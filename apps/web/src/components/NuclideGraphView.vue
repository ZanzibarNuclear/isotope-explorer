<script setup lang="ts">
import { computed, ref, watch } from "vue";
import type { NuclideInfo, StepInfo } from "@wasm/nuclear_sim_wasm.js";
import { PERIODIC_TABLE } from "../data/periodic-table-layout";

const props = defineProps<{
  steps: StepInfo[];
  cursor: number;
  fissionTails?: {
    following_light: boolean;
    light: StepInfo[];
    heavy: StepInfo[];
  };
}>();

const emit = defineEmits<{
  (e: "go-to-step", index: number): void;
}>();

const CELL = 72;
const NODE_W = 56;
const NODE_H = 50;
const NODE_HW = NODE_W / 2;
const NODE_HH = NODE_H / 2;
const VIEW_COLS = 7;
const VIEW_ROWS = 5;
const ZOOM_LEVELS = [1, 0.75, 0.5, "fit"] as const;

type ZoomLevel = (typeof ZOOM_LEVELS)[number];

const zoomLevel = ref<ZoomLevel>(1);
const panOffset = ref({ x: 0, y: 0 });
const dragState = ref<{
  pointerId: number;
  startClientX: number;
  startClientY: number;
  startPanX: number;
  startPanY: number;
  svgWidth: number;
  svgHeight: number;
} | null>(null);

const ELEMENT_SYMBOL_BY_Z = new Map(PERIODIC_TABLE.map((e) => [e.z, e.symbol]));
const elementSymbol = (z: number) => ELEMENT_SYMBOL_BY_Z.get(z) ?? "";

const HALF_LIFE_INFINITY = "∞";

function decaySymbol(mode?: string): string {
  switch (mode) {
    case "alpha":
      return "α";
    case "beta-minus":
      return "β−";
    case "beta-plus":
      return "β+";
    case "electron-capture":
      return "ε";
    case "isomeric-transition":
      return "IT";
    default:
      return mode ?? "";
  }
}

function edgeLabel(step: StepInfo): string {
  switch (step.event_type) {
    case "neutron-absorbed":
      return "+n";
    case "decay":
      return decaySymbol(step.detail?.decay_mode);
    case "fission":
      return "fission";
    default:
      return "";
  }
}

function formatHalfLife(s: number): string {
  const min = 60,
    hr = 3600,
    day = 86400,
    yr = 365.25 * day;
  if (s >= yr * 1e9) return `${(s / (yr * 1e9)).toPrecision(3)} Gyr`;
  if (s >= yr * 1e6) return `${(s / (yr * 1e6)).toPrecision(3)} Myr`;
  if (s >= yr * 1e3) return `${(s / (yr * 1e3)).toPrecision(3)} kyr`;
  if (s >= yr) return `${Math.round(s / yr).toLocaleString()} yr`;
  if (s >= day) return `${(s / day).toFixed(1)} d`;
  if (s >= hr) return `${(s / hr).toFixed(1)} h`;
  if (s >= min) return `${(s / min).toFixed(1)} min`;
  if (s >= 1)
    return `${s < 100 ? s.toPrecision(3) : Math.round(s).toLocaleString()} s`;
  return `${s.toPrecision(2)} s`;
}

function halfLifeDisplay(opts: {
  unknown: boolean;
  isStable: boolean;
  halfLife: number | null | undefined;
}): string {
  if (opts.unknown) return "??";
  if (opts.isStable) return HALF_LIFE_INFINITY;
  if (typeof opts.halfLife === "number") return formatHalfLife(opts.halfLife);
  return "—";
}

interface RawNode {
  z: number;
  n: number;
  notation: string;
  index?: number;
  isStable: boolean;
  unknown: boolean;
  ghost: boolean;
  halfLife: number | null | undefined;
}

interface RawEdge {
  fromZ: number;
  fromN: number;
  toZ: number;
  toN: number;
  label: string;
  kind: "neutron" | "decay" | "fission";
  ghost: boolean;
}

interface NuclidePoint {
  z: number;
  n: number;
}

