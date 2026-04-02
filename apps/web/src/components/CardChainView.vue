<script setup lang="ts">
import { computed } from "vue";
import type { StepInfo } from "@wasm/nuclear_sim_wasm.js";

const props = defineProps<{
  steps: StepInfo[];
  cursor: number;
  followingHeavy?: boolean;
}>();

const emit = defineEmits<{
  (e: "go-to-step", index: number): void;
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
  isFission: false;
}
interface FissionSplitItem {
  kind: "fission-split";
  step: StepInfo;               // the fission step itself
  followedIsHeavy: boolean;
  label: string;
  isActive: boolean;
}

type RenderItem = CardItem | ConnectorItem | FissionSplitItem;

// ── Formatters ───────────────────────────────────────────────────────────────

const HALF_LIFE_INFINITY = "\u221e";

function formatDecayMode(mode: string): string {
  switch (mode) {
    case "alpha":              return "\u03b1";
    case "beta-minus":         return "\u03b2\u2212";
    case "beta-plus":          return "\u03b2+";
    case "electron-capture":   return "\u03b5/EC";
    case "isomeric-transition":return "IT";
    default:                   return mode;
  }
}

function formatHalfLife(s: number | null): string {
  if (s === null) return HALF_LIFE_INFINITY;
  const min = 60, hr = 3600, day = 86400, yr = 365.25 * day;
  if (s >= yr * 1e9) return `${(s / (yr * 1e9)).toPrecision(3)} Gyr`;
  if (s >= yr * 1e6) return `${(s / (yr * 1e6)).toPrecision(3)} Myr`;
  if (s >= yr * 1e3) return `${(s / (yr * 1e3)).toPrecision(3)} kyr`;
  if (s >= yr)       return `${Math.round(s / yr).toLocaleString()} yr`;
  if (s >= day)      return `${(s / day).toFixed(1)} d`;
  if (s >= hr)       return `${(s / hr).toFixed(1)} h`;
  if (s >= min)      return `${(s / min).toFixed(1)} min`;
  if (s >= 1)        return `${s < 100 ? s.toPrecision(3) : Math.round(s).toLocaleString()} s`;
  return `${s.toPrecision(2)} s`;
}

function halfLifeDisplay(step: StepInfo): string {
  if (step.nuclide_is_stable) return HALF_LIFE_INFINITY;
  if (step.nuclide_half_life_s != null) return formatHalfLife(step.nuclide_half_life_s);
  return "\u2014";
}

// ── Render list ──────────────────────────────────────────────────────────────

const renderItems = computed((): RenderItem[] => {
  const items: RenderItem[] = [];
  const followedIsHeavy = props.followingHeavy ?? true;

  for (const step of props.steps) {
    const isActive = step.index === props.cursor;

    switch (step.event_type) {
      case "start":
        items.push({ kind: "card", step, isActive });
        break;

      case "neutron-absorbed":
        items.push({ kind: "connector", label: "+ neutron", isFission: false });
        items.push({ kind: "card", step, isActive });
        break;

      case "fission": {
        const n = step.detail?.neutrons_released;
        const label = n ? `fission +${n}n` : "fission";
        items.push({ kind: "fission-split", step, followedIsHeavy, label, isActive });
        break;
      }

      case "decay": {
        const mode = step.detail?.decay_mode;
        items.push({ kind: "connector", label: mode ? formatDecayMode(mode) : "decay", isFission: false });
        items.push({ kind: "card", step, isActive });
        break;
      }

      case "stable":
        items.push({ kind: "connector", label: "", isFission: false });
        items.push({ kind: "card", step, isActive });
        break;
    }
  }

  return items;
});
</script>

<template>
  <div class="card-chain">
    <template v-for="(item, idx) in renderItems" :key="idx">

      <!-- Isotope card -->
      <div
        v-if="item.kind === 'card'"
        class="iso-card"
        :class="{
          active: item.isActive,
          stable: item.step.nuclide_is_stable,
        }"
        @click="emit('go-to-step', item.step.index)"
      >
        <div class="card-notation">{{ item.step.nuclide.notation }}</div>
        <div class="card-hl">{{ halfLifeDisplay(item.step) }}</div>
      </div>

      <!-- Straight connector (decay / neutron / stable) -->
      <div v-else-if="item.kind === 'connector'" class="connector">
        <div class="connector-stem"></div>
        <div class="connector-label" v-if="item.label">{{ item.label }}</div>
        <div class="connector-arrow">&#8595;</div>
      </div>

      <!-- Fission split -->
      <div v-else-if="item.kind === 'fission-split'" class="fission-split">
        <!-- fan label + arrows -->
        <div class="fan-head">
          <div class="fan-stem"></div>
          <div class="fan-label">{{ item.label }}</div>
          <div class="fan-arrows">
            <span class="fan-arrow-left">&#8601;</span>
            <span class="fan-arrow-right">&#8600;</span>
          </div>
        </div>
        <!-- fragment cards side by side -->
        <div class="frag-row">
          <div
            class="frag-card"
            :class="{
              followed: !item.followedIsHeavy,
              unfollowed: item.followedIsHeavy,
              active: item.isActive && !item.followedIsHeavy,
            }"
            @click="emit('go-to-step', item.step.index)"
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
            @click="emit('go-to-step', item.step.index)"
          >
            <div class="frag-tag">heavy</div>
            <div class="card-notation">{{ item.step.detail?.heavy_fragment?.notation }}</div>
          </div>
        </div>
        <!-- continuation stem from the followed fragment down to the next connector -->
        <div class="frag-continuation" :class="{ 'is-heavy': item.followedIsHeavy }">
          <div class="continuation-stem"></div>
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
  opacity: 0.4;
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

/* Stem from the followed fragment card down to the next connector */
.frag-continuation {
  width: 100%;
  display: flex;
  /* shift center to align under heavy (right) or light (left) fragment */
  justify-content: center;
}
.frag-continuation.is-heavy {
  padding-left: calc(7.5rem + 1rem); /* card width + gap */
}
.frag-continuation:not(.is-heavy) {
  padding-right: calc(7.5rem + 1rem);
}
.continuation-stem {
  width: 1px;
  height: 10px;
  background: #30363d;
}
</style>
