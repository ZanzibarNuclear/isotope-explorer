# Isotope Explorer -- Design Document

## Concept

Isotope Explorer is an interactive web application that lets people throw
neutrons at atomic nuclei and watch what happens. Starting from any isotope the
user fires a neutron -- slow (thermal) or fast (energetic) -- into the nucleus.
The neutron is absorbed, producing a new isotope. We then simulate whatever
comes next: the nucleus may be stable, it may undergo radioactive decay, or it
may fission into lighter fragments. Each of those products may themselves be
unstable, so we follow the chain until every product has reached a stable
isotope. The user can step forward and backward through this progression to
study each stage at their own pace.

The goal is education and curiosity, not rigorous simulation. We use real
isotope data where practical and simplify where needed to keep the experience
fun and approachable.

## Core Loop

1. **Choose a starting isotope.** The user picks an element and isotope (e.g.
   U-235) or selects from presets.
2. **Fire a neutron.** The user chooses slow (thermal) or fast (energetic).
   The neutron is absorbed: (Z, N) -> (Z, N+1).
3. **Resolve the new isotope.** The resulting nucleus is checked:
   - **Stable** -- nothing further happens. Display the result.
   - **Fission** -- the nucleus splits into two (or more) lighter fragments
     plus free neutrons. Each fragment enters its own resolution chain.
   - **Decay** -- the nucleus undergoes radioactive decay (alpha, beta-minus,
     beta-plus, gamma, etc.) producing a new nuclide. Repeat from this step.
4. **Build a step history.** Every transition (absorption, fission split, decay
   event) is recorded as a step. The user can walk forward and backward through
   the sequence.
5. **Repeat.** The user can fire another neutron into any of the resulting
   stable products, or start fresh.

## Neutron Types

- **Slow (thermal) neutron** -- low kinetic energy (~0.025 eV). Favors
  absorption and fission in fissile isotopes (U-235, Pu-239).
- **Fast neutron** -- high kinetic energy (~1-2 MeV). Can cause fission in
  fissionable isotopes (U-238) and has different cross-section behavior.

The distinction affects which outcomes are possible and their probabilities.

## Outcomes After Neutron Absorption

After the target nucleus absorbs a neutron, several things can happen depending
on the isotope and neutron energy:

| Outcome            | Description                                       |
|--------------------|---------------------------------------------------|
| Stable             | New isotope is stable. Chain ends.                |
| Fission            | Nucleus splits into lighter fragments + neutrons. |
| Beta-minus decay   | Neutron -> proton + electron + antineutrino.      |
| Beta-plus decay    | Proton -> neutron + positron + neutrino.          |
| Alpha decay        | Emits He-4 nucleus (Z-2, N-2).                   |
| Gamma emission     | Excited nucleus emits a photon (no Z/N change).   |
| Neutron emission   | Nucleus ejects a neutron (back to original Z, N). |

For the first version we focus on: stable, fission, beta-minus, beta-plus, and
alpha decay. Gamma emission and neutron emission can be added later.

## Step History Model

The simulation produces a tree of events (fission branches), but we present it
as a linear walkable sequence with branch points.

```
Step 0: U-235 (starting isotope)
Step 1: U-235 + n -> U-236 (neutron absorbed)
Step 2: U-236 -> Ba-141 + Kr-92 + 3n (fission)
Step 3a: Ba-141 -> La-141 (beta-minus decay)
Step 3b: Kr-92 -> Kr-92 (stable -- chain ends)
Step 4a: La-141 -> Ce-141 (beta-minus decay)
...
```

The user sees one "current step" at a time and can navigate with
forward/backward controls. At branch points (fission) the user can choose
which fragment to follow.

## Data Requirements

We need nuclear data for each isotope we support:

- **Stability**: is this nuclide stable?
- **Decay modes**: what kind(s) of decay, branching ratios, half-lives
- **Fission data**: which isotopes are fissile/fissionable, typical fragment
  distributions
- **Cross sections** (simplified): likelihood of absorption vs. scattering for
  thermal and fast neutrons

## Data Sources