function centeredAxisBounds(value: number, visibleCells: number, minValue = 0) {
  let min = value - Math.floor(visibleCells / 2);
  let max = min + visibleCells - 1;
  if (min < minValue) {
    max += minValue - min;
    min = minValue;
  }
  return { min, max };
}

function zoomLabel(level: ZoomLevel): string {
  if (level === "fit") return "Fit";
  return `${Math.round(level * 100)}%`;
}

function recenterGraph() {
  panOffset.value = { x: 0, y: 0 };
}

watch(
  () => [props.cursor, zoomLevel.value] as const,
  () => {
    recenterGraph();
  },
);

const layout = computed(() => {
  const steps = props.steps;
  if (steps.length === 0) return null;

  const start = steps[0].nuclide;

  const nodeMap = new Map<string, RawNode>();
  const edgeMap = new Map<string, RawEdge>();

  const key = (z: number, n: number) => `${z}:${n}`;

  function addNode(
    z: number,
    n: number,
    notation: string,
    opts: {
      index?: number;
      isStable?: boolean;
      unknown?: boolean;
      ghost?: boolean;
      halfLife?: number | null;
    },
  ) {
    const k = key(z, n);
    const existing = nodeMap.get(k);
    if (existing) {
      // Real step data (non-ghost) wins over a placeholder ghost.
      if (existing.ghost && !opts.ghost) {
        existing.ghost = false;
        existing.index = opts.index ?? existing.index;
        existing.isStable = opts.isStable ?? existing.isStable;
        existing.unknown = opts.unknown ?? existing.unknown;
        existing.halfLife = opts.halfLife;
        existing.notation = notation;
      } else if (existing.index === undefined && opts.index !== undefined) {
        existing.index = opts.index;
      }
      return;
    }
    nodeMap.set(k, {
      z,
      n,
      notation,
      index: opts.index,
      isStable: opts.isStable ?? false,
      unknown: opts.unknown ?? false,
      ghost: opts.ghost ?? false,
      halfLife: opts.halfLife,
    });
  }

  function addEdge(edge: RawEdge) {
    const k = [
      edge.fromZ,
      edge.fromN,
      edge.toZ,
      edge.toN,
      edge.kind,
      edge.label,
    ].join(":");
    const existing = edgeMap.get(k);
    if (existing) {
      if (existing.ghost && !edge.ghost) existing.ghost = false;
      return;
    }
    edgeMap.set(k, edge);
  }

  function addFragmentNode(fragment: NuclideInfo, ghost: boolean) {
    // Fragment half_life_s convention: undefined = unknown, null = stable.
    const hl = fragment.half_life_s;
    addNode(fragment.z, fragment.n, fragment.notation, {
      ghost,
      unknown: hl === undefined,
      isStable: hl === null,
      halfLife: typeof hl === "number" ? hl : undefined,
    });
  }

  function edgeKindForStep(step: StepInfo): RawEdge["kind"] {
    return step.event_type === "neutron-absorbed"
      ? "neutron"
      : step.event_type === "fission"
        ? "fission"
        : "decay";
  }

  function addRevealedTail(
    parent: NuclidePoint,
    fragment: NuclideInfo | undefined,
    tail: StepInfo[],
    activeLeg: boolean,
  ) {
    if (!fragment || tail.length === 0) return;

    addFragmentNode(fragment, false);
    addEdge({
      fromZ: parent.z,
      fromN: parent.n,
      toZ: fragment.z,
      toN: fragment.n,
      label: "",
      kind: "fission",
      ghost: false,
    });

    let previous: NuclidePoint = fragment;
    for (const step of tail) {
      if (step.event_type === "stable") continue;
      addNode(step.nuclide.z, step.nuclide.n, step.nuclide.notation, {
        index: activeLeg ? step.index : undefined,
        isStable: step.nuclide_is_stable,
        unknown: !step.nuclide_in_database,
        halfLife: step.nuclide_half_life_s,
      });
      addEdge({
        fromZ: previous.z,
        fromN: previous.n,
        toZ: step.nuclide.z,
        toN: step.nuclide.n,
        label: edgeLabel(step),
        kind: edgeKindForStep(step),
        ghost: false,
      });
      previous = step.nuclide;
    }
  }

  for (let i = 0; i < steps.length; i++) {
    const step = steps[i];
    if (step.event_type === "stable") continue;

    addNode(step.nuclide.z, step.nuclide.n, step.nuclide.notation, {
      index: step.index,
      isStable: step.nuclide_is_stable,
      unknown: !step.nuclide_in_database,
      halfLife: step.nuclide_half_life_s,
    });

    if (i > 0) {
      const prev = steps[i - 1];
      addEdge({
        fromZ: prev.nuclide.z,
        fromN: prev.nuclide.n,
        toZ: step.nuclide.z,
        toN: step.nuclide.n,
        label: edgeLabel(step),
        kind: edgeKindForStep(step),
        ghost: false,
      });

      if (step.event_type === "fission") {
        const followed = step.nuclide;
        const heavy = step.detail?.heavy_fragment;
        const light = step.detail?.light_fragment;
        const isFollowed = (frag: { z: number; n: number } | undefined) =>
          !!frag && frag.z === followed.z && frag.n === followed.n;
        const other = isFollowed(heavy) ? light : heavy;
        if (other) {
          addFragmentNode(other, true);
          addEdge({
            fromZ: prev.nuclide.z,
            fromN: prev.nuclide.n,
            toZ: other.z,
            toN: other.n,
            label: "",
            kind: "fission",
            ghost: true,
          });
        }
      }
    }
  }

  const fi = steps.findIndex((step) => step.event_type === "fission");
  const fissionStep = fi >= 0 ? steps[fi] : undefined;
  const fissionParent = fi > 0 ? steps[fi - 1].nuclide : fissionStep?.detail?.parent;
  if (fissionStep && fissionParent && props.fissionTails) {
    addRevealedTail(
      fissionParent,
      fissionStep.detail?.light_fragment,
      props.fissionTails.light,
      props.fissionTails.following_light,
    );
    addRevealedTail(
      fissionParent,
      fissionStep.detail?.heavy_fragment,
      props.fissionTails.heavy,
      !props.fissionTails.following_light,
    );
  }

  const nodes = Array.from(nodeMap.values());
  const edges = Array.from(edgeMap.values());

  let dataMinZ = start.z,
    dataMaxZ = start.z,
    dataMinN = start.n,
    dataMaxN = start.n;
  for (const node of nodes) {
    if (node.z < dataMinZ) dataMinZ = node.z;
    if (node.z > dataMaxZ) dataMaxZ = node.z;
    if (node.n < dataMinN) dataMinN = node.n;
    if (node.n > dataMaxN) dataMaxN = node.n;
  }

  const cursorStep = steps[props.cursor] ?? steps[0];
  const activeNuclide = cursorStep.nuclide;
  const currentZoomLevel = zoomLevel.value;
  const minNForLabels = dataMinN === 0 ? -1 : 0;
  const fitMinN = Math.max(minNForLabels, dataMinN - 2);
  const fitMaxN = dataMaxN + 1;
  const fitMinZ = Math.max(0, dataMinZ - 2);
  const fitMaxZ = dataMaxZ + 1;
  const viewCols =
    currentZoomLevel === "fit"
      ? fitMaxN - fitMinN + 1
      : Math.round(VIEW_COLS / currentZoomLevel);
  const viewRows =
    currentZoomLevel === "fit"
      ? fitMaxZ - fitMinZ + 1
      : Math.round(VIEW_ROWS / currentZoomLevel);
  const nBounds =
    currentZoomLevel === "fit"
      ? { min: fitMinN, max: fitMaxN }
      : centeredAxisBounds(
          activeNuclide.n,
          viewCols,
          activeNuclide.n === 0 ? -1 : 0,
        );
  const zBounds =
    currentZoomLevel === "fit"
      ? { min: fitMinZ, max: fitMaxZ }
      : centeredAxisBounds(activeNuclide.z, viewRows);
  const minN = nBounds.min,
    maxN = nBounds.max;
  const minZ = zBounds.min,
    maxZ = zBounds.max;

  const cols = maxN - minN + 1;
  const rows = maxZ - minZ + 1;
  const width = cols * CELL;
  const height = rows * CELL;

  const nodeX = (n: number) => (n - minN) * CELL + CELL / 2;
  const nodeY = (z: number) => (maxZ - z) * CELL + CELL / 2;

  const activeKey = cursorStep
    ? key(cursorStep.nuclide.z, cursorStep.nuclide.n)
    : null;

  const placedNodes = nodes.map((node) => ({
    ...node,
    isActive: activeKey === key(node.z, node.n),
    isStart: node.z === start.z && node.n === start.n,
    x: nodeX(node.n),
    y: nodeY(node.z),
    halfLifeText: halfLifeDisplay(node),
  }));

  // Clip a ray from a rect's center to its boundary along (ux, uy).
  function rectClip(ux: number, uy: number) {
    const ax = Math.abs(ux),
      ay = Math.abs(uy);
    const tx = ax > 0 ? NODE_HW / ax : Infinity;
    const ty = ay > 0 ? NODE_HH / ay : Infinity;
    return Math.min(tx, ty);
  }

  const placedEdges = edges.flatMap((e) => {
    const x1 = nodeX(e.fromN),
      y1 = nodeY(e.fromZ);
    const x2 = nodeX(e.toN),
      y2 = nodeY(e.toZ);
    const dx = x2 - x1,
      dy = y2 - y1;
    const len = Math.hypot(dx, dy);
    if (len === 0) return [];
    const ux = dx / len,
      uy = dy / len;
    const t = rectClip(ux, uy);
    return [
      {
        ...e,
        x1: x1 + ux * t,
        y1: y1 + uy * t,
        x2: x2 - ux * t,
        y2: y2 - uy * t,
        midX: (x1 + ux * t + x2 - ux * t) / 2,
        midY: (y1 + uy * t + y2 - uy * t) / 2,
      },
    ];
  });

  // Grid and labels track the current viewBox, so after manual panning the
  // axes still describe the area the user is looking at.
  const viewMinX = panOffset.value.x;
  const viewMaxX = panOffset.value.x + width;
  const viewMinY = panOffset.value.y;
  const viewMaxY = panOffset.value.y + height;

  const firstNBoundary = Math.max(0, minN + Math.floor(viewMinX / CELL));
  const lastNBoundary = Math.max(
    firstNBoundary,
    minN + Math.ceil(viewMaxX / CELL),
  );
  const vGridLines: { x: number }[] = [];
  for (let n = firstNBoundary; n <= lastNBoundary; n++) {
    vGridLines.push({ x: (n - minN) * CELL });
  }

  const firstZBoundary = Math.max(0, maxZ - Math.ceil(viewMaxY / CELL));
  const lastZBoundary = Math.max(
    firstZBoundary,
    maxZ - Math.floor(viewMinY / CELL),
  );
  const hGridLines: { y: number }[] = [];
  for (let z = firstZBoundary; z <= lastZBoundary; z++) {
    hGridLines.push({ y: (maxZ - z) * CELL });
  }

  const nTicks: { n: number; x: number }[] = [];
  const firstNLabel = firstNBoundary;
  const lastNLabel = Math.max(firstNLabel, lastNBoundary - 1);
  for (let n = firstNLabel; n <= lastNLabel; n++) {
    nTicks.push({ n, x: nodeX(n) });
  }

  const zTicks: { z: number; y: number; symbol: string }[] = [];
  const firstZLabel = Math.max(0, maxZ - Math.ceil(viewMaxY / CELL) + 1);
  const lastZLabel = Math.max(firstZLabel, maxZ - Math.floor(viewMinY / CELL));
  for (let z = firstZLabel; z <= lastZLabel; z++) {
    zTicks.push({ z, y: nodeY(z), symbol: elementSymbol(z) });
  }

  const bottomLeftN = minN + Math.floor(viewMinX / CELL);
  const bottomLeftZ = firstZLabel;
  const nLabelTicks = nTicks.filter((t) => t.n !== bottomLeftN);
  const zLabelTicks = zTicks.filter((t) => t.z !== bottomLeftZ);

  return {
    nodes: placedNodes,
    edges: placedEdges,
    width,
    height,
    viewBox: `${panOffset.value.x} ${panOffset.value.y} ${width} ${height}`,
    viewBoxX: panOffset.value.x,
    viewBoxY: panOffset.value.y,
    cellSize: CELL,
    nodeW: NODE_W,
    nodeH: NODE_H,
    nodeHW: NODE_HW,
    nodeHH: NODE_HH,
    bounds: { minN, maxN, minZ, maxZ, dataMinN, dataMaxN, dataMinZ, dataMaxZ },
    zoomLevel: zoomLevel.value,
    origin: {
      x: -minN * CELL,
      y: (maxZ + 1) * CELL,
    },
    vGridLines,
    hGridLines,
    nTicks,
    zTicks,
    nLabelTicks,
    zLabelTicks,
  };
});

