<script setup lang="ts">
import { computed, ref, watch } from "vue";
import type { StepInfo } from "@wasm/nuclear_sim_wasm.js";

const props = defineProps<{
  steps: StepInfo[];
  cursor: number;
  followingHeavy?: boolean;
  session: unknown;
}>();

const emit = defineEmits<{
  (e: "go-to-step", index: number): void;
  (e: "go-to-branch-step", leg: "light" | "heavy", fissionIndex: number, offset: number): void;
}>();

// ── Types ────────────────────────────────────────────────────────────────────

interface CardItem {
  kind: "card";
  step: StepInfo;
  isActive: boolean;
}
interface ConnectorItem {
  kind: "connector";
  label: string;
}
interface FissionSplitItem {
  kind: "fission-split";
  step: StepInfo;
  followedIsHeavy: boolean;
  label: string;
  isActive: boolean;
}

type RenderItem = CardItem | ConnectorItem | FissionSplitItem;

type RenderBlock =
  | { kind: "segment"; items: RenderItem[] }
  | { kind: "fission"; step: StepInfo; followedIsHeavy: boolean; label: string; isActive: boolean }
  | { kind: "parallel"; lightItems: RenderItem[]; heavyItems: RenderItem[] };

// ── Formatters ───────────────────────────────────────────────────────────────

const HALF_LIFE_INFINITY = "\u221e";

function formatDecayMode(mode: string): string {
  switch (mode) {
    case "alpha":
      return "\u03b1";
    case "beta-minus":
      return "\u03b2\u2212";
    case "beta-plus":
      return "\u03b2+";
    case "electron-capture":
      return "\u03b5/EC";
    case "isomeric-transition":
      return "IT";
    default:
      return mode;
  }
}

function formatHalfLife(s: number | null): string {
  if (s === null) return HALF_LIFE_INFINITY;
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
  if (s >= 1) return `${s < 100 ? s.toPrecision(3) : Math.round(s).toLocaleString()} s`;
  return `${s.toPrecision(2)} s`;
}

function halfLifeDisplay(step: StepInfo): string {
  if (!step.nuclide_in_database) return "??";
  if (step.nuclide_is_stable) return HALF_LIFE_INFINITY;
  if (step.nuclide_half_life_s != null) return formatHalfLife(step.nuclide_half_life_s);
  return "\u2014";
}

function makeLineItems(steps: StepInfo[], cursor: number, cursorActive: boolean): RenderItem[] {
  const items: RenderItem[] = [];
  for (const step of steps) {
    const isActive = cursorActive && step.index === cursor;

    switch (step.event_type) {
      case "start":
        items.push({ kind: "card", step, isActive });
        break;

      case "neutron-absorbed":
        items.push({ kind: "connector", label: "+ neutron" });
        items.push({ kind: "card", step, isActive });
        break;

      case "fission": {
        const n = step.detail?.neutrons_released;
        const e = step.detail?.energy === "fast" ? "fast n" : "n";
        const label = n ? `+ ${e} → fission +${n}n` : `+ ${e} → fission`;
        items.push({
          kind: "fission-split",
          step,
          followedIsHeavy: props.followingHeavy ?? true,
          label,
          isActive,
        });
        break;
      }

      case "decay": {
        const mode = step.detail?.decay_mode;
        items.push({
          kind: "connector",
          label: mode ? formatDecayMode(mode) : "decay",
        });
        items.push({ kind: "card", step, isActive });
        break;
      }

      case "stable":
        // The stable isotope was already shown by the preceding decay card;
        // skip to avoid a duplicate final card.
        break;
    }
  }
  return items;
}

const fissionIndex = computed(() => props.steps.findIndex((s) => s.event_type === "fission"));

const followedTail = computed(() => {
  const fi = fissionIndex.value;
  if (fi < 0) return [];
  return props.steps.slice(fi + 1);
});

const lightPreview = ref<StepInfo[]>([]);
const heavyPreview = ref<StepInfo[]>([]);

