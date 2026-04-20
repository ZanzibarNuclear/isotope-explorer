# Isotope Explorer -- Implementation Plan

## What Exists Today

- Rust workspace with two crates: `nuclear-sim` (core) and `nuclear-sim-wasm`
  (WASM bridge)
- `Nuclide` struct: compositional model (Z, N) with validation and element
  symbol lookup. Supports notation like "U-235".
- `Nucleus` struct: thin wrapper around Nuclide, placeholder for dynamic state
- Element symbol table for Z = 1-118
- WASM bindings: `NuclideInfo` class exposed to JS, `sim_version()` function
- Vue 3 app with Vite: dark-themed layout with viewport placeholder (left) and
  details panel (right). Loads WASM, displays U-235 sample data.
- Build pipeline: `npm run dev` builds WASM then starts Vite dev server

## Implementation Phases

### Phase 1: Data Extraction Pipeline

Build tooling to pull nuclear data from authoritative sources and output a
Rust-friendly dataset. This is a prerequisite for everything else.

We have two paths (see design doc). Start with Path A (Python extraction) for
speed, migrate to Path B (direct parsing) as needed.

**Path A -- Python extraction (quick start)**

1. **Extract isotope properties from mendeleev**
   - Install mendeleev, query its SQLite DB or use its fetch API
   - Pull: Z, A, atomic mass, natural abundance, half-life, spin, parity,
     is_radioactive for all isotopes
   - Pull: element symbol, name, group, period, block for Z=1..118
   - Output as JSON (or generated Rust source)

2. **Extract decay data from radioactivedecay**
   - Install radioactivedecay, read its ICRP-107 dataset
   - For each nuclide: decay modes, branching fractions, daughter nuclides
   - Cross-reference half-lives with mendeleev data
   - Output as JSON keyed by (Z, A)