function onNodeClick(node: { index?: number }) {
  if (node.index === undefined) return;
  recenterGraph();
  emit("go-to-step", node.index);
}

function onGraphPointerDown(event: PointerEvent) {
  if (!layout.value || event.button !== 0) return;
  const svg = event.currentTarget as SVGSVGElement;
  const rect = svg.getBoundingClientRect();
  if (rect.width === 0 || rect.height === 0) return;

  svg.setPointerCapture(event.pointerId);
  dragState.value = {
    pointerId: event.pointerId,
    startClientX: event.clientX,
    startClientY: event.clientY,
    startPanX: panOffset.value.x,
    startPanY: panOffset.value.y,
    svgWidth: rect.width,
    svgHeight: rect.height,
  };
}

function onGraphPointerMove(event: PointerEvent) {
  const drag = dragState.value;
  const currentLayout = layout.value;
  if (!drag || !currentLayout || drag.pointerId !== event.pointerId) return;

  const dx =
    ((event.clientX - drag.startClientX) / drag.svgWidth) * currentLayout.width;
  const dy =
    ((event.clientY - drag.startClientY) / drag.svgHeight) *
    currentLayout.height;
  panOffset.value = {
    x: drag.startPanX - dx,
    y: drag.startPanY - dy,
  };
}