watch(
  () => [props.steps, props.session, fissionIndex.value] as const,
  () => {
    const fi = fissionIndex.value;
    const session = props.session as { decay_chain_preview?: (z: number, n: number) => StepInfo[] } | null;
    if (fi < 0 || !session?.decay_chain_preview) {
      lightPreview.value = [];
      heavyPreview.value = [];
      return;
    }
    const f = props.steps[fi];
    const l = f?.detail?.light_fragment;
    const h = f?.detail?.heavy_fragment;
    if (!l || !h) {
      lightPreview.value = [];
      heavyPreview.value = [];
      return;
    }
    try {
      lightPreview.value = session.decay_chain_preview(l.z, l.n);
      heavyPreview.value = session.decay_chain_preview(h.z, h.n);
    } catch {
      lightPreview.value = [];
      heavyPreview.value = [];
    }
  },
  { immediate: true, deep: true }
);

const fh = computed(() => props.followingHeavy ?? true);

const lightLegSteps = computed(() => (fh.value ? lightPreview.value : followedTail.value));
const heavyLegSteps = computed(() => (fh.value ? followedTail.value : heavyPreview.value));

const renderBlocks = computed((): RenderBlock[] => {
  const fi = fissionIndex.value;
  const cur = props.cursor;

  if (fi < 0) {
    return [{ kind: "segment", items: makeLineItems(props.steps, cur, true) }];
  }

  const fissionStep = props.steps[fi]!;
  const n = fissionStep.detail?.neutrons_released;
  const e = fissionStep.detail?.energy === "fast" ? "fast n" : "n";
  const label = n ? `+ ${e} → fission +${n}n` : `+ ${e} → fission`;

  return [
    { kind: "segment", items: makeLineItems(props.steps.slice(0, fi), cur, true) },
    {
      kind: "fission",
      step: fissionStep,
      followedIsHeavy: fh.value,
      label,
      isActive: fissionStep.index === cur,
    },
    {
      kind: "parallel",
      lightItems: makeLineItems(lightLegSteps.value, cur, !fh.value),
      heavyItems: makeLineItems(heavyLegSteps.value, cur, fh.value),
    },
  ];
});

function onSegmentCardClick(step: StepInfo) {
  emit("go-to-step", step.index);
}

function onLegCardClick(leg: "light" | "heavy", step: StepInfo) {
  const fi = fissionIndex.value;
  if (fi < 0) return;
  const preview =
    (leg === "light" && fh.value) || (leg === "heavy" && !fh.value);
  if (preview) {
    emit("go-to-branch-step", leg, fi, step.index);
  } else {
    emit("go-to-step", step.index);
  }
}

function onFissionFragClick(stepIndex: number) {
  emit("go-to-step", stepIndex);
}
</script>