We need data from several domains: isotope identity/properties, decay chains,
and neutron interaction physics. Below we catalog the authoritative primary
databases, the open-source libraries that wrap them, and our strategy for
bringing the data into Rust.

### Primary databases (authoritative upstream sources)

#### IAEA LiveChart of Nuclides

The International Atomic Energy Agency's LiveChart is the most comprehensive
and user-friendly source for nuclear data. Covers 4,000+ isotopes including
nuclear structure, decay properties, and radiation types.

**Key for us:**
- Decay modes, branching ratios, half-lives, daughter products
- Nuclear structure (spin, parity, excitation states)
- The LiveChart Data Download API allows programmatic CSV retrieval -- ideal
  for building our extraction pipeline

URL: https://www-nds.iaea.org/relnsd/vcharthtml/VChartHTML.html
API: https://www-nds.iaea.org/relnsd/v1/data

#### NIST Atomic Weights and Isotopic Compositions

The National Institute of Standards and Technology is the authoritative U.S.
source for precise atomic weights and isotopic compositions of stable isotopes.

**Key for us:**
- Standardized relative atomic masses (defining the "ground state" starting
  point of our model)
- Natural isotopic abundances

URL: https://www.nist.gov/pml/atomic-weights-and-isotopic-compositions

#### NNDC / Brookhaven National Laboratory

The National Nuclear Data Center hosts several deep-level databases used by
professional nuclear physicists:

- **ENSDF (Evaluated Nuclear Structure Data File)**: The principal source for
  nuclear structure and decay data. Standard text-based format -- we could
  write a Rust parser for ENSDF files, allowing the app to load local data
  without API calls.
- **NuDat 3**: Web-based tool for exploring ENSDF data. Allows CSV exports of
  ground and isomeric state information.
- **ENDF (Evaluated Nuclear Data File)**: Crucial for neutron-induced fission
  and capture -- contains cross-section data (the probability of a reaction
  occurring at a given neutron energy).

URL: https://www.nndc.bnl.gov/

#### OECD Nuclear Energy Agency (NEA)

The NEA provides JANIS, a tool for visualizing and manipulating nuclear data
from multiple international libraries (ENDF/B, JEFF, JENDL, TENDL). Useful as
a cross-reference and for exploring data before committing to a specific
library.

URL: https://www.oecd-nea.org/jcms/pl_39910/janis

### Open-source libraries (convenient wrappers)

These Python/C++ projects wrap the primary databases above and provide
convenient programmatic access. We can use them as extraction tools (read their
data, output Rust-friendly formats) without porting the libraries themselves.

#### mendeleev (Python, MIT license)

Element- and isotope-property library backed by an SQLite database. Sources
its data from IUPAC, NIST, NUBASE, and AME.

**What we take:** Isotope identity (Z, A, mass, abundance, half-life, spin,
parity, is_radioactive), element metadata (symbol, name, group, period, block).

**Gap:** No detailed branching ratios, daughter products, cross-sections, or
fission data.

Repository: https://github.com/lmmentel/mendeleev

#### radioactivedecay (Python, MIT license)

Decay-chain calculation library. Ships two datasets: ICRP-107 (1252 nuclides,
default) and NUBASE (~3000+ nuclides). Solves Bateman equations via matrix
exponentials.

**What we take:** Decay modes per nuclide, branching fractions, progeny
(daughter nuclides), half-lives.

**Gap:** No element properties, isotope masses, neutron interaction data, or
fission yields. We do NOT need its matrix-exponential solver -- our simulation
steps through one decay at a time.

Repository: https://github.com/radioactivedecay/radioactivedecay

#### OpenMC (C++, MIT license)

Monte Carlo particle transport code for neutronics. Reads cross-section
libraries from ENDF/B, JEFF, and TENDL.

**What we take:** Neutron absorption cross-sections (thermal and fast), fission
cross-sections and probability, fission product yield distributions.

**Gap:** We do NOT need the full transport engine, geometry, tallies, depletion
solvers, or any reactor-physics machinery. We extract only the cross-section
and yield tables.

Repository: https://github.com/openmc-dev/openmc

### How the sources combine