function onGraphPointerEnd(event: PointerEvent) {
  if (dragState.value?.pointerId !== event.pointerId) return;
  (event.currentTarget as SVGSVGElement).releasePointerCapture(event.pointerId);
  dragState.value = null;
}
</script>

<template>
  <div class="nuclide-graph">
    <div v-if="layout" class="axis-labels">
      <span class="axis-label">Z (protons) &uarr;</span>
      <span class="axis-sep">&middot;</span>
      <span class="axis-label">N (neutrons) &rarr;</span>
    </div>
    <div class="graph-frame" v-if="layout">
      <div class="zoom-control" aria-label="Graph zoom">
        <button
          type="button"
          class="zoom-btn recenter-btn"
          :disabled="panOffset.x === 0 && panOffset.y === 0"
          @click="recenterGraph">
          Center
        </button>
        <button
          v-for="level in ZOOM_LEVELS"
          :key="level"
          type="button"
          class="zoom-btn"
          :class="{ active: zoomLevel === level }"
          :aria-pressed="zoomLevel === level"
          @click="zoomLevel = level">
          {{ zoomLabel(level) }}
        </button>
      </div>
      <svg
        class="graph"
        :class="{ panning: dragState }"
        :viewBox="layout.viewBox"
        preserveAspectRatio="xMidYMid meet"
        @pointerdown="onGraphPointerDown"
        @pointermove="onGraphPointerMove"
        @pointerup="onGraphPointerEnd"
        @pointercancel="onGraphPointerEnd"
        @lostpointercapture="dragState = null">
        <defs>
          <marker
            id="ng-arrow"
            viewBox="0 0 10 10"
            refX="9"
            refY="5"
            markerWidth="6"
            markerHeight="6"
            orient="auto">
            <path d="M 0 0 L 10 5 L 0 10 z" fill="#8b949e" />
          </marker>
          <marker
            id="ng-arrow-fission"
            viewBox="0 0 10 10"
            refX="9"
            refY="5"
            markerWidth="6"
            markerHeight="6"
            orient="auto">
            <path d="M 0 0 L 10 5 L 0 10 z" fill="#f0883e" />
          </marker>
          <marker
            id="ng-arrow-ghost"
            viewBox="0 0 10 10"
            refX="9"
            refY="5"
            markerWidth="6"
            markerHeight="6"
            orient="auto">
            <path d="M 0 0 L 10 5 L 0 10 z" fill="#484f58" />
          </marker>
        </defs>

        <g class="grid">
          <line
            v-for="(line, i) in layout.vGridLines"
            :key="'v' + i"
            :x1="line.x"
            :y1="layout.viewBoxY"
            :x2="line.x"
            :y2="layout.viewBoxY + layout.height" />
          <line
            v-for="(line, i) in layout.hGridLines"
            :key="'h' + i"
            :x1="layout.viewBoxX"
            :y1="line.y"
            :x2="layout.viewBoxX + layout.width"
            :y2="line.y" />
          <line
            class="axis-line"
            :x1="layout.origin.x"
            :y1="layout.viewBoxY"
            :x2="layout.origin.x"
            :y2="layout.viewBoxY + layout.height" />
          <line
            class="axis-line"
            :x1="layout.viewBoxX"
            :y1="layout.origin.y"
            :x2="layout.viewBoxX + layout.width"
            :y2="layout.origin.y" />
        </g>

        <g class="ticks">
          <text
            v-for="(t, i) in layout.nLabelTicks"
            :key="'nt' + i"
            :x="t.x"
            :y="layout.viewBoxY + layout.height - layout.cellSize / 2 + 4"
            class="tick-label tick-n"
            text-anchor="middle">
            {{ t.n }}
          </text>
          <text
            v-for="(t, i) in layout.zLabelTicks"
            :key="'zt' + i"
            :x="layout.viewBoxX + layout.cellSize / 2"
            :y="t.y - 6"
            class="tick-label tick-z-symbol"
            text-anchor="middle">
            {{ t.symbol }}
          </text>
          <text
            v-for="(t, i) in layout.zLabelTicks"
            :key="'zn' + i"
            :x="layout.viewBoxX + layout.cellSize / 2"
            :y="t.y + 8"
            class="tick-label tick-z-number"
            text-anchor="middle">
            {{ t.z }}
          </text>
        </g>

        <g class="edges">
          <g
            v-for="(edge, i) in layout.edges"
            :key="'e' + i"
            class="edge"
            :class="[edge.kind, { ghost: edge.ghost }]">
            <line
              :x1="edge.x1"
              :y1="edge.y1"
              :x2="edge.x2"
              :y2="edge.y2"
              :marker-end="
                edge.ghost
                  ? 'url(#ng-arrow-ghost)'
                  : edge.kind === 'fission'
                    ? 'url(#ng-arrow-fission)'
                    : 'url(#ng-arrow)'
              " />
            <text
              v-if="edge.label"
              :x="edge.midX"
              :y="edge.midY"
              class="edge-label"
              text-anchor="middle"
              dominant-baseline="middle">
              {{ edge.label }}
            </text>
          </g>
        </g>

        <g class="nodes">
          <g
            v-for="(node, i) in layout.nodes"
            :key="'n' + i"
            class="node"
            :class="{
              active: node.isActive,
              stable: node.isStable,
              unknown: node.unknown,
              ghost: node.ghost,
              start: node.isStart,
              clickable: node.index !== undefined,
            }"
            :transform="`translate(${node.x}, ${node.y})`"
            @pointerdown.stop
            @click="onNodeClick(node)">
            <rect
              :x="-layout.nodeHW"
              :y="-layout.nodeHH"
              :width="layout.nodeW"
              :height="layout.nodeH"
              rx="5"
              ry="5" />
            <text class="node-notation" text-anchor="middle" y="-2">
              {{ node.notation }}
            </text>
            <text class="node-halflife" text-anchor="middle" y="13">
              {{ node.halfLifeText }}
            </text>
          </g>
        </g>
      </svg>
    </div>
  </div>