<template>
  <div class="card-chain">
    <template v-for="(block, bi) in renderBlocks" :key="bi">
      <!-- Linear segment (prefix or full chain when no split) -->
      <template v-if="block.kind === 'segment'">
        <template v-for="(item, idx) in block.items" :key="'s-' + bi + '-' + idx">
          <div
            v-if="item.kind === 'card'"
            class="iso-card"
            :class="{
              active: item.isActive,
              stable: item.step.nuclide_is_stable,
              unknown: !item.step.nuclide_in_database,
            }"
            @click="onSegmentCardClick(item.step)"
          >
            <div class="card-notation">{{ item.step.nuclide.notation }}</div>
            <div class="card-hl">{{ halfLifeDisplay(item.step) }}</div>
          </div>

          <div v-else-if="item.kind === 'connector'" class="connector">
            <div class="connector-stem"></div>
            <div class="connector-label" v-if="item.label">{{ item.label }}</div>
            <div class="connector-arrow">&#8595;</div>
          </div>

          <!-- Nested fission in linear segment (rare); keep old single-column fan -->
          <div v-else-if="item.kind === 'fission-split'" class="fission-split fission-split-inline">
            <div class="fan-head">
              <div class="fan-stem"></div>
              <div class="fan-label">{{ item.label }}</div>
              <div class="fan-arrows">
                <span class="fan-arrow-left">&#8601;</span>
                <span class="fan-arrow-right">&#8600;</span>
              </div>
            </div>
            <div class="frag-row">
              <div
                class="frag-card"
                :class="{
                  followed: !item.followedIsHeavy,
                  unfollowed: item.followedIsHeavy,
                  active: item.isActive && !item.followedIsHeavy,
                }"
                @click="onFissionFragClick(item.step.index)"
              >
                <div class="frag-tag">light</div>
                <div class="card-notation">{{ item.step.detail?.light_fragment?.notation }}</div>
              </div>
              <div
                class="frag-card"
                :class="{
                  followed: item.followedIsHeavy,
                  unfollowed: !item.followedIsHeavy,
                  active: item.isActive && item.followedIsHeavy,
                }"
                @click="onFissionFragClick(item.step.index)"
              >
                <div class="frag-tag">heavy</div>
                <div class="card-notation">{{ item.step.detail?.heavy_fragment?.notation }}</div>
              </div>
            </div>
            <div class="frag-continuation" :class="{ 'is-heavy': item.followedIsHeavy }">
              <div class="continuation-stem"></div>
            </div>
          </div>
        </template>
      </template>

      <!-- Standalone fission row before parallel legs -->
      <div v-else-if="block.kind === 'fission'" class="fission-split fission-split-parallel">
        <div class="fan-head">
          <div class="fan-stem"></div>
          <div class="fan-label">{{ block.label }}</div>
          <div class="fan-arrows">
            <span class="fan-arrow-left">&#8601;</span>
            <span class="fan-arrow-right">&#8600;</span>
          </div>
        </div>
        <div class="frag-row">
          <div
            class="frag-card"
            :class="{
              followed: !block.followedIsHeavy,
              unfollowed: block.followedIsHeavy,
              active: block.isActive && !block.followedIsHeavy,
            }"
            @click="onFissionFragClick(block.step.index)"
          >
            <div class="frag-tag">light</div>
            <div class="card-notation">{{ block.step.detail?.light_fragment?.notation }}</div>
          </div>
          <div
            class="frag-card"
            :class="{
              followed: block.followedIsHeavy,
              unfollowed: !block.followedIsHeavy,
              active: block.isActive && block.followedIsHeavy,
            }"
            @click="onFissionFragClick(block.step.index)"
          >
            <div class="frag-tag">heavy</div>
            <div class="card-notation">{{ block.step.detail?.heavy_fragment?.notation }}</div>
          </div>
        </div>
      </div>

      <!-- Two decay columns after fission -->
      <div v-else-if="block.kind === 'parallel'" class="parallel-columns">
        <div class="parallel-column" :class="{ dimmed: fh }">
          <div class="column-label">Light</div>
          <div class="column-stem"></div>
          <template v-for="(item, idx) in block.lightItems" :key="'L-' + idx">
            <div
              v-if="item.kind === 'card'"
              class="iso-card"
              :class="{
                active: item.isActive,
                stable: item.step.nuclide_is_stable,
              }"
              @click="onLegCardClick('light', item.step)"
            >
              <div class="card-notation">{{ item.step.nuclide.notation }}</div>
              <div class="card-hl">{{ halfLifeDisplay(item.step) }}</div>
            </div>
            <div v-else-if="item.kind === 'connector'" class="connector">
              <div class="connector-stem"></div>
              <div class="connector-label" v-if="item.label">{{ item.label }}</div>
              <div class="connector-arrow">&#8595;</div>
            </div>
          </template>
        </div>

        <div class="parallel-column" :class="{ dimmed: !fh }">
          <div class="column-label">Heavy</div>
          <div class="column-stem"></div>
          <template v-for="(item, idx) in block.heavyItems" :key="'H-' + idx">
            <div
              v-if="item.kind === 'card'"
              class="iso-card"
              :class="{
                active: item.isActive,
                stable: item.step.nuclide_is_stable,
              }"
              @click="onLegCardClick('heavy', item.step)"
            >
              <div class="card-notation">{{ item.step.nuclide.notation }}</div>
              <div class="card-hl">{{ halfLifeDisplay(item.step) }}</div>
            </div>
            <div v-else-if="item.kind === 'connector'" class="connector">
              <div class="connector-stem"></div>
              <div class="connector-label" v-if="item.label">{{ item.label }}</div>
              <div class="connector-arrow">&#8595;</div>
            </div>
          </template>
        </div>
      </div>
    </template>
  </div>
</template>

<style scoped>
/* ── Container ── */
.card-chain {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 1.25rem 1rem;
  gap: 0;
  width: 100%;
  max-width: 52rem;
  margin: 0 auto;
}

