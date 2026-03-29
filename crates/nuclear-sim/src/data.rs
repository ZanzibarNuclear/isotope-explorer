//! Nuclear data tables: properties, decay modes, and fission products for known nuclides.
//!
//! Phase 0: hand-curated stub data for the U-235 thermal fission happy path.
//! Will be replaced with extracted data from IAEA / ENSDF / ENDF later.

use crate::Nuclide;
use std::collections::HashMap;

/// How a radioactive nuclide transforms.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DecayMode {
    /// Emits an alpha particle (He-4): Z-2, N-2
    Alpha,
    /// Neutron converts to proton: Z+1, N-1
    BetaMinus,
    /// Proton converts to neutron: Z-1, N+1
    BetaPlus,
    /// Proton captures orbital electron: Z-1, N+1 (same daughter as BetaPlus)
    ElectronCapture,
    /// Excited state emits gamma photon: Z and N unchanged
    IsomericTransition,
}

impl DecayMode {
    /// Compute the daughter nuclide after this decay.
    pub fn daughter(&self, parent: &Nuclide) -> Option<Nuclide> {
        match self {
            DecayMode::Alpha => Nuclide::try_new(
                parent.z().checked_sub(2)?,
                parent.n().checked_sub(2)?,
            )
            .ok(),
            DecayMode::BetaMinus => {
                Nuclide::try_new(parent.z() + 1, parent.n().checked_sub(1)?).ok()
            }
            DecayMode::BetaPlus | DecayMode::ElectronCapture => {
                Nuclide::try_new(parent.z().checked_sub(1)?, parent.n() + 1).ok()
            }
            DecayMode::IsomericTransition => Some(*parent),
        }
    }
}

/// A single decay branch: mode + probability (0.0..=1.0).
#[derive(Debug, Clone)]
pub struct DecayBranch {
    pub mode: DecayMode,
    pub fraction: f64,
}

/// A pair of fission fragments with their yield probability.
#[derive(Debug, Clone)]
pub struct FissionProduct {
    pub light: Nuclide,
    pub heavy: Nuclide,
    pub neutrons_released: u8,
    pub fraction: f64,
}

/// What kind of nuclide is this?
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Stability {
    /// Will not spontaneously decay.
    Stable,
    /// Will decay via one or more modes.
    Radioactive,
}

/// Everything we know about a specific nuclide.
#[derive(Debug, Clone)]
pub struct NuclideData {
    pub stability: Stability,
    /// Half-life in seconds. None for stable isotopes.
    pub half_life_s: Option<f64>,
    /// Decay branches (empty for stable isotopes). Fractions should sum to ~1.0.
    pub decay_modes: Vec<DecayBranch>,
    /// Can this nuclide undergo fission when hit by a thermal neutron?
    pub fissile: bool,
    /// Fission product pairs (empty if not fissile). Fractions should sum to ~1.0.
    pub fission_products: Vec<FissionProduct>,
}

/// The nuclear data catalog. Keyed by (Z, N) via Nuclide.
pub struct NuclideDatabase {
    data: HashMap<Nuclide, NuclideData>,
}

impl NuclideDatabase {
    /// Look up data for a nuclide. Returns None if not in our dataset.
    pub fn get(&self, nuclide: &Nuclide) -> Option<&NuclideData> {
        self.data.get(nuclide)
    }