</template>

<style scoped>
.nuclide-graph {
  display: flex;
  flex-direction: column;
  align-items: stretch;
  box-sizing: border-box;
  padding: 1rem 1.25rem;
  width: 100%;
}

.axis-labels {
  display: flex;
  gap: 0.5rem;
  font-size: 0.75rem;
  color: #6e7681;
  margin-bottom: 0.5rem;
}
.axis-sep {
  color: #30363d;
}

.graph-frame {
  position: relative;
  overflow: hidden;
  border: 1px solid #21262d;
  border-radius: 8px;
  background: #0d1117;
  display: flex;
  max-width: 100%;
  height: clamp(340px, calc(100vh - 15rem), 720px);
}

.zoom-control {
  position: absolute;
  top: 0.6rem;
  right: 1.5rem;
  z-index: 1;
  display: flex;
  gap: 2px;
  padding: 2px;
  border: 1px solid #30363d;
  border-radius: 6px;
  background: #0d1117d9;
  backdrop-filter: blur(4px);
}

.zoom-btn {
  padding: 0.2rem 0.45rem;
  border: none;
  border-radius: 4px;
  background: transparent;
  color: #8b949e;
  font-size: 0.72rem;
  font-weight: 600;
  cursor: pointer;
}

.zoom-btn:hover {
  color: #e6edf3;
  background: #21262d;
}

