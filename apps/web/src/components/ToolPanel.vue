<script setup lang="ts">
import { computed } from "vue";
import type { SimState } from "@wasm/nuclear_sim_wasm.js";
import { PERIODIC_TABLE } from "../data/periodic-table-layout";

const props = defineProps<{
  sessionReady: boolean;
  simState: SimState | null;
  stepByStep: boolean;
  wasmError: string | null;
  wasmVersion: string;
}>();

const emit = defineEmits<{
  (e: "update:stepByStep", value: boolean): void;
  (e: "fire-neutron", energy: "slow" | "fast"): void;
  (e: "induce-decay"): void;
  (e: "step-back"): void;
  (e: "step-forward"): void;
  (e: "switch-branch", fragment: "light" | "heavy"): void;
}>();

const ELEMENT_NAME_BY_Z = new Map(PERIODIC_TABLE.map(e => [e.z, e.name]));

const canStepBack = computed(() => props.simState && props.simState.cursor > 0);
const canStepForward = computed(
  () => props.simState && props.simState.cursor < props.simState.step_count - 1
);
const canFire = computed(() => props.simState?.can_fire ?? false);
const canDecay = computed(() => props.simState?.can_decay ?? false);

function elementName(z: number): string {
  return ELEMENT_NAME_BY_Z.get(z) ?? "Unknown";
}
</script>

<template>
  <aside class="panel" aria-label="Controls">
    <h2 class="panel-title">Controls</h2>
    <p v-if="wasmError" class="error">{{ wasmError }}</p>

    <div class="section" v-if="sessionReady">
      <h2>Mode</h2>
      <div class="mode-toggle">
        <button class="mode-btn" :class="{ active: stepByStep }" @click="emit('update:stepByStep', true)">
          Step-by-step
        </button>
        <button class="mode-btn" :class="{ active: !stepByStep }" @click="emit('update:stepByStep', false)">
          Auto-chain
        </button>
      </div>
    </div>

    <div class="section" v-if="simState">
      <h2>Actions</h2>
      <div class="action-btns">
        <button class="fire-btn slow" :disabled="!canFire" @click="emit('fire-neutron', 'slow')">
          Thermal
        </button>
        <button class="fire-btn fast" :disabled="!canFire" @click="emit('fire-neutron', 'fast')">
          Fast
        </button>
        <button class="fire-btn decay" :disabled="!canDecay" @click="emit('induce-decay')">
          Induce Decay
        </button>
      </div>
    </div>

    <div class="section" v-if="simState && simState.step_count > 1">
      <h2>Navigate</h2>
      <div class="nav-row">
        <button class="nav-btn" :disabled="!canStepBack" @click="emit('step-back')">&larr; Back</button>
        <span class="step-counter">
          Step {{ simState.cursor + 1 }} of {{ simState.step_count }}
        </span>
        <button class="nav-btn" :disabled="!canStepForward" @click="emit('step-forward')">
          Forward &rarr;
        </button>
      </div>
    </div>

    <div class="section" v-if="simState?.has_fission_branch">
      <h2>Fission Fragment</h2>
      <div class="branch-btns">
        <button class="branch-btn" :class="{ selected: simState.following_heavy }" @click="emit('switch-branch', 'heavy')">
          Heavy fragment
        </button>
        <button class="branch-btn" :class="{ selected: !simState.following_heavy }" @click="emit('switch-branch', 'light')">
          Light fragment
        </button>
      </div>
    </div>

    <div class="section" v-if="simState">
      <h2>Highlighted Isotope</h2>
      <div class="detail-card" :class="{ unknown: !simState.current_step.nuclide_in_database }">
        <div class="detail-nuclide">{{ simState.current_step.nuclide.notation }}</div>
        <div class="detail-element-name">{{ elementName(simState.current_step.nuclide.z) }}</div>
        <div v-if="!simState.current_step.nuclide_in_database" class="detail-unknown">
          ?? No data available for this nuclide
        </div>
        <div class="detail-desc">{{ simState.current_step.description }}</div>
        <dl class="detail-props">
          <dt>Type</dt>
          <dd>{{ simState.current_step.event_type }}</dd>
          <dt>Z / N / A</dt>
          <dd>
            {{ simState.current_step.nuclide.z }} /
            {{ simState.current_step.nuclide.n }} /
            {{ simState.current_step.nuclide.a }}
          </dd>
          <template v-if="simState.current_step.detail?.decay_mode">
            <dt>Decay mode</dt>
            <dd>{{ simState.current_step.detail.decay_mode }}</dd>
          </template>
          <template v-if="simState.current_step.detail?.neutrons_released">
            <dt>Free neutrons</dt>
            <dd>{{ simState.current_step.detail.neutrons_released }}</dd>
          </template>
        </dl>
      </div>
    </div>

    <p class="version">sim v{{ wasmVersion }}</p>
  </aside>
