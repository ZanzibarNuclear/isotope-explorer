<script setup lang="ts">
import { onMounted, ref } from "vue";
import type { SimState, StepInfo } from "@wasm/nuclear_sim_wasm.js";
import PeriodicTablePicker from "./components/PeriodicTablePicker.vue";
import QuickPickList from "./components/QuickPickList.vue";
import ChainView from "./components/ChainView.vue";
import CardChainView from "./components/CardChainView.vue";
import NuclideGraphView from "./components/NuclideGraphView.vue";
import ToolPanel from "./components/ToolPanel.vue";

type PickerView = "table" | "quick";
const pickerView = ref<PickerView>("quick");
const pickerOpen = ref(true);

type ChainViewMode = "list" | "cards" | "graph";
const chainViewMode = ref<ChainViewMode>("graph");

const stepByStep = ref(true);

const wasmVersion = ref("...");
const wasmError = ref<string | null>(null);
const session = ref<any>(null);
const simState = ref<SimState | null>(null);
const allSteps = ref<StepInfo[]>([]);
const startingIsotope = ref<string | null>(null);

interface FissionTails {
  following_light: boolean;
  light: StepInfo[];
  heavy: StepInfo[];
}
const fissionTails = ref<FissionTails | undefined>(undefined);

function refreshState() {
  if (!session.value) return;
  simState.value = session.value.state();
  allSteps.value = session.value.all_steps();
  if (stepByStep.value && simState.value?.has_fission_branch) {
    fissionTails.value = session.value.fission_tails() ?? undefined;
  } else {
    fissionTails.value = undefined;
  }
}

function onSelectIsotope(z: number, n: number) {
  if (!session.value) return;
  try {
    session.value.set_isotope(z, n);
    refreshState();
    startingIsotope.value = simState.value?.current_step.nuclide.notation ?? null;
    pickerOpen.value = false;
  } catch (e) {
    wasmError.value = e instanceof Error ? e.message : String(e);
  }
}

function clearIsotope() {
  simState.value = null;
  allSteps.value = [];
  startingIsotope.value = null;
  fissionTails.value = undefined;
  pickerOpen.value = true;
}

function induceDecay() {
  if (!session.value) return;
  try {
    if (stepByStep.value) {
      session.value.induce_decay();
    } else {
      session.value.induce_decay_chain();
    }
    refreshState();
  } catch (e) {
    wasmError.value = e instanceof Error ? e.message : String(e);
  }
}

function fireNeutron(energy: "slow" | "fast") {
  if (!session.value) return;
  try {
    if (stepByStep.value) {
      session.value.fire_neutron_step(energy);
    } else {
      session.value.fire_neutron(energy);
    }
    refreshState();
  } catch (e) {
    wasmError.value = e instanceof Error ? e.message : String(e);
  }
}

function stepBack() {
  if (!session.value) return;
  try {
    session.value.step_back();
    refreshState();
  } catch (e) {
    /* at start */
  }
}

function stepForward() {
  if (!session.value) return;
  try {
    session.value.step_forward();
    refreshState();
  } catch (e) {
    /* at end */
  }
}

function goToStep(index: number) {
  if (!session.value) return;
  try {
    session.value.go_to_step(index);
    refreshState();
  } catch (e) {
    /* invalid */
  }
}

function switchBranch(fragment: "light" | "heavy") {
  if (!session.value) return;
  try {
    if (stepByStep.value) {
      session.value.switch_branch_step(fragment);
    } else {
      session.value.switch_branch(fragment);
    }
    refreshState();
  } catch (e) {
    wasmError.value = e instanceof Error ? e.message : String(e);
  }
}

function onSwitchFragment(leg: "light" | "heavy") {
  switchBranch(leg);
}

function onGoToBranchStep(leg: "light" | "heavy", fissionIndex: number, offset: number) {
  if (!session.value) return;
  try {
    const wantHeavy = leg === "heavy";
    if (simState.value && simState.value.following_heavy !== wantHeavy) {
      session.value.switch_branch(leg);
    }
    session.value.go_to_step(fissionIndex + 1 + offset);
    refreshState();
  } catch (e) {
    wasmError.value = e instanceof Error ? e.message : String(e);
  }
}

onMounted(async () => {
  try {
    const mod = await import("@wasm/nuclear_sim_wasm.js");
    wasmVersion.value = mod.sim_version();
    const s = new mod.SimSession();
    session.value = s;
  } catch (e) {
    wasmError.value = e instanceof Error ? e.message : String(e);
  }
});
</script>

