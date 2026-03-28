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

**Path B -- Direct database access (future)**

5. **IAEA LiveChart API**
   - Use the LiveChart Data Download API to pull CSV data directly
   - Covers 4,000+ isotopes: decay modes, branching ratios, half-lives,
     nuclear structure
   - Can replace or supplement mendeleev + radioactivedecay extraction
   - Advantage: single authoritative source, no Python library dependency

6. **Rust ENSDF parser**
   - ENSDF is a standard text-based format for nuclear structure data
   - Write a Rust parser to load local ENSDF data files directly
   - Allows offline operation with no API or Python dependency
   - Could become a standalone Rust crate for the community

7. **NIST data for precision masses**
   - Pull atomic weights and isotopic compositions from NIST
   - Use as the authoritative source for stable isotope starting points

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

## Open Questions

These will be resolved as we go:

- **Data source strategy**: Start with Python library extraction (Path A) for
  speed. Evaluate whether the IAEA LiveChart API or a Rust ENSDF parser
  (Path B) should replace it. The IAEA API may be the sweet spot -- single
  source, CSV output, no Python dependency, 4,000+ nuclides.
- **ENDF extraction complexity**: The ENDF/B format is notoriously gnarly.
  OpenMC has parsers, but we may want to use its Python API rather than
  parsing raw ENDF files. The NNDC and NEA/JANIS tools can also export
  cross-section data in simpler formats. Explore before committing.
- **Rust ENSDF parser scope**: Building a full ENSDF parser is a significant
  effort but could become a useful standalone crate. Decide whether this is
  worth pursuing early or should wait until Path A proves limiting.
- **Dataset size vs. WASM binary size**: Embedding data for 500+ nuclides
  with cross-sections could add meaningful size to the WASM binary. May need
  to benchmark and decide on a compression or lazy-loading strategy.
- **Fission product model**: Use the full ENDF yield distribution
  (probabilistic) or simplify to the top few product pairs per fissile
  isotope? Start simple, expand if needed.
- **Visualization tech**: Canvas 2D, WebGL, or a library like PixiJS /
  Three.js?
- **State management**: Vue reactive state sufficient, or use Pinia for more
  structure?
- **Decay chain depth**: How many steps do we follow before cutting off?
  Some chains are very long.
- **Branching UX**: How to present fission branches intuitively? Tree view?
  Tabs? Sequential with "follow fragment A / follow fragment B" choice?

## Suggested First Steps

1. Build the Python extraction script for mendeleev (isotope properties)
2. Add radioactivedecay extraction (decay modes, branching, daughters)
3. Explore OpenMC / ENDF data access for cross-sections and fission yields
4. Merge into a single JSON dataset, validate decay chain completeness
5. Implement `NuclideData` struct in Rust, load the extracted dataset
6. Implement decay logic (alpha, beta-minus) and test with known chains
7. Implement neutron absorption -> fission for U-235 + thermal neutron
8. Build the step history model with forward/backward navigation
9. Expose the simulation session via WASM
10. Wire up the Vue UI: isotope picker, fire button, step navigator
11. Add basic canvas visualization
