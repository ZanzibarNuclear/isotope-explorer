<script setup lang="ts">
import { onMounted, ref } from "vue";

const wasmVersion = ref<string>("…");
const wasmError = ref<string | null>(null);

onMounted(async () => {
  try {
    const mod = await import("@wasm/nuclear_sim_wasm.js");
    wasmVersion.value = mod.sim_version();
  } catch (e) {
    wasmError.value = e instanceof Error ? e.message : String(e);
  }
});
</script>

<template>
  <div class="app">
    <header class="header">
      <h1>Isotopic Explorer</h1>
      <p class="subtitle">Neutrons, nucleus, energy — sim core: Rust / WASM · UI: Vue</p>
    </header>

    <main class="main">
      <section class="viewport" aria-label="Nucleus visualization">
        <div class="viewport-placeholder">
          Nucleus + neutron paths (canvas / WebGL) will live here.
        </div>
      </section>

      <aside class="panel" aria-label="Simulation details">
        <h2>Details</h2>
        <dl class="details">
          <dt>WASM sim</dt>
          <dd>
            <span v-if="wasmError" class="error">{{ wasmError }}</span>
            <span v-else>v{{ wasmVersion }}</span>
          </dd>
        </dl>
        <p class="hint">
          Run <code>npm run wasm</code> from the repo root if the module failed to load.
        </p>
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
  font-family:
    system-ui,
    -apple-system,
    Segoe UI,
    Roboto,
    sans-serif;
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
  grid-template-columns: 1fr minmax(260px, 320px);
  gap: 0;
  min-height: 0;
}

@media (max-width: 720px) {
  .main {
    grid-template-columns: 1fr;
  }
}

.viewport {
  padding: 1rem;
  min-height: 280px;
}

.viewport-placeholder {
  height: 100%;
  min-height: 240px;
  border-radius: 8px;
  border: 1px dashed #484f58;
  display: flex;
  align-items: center;
  justify-content: center;
  text-align: center;
  color: #8b949e;
  font-size: 0.9rem;
  padding: 1rem;
}

.panel {
  padding: 1rem 1.25rem;
  border-left: 1px solid #30363d;
  background: #161b22;
}

@media (max-width: 720px) {
  .panel {
    border-left: none;
    border-top: 1px solid #30363d;
  }
}

.panel h2 {
  margin: 0 0 0.75rem;
  font-size: 1rem;
  font-weight: 600;
}

.details {
  margin: 0;
  display: grid;
  grid-template-columns: auto 1fr;
  gap: 0.35rem 1rem;
  font-size: 0.875rem;
}

.details dt {
  color: #8b949e;
}

.details dd {
  margin: 0;
}

.error {
  color: #f85149;
}

.hint {
  margin: 1rem 0 0;
  font-size: 0.8rem;
  color: #6e7681;
}

.hint code {
  font-size: 0.85em;
  background: #21262d;
  padding: 0.15em 0.4em;
  border-radius: 4px;
}
</style>
