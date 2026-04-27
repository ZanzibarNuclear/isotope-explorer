<script setup lang="ts">
import { computed } from "vue";
import type { StepInfo } from "@wasm/nuclear_sim_wasm.js";

const props = defineProps<{
  steps: StepInfo[];
  cursor: number;
}>();

const emit = defineEmits<{
  (e: "go-to-step", index: number): void;
}>();

const CELL = 64;
const NODE_R = 24;

function decaySymbol(mode?: string): string {
  switch (mode) {
    case "alpha": return "α";
    case "beta-minus": return "β−";
    case "beta-plus": return "β+";
    case "electron-capture": return "ε";
    case "isomeric-transition": return "IT";
    default: return mode ?? "";
  }
}

function edgeLabel(step: StepInfo): string {
  switch (step.event_type) {
    case "neutron-absorbed": return "+n";
    case "decay": return decaySymbol(step.detail?.decay_mode);
    case "fission": return "fission";
    default: return "";
  }
}

interface RawNode {
  z: number;
  n: number;
  notation: string;
  index?: number;
  isStable: boolean;
  unknown: boolean;
  ghost: boolean;
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

const layout = computed(() => {
  const steps = props.steps;
  if (steps.length === 0) {
    return null;
  }

  const start = steps[0].nuclide;

  const nodeMap = new Map<string, RawNode>();
  const edges: RawEdge[] = [];

  function key(z: number, n: number) { return `${z}:${n}`; }

  function addNode(z: number, n: number, notation: string, opts: {
    index?: number; isStable?: boolean; unknown?: boolean; ghost?: boolean;
  }) {
    const k = key(z, n);
    const existing = nodeMap.get(k);
    if (existing) {
      // Prefer non-ghost data and earliest step index for navigation.
      if (existing.ghost && !opts.ghost) {
        existing.ghost = false;
        existing.index = opts.index ?? existing.index;
        existing.isStable = opts.isStable ?? existing.isStable;
        existing.unknown = opts.unknown ?? existing.unknown;
        existing.notation = notation;
      } else if (existing.index === undefined && opts.index !== undefined) {
        existing.index = opts.index;
      }
      return;
    }
    nodeMap.set(k, {
      z, n, notation,
      index: opts.index,
      isStable: opts.isStable ?? false,
      unknown: opts.unknown ?? false,
      ghost: opts.ghost ?? false,
    });
  }

  for (let i = 0; i < steps.length; i++) {
    const step = steps[i];
    if (step.event_type === "stable") continue;

    addNode(step.nuclide.z, step.nuclide.n, step.nuclide.notation, {
      index: step.index,
      isStable: step.nuclide_is_stable,
      unknown: !step.nuclide_in_database,
    });

    if (i > 0) {
      const prev = steps[i - 1];
      const kind: RawEdge["kind"] =
        step.event_type === "neutron-absorbed" ? "neutron" :
        step.event_type === "fission" ? "fission" : "decay";

      edges.push({
        fromZ: prev.nuclide.z, fromN: prev.nuclide.n,
        toZ: step.nuclide.z, toN: step.nuclide.n,
        label: edgeLabel(step),
        kind,
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
          addNode(other.z, other.n, other.notation, { ghost: true });
          edges.push({
            fromZ: prev.nuclide.z, fromN: prev.nuclide.n,
            toZ: other.z, toN: other.n,
            label: "",
            kind: "fission",
            ghost: true,
          });
        }
      }
    }
  }

  const nodes = Array.from(nodeMap.values());

  // Symmetric extent around the starting isotope.
  let radZ = 1, radN = 1;
  for (const node of nodes) {
    radZ = Math.max(radZ, Math.abs(node.z - start.z));
    radN = Math.max(radN, Math.abs(node.n - start.n));
  }
  radZ += 1;
  radN += 1;

  const minZ = start.z - radZ;
  const maxZ = start.z + radZ;
  const minN = start.n - radN;
  const maxN = start.n + radN;

  const cols = maxN - minN + 1;
  const rows = maxZ - minZ + 1;
  const width = cols * CELL;
  const height = rows * CELL;

  const nodeX = (n: number) => (n - minN) * CELL + CELL / 2;
  const nodeY = (z: number) => (maxZ - z) * CELL + CELL / 2;

  const cursorStep = steps[props.cursor];
  const activeKey = cursorStep ? key(cursorStep.nuclide.z, cursorStep.nuclide.n) : null;

  const placedNodes = nodes.map(node => ({
    ...node,
    isActive: activeKey === key(node.z, node.n),
    isStart: node.z === start.z && node.n === start.n,
    x: nodeX(node.n),
    y: nodeY(node.z),
  }));

  const placedEdges = edges.flatMap(e => {
    const x1 = nodeX(e.fromN), y1 = nodeY(e.fromZ);
    const x2 = nodeX(e.toN), y2 = nodeY(e.toZ);
    const dx = x2 - x1, dy = y2 - y1;
    const len = Math.hypot(dx, dy);
    if (len === 0) return [];
    const ux = dx / len, uy = dy / len;
    return [{
      ...e,
      x1: x1 + ux * NODE_R,
      y1: y1 + uy * NODE_R,
      x2: x2 - ux * NODE_R,
      y2: y2 - uy * NODE_R,
      midX: (x1 + x2) / 2,
      midY: (y1 + y2) / 2,
    }];
  });

  // Axis tick positions (every cell).
  const nTicks: { n: number; x: number }[] = [];
  for (let n = minN; n <= maxN; n++) nTicks.push({ n, x: nodeX(n) });
  const zTicks: { z: number; y: number }[] = [];
  for (let z = minZ; z <= maxZ; z++) zTicks.push({ z, y: nodeY(z) });

  return {
    nodes: placedNodes,
    edges: placedEdges,
    width,
    height,
    viewBox: `0 0 ${width} ${height}`,
    cellSize: CELL,
    bounds: { minN, maxN, minZ, maxZ },
    nTicks,
    zTicks,
  };
});

function onNodeClick(node: { index?: number }) {
  if (node.index !== undefined) emit("go-to-step", node.index);
}
</script>

<template>
  <div class="nuclide-graph">
    <div v-if="layout" class="axis-labels">
      <span class="axis-label">Z (protons) &uarr;</span>
      <span class="axis-sep">&middot;</span>
      <span class="axis-label">N (neutrons) &rarr;</span>
    </div>
    <div class="graph-scroll" v-if="layout">
      <svg class="graph" :width="layout.width" :height="layout.height" :viewBox="layout.viewBox">
        <defs>
          <marker id="ng-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto">
            <path d="M 0 0 L 10 5 L 0 10 z" fill="#8b949e" />
          </marker>
          <marker id="ng-arrow-fission" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto">
            <path d="M 0 0 L 10 5 L 0 10 z" fill="#f0883e" />
          </marker>
          <marker id="ng-arrow-ghost" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto">
            <path d="M 0 0 L 10 5 L 0 10 z" fill="#484f58" />
          </marker>
        </defs>

