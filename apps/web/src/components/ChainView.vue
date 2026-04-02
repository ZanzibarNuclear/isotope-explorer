<script setup lang="ts">
import type { StepInfo } from "@wasm/nuclear_sim_wasm.js";

const props = defineProps<{
  steps: StepInfo[];
  cursor: number;
}>();

const emit = defineEmits<{
  (e: "go-to-step", index: number): void;
}>();

function eventIcon(type: string): string {
  switch (type) {
    case "start": return "&#9679;";
    case "neutron-absorbed": return "&#10141;";
    case "fission": return "&#10038;";
    case "decay": return "&#8595;";
    case "stable": return "&#9632;";
    default: return "?";
  }
}

const HALF_LIFE_INFINITY = "\u221e";

function formatDecayMode(mode: string): string {
  switch (mode) {
    case "alpha": return "\u03b1";
    case "beta-minus": return "\u03b2\u2212";
    case "beta-plus": return "\u03b2+";
    case "electron-capture": return "\u03b5 / EC";
    case "isomeric-transition": return "IT";
    default: return mode;
  }
}

function formatHalfLife(seconds: number): string {
  const minute = 60;
  const hour = 3600;
  const day = 24 * hour;
  const year = 365.25 * day;
  if (seconds >= year * 1e9) return `${(seconds / (year * 1e9)).toPrecision(3)} Gyr`;
  if (seconds >= year * 1e6) return `${(seconds / (year * 1e6)).toPrecision(3)} Myr`;
  if (seconds >= year * 1e3) return `${(seconds / (year * 1e3)).toPrecision(3)} kyr`;
  if (seconds >= year) return `${Math.round(seconds / year).toLocaleString()} yr`;
  if (seconds >= day) return `${(seconds / day).toFixed(1)} d`;
  if (seconds >= hour) return `${(seconds / hour).toFixed(1)} h`;
  if (seconds >= minute) return `${(seconds / minute).toFixed(1)} min`;
  if (seconds >= 1) return `${seconds < 100 ? seconds.toPrecision(3) : Math.round(seconds).toLocaleString()} s`;
  return `${seconds.toPrecision(2)} s`;
}

function chainStepMeta(step: StepInfo): string {
  if (step.event_type === "decay" && step.detail?.decay_mode) {
    return formatDecayMode(step.detail.decay_mode);
  }
  if (step.nuclide_is_stable) return HALF_LIFE_INFINITY;
  const hl = step.nuclide_half_life_s;
  if (hl != null && Number.isFinite(hl)) return formatHalfLife(hl);
  return "\u2014";
}
</script>

<template>
  <div class="chain">
    <div
      v-for="step in steps"
      :key="step.index"
      class="chain-step"
      :class="{
        active: step.index === cursor,
        fission: step.event_type === 'fission',
        stable: step.event_type === 'stable',
      }"
      @click="emit('go-to-step', step.index)"
    >
      <span class="chain-icon" v-html="eventIcon(step.event_type)"></span>
      <span class="chain-label">{{ step.nuclide.notation }}</span>
      <span
        class="chain-meta"
        :class="step.event_type === 'decay' && step.detail?.decay_mode ? 'is-decay-mode' : 'is-half-life'"
        :title="
          step.event_type === 'decay' && step.detail?.decay_mode
            ? 'Decay mode'
            : 'Half-life (stable = ' + HALF_LIFE_INFINITY + ')'
        "
      >{{ chainStepMeta(step) }}</span>
      <span class="chain-type">{{ step.event_type }}</span>
    </div>
  </div>
</template>

<style scoped>
.chain {
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.chain-step {
  display: grid;
  grid-template-columns: 1.2rem minmax(4.5rem, auto) minmax(5rem, auto) 1fr;
  align-items: center;
  column-gap: 0.75rem;
  padding: 0.5rem 0.75rem;
  border-radius: 6px;
  cursor: pointer;
  transition: background 0.15s;
  font-size: 0.9rem;
}
.chain-step:hover {
  background: #1c2128;
}
.chain-step.active {
  background: #1f6feb33;
  outline: 1px solid #1f6feb;
}
.chain-step.fission {
  border-left: 3px solid #f0883e;
}
.chain-step.stable {
  border-left: 3px solid #3fb950;
}
.chain-icon {
  font-size: 0.85rem;
  text-align: center;
  color: #8b949e;
}
.chain-label {
  font-weight: 600;
}
.chain-type {
  color: #8b949e;
  font-size: 0.8rem;
}
.chain-meta {
  font-size: 0.75rem;
  font-variant-numeric: tabular-nums;
  white-space: nowrap;
}
.chain-meta.is-half-life {
  color: #6e7681;
}
.chain-meta.is-decay-mode {
  color: #d2a8ff;
  background: #a371f718;
  border: 1px solid #a371f730;
  border-radius: 3px;
  padding: 0 0.35rem;
  font-size: 0.78rem;
  line-height: 1.5;
}
.chain-step.stable .chain-meta.is-half-life {
  color: #3fb95099;
}
</style>