/* ── Isotope card ── */
.iso-card {
  width: 9rem;
  padding: 0.5rem 0.75rem;
  background: #21262d;
  border: 1px solid #30363d;
  border-radius: 8px;
  text-align: center;
  cursor: pointer;
  transition: border-color 0.12s, background 0.12s;
}
.iso-card:hover {
  border-color: #58a6ff;
  background: #1c2128;
}
.iso-card.active {
  border-color: #58a6ff;
  background: #1f6feb22;
}
.iso-card.stable {
  border-color: #3fb950;
}
.iso-card.stable.active {
  background: #3fb95015;
}
.iso-card.unknown {
  border-color: #d29922;
  border-style: dashed;
  opacity: 0.7;
}
.iso-card.unknown .card-hl {
  color: #d29922;
}

.card-notation {
  font-size: 1rem;
  font-weight: 700;
  letter-spacing: 0.02em;
}
.card-hl {
  font-size: 0.72rem;
  color: #6e7681;
  margin-top: 0.2rem;
  font-variant-numeric: tabular-nums;
}
.iso-card.stable .card-hl {
  color: #3fb95080;
}

/* ── Straight connector ── */
.connector {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0;
}
.connector-stem {
  width: 1px;
  height: 10px;
  background: #30363d;
}
.connector-label {
  font-size: 0.72rem;
  color: #8b949e;
  line-height: 1.6;
  padding: 0 0.3rem;
  background: #0f1419;
}
.connector-arrow {
  color: #484f58;
  font-size: 0.9rem;
  line-height: 1;
}

/* ── Fission split ── */
.fission-split {
  display: flex;
  flex-direction: column;
  align-items: center;
}

.fission-split-parallel {
  margin-bottom: 0.25rem;
}

.fan-head {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0;
}
.fan-stem {
  width: 1px;
  height: 10px;
  background: #30363d;
}
.fan-label {
  font-size: 0.72rem;
  color: #f0883e;
  padding: 0 0.3rem;
  background: #0f1419;
  line-height: 1.6;
}
.fan-arrows {
  display: flex;
  gap: 7rem;
  line-height: 1;
  margin-top: 2px;
}
.fan-arrow-left,
.fan-arrow-right {
  font-size: 1.1rem;
  color: #f0883e;
}

.frag-row {
  display: flex;
  gap: 1rem;
  align-items: flex-start;
}

.frag-card {
  width: 7.5rem;
  padding: 0.4rem 0.6rem;
  background: #21262d;
  border: 1px solid #30363d;
  border-radius: 8px;
  text-align: center;
  cursor: pointer;
  transition: border-color 0.12s, background 0.12s;
}
.frag-card:hover {
  border-color: #58a6ff;
}
.frag-card.followed {
  border-color: #f0883e;
  background: #f0883e0e;
}
.frag-card.unfollowed {
  opacity: 0.55;
}
.frag-card.active {
  outline: 1px solid #f0883e;
}

.frag-tag {
  font-size: 0.6rem;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  color: #484f58;
  margin-bottom: 0.15rem;
}

.frag-continuation {
  width: 100%;
  display: flex;
  justify-content: center;
}
.frag-continuation.is-heavy {
  padding-left: calc(7.5rem + 1rem);
}
.frag-continuation:not(.is-heavy) {
  padding-right: calc(7.5rem + 1rem);
}
.continuation-stem {
  width: 1px;
  height: 10px;
  background: #30363d;
}

/* ── Parallel columns after fission ── */
.parallel-columns {
  display: flex;
  flex-direction: row;
  justify-content: center;
  align-items: flex-start;
  gap: 1.5rem;
  width: 100%;
  padding: 0 0.5rem 1rem;
}

.parallel-column {
  display: flex;
  flex-direction: column;
  align-items: center;
  flex: 1;
  min-width: 0;
  max-width: 12rem;
  transition: opacity 0.15s;
}

.parallel-column.dimmed {
  opacity: 0.45;
}

.parallel-column:not(.dimmed) {
  opacity: 1;
}

.column-label {
  font-size: 0.65rem;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: #6e7681;
  margin-bottom: 0.2rem;
}

.column-stem {
  width: 1px;
  height: 12px;
  background: #30363d;
  margin-bottom: 2px;
}
</style>