    /// How many nuclides are in the database.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

// ---------------------------------------------------------------------------
// Stub dataset: U-235 thermal fission happy path
// ---------------------------------------------------------------------------

fn n(z: u16, n: u16) -> Nuclide {
    Nuclide::try_new(z, n).expect("valid nuclide in stub data")
}

fn stable() -> NuclideData {
    NuclideData {
        stability: Stability::Stable,
        half_life_s: None,
        decay_modes: vec![],
        fissile: false,
        fission_products: vec![],
    }
}

fn radioactive(half_life_s: f64, modes: Vec<DecayBranch>) -> NuclideData {
    NuclideData {
        stability: Stability::Radioactive,
        half_life_s: Some(half_life_s),
        decay_modes: modes,
        fissile: false,
        fission_products: vec![],
    }
}

/// Build the Phase 0 stub database.
///
/// Covers the U-235 thermal fission happy path:
///   U-235 + n -> U-236 -> fission -> Ba-141 + Kr-92 + 3n
///   Ba-141 -> La-141 -> Ce-141 -> Pr-141 (stable)
///   Kr-92  -> Rb-92  -> Sr-92  -> Y-92   -> Zr-92 (stable)
///
/// Plus a few other interesting nuclides for testing.
pub fn build_stub_database() -> NuclideDatabase {
    let mut data = HashMap::new();

    // -- U-235: fissile, the star of the show --
    data.insert(
        n(92, 143), // U-235
        NuclideData {
            stability: Stability::Radioactive,
            half_life_s: Some(2.22e16), // 704 million years
            decay_modes: vec![DecayBranch {
                mode: DecayMode::Alpha,
                fraction: 1.0,
            }],
            fissile: true,
            fission_products: vec![FissionProduct {
                light: n(36, 56), // Kr-92
                heavy: n(56, 85), // Ba-141
                neutrons_released: 3,
                fraction: 1.0, // simplified: one product pair for now
            }],
        },
    );

    // -- U-236: compound nucleus after neutron absorption --
    // In reality U-236 is what fissions, but for the user experience we
    // model the fission as happening from U-235 + neutron. U-236 as a
    // standalone isotope is a long-lived alpha emitter.
    data.insert(
        n(92, 144), // U-236
        radioactive(
            7.39e14, // 23.4 million years
            vec![DecayBranch {
                mode: DecayMode::Alpha,
                fraction: 1.0,
            }],
        ),
    );

    // -- Ba-141 decay chain: Ba-141 -> La-141 -> Ce-141 -> Pr-141 (stable) --

    data.insert(
        n(56, 85), // Ba-141, half-life 18.27 minutes
        radioactive(
            1096.2,
            vec![DecayBranch {
                mode: DecayMode::BetaMinus,
                fraction: 1.0,
            }],
        ),
    );

    data.insert(
        n(57, 84), // La-141, half-life 3.92 hours
        radioactive(
            14112.0,
            vec![DecayBranch {
                mode: DecayMode::BetaMinus,
                fraction: 1.0,
            }],
        ),
    );

    data.insert(
        n(58, 83), // Ce-141, half-life 32.5 days
        radioactive(
            2.808e6,
            vec![DecayBranch {
                mode: DecayMode::BetaMinus,
                fraction: 1.0,
            }],
        ),
    );

    data.insert(n(59, 82), stable()); // Pr-141 (stable)

    // -- Kr-92 decay chain: Kr-92 -> Rb-92 -> Sr-92 -> Y-92 -> Zr-92 (stable) --

    data.insert(
        n(36, 56), // Kr-92, half-life 1.84 seconds
        radioactive(
            1.84,
            vec![DecayBranch {
                mode: DecayMode::BetaMinus,
                fraction: 1.0,
            }],
        ),
    );

    data.insert(
        n(37, 55), // Rb-92, half-life 4.49 seconds
        radioactive(
            4.49,
            vec![DecayBranch {
                mode: DecayMode::BetaMinus,
                fraction: 1.0,
            }],
        ),
    );

    data.insert(
        n(38, 54), // Sr-92, half-life 2.66 hours
        radioactive(
            9576.0,
            vec![DecayBranch {
                mode: DecayMode::BetaMinus,
                fraction: 1.0,
            }],
        ),
    );

    data.insert(
        n(39, 53), // Y-92, half-life 3.54 hours
        radioactive(
            12744.0,
            vec![DecayBranch {
                mode: DecayMode::BetaMinus,
                fraction: 1.0,
            }],
        ),
    );

    data.insert(n(40, 52), stable()); // Zr-92 (stable)

    // -- Bonus nuclides for variety --

    // Co-60: classic beta-minus emitter (gamma source)
    data.insert(
        n(27, 33), // Co-60, half-life 5.27 years
        radioactive(
            1.663e8,
            vec![DecayBranch {
                mode: DecayMode::BetaMinus,
                fraction: 1.0,
            }],
        ),
    );

    data.insert(n(28, 32), stable()); // Ni-60 (stable, daughter of Co-60)

    // C-14: famous beta-minus emitter (carbon dating)
    data.insert(
        n(6, 8), // C-14, half-life 5730 years
        radioactive(
            1.808e11,
            vec![DecayBranch {
                mode: DecayMode::BetaMinus,
                fraction: 1.0,
            }],
        ),
    );

    data.insert(n(7, 7), stable()); // N-14 (stable, daughter of C-14)

    // Pu-239: another fissile isotope (stub with alpha decay only for now)
    data.insert(
        n(94, 145), // Pu-239, half-life 24,110 years
        NuclideData {
            stability: Stability::Radioactive,
            half_life_s: Some(7.61e11),
            decay_modes: vec![DecayBranch {
                mode: DecayMode::Alpha,
                fraction: 1.0,
            }],
            fissile: true,
            fission_products: vec![], // TODO: add Pu-239 fission products
        },
    );

    // U-238: fertile, alpha emitter
    data.insert(
        n(92, 146), // U-238, half-life 4.47 billion years
        radioactive(
            1.41e17,
            vec![DecayBranch {
                mode: DecayMode::Alpha,
                fraction: 1.0,
            }],
        ),
    );

    // Th-234: daughter of U-238 alpha decay
    data.insert(
        n(90, 144), // Th-234, half-life 24.1 days
        radioactive(
            2.082e6,
            vec![DecayBranch {
                mode: DecayMode::BetaMinus,
                fraction: 1.0,
            }],
        ),
    );

    // Th-231: daughter of U-235 alpha decay
    data.insert(
        n(90, 141), // Th-231, half-life 25.5 hours
        radioactive(
            91800.0,
            vec![DecayBranch {
                mode: DecayMode::BetaMinus,
                fraction: 1.0,
            }],
        ),
    );

    // He-4: alpha particle (stable)
    data.insert(n(2, 2), stable());

    NuclideDatabase { data }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stub_database_loads() {
        let db = build_stub_database();
        assert!(db.len() > 10);
    }

    #[test]
    fn u235_is_fissile() {
        let db = build_stub_database();
        let u235 = Nuclide::uranium_235();
        let data = db.get(&u235).expect("U-235 should be in stub data");
        assert!(data.fissile);
        assert_eq!(data.fission_products.len(), 1);
    }

    #[test]
    fn ba141_decays_to_la141() {
        let db = build_stub_database();
        let ba141 = n(56, 85);
        let data = db.get(&ba141).expect("Ba-141 should be in stub data");
        assert_eq!(data.decay_modes.len(), 1);
        assert_eq!(data.decay_modes[0].mode, DecayMode::BetaMinus);

        let daughter = DecayMode::BetaMinus.daughter(&ba141).unwrap();
        assert_eq!(daughter.z(), 57); // La
        assert_eq!(daughter.mass_number(), 141);
    }

    #[test]
    fn pr141_is_stable() {
        let db = build_stub_database();
        let pr141 = n(59, 82);
        let data = db.get(&pr141).expect("Pr-141 should be in stub data");
        assert_eq!(data.stability, Stability::Stable);
    }

    #[test]
    fn zr92_is_stable() {
        let db = build_stub_database();
        let zr92 = n(40, 52);
        let data = db.get(&zr92).expect("Zr-92 should be in stub data");
        assert_eq!(data.stability, Stability::Stable);
    }

    #[test]
    fn kr92_chain_reaches_zr92() {
        let db = build_stub_database();
        let mut current = n(36, 56); // Kr-92
        let mut steps = 0;
        loop {
            let data = db.get(&current).expect("nuclide should be in stub data");
            if data.stability == Stability::Stable {
                break;
            }
            current = data.decay_modes[0].mode.daughter(&current).unwrap();
            steps += 1;
            assert!(steps < 20, "decay chain too long -- possible loop");
        }
        assert_eq!(current, n(40, 52)); // Zr-92
        assert_eq!(steps, 4);
    }
}