        <g class="grid">
          <line v-for="(t, i) in layout.nTicks" :key="'v'+i"
            :x1="t.x - layout.cellSize / 2" :y1="0"
            :x2="t.x - layout.cellSize / 2" :y2="layout.height" />
          <line :x1="layout.width" :y1="0" :x2="layout.width" :y2="layout.height" />
          <line v-for="(t, i) in layout.zTicks" :key="'h'+i"
            :x1="0" :y1="t.y - layout.cellSize / 2"
            :x2="layout.width" :y2="t.y - layout.cellSize / 2" />
          <line :x1="0" :y1="layout.height" :x2="layout.width" :y2="layout.height" />
        </g>

        <g class="ticks">
          <text v-for="(t, i) in layout.nTicks" :key="'nt'+i"
            :x="t.x" :y="layout.height - 4" class="tick-label" text-anchor="middle">{{ t.n }}</text>
          <text v-for="(t, i) in layout.zTicks" :key="'zt'+i"
            :x="4" :y="t.y" class="tick-label" dominant-baseline="middle">{{ t.z }}</text>
        </g>

        <g class="edges">
          <g v-for="(edge, i) in layout.edges" :key="'e'+i"
            class="edge" :class="[edge.kind, { ghost: edge.ghost }]">
            <line :x1="edge.x1" :y1="edge.y1" :x2="edge.x2" :y2="edge.y2"
              :marker-end="edge.ghost ? 'url(#ng-arrow-ghost)' : edge.kind === 'fission' ? 'url(#ng-arrow-fission)' : 'url(#ng-arrow)'" />
            <text v-if="edge.label" :x="edge.midX" :y="edge.midY"
              class="edge-label" text-anchor="middle" dy="-4">{{ edge.label }}</text>
          </g>
        </g>

        <g class="nodes">
          <g v-for="(node, i) in layout.nodes" :key="'n'+i"
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
            @click="onNodeClick(node)">
            <circle :r="24" />
            <text class="node-label" text-anchor="middle" dy="0.35em">{{ node.notation }}</text>
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
.axis-sep { color: #30363d; }

.graph-scroll {
  overflow: auto;
  border: 1px solid #21262d;
  border-radius: 8px;
  background: #0d1117;
  max-width: 100%;
}

.graph { display: block; }

.grid line {
  stroke: #161b22;
  stroke-width: 1;
}

.ticks .tick-label {
  fill: #484f58;
  font-size: 9px;
  font-family: ui-monospace, monospace;
  pointer-events: none;
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
.edge.fission .edge-label { fill: #f0883e; }

.node circle {
  fill: #21262d;
  stroke: #30363d;
  stroke-width: 1.5;
  transition: stroke 0.12s, fill 0.12s;
}
.node.clickable { cursor: pointer; }
.node.clickable:hover circle {
  stroke: #58a6ff;
}
.node.active circle {
  fill: #1f6feb33;
  stroke: #58a6ff;
  stroke-width: 2.5;
}
.node.stable circle {
  stroke: #3fb950;
}
.node.stable.active circle {
  fill: #3fb95022;
}
.node.unknown circle {
  stroke: #d29922;
  stroke-dasharray: 3 2;
}
.node.ghost circle {
  stroke: #484f58;
  fill: #161b22;
  stroke-dasharray: 3 2;
}
.node.start circle {
  stroke-width: 2;
}
.node.start:not(.active) circle {
  stroke: #58a6ff80;
}
.node-label {
  fill: #e6edf3;
  font-size: 11px;
  font-weight: 700;
  font-family: ui-sans-serif, system-ui;
  pointer-events: none;
}
.node.ghost .node-label { fill: #6e7681; font-weight: 600; }
</style>
