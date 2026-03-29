//! Simulation engine: neutron absorption, fission, decay, and step history.
//!
//! The play loop:
//!   1. User picks a starting isotope
//!   2. User fires a neutron (slow or fast)
//!   3. Neutron is absorbed, creating a compound nucleus (A+1)
//!   4. We resolve what happens: fission or radioactive decay
//!   5. If fission: split into fragments + free neutrons, user picks a fragment to follow
//!   6. If radioactive: decay step by step until stable
//!   7. User can walk forward/backward through the history

use crate::data::{DecayMode, FissionProduct, NuclideData, NuclideDatabase, Stability};
use crate::Nuclide;

/// Neutron energy regime.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NeutronEnergy {
    /// Thermal neutron (~0.025 eV). Favors fission in fissile isotopes.
    Slow,
    /// Fast neutron (~1-2 MeV). Can cause fission in fertile isotopes.
    Fast,
}

/// What happened at a single step in the simulation.
#[derive(Debug, Clone)]
pub enum SimEvent {
    /// Starting isotope chosen by the user.
    Start {
        nuclide: Nuclide,
    },
    /// Neutron was absorbed, forming a compound nucleus.
    NeutronAbsorbed {
        target: Nuclide,
        energy: NeutronEnergy,
        compound: Nuclide,
    },
    /// Compound nucleus underwent fission.
    Fission {
        parent: Nuclide,
        light: Nuclide,
        heavy: Nuclide,
        neutrons_released: u8,
    },
    /// Nuclide decayed via a specific mode.
    Decay {
        parent: Nuclide,
        mode: DecayMode,
        daughter: Nuclide,
    },
    /// Nuclide is stable -- chain ends here.
    Stable {
        nuclide: Nuclide,
    },
}

impl SimEvent {
    /// The nuclide that results from this event (the "current" nuclide after this step).
    pub fn resulting_nuclide(&self) -> Nuclide {
        match self {
            SimEvent::Start { nuclide } => *nuclide,
            SimEvent::NeutronAbsorbed { compound, .. } => *compound,
            SimEvent::Fission { heavy, .. } => *heavy, // default to heavy fragment
            SimEvent::Decay { daughter, .. } => *daughter,
            SimEvent::Stable { nuclide } => *nuclide,
        }
    }
}

/// At a fission point, the user must choose which fragment to follow.
#[derive(Debug, Clone)]
pub struct FissionBranch {
    /// Index of the step where fission occurred.
    pub fission_step: usize,
    /// The light fragment.
    pub light: Nuclide,
    /// The heavy fragment.
    pub heavy: Nuclide,
}

/// A simulation session. Tracks the full event history and navigation state.
pub struct Simulation {
    db: NuclideDatabase,
    /// The event log. Index 0 is the Start event.
    steps: Vec<SimEvent>,
    /// Current position in the step history (index into `steps`).
    cursor: usize,
    /// Fission branches encountered. When the user is at a fission step,
    /// they can choose which fragment to follow, which replaces the
    /// continuation from that point.
    fission_branches: Vec<FissionBranch>,
}

/// Errors that can occur during simulation.
#[derive(Debug, Clone, PartialEq)]
pub enum SimError {
    /// No starting isotope has been set.
    NoStart,
    /// The simulation chain has already terminated at a stable isotope.
    AlreadyStable,
    /// Nuclide not found in the database.
    UnknownNuclide(String),
    /// Neutron has already been fired; advance through the chain first.
    NeutronAlreadyFired,
    /// Can't go back past the start.
    AtStart,
    /// Can't go forward past the end.
    AtEnd,
    /// Invalid branch index.
    InvalidBranch,
    /// Cannot fire neutron at a nuclide with no neutron interaction data.
    NotFissile(String),
}

impl std::fmt::Display for SimError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SimError::NoStart => write!(f, "no starting isotope set"),
            SimError::AlreadyStable => write!(f, "chain has ended at a stable isotope"),
            SimError::UnknownNuclide(s) => write!(f, "unknown nuclide: {s}"),
            SimError::NeutronAlreadyFired => write!(f, "neutron already fired"),
            SimError::AtStart => write!(f, "already at start"),
            SimError::AtEnd => write!(f, "already at end of chain"),
            SimError::InvalidBranch => write!(f, "invalid branch index"),
            SimError::NotFissile(s) => write!(f, "{s} is not fissile"),
        }
    }
}