</template>

<style scoped>
.panel {
  align-self: start;
  justify-self: end;
  width: 100%;
  box-sizing: border-box;
  padding: 0 1.25rem 1rem;
  border-left: 1px solid #30363d;
  background: #161b22;
  overflow-y: auto;
}

@media (max-width: 720px) {
  .panel {
    border-left: none;
    border-top: 1px solid #30363d;
  }
}

.panel-title {
  margin: 0 0 0.75rem;
  padding: 0.6rem 0 0;
  font-size: 0.8rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.07em;
  color: #6e7681;
}

.section {
  margin-bottom: 1.25rem;
}

.section h2 {
  margin: 0 0 0.5rem;
  font-size: 0.85rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: #8b949e;
}

.action-btns {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
}

.fire-btn {
  flex: 1;
  min-width: 7rem;
  padding: 0.5rem;
  border: none;
  border-radius: 6px;
  font-size: 0.9rem;
  font-weight: 600;
  cursor: pointer;
  transition: opacity 0.15s;
}

.fire-btn:disabled {
  opacity: 0.35;
  cursor: not-allowed;
}

.fire-btn.slow {
  background: #1f6feb;
  color: #fff;
}

.fire-btn.slow:not(:disabled):hover {
  background: #388bfd;
}

.fire-btn.fast {
  background: #f0883e;
  color: #fff;
}

.fire-btn.fast:not(:disabled):hover {
  background: #f39c55;
}

.fire-btn.decay {
  background: #8957e5;
  color: #fff;
}

.fire-btn.decay:not(:disabled):hover {
  background: #a371f7;
}

.mode-toggle {
  display: flex;
  gap: 2px;
}

.mode-btn {
  flex: 1;
  padding: 0.3rem 0.5rem;
  border: 1px solid #30363d;
  border-radius: 6px;
  background: #0d1117;
  color: #6e7681;
  font-size: 0.8rem;
  font-weight: 500;
  cursor: pointer;
  transition: background 0.12s, color 0.12s, border-color 0.12s;
}

.mode-btn:hover {
  color: #e6edf3;
  border-color: #484f58;
}

.mode-btn.active {
  background: #1f6feb18;
  color: #58a6ff;
  border-color: #1f6feb;
}

.nav-row {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.nav-btn {
  padding: 0.35rem 0.7rem;
  border: 1px solid #30363d;
  border-radius: 6px;
  background: #21262d;
  color: #e6edf3;
  font-size: 0.85rem;
  cursor: pointer;
}

.nav-btn:disabled {
  opacity: 0.35;
  cursor: not-allowed;
}

.nav-btn:not(:disabled):hover {
  border-color: #58a6ff;
}

.step-counter {
  flex: 1;
  text-align: center;
  font-size: 0.85rem;
  color: #8b949e;
}

.branch-btns {
  display: flex;
  gap: 0.5rem;
}

.branch-btn {
  flex: 1;
  padding: 0.4rem;
  border: 1px solid #30363d;
  border-radius: 6px;
  background: #21262d;
  color: #e6edf3;
  font-size: 0.85rem;
  cursor: pointer;
}

.branch-btn.selected {
  border-color: #58a6ff;
  background: #1f6feb33;
}

.branch-btn:hover {
  border-color: #58a6ff;
}

.detail-card {
  background: #0d1117;
  border: 1px solid #30363d;
  border-radius: 8px;
  padding: 0.75rem;
}

.detail-nuclide {
  font-size: 1.4rem;
  font-weight: 700;
  margin-bottom: 0.1rem;
}

.detail-element-name {
  font-size: 0.85rem;
  color: #aaa;
  margin-bottom: 0.25rem;
}

.detail-desc {
  font-size: 0.85rem;
  color: #8b949e;
  margin-bottom: 0.6rem;
}

.detail-props {
  margin: 0;
  display: grid;
  grid-template-columns: auto 1fr;
  gap: 0.25rem 0.75rem;
  font-size: 0.825rem;
}

.detail-props dt {
  color: #8b949e;
}

.detail-props dd {
  margin: 0;
}

.detail-card.unknown {
  border-color: #d29922;
  border-style: dashed;
}

.detail-unknown {
  font-size: 0.85rem;
  font-weight: 600;
  color: #d29922;
  margin-bottom: 0.4rem;
}

.error {
  color: #f85149;
  font-size: 0.875rem;
  margin: 0 0 1rem;
}

.version {
  margin: 1.5rem 0 0;
  font-size: 0.75rem;
  color: #484f58;
}
</style>
