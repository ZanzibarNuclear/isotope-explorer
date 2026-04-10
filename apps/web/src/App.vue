<script setup lang="ts">
import { onMounted, ref, computed } from "vue";
import type { SimState, StepInfo } from "@wasm/nuclear_sim_wasm.js";
import PeriodicTablePicker from "./components/PeriodicTablePicker.vue";
import QuickPickList from "./components/QuickPickList.vue";
import ChainView from "./components/ChainView.vue";
import CardChainView from "./components/CardChainView.vue";

type PickerView = "table" | "quick";
const pickerView = ref<PickerView>("quick");

type ChainViewMode = "list" | "cards";
const chainViewMode = ref<ChainViewMode>("cards");

const wasmVersion = ref("...");
const wasmError = ref<string | null>(null);
const session = ref<any>(null);
const simState = ref<SimState | null>(null);
const allSteps = ref<StepInfo[]>([]);

const canStepBack = computed(() => simState.value && simState.value.cursor > 0);
const canStepForward = computed(
  () => simState.value && simState.value.cursor < simState.value.step_count - 1
);
const canFire = computed(() => simState.value?.can_fire ?? false);
const canDecay = computed(() => simState.value?.can_decay ?? false);

function refreshState() {
  if (!session.value) return;
  simState.value = session.value.state();
  allSteps.value = session.value.all_steps();
}

function onSelectIsotope(z: number, n: number) {
  if (!session.value) return;
  try {
    session.value.set_isotope(z, n);
    refreshState();
  } catch (e) {
    wasmError.value = e instanceof Error ? e.message : String(e);
  }
}

function induceDecay() {
  if (!session.value) return;
  try {
    session.value.induce_decay();
    refreshState();
  } catch (e) {
    wasmError.value = e instanceof Error ? e.message : String(e);
  }
}

function fireNeutron(energy: "slow" | "fast") {
  if (!session.value) return;
  try {
    session.value.fire_neutron(energy);
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
    session.value.switch_branch(fragment);
    refreshState();
  } catch (e) {
    wasmError.value = e instanceof Error ? e.message : String(e);
  }
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
      <p class="subtitle">Fire neutrons at nuclei and see what happens</p>
    </header>

    <!-- Isotope picker (full width) -->
    <section v-if="session" class="picker-area">
      <div class="picker-toggle">
        <button
          class="toggle-btn"
          :class="{ active: pickerView === 'table' }"
          @click="pickerView = 'table'"
        >
          Periodic Table
        </button>
        <button
          class="toggle-btn"
          :class="{ active: pickerView === 'quick' }"
          @click="pickerView = 'quick'"
        >
          Quick Pick
        </button>
      </div>
      <PeriodicTablePicker
        v-if="pickerView === 'table'"
        :session="session"
        @select-isotope="onSelectIsotope"
      />
      <QuickPickList
        v-else
        :session="session"
        @select-isotope="onSelectIsotope"
      />
    </section>

    <main class="main">
      <!-- Left: chain visualization -->
      <section class="viewport" aria-label="Reaction chain">
        <div class="chain-view-toggle" v-if="simState">
          <button class="chain-toggle-btn" :class="{ active: chainViewMode === 'list' }" @click="chainViewMode = 'list'">List</button>
          <button class="chain-toggle-btn" :class="{ active: chainViewMode === 'cards' }" @click="chainViewMode = 'cards'">Cards</button>
        </div>
        <div v-if="!simState" class="viewport-placeholder">
          Choose an isotope to begin.
        </div>
        <ChainView
          v-else-if="chainViewMode === 'list'"
          :steps="allSteps"
          :cursor="simState.cursor"
          @go-to-step="goToStep"
        />
        <CardChainView
          v-else
          :session="session"
          :steps="allSteps"
          :cursor="simState.cursor"
          :following-heavy="simState.following_heavy"
          @go-to-step="goToStep"
          @go-to-branch-step="onGoToBranchStep"
        />
      </section>

      <!-- Right: controls and details -->
      <aside class="panel" aria-label="Controls">
        <!-- Error display -->
        <p v-if="wasmError" class="error">{{ wasmError }}</p>

        <!-- Action controls -->
        <div class="section" v-if="simState">
          <h2>Actions</h2>
          <div class="action-btns">
            <button class="fire-btn slow" :disabled="!canFire" @click="fireNeutron('slow')">
              Slow Neutron
            </button>
            <button class="fire-btn fast" :disabled="!canFire" @click="fireNeutron('fast')">
              Fast Neutron
            </button>
            <button class="fire-btn decay" :disabled="!canDecay" @click="induceDecay">
              Induce Decay
            </button>
          </div>
        </div>

        <!-- Step navigator -->
        <div class="section" v-if="simState && simState.step_count > 1">
          <h2>Navigate</h2>
          <div class="nav-row">
            <button class="nav-btn" :disabled="!canStepBack" @click="stepBack">&larr; Back</button>
            <span class="step-counter">
              Step {{ simState.cursor + 1 }} of {{ simState.step_count }}
            </span>
            <button class="nav-btn" :disabled="!canStepForward" @click="stepForward">
              Forward &rarr;
            </button>
          </div>
        </div>

        <!-- Fission branch selector -->
        <div class="section" v-if="simState?.has_fission_branch">
          <h2>Fission Fragment</h2>
          <div class="branch-btns">
            <button
              class="branch-btn"
              :class="{ selected: !simState.following_heavy }"
              @click="switchBranch('light')"
            >
              Light fragment
            </button>
            <button
              class="branch-btn"
              :class="{ selected: simState.following_heavy }"
              @click="switchBranch('heavy')"
            >
              Heavy fragment
            </button>
          </div>
        </div>

        <!-- Current step detail -->
        <div class="section" v-if="simState">
          <h2>Current Step</h2>
          <div class="detail-card" :class="{ unknown: !simState.current_step.nuclide_in_database }">
            <div class="detail-nuclide">{{ simState.current_step.nuclide.notation }}</div>
            <div v-if="!simState.current_step.nuclide_in_database" class="detail-unknown">?? No data available for this nuclide</div>
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
    </main>
  </div>
</template>

<style scoped>
.app {
  min-height: 100vh;
  display: flex;
  flex-direction: column;
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
  .main { grid-template-columns: 1fr; }
}

/* -- Viewport / chain -- */
.viewport {
  padding: 1.5rem;
  overflow-y: auto;
}
.viewport-placeholder {
  height: 100%;
  min-height: 240px;
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
  gap: 4px;
  padding: 0.5rem 0.75rem 0;
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

/* -- Panel -- */
.panel {
  padding: 1rem 1.25rem;
  border-left: 1px solid #30363d;
  background: #161b22;
  overflow-y: auto;
}
@media (max-width: 720px) {
  .panel { border-left: none; border-top: 1px solid #30363d; }
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

/* -- Picker area -- */
.picker-area {
  border-bottom: 1px solid #30363d;
}

.picker-toggle {
  display: flex;
  gap: 2px;
  padding: 0.5rem 1.25rem 0;
  background: #0d1117;
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
}

/* -- Action buttons -- */
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

/* -- Navigation -- */
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

/* -- Branch selector -- */
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

/* -- Detail card -- */
.detail-card {
  background: #0d1117;
  border: 1px solid #30363d;
  border-radius: 8px;
  padding: 0.75rem;
}
.detail-nuclide {
  font-size: 1.4rem;
  font-weight: 700;
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

/* -- Unknown nuclide indicator -- */
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