impl std::error::Error for SimError {}

impl Simulation {
    /// Create a new simulation with the given database.
    pub fn new(db: NuclideDatabase) -> Self {
        Self {
            db,
            steps: Vec::new(),
            cursor: 0,
            fission_branches: Vec::new(),
        }
    }

    /// Set the starting isotope. Resets any existing simulation.
    pub fn set_isotope(&mut self, z: u16, n: u16) -> Result<(), SimError> {
        let nuclide =
            Nuclide::try_new(z, n).map_err(|e| SimError::UnknownNuclide(e.to_string()))?;
        self.steps.clear();
        self.fission_branches.clear();
        self.steps.push(SimEvent::Start { nuclide });
        self.cursor = 0;
        Ok(())
    }

    /// Fire a neutron at the current nuclide.
    ///
    /// This adds a NeutronAbsorbed event (target absorbs neutron, forming A+1 compound),
    /// then resolves the outcome: fission (if fissile + slow neutron) or identifies the
    /// compound as radioactive/stable.
    pub fn fire_neutron(&mut self, energy: NeutronEnergy) -> Result<(), SimError> {
        if self.steps.is_empty() {
            return Err(SimError::NoStart);
        }

        // Can only fire at the end of the current chain
        if self.cursor != self.steps.len() - 1 {
            return Err(SimError::NeutronAlreadyFired);
        }

        // Can't fire if already at a stable endpoint with events after start
        if self.steps.len() > 1 {
            if let Some(SimEvent::Stable { .. }) = self.steps.last() {
                return Err(SimError::AlreadyStable);
            }
        }

        let target = self.current_nuclide();

        // Absorb the neutron: Z stays the same, N+1
        let compound = Nuclide::try_new(target.z(), target.n() + 1)
            .map_err(|e| SimError::UnknownNuclide(e.to_string()))?;

        self.steps.push(SimEvent::NeutronAbsorbed {
            target,
            energy,
            compound,
        });
        self.cursor = self.steps.len() - 1;

        // Now resolve what happens to the compound nucleus.
        // Check the *target* for fissile flag (U-235 is fissile, the compound U-236 fissions).
        let target_data = self.db.get(&target);

        if let Some(data) = target_data {
            if data.fissile && energy == NeutronEnergy::Slow && !data.fission_products.is_empty() {
                // Fission! Use the first product pair for now.
                let product = &data.fission_products[0];
                self.resolve_fission(compound, product.clone());
                return Ok(());
            }
        }

        // No fission -- resolve the compound nucleus
        self.resolve_nuclide(compound);
        Ok(())
    }

    /// Resolve a fission event.
    fn resolve_fission(&mut self, parent: Nuclide, product: FissionProduct) {
        let fission_step = self.steps.len();

        self.steps.push(SimEvent::Fission {
            parent,
            light: product.light,
            heavy: product.heavy,
            neutrons_released: product.neutrons_released,
        });

        self.fission_branches.push(FissionBranch {
            fission_step,
            light: product.light,
            heavy: product.heavy,
        });

        self.cursor = self.steps.len() - 1;

        // Auto-follow the heavy fragment's decay chain
        self.follow_decay_chain(product.heavy);
    }