.zoom-btn.active {
  color: #58a6ff;
  background: #1f6feb33;
}

.zoom-btn:disabled {
  color: #484f58;
  cursor: not-allowed;
  background: transparent;
}

.recenter-btn {
  margin-right: 2px;
}

.graph {
  display: block;
  width: 100%;
  height: 100%;
  cursor: grab;
  touch-action: none;
  user-select: none;
}

.graph.panning {
  cursor: grabbing;
}

.grid line {
  stroke: #161b22;
  stroke-width: 1;
}
.grid .axis-line {
  stroke: #30363d;
  stroke-width: 2;
}

.ticks .tick-label {
  font-family: ui-monospace, monospace;
  pointer-events: none;
  fill: #484f58;
}
.tick-n {
  font-size: 9px;
}
.tick-z-symbol {
  font-size: 11px;
  font-weight: 700;
  fill: #6e7681;
}
.tick-z-number {
  font-size: 9px;
}

.edge line {
  stroke: #8b949e;
  stroke-width: 1.5;
}
.edge.fission line {
  stroke: #f0883e;
  stroke-width: 2;
}
.edge.ghost line {
  stroke: #484f58;
  stroke-dasharray: 4 3;
  stroke-width: 1.25;
}
.edge-label {
  fill: #c9d1d9;
  font-size: 11px;
  font-family: ui-sans-serif, system-ui;
  paint-order: stroke;
  stroke: #0d1117;
  stroke-width: 3;
  stroke-linejoin: round;
  pointer-events: none;
}
.edge.fission .edge-label {
  fill: #f0883e;
}