<template>
  <div class="app">
    <header class="header">
      <h1>Isotope Explorer</h1>
      <p class="subtitle">Pick an isotope, fire neutrons, induce decay, and see what happens</p>
    </header>

    <!-- Isotope picker (full width) -->
    <section v-if="session" class="picker-area">
      <!-- Collapsed: show selected isotope + change button -->
      <div v-if="!pickerOpen && startingIsotope" class="picker-summary">
        <span class="picker-summary-label">Starting Isotope</span>
        <span class="picker-summary-sep">·</span>
        <span class="picker-summary-value">{{ startingIsotope }}</span>
        <button class="picker-clear-btn" @click="clearIsotope">Change</button>
      </div>

      <!-- Expanded: full picker UI -->
      <template v-else>
        <h2 class="picker-heading">Choose an Isotope</h2>
        <div class="picker-toggle">
          <div class="picker-tabs">
            <button class="toggle-btn" :class="{ active: pickerView === 'quick' }" @click="pickerView = 'quick'">
              Quick Pick
            </button>
            <button class="toggle-btn" :class="{ active: pickerView === 'table' }" @click="pickerView = 'table'">
              Periodic Table
            </button>
          </div>
          <div class="isotope-legend" aria-label="Isotope color legend">
            <span class="legend-item"><span class="legend-swatch stable"></span>Stable</span>
            <span class="legend-item"><span class="legend-swatch fissile"></span>Fissile</span>
            <span class="legend-item"><span class="legend-swatch radioactive"></span>Radioactive</span>
          </div>
        </div>
        <PeriodicTablePicker v-if="pickerView === 'table'" :session="session" @select-isotope="onSelectIsotope" />
        <QuickPickList v-else :session="session" @select-isotope="onSelectIsotope" />
      </template>
    </section>

    <main v-if="!pickerOpen" class="main">
      <!-- Left: chain visualization -->
      <section class="viewport" aria-label="Reaction chain">
        <div class="viewport-header">
          <h2 class="panel-title">Action Viewer</h2>
          <div class="chain-view-toggle" v-if="simState">
            <button class="chain-toggle-btn" :class="{ active: chainViewMode === 'graph' }"
              @click="chainViewMode = 'graph'">Graph</button>
            <button class="chain-toggle-btn" :class="{ active: chainViewMode === 'cards' }"
              @click="chainViewMode = 'cards'">Cards</button>
            <button class="chain-toggle-btn" :class="{ active: chainViewMode === 'list' }"
              @click="chainViewMode = 'list'">List</button>
          </div>
        </div>
        <div v-if="!simState" class="viewport-placeholder">
          Choose an isotope to begin.
        </div>
        <ChainView v-else-if="chainViewMode === 'list'" :steps="allSteps" :cursor="simState.cursor"
          @go-to-step="goToStep" />
        <NuclideGraphView v-else-if="chainViewMode === 'graph'" :steps="allSteps" :cursor="simState.cursor"
          @go-to-step="goToStep" />
        <CardChainView v-else :session="session" :steps="allSteps" :cursor="simState.cursor"
          :following-heavy="simState.following_heavy" :step-by-step="stepByStep" :fission-tails="fissionTails"
          @go-to-step="goToStep" @go-to-branch-step="onGoToBranchStep" @switch-fragment="onSwitchFragment" />
      </section>

      <ToolPanel
        v-model:step-by-step="stepByStep"
        :session-ready="Boolean(session)"
        :sim-state="simState"
        :wasm-error="wasmError"
        :wasm-version="wasmVersion"
        @fire-neutron="fireNeutron"
        @induce-decay="induceDecay"
        @step-back="stepBack"
        @step-forward="stepForward"
        @switch-branch="switchBranch"
      />
    </main>
  </div>
</template>

<style>
html,
body,
#app {
  margin: 0;
  width: 100%;
  height: 100%;
}
</style>

<style scoped>
.app {
  height: 100vh;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background: #0f1419;
  color: #e6edf3;
  font-family: system-ui, -apple-system, Segoe UI, Roboto, sans-serif;
}

.header {
  padding: 1rem 1.25rem;
  border-bottom: 1px solid #30363d;
}

.header h1 {
  margin: 0;
  font-size: 1.35rem;
  font-weight: 600;
}

.subtitle {
  margin: 0.35rem 0 0;
  font-size: 0.875rem;
  color: #8b949e;
}

.main {
  flex: 1;
  display: grid;
  grid-template-columns: 1fr minmax(280px, 340px);
  gap: 0;
  min-height: 0;
}