    /// Follow a nuclide's decay chain until it reaches stability or we run out of data.
    fn follow_decay_chain(&mut self, start: Nuclide) {
        let mut current = start;
        let max_steps = 50; // safety limit
        let mut count = 0;

        loop {
            if count >= max_steps {
                break;
            }
            count += 1;

            let data = match self.db.get(&current) {
                Some(d) => d.clone(),
                None => {
                    // Unknown nuclide -- treat as stable (we don't have data for it)
                    self.steps.push(SimEvent::Stable { nuclide: current });
                    self.cursor = self.steps.len() - 1;
                    break;
                }
            };

            match data.stability {
                Stability::Stable => {
                    self.steps.push(SimEvent::Stable { nuclide: current });
                    self.cursor = self.steps.len() - 1;
                    break;
                }
                Stability::Radioactive => {
                    if data.decay_modes.is_empty() {
                        // No decay data -- treat as stable
                        self.steps.push(SimEvent::Stable { nuclide: current });
                        self.cursor = self.steps.len() - 1;
                        break;
                    }

                    // Pick the dominant decay mode (highest branching fraction).
                    // TODO: use RNG weighted by branching fractions for variety.
                    let branch = data
                        .decay_modes
                        .iter()
                        .max_by(|a, b| a.fraction.partial_cmp(&b.fraction).unwrap())
                        .unwrap();

                    let daughter = match branch.mode.daughter(&current) {
                        Some(d) => d,
                        None => {
                            // Decay math failed (shouldn't happen with valid data)
                            self.steps.push(SimEvent::Stable { nuclide: current });
                            self.cursor = self.steps.len() - 1;
                            break;
                        }
                    };

                    self.steps.push(SimEvent::Decay {
                        parent: current,
                        mode: branch.mode,
                        daughter,
                    });
                    self.cursor = self.steps.len() - 1;

                    current = daughter;
                }
            }
        }
    }

    /// Resolve a nuclide: if radioactive, follow its decay chain; if stable, mark it.
    fn resolve_nuclide(&mut self, nuclide: Nuclide) {
        self.follow_decay_chain(nuclide);
    }

    /// Switch to following a different fission fragment. This truncates the
    /// chain after the fission step and follows the chosen fragment instead.
    pub fn switch_branch(&mut self, branch_index: usize, follow_light: bool) -> Result<(), SimError> {
        let branch = self
            .fission_branches
            .get(branch_index)
            .ok_or(SimError::InvalidBranch)?
            .clone();

        let fragment = if follow_light {
            branch.light
        } else {
            branch.heavy
        };

        // Truncate everything after the fission step
        self.steps.truncate(branch.fission_step + 1);
        self.cursor = branch.fission_step;

        // Follow the chosen fragment
        self.follow_decay_chain(fragment);
        Ok(())
    }

    // -- Navigation --

    /// Move cursor forward one step. Returns the event at the new position.
    pub fn step_forward(&mut self) -> Result<&SimEvent, SimError> {
        if self.cursor >= self.steps.len() - 1 {
            return Err(SimError::AtEnd);
        }
        self.cursor += 1;
        Ok(&self.steps[self.cursor])
    }

    /// Move cursor backward one step. Returns the event at the new position.
    pub fn step_back(&mut self) -> Result<&SimEvent, SimError> {
        if self.cursor == 0 {
            return Err(SimError::AtStart);
        }
        self.cursor -= 1;
        Ok(&self.steps[self.cursor])
    }

    /// Jump to a specific step.
    pub fn go_to_step(&mut self, index: usize) -> Result<&SimEvent, SimError> {
        if index >= self.steps.len() {
            return Err(SimError::AtEnd);
        }
        self.cursor = index;
        Ok(&self.steps[self.cursor])
    }

    // -- Queries --

    /// The current event at the cursor.
    pub fn current_event(&self) -> Option<&SimEvent> {
        self.steps.get(self.cursor)
    }

    /// The nuclide at the current cursor position.
    pub fn current_nuclide(&self) -> Nuclide {
        self.steps
            .get(self.cursor)
            .map(|e| e.resulting_nuclide())
            .expect("simulation should have at least a Start event")
    }

    /// Current step index.
    pub fn cursor(&self) -> usize {
        self.cursor
    }

    /// Total number of steps.
    pub fn step_count(&self) -> usize {
        self.steps.len()
    }

    /// All steps in the history.
    pub fn steps(&self) -> &[SimEvent] {
        &self.steps
    }

    /// All fission branch points.
    pub fn fission_branches(&self) -> &[FissionBranch] {
        &self.fission_branches
    }

    /// Is the chain complete (ended at a stable nuclide)?
    pub fn is_complete(&self) -> bool {
        matches!(self.steps.last(), Some(SimEvent::Stable { .. }))
    }