3. **Extract neutron data from OpenMC / ENDF**
   - Download ENDF/B cross-section library (or use OpenMC's data API)
   - For a curated set of isotopes: thermal and fast neutron absorption
     cross-sections, fission cross-sections
   - Extract fission product yield distributions for fissile isotopes
     (U-235, U-238, Pu-239, and others)
   - Output as JSON

4. **Merge and curate**
   - Combine the three outputs into a single dataset keyed by (Z, N)
   - Start with ~200-500 nuclides: actinides, fission products, common
     elements, and notable decay chain members
   - Validate: every daughter nuclide in a decay chain should also be in
     the dataset (or flagged as needing addition)
   - Generate a Rust source file or JSON blob for embedding in the binary

**Path B -- Direct authoritative sources (replaces Path A)**

Path B eliminates the mendeleev and radioactivedecay Python library
dependencies by fetching data directly from authoritative sources. The
extraction scripts still use Python (for HTTP requests and CSV/text
parsing) but have no specialized nuclear-data library dependencies.

5. **IAEA LiveChart API → decay data + nuclear structure**
   - GET `https://www-nds.iaea.org/relnsd/v1/data?fields=ground_states&nuclides=all`
   - Returns CSV with ~3,500 ground-state nuclides
   - Provides: Z, N, half-life (seconds), decay modes with branching
     fractions, spin, parity
   - Replaces both mendeleev (isotope properties) and radioactivedecay
     (ICRP-107 decay chains) with a single authoritative source
   - Script: `scripts/pathb/fetch_iaea_decays.py`

6. **NIST Atomic Weights (not needed)**
   - The IAEA LiveChart API already provides atomic mass (in micro-AMU)
     and natural abundance for 3,356 of 3,383 nuclides
   - No separate NIST fetch required; this fixes the `atomic_mass` null
     bug from Path A without an additional data source

7. **Curated ENDF/B-VIII.0 thermal cross-sections**
   - Expand the existing 17 hand-curated isotopes to ~80-100 using
     published ENDF/B-VIII.0 reference tables
   - Provides: sigma_capture, sigma_fission, sigma_elastic,
     sigma_absorption, nu_bar at thermal energy (0.0253 eV)
   - Coverage: actinides (~10), structural/moderator materials (~15),
     neutron poisons (~10), important fission products (~50+)
   - Full ENDF-6 cross-section parsing (MF=3) is not worth the
     complexity -- point values at thermal energy are sufficient for
     our simulation's fission-vs-capture decisions
   - Script: `scripts/pathb/build_thermal_xs.py`

8. **ENDF/B-VIII.0 fission yield sublibrary**
   - Download and parse the neutron-induced fission yield (nfy) files
     from NNDC for ~40 fissile parents
   - The fission yield sublibrary (MF=8, MT=454) uses a simple flat
     record format -- much simpler than full ENDF-6 cross-sections
   - For each parent: extract top 10-15 product pairs (covers >80%
     of total yield) at thermal and fast energies
   - Expands from current 4 fissile parents to all ~40 in ENDF/B-VIII.0
   - Script: `scripts/pathb/parse_fission_yields.py`

9. **Merge and validate**
   - Combine all four outputs into `crates/nuclear-sim/data/nuclide_data.json`
   - Include ALL ~3,500 nuclides (decay data, masses, stability)
   - Add cross-sections for ~80-100, fission yields for ~40 parents
   - Validate: decay chain completeness, branching fraction sums,
     fission mass conservation, regression against current 746 nuclides
   - Estimated size: ~750 KB JSON, ~150-200 KB gzipped in WASM binary
   - Script: `scripts/pathb/merge_pathb.py`

**Rust ENSDF parser (deferred)**

Building a Rust parser for ENSDF files could become a useful standalone
crate, but is not needed now. The IAEA LiveChart API provides the same
data (it's backed by ENSDF) in an easier-to-consume format. Revisit if
we need offline-only operation or sub-second data refresh.

### Phase 2: Nuclear Data and Simulation Engine (Rust)

The core work. Everything downstream depends on having real isotope data and
simulation logic in Rust.

1. **Isotope data tables**
   - Define a `NuclideData` struct: stability flag, half-life, decay modes
     with branching ratios, fissile/fissionable flags, atomic mass, abundance
   - Load the extracted dataset (compiled-in via `include!` or `include_str!`)
   - Provide a lookup: given (Z, N), return `Option<&NuclideData>`

2. **Decay simulation**
   - Model decay types: alpha, beta-minus, beta-plus, EC, IT (as an enum)
   - Given a nuclide + its data, pick a decay mode by branching ratio and
     compute the daughter nuclide
   - Handle decay chains: follow until reaching a stable isotope

3. **Fission simulation**
   - Given a fissile nuclide + neutron energy, pick a fragment pair from the
     ENDF yield distribution
   - Determine how many free neutrons are released
   - Decide fission vs. capture based on cross-section ratio at the given
     neutron energy

4. **Neutron absorption logic**
   - Given a nuclide and neutron energy (thermal or fast), determine what
     happens: stable, fission, or decay
   - Use extracted cross-section data to weight the outcome probabilities
   - Handle "unknown isotope" gracefully (not in dataset)

5. **Step history / event log**
   - Define event types: NeutronAbsorbed, Fission, Decay, Stable
   - Build a tree structure of events with navigation (parent, children)
   - Support forward/backward traversal and branch selection

### Phase 3: WASM API Expansion

Expose the simulation engine to JavaScript.

1. **Simulation session object**
   - WASM class that holds the current simulation state
   - Methods: `set_isotope(z, n)`, `fire_neutron(energy)`, `step_forward()`,
     `step_backward()`, `current_step()`, `select_branch(index)`

2. **Step/event data transfer**
   - WASM structs for each event type, readable from JS
   - Current nuclide state at each step
   - Branch information at fission points

3. **Data queries**
   - Look up isotope info: is it stable? what are its decay modes?
   - List available presets / interesting starting isotopes

### Phase 4: Vue UI -- Controls and Information

Build out the interactive UI in the panel.

1. **Isotope selector**
   - Element dropdown or periodic table picker
   - Mass number input or slider
   - Preset buttons (U-235, Pu-239, etc.)

2. **Neutron controls**
   - "Fire Slow Neutron" and "Fire Fast Neutron" buttons
   - Disabled state when simulation is mid-chain
   - Visual feedback on fire

3. **Step navigator**
   - Back / Forward buttons
   - Step counter ("Step 3 of 7")
   - Branch selector at fission points (dropdown or tabs)

4. **Current state display**
   - Nuclide notation, Z/N/A
   - What happened at this step (absorbed, decayed, fissioned)
   - Properties: stability, half-life, decay mode

5. **Chain summary**
   - Compact list or diagram of the full chain
   - Highlight current step
   - Show branches

### Phase 5: Viewport Visualization

The visual centerpiece. Can be developed incrementally.

1. **Basic nucleus rendering**
   - Canvas 2D: draw the nucleus as a styled circle with Z/N label
   - Animate neutron approach (small dot moving toward nucleus)
   - Animate absorption (neutron merges into nucleus, size/color shifts)

2. **Fission animation**
   - Nucleus wobbles, then splits into two smaller circles
   - Free neutrons fly off
   - Fragments settle into their positions

3. **Decay animation**
   - Alpha: small cluster (He-4) ejected from nucleus
   - Beta: small particle (electron/positron) emitted
   - Nucleus transitions to new state

4. **Polish and iteration**
   - Individual nucleon visualization (dots for protons/neutrons)
   - Color coding (protons vs. neutrons)
   - Particle trails, glow effects
   - Responsive sizing

### Phase 6: Polish and Extended Features

1. **Interesting presets and guided experiences**
   - "Try U-235 fission" walkthrough
   - Notable decay chains (Thorium series, Uranium series)
   - Quiz mode: guess what happens next

2. **Expanded isotope coverage**
   - More complete data tables
   - Handle edge cases (very short-lived isotopes, exotic decays)

3. **UI polish**
   - Tooltips, help text
   - Keyboard navigation (arrow keys for step forward/back)
   - Mobile-friendly layout adjustments
   - Loading states, error handling

## Resolved Decisions

- **Data source strategy**: Path B (direct authoritative sources) replaces
  Path A. IAEA LiveChart API for decay data, NIST for masses, curated
  ENDF/B-VIII.0 values for cross-sections, ENDF nfy sublibrary for fission
  yields. Python scripts remain for extraction but have no specialized
  nuclear-data library dependencies.
- **ENDF extraction complexity**: We do NOT parse full ENDF-6 cross-section
  files (MF=3). Thermal point values are curated from published reference
  tables (~80-100 isotopes). We DO parse the fission yield sublibrary
  (MF=8, MT=454) which has a much simpler flat-list format.
- **Rust ENSDF parser scope**: Deferred. The IAEA LiveChart API provides the
  same data in an easier format. Revisit only if offline operation is needed.
- **Dataset size vs. WASM binary size**: ~3,500 nuclides with all fields
  estimates to ~750 KB JSON, ~150-200 KB gzipped. Well within budget for a
  WASM binary. No compression or lazy-loading needed.
- **Fission product model**: Top 10-15 product pairs per fissile parent
  (covers >80% of yield). Probabilistic selection from these pairs.

## Open Questions

- **Visualization tech**: Canvas 2D, WebGL, or a library like PixiJS /
  Three.js?
- **State management**: Vue reactive state sufficient, or use Pinia for more
  structure?
- **Decay chain depth**: How many steps do we follow before cutting off?
  Some chains are very long.
- **Branching UX**: How to present fission branches intuitively? Tree view?
  Tabs? Sequential with "follow fragment A / follow fragment B" choice?
