<script setup lang="ts">
import { onMounted, ref, computed } from "vue";
import type { SimState, StepInfo } from "@wasm/nuclear_sim_wasm.js";
import PeriodicTablePicker from "./components/PeriodicTablePicker.vue";

const wasmVersion = ref("...");
const wasmError = ref<string | null>(null);
const session = ref<any>(null);
const simState = ref<SimState | null>(null);
const allSteps = ref<StepInfo[]>([]);

const canStepBack = computed(() => simState.value && simState.value.cursor > 0);
const canStepForward = computed(
  () => simState.value && simState.value.cursor < simState.value.step_count - 1
);
const canFire = computed(() => {
  if (!simState.value) return false;
  // Can fire if we're at the start (only Start event) or the chain is complete
  // and user is at the end
  const s = simState.value;
  return s.step_count === 1 && s.current_step.event_type === "start";
});

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

    <!-- Periodic table picker (full width) -->
    <section v-if="session" class="picker-area">
      <PeriodicTablePicker :session="session" @select-isotope="onSelectIsotope" />
    </section>

    <main class="main">
      <!-- Left: chain visualization -->
      <section class="viewport" aria-label="Reaction chain">
        <div v-if="!simState" class="viewport-placeholder">
          Choose an isotope to begin.
        </div>
        <div v-else class="chain">
          <div
            v-for="step in allSteps"
            :key="step.index"
            class="chain-step"
            :class="{
              active: step.index === simState.cursor,
              fission: step.event_type === 'fission',
              stable: step.event_type === 'stable',
            }"
            @click="goToStep(step.index)"
          >
            <span class="chain-icon" v-html="eventIcon(step.event_type)"></span>
            <span class="chain-label">{{ step.nuclide.notation }}</span>
            <span class="chain-type">{{ step.event_type }}</span>
          </div>
        </div>
      </section>

      <!-- Right: controls and details -->
      <aside class="panel" aria-label="Controls">
        <!-- Error display -->
        <p v-if="wasmError" class="error">{{ wasmError }}</p>

        <!-- Neutron controls -->
        <div class="section" v-if="simState">
          <h2>Fire Neutron</h2>
          <div class="neutron-btns">
            <button class="fire-btn slow" :disabled="!canFire" @click="fireNeutron('slow')">
              Slow Neutron
            </button>
            <button class="fire-btn fast" :disabled="!canFire" @click="fireNeutron('fast')">
              Fast Neutron
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
          <div class="detail-card">
            <div class="detail-nuclide">{{ simState.current_step.nuclide.notation }}</div>
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

.chain {
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.chain-step {
  display: flex;
  align-items: center;
  gap: 0.6rem;
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
  width: 1.2rem;
  text-align: center;
  color: #8b949e;
}
.chain-label {
  font-weight: 600;
  min-width: 4.5rem;
}
.chain-type {
  color: #8b949e;
  font-size: 0.8rem;
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

/* -- Fire buttons -- */
.neutron-btns {
  display: flex;
  gap: 0.5rem;
}
.fire-btn {
  flex: 1;
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