.node rect {
  fill: #21262d;
  stroke: #30363d;
  stroke-width: 1.5;
  transition:
    stroke 0.12s,
    fill 0.12s;
}
.node.clickable {
  cursor: pointer;
}
.node.clickable:hover rect {
  stroke: #58a6ff;
}
.node.active rect {
  fill: #1f6feb33;
  stroke: #58a6ff;
  stroke-width: 2.5;
}
.node.stable rect {
  stroke: #3fb950;
}
.node.stable.active rect {
  fill: #3fb95022;
}
.node.unknown rect {
  stroke: #d29922;
  stroke-dasharray: 3 2;
}
.node.ghost rect {
  stroke: #484f58;
  fill: #161b22;
  stroke-dasharray: 3 2;
}
.node.start rect {
  stroke-width: 2;
}
.node.start:not(.active):not(.stable) rect {
  stroke: #58a6ff80;
}

.node-notation {
  fill: #e6edf3;
  font-size: 12px;
  font-weight: 700;
  font-family: ui-sans-serif, system-ui;
  pointer-events: none;
}
.node-halflife {
  fill: #8b949e;
  font-size: 9px;
  font-family: ui-monospace, monospace;
  font-variant-numeric: tabular-nums;
  pointer-events: none;
}
.node.stable .node-halflife {
  fill: #3fb95099;
}
.node.stable .node-halflife {
  font-size: 16px;
}
.node.unknown .node-halflife {
  fill: #d29922;
}
.node.ghost .node-notation {
  fill: #6e7681;
  font-weight: 600;
}
.node.ghost .node-halflife {
  fill: #484f58;
}
</style>
