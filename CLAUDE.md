# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

### Development

```bash
# Build WASM and start Vue dev server (primary dev command)
npm run dev

# Build WASM only (required before running the web app standalone)
npm run wasm

# Production build
npm run build

# Preview production build
npm run preview
```

The `wasm` step compiles `crates/nuclear-sim-wasm` with wasm-pack and outputs to `apps/web/src/wasm-pkg/`. This must run before the Vue dev server can resolve the `@wasm` alias.

### Rust

```bash
# Run Rust tests (core simulation logic)
cargo test

# Test a specific module
cargo test -p nuclear-sim

# Check Rust code without building WASM
cargo check
```

## Architecture

This is a **Rust + WASM core** with a **Vue 3 TypeScript frontend**.

```
crates/
  nuclear-sim/          # Pure Rust simulation engine — no web deps, fully testable
  nuclear-sim-wasm/     # Thin wasm-bindgen bridge exposing SimSession to JS
apps/web/               # Vue 3 + Vite frontend
docs/                   # design.md (architecture), plan.md (phased roadmap)
```

### Rust Layer (`crates/nuclear-sim`)

Core domain types: `Nuclide` (Z, N), `Nucleus`, `NuclideData` (decay modes, stability), `Simulation` (step history + cursor). The simulation records every transition (absorption, decay, fission) as a `SimEvent` and supports forward/backward navigation through a step history including fission branches.

### WASM Bridge (`crates/nuclear-sim-wasm`)

Exposes `SimSession` as a JS class. Key methods: `set_isotope()`, `fire_neutron()`, `step_forward()`, `step_backward()`, `go_to_step()`, `induce_decay()`, `switch_branch()`. Returns `SimState` and `StepInfo` structs serialized via wasm-bindgen.

### Vue Frontend (`apps/web/src`)

- `main.ts`: loads WASM (async `init()`), then mounts `App.vue`
- `App.vue`: single-file component managing all state via Vue 3 composition API (`ref`, `computed`) — no Pinia/Vuex
- Two-panel layout: left panel shows clickable step chain with fission branches; right panel has isotope selector, neutron fire controls, and step navigator

The `@wasm` path alias (configured in `vite.config.ts` and `tsconfig.json`) resolves to the wasm-pack output directory.

### Data Status

The simulation currently uses **stub/placeholder nuclear data**. Phase 1 of `docs/plan.md` covers extracting real data from mendeleev (isotope properties), radioactivedecay (ICRP-107 decay chains), and ENDF/B (neutron cross-sections + fission yields) into a Rust-friendly dataset.