@media (max-width: 720px) {
  .main {
    grid-template-columns: 1fr;
  }
}

/* -- Panel title -- */
.panel-title {
  margin: 0 0 0.75rem;
  padding: 0.6rem 0 0;
  font-size: 0.8rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.07em;
  color: #6e7681;
}

/* -- Viewport / chain -- */
.viewport {
  padding: 0;
  min-width: 0;
  overflow-y: auto;
}

.viewport-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.5rem 0.5rem 0 1.25rem;
}

.viewport-placeholder {
  height: 100%;
  min-height: 240px;
  margin: 1.5rem;
  border-radius: 8px;
  border: 1px dashed #484f58;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #8b949e;
  font-size: 0.95rem;
}

/* -- Chain view toggle -- */
.chain-view-toggle {
  display: flex;
  justify-content: flex-end;
  gap: 4px;
  padding: 0.4rem 0.75rem;
}

.chain-toggle-btn {
  padding: 0.2rem 0.6rem;
  border: 1px solid #30363d;
  border-radius: 4px;
  background: transparent;
  color: #6e7681;
  font-size: 0.75rem;
  cursor: pointer;
  transition: color 0.12s, border-color 0.12s;
}

.chain-toggle-btn:hover {
  color: #e6edf3;
  border-color: #484f58;
}

.chain-toggle-btn.active {
  color: #e6edf3;
  border-color: #58a6ff;
  background: #1f6feb18;
}

/* -- Picker area -- */
.picker-area {
  border-bottom: 1px solid #30363d;
}

.picker-summary {
  display: flex;
  align-items: center;
  gap: 0.6rem;
  padding: 0.6rem 1.25rem;
  background: #0d1117;
  font-size: 0.875rem;
}

.picker-summary-label {
  color: #8b949e;
  font-weight: 500;
}

.picker-summary-sep {
  color: #484f58;
}

.picker-summary-value {
  font-weight: 700;
  color: #e6edf3;
  font-size: 1rem;
}

.picker-clear-btn {
  padding: 0.25rem 0.7rem;
  border: 1px solid #30363d;
  border-radius: 6px;
  background: #21262d;
  color: #8b949e;
  font-size: 0.8rem;
  cursor: pointer;
  transition: color 0.12s, border-color 0.12s;
}

.picker-clear-btn:hover {
  color: #e6edf3;
  border-color: #58a6ff;
}

.picker-heading {
  margin: 0;
  padding: 0.6rem 1.25rem 0.25rem;
  font-size: 0.95rem;
  font-weight: 600;
  color: #e6edf3;
  background: #0d1117;
}

.picker-toggle {
  display: flex;
  align-items: flex-end;
  justify-content: space-between;
  gap: 1rem;
  padding: 0.5rem 1.25rem 0;
  border-bottom: 1px solid #30363d;
  background: #0d1117;
}

.picker-tabs {
  display: flex;
  gap: 2px;
}

.toggle-btn {
  padding: 0.3rem 0.75rem;
  border: 1px solid #30363d;
  border-bottom: none;
  border-radius: 6px 6px 0 0;
  background: #161b22;
  color: #8b949e;
  font-size: 0.8rem;
  font-weight: 500;
  cursor: pointer;
  transition: background 0.12s, color 0.12s;
}

.toggle-btn:hover {
  background: #21262d;
  color: #e6edf3;
}

.toggle-btn.active {
  background: #21262d;
  color: #e6edf3;
  border-color: #30363d;
  border-bottom-color: #21262d;
  margin-bottom: -1px;
  padding-bottom: calc(0.3rem + 1px);
}

.isotope-legend {
  display: flex;
  align-items: center;
  gap: 0.9rem;
  padding-bottom: 0.35rem;
  color: #8b949e;
  font-size: 0.75rem;
}

.legend-item {
  display: inline-flex;
  align-items: center;
  gap: 0.3rem;
  white-space: nowrap;
}

.legend-swatch {
  width: 0.7rem;
  height: 0.7rem;
  border-radius: 2px;
  background: #8b949e;
}

.legend-swatch.stable {
  background: #3fb950;
}

.legend-swatch.fissile {
  background: #f0883e;
}

.legend-swatch.radioactive {
  background: #8b949e;
}

@media (max-width: 640px) {
  .picker-toggle {
    align-items: flex-start;
    flex-direction: column;
  }

  .isotope-legend {
    padding-bottom: 0;
  }
}

</style>