| Need                        | IAEA | NIST | NNDC/ENSDF | ENDF | mendeleev | rd   | OpenMC |
|-----------------------------|------|------|------------|------|-----------|------|--------|
| Element properties          | --   | Yes  | --         | --   | Yes       | --   | --     |
| Isotope mass / abundance    | Yes  | Yes  | Yes        | --   | Yes       | --   | --     |
| Stability                   | Yes  | --   | Yes        | --   | Yes       | Yes  | --     |
| Decay modes + branching     | Yes  | --   | Yes        | --   | --        | Yes  | --     |
| Daughter products           | Yes  | --   | Yes        | --   | --        | Yes  | --     |
| Neutron cross-sections      | --   | --   | --         | Yes  | --        | --   | Yes    |
| Fission probability         | --   | --   | --         | Yes  | --        | --   | Yes    |
| Fission product yields      | --   | --   | --         | Yes  | --        | --   | Yes    |

(rd = radioactivedecay)

### Data strategy

We have two complementary paths for getting data into the app:

**Path A -- Python extraction scripts** (quick start): Use mendeleev,
radioactivedecay, and OpenMC's Python API to extract data and output
Rust-friendly JSON. This gets us moving fast with well-curated data.

**Path B -- Direct database parsing** (long-term): Write a Rust ENSDF parser
and/or use the IAEA LiveChart CSV API to pull data directly. This eliminates
the Python dependency and gives us full control over coverage and freshness.

We start with Path A for speed, then migrate to Path B as needed. Both paths
produce the same output: a compiled-in dataset that ships inside the WASM
binary (no runtime fetches). The extraction tooling lives in the repo and can
be re-run to update or expand the dataset.

Coverage target: start with ~200-500 nuclides covering actinides, common
fission products, and elements users are likely to try. Provide graceful
"unknown isotope" handling for nuclides outside the dataset.

## User Interface

### Layout (already scaffolded)

- **Viewport (left)**: Visual representation of the nucleus, neutron approach,
  and reaction animations. Canvas or WebGL.
- **Panel (right)**: Controls and information display.

### Panel Sections

1. **Isotope Selector** -- pick element + mass number, or choose a preset
2. **Neutron Controls** -- "Fire Slow Neutron" / "Fire Fast Neutron" buttons
3. **Step Navigator** -- back/forward through the reaction chain, branch
   selector at fission points
4. **Current State Display** -- shows the current nuclide, what happened at
   this step, and properties (Z, N, A, stability, decay mode)
5. **Chain Summary** -- compact view of the full progression so far

### Viewport Visualization

Start simple and iterate:
- Phase 1: Stylized circle representing the nucleus, with Z/N displayed.
  Animate the neutron approaching and being absorbed. Show fission as the
  circle splitting.
- Phase 2: Individual nucleon dots (protons and neutrons) in a cluster.
  Animate absorption, rearrangement, fission, decay emissions.
- Phase 3: Particle effects, energy visualization, more polish.

## Architecture

### Rust Core (`nuclear-sim`)

The simulation engine, compiled to WASM for the browser. Pure Rust, no web
dependencies. Responsible for:

- Nuclide model (exists: Z, N, A, element symbol, notation)
- Nuclear data tables (stability, decay modes, fission data)
- Simulation logic: given a nuclide + neutron energy, compute what happens
- Step/history model: record the chain of events
- Navigation: step forward/backward, follow branches

### WASM Bridge (`nuclear-sim-wasm`)

Thin layer exposing the Rust API to JavaScript via wasm-bindgen. Translates
Rust types into JS-friendly structures.

### Vue Frontend (`apps/web`)

Handles all presentation and user interaction:

- State management (current simulation, step position)
- UI controls (isotope picker, neutron buttons, step navigator)
- Viewport rendering (canvas/WebGL nucleus visualization)
- Responsive layout

## Non-Goals (for now)

- Accurate energy accounting (binding energy, kinetic energy of products)
- Neutron chain reactions (multiple simultaneous fissions)
- Time-based simulation (real-time decay)
- Multiplayer or social features
- Mobile-native apps