    /// Look up data for a nuclide in the database.
    pub fn lookup(&self, nuclide: &Nuclide) -> Option<&NuclideData> {
        self.db.get(nuclide)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::build_stub_database;

    fn new_sim() -> Simulation {
        Simulation::new(build_stub_database())
    }

    #[test]
    fn set_isotope_and_read_back() {
        let mut sim = new_sim();
        sim.set_isotope(92, 143).unwrap(); // U-235
        assert_eq!(sim.current_nuclide().notation(), "U-235");
        assert_eq!(sim.step_count(), 1);
        assert_eq!(sim.cursor(), 0);
    }

    #[test]
    fn fire_slow_neutron_at_u235_causes_fission() {
        let mut sim = new_sim();
        sim.set_isotope(92, 143).unwrap();
        sim.fire_neutron(NeutronEnergy::Slow).unwrap();

        // Should have: Start, NeutronAbsorbed, Fission, then decay chain to stable
        assert!(sim.step_count() > 3);
        assert!(sim.is_complete());

        // Check that fission happened
        let has_fission = sim.steps().iter().any(|e| matches!(e, SimEvent::Fission { .. }));
        assert!(has_fission, "should have a fission event");

        // Chain should end at a stable nuclide
        let last = sim.steps().last().unwrap();
        assert!(matches!(last, SimEvent::Stable { .. }));
    }

    #[test]
    fn u235_fission_chain_ends_at_stable() {
        let mut sim = new_sim();
        sim.set_isotope(92, 143).unwrap();
        sim.fire_neutron(NeutronEnergy::Slow).unwrap();

        // The heavy fragment chain (Ba-141) should end at Pr-141
        assert!(sim.is_complete());
        let final_nuclide = sim.current_nuclide();
        // Default follows heavy fragment (Ba-141 -> ... -> Pr-141)
        assert_eq!(final_nuclide.notation(), "Pr-141");
    }

    #[test]
    fn can_switch_to_light_fragment() {
        let mut sim = new_sim();
        sim.set_isotope(92, 143).unwrap();
        sim.fire_neutron(NeutronEnergy::Slow).unwrap();

        // Switch to light fragment (Kr-92 chain)
        assert_eq!(sim.fission_branches().len(), 1);
        sim.switch_branch(0, true).unwrap();

        // Should now end at Zr-92
        assert!(sim.is_complete());
        assert_eq!(sim.current_nuclide().notation(), "Zr-92");
    }

    #[test]
    fn navigation_forward_and_back() {
        let mut sim = new_sim();
        sim.set_isotope(92, 143).unwrap();
        sim.fire_neutron(NeutronEnergy::Slow).unwrap();

        // Go back to start
        sim.go_to_step(0).unwrap();
        assert_eq!(sim.cursor(), 0);
        assert_eq!(sim.current_nuclide().notation(), "U-235");

        // Step forward
        sim.step_forward().unwrap();
        assert_eq!(sim.cursor(), 1);

        // Step back
        sim.step_back().unwrap();
        assert_eq!(sim.cursor(), 0);
    }

    #[test]
    fn fire_neutron_at_c14_no_fission() {
        let mut sim = new_sim();
        sim.set_isotope(6, 8).unwrap(); // C-14
        sim.fire_neutron(NeutronEnergy::Slow).unwrap();

        // C-14 is not fissile, so neutron absorption creates C-15
        // C-15 is not in our stub data, so it should be treated as stable (unknown)
        assert!(sim.is_complete());

        // No fission events
        let has_fission = sim.steps().iter().any(|e| matches!(e, SimEvent::Fission { .. }));
        assert!(!has_fission);
    }

    #[test]
    fn cannot_step_back_past_start() {
        let mut sim = new_sim();
        sim.set_isotope(92, 143).unwrap();
        assert!(sim.step_back().is_err());
    }

    #[test]
    fn cannot_fire_twice_without_reset() {
        let mut sim = new_sim();
        sim.set_isotope(92, 143).unwrap();
        sim.fire_neutron(NeutronEnergy::Slow).unwrap();
        // Chain is complete at stable -- can't fire again
        assert!(sim.fire_neutron(NeutronEnergy::Slow).is_err());
    }
}
