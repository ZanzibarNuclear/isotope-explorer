//! A **nuclide** is a nucleus with a definite proton and neutron count (isotope of an element).
//!
//! This is compositional data only: no energies, half-lives, or cross sections yet. Those belong
//! in nuclear-data tables you will load or embed later.

use std::fmt;

use crate::elements;

/// Proton number Z and neutron number N define a nuclide; mass number A = Z + N.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Nuclide {
    z: u16,
    n: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NuclideError {
    ProtonsOutOfRange { z: u16 },
    NeutronsOutOfRange { n: u16 },
    MassNumberOutOfRange { a: u32 },
}

impl fmt::Display for NuclideError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NuclideError::ProtonsOutOfRange { z } => {
                write!(f, "atomic number Z={z} is outside 1..=118")
            }
            NuclideError::NeutronsOutOfRange { n } => {
                write!(f, "neutron number N={n} is outside 0..=200")
            }
            NuclideError::MassNumberOutOfRange { a } => {
                write!(f, "mass number A={a} is outside 2..=300")
            }
        }
    }
}

impl std::error::Error for NuclideError {}

/// Bounds wide enough for known nuclides and a little headroom for tabulated data later.
const Z_MAX: u16 = 118;
const N_MAX: u16 = 200;
const A_MIN: u32 = 2;
const A_MAX: u32 = 300;

impl Nuclide {
    /// Construct a nuclide from proton count `z` and neutron count `n`.
    pub fn try_new(z: u16, n: u16) -> Result<Self, NuclideError> {
        if !(1..=Z_MAX).contains(&z) {
            return Err(NuclideError::ProtonsOutOfRange { z });
        }
        if n > N_MAX {
            return Err(NuclideError::NeutronsOutOfRange { n });
        }
        let a = u32::from(z) + u32::from(n);
        if !(A_MIN..=A_MAX).contains(&a) {
            return Err(NuclideError::MassNumberOutOfRange { a });
        }
        Ok(Self { z, n })
    }

    pub fn z(&self) -> u16 {
        self.z
    }

    pub fn n(&self) -> u16 {
        self.n
    }

    pub fn mass_number(&self) -> u16 {
        self.z + self.n
    }

    pub fn element_symbol(&self) -> &'static str {
        elements::symbol(self.z).unwrap_or("?")
    }

    /// Human-readable label, e.g. `"U-235"`, `"C-14"`.
    pub fn notation(&self) -> String {
        format!("{}-{}", self.element_symbol(), self.mass_number())
    }

    // --- Common examples (sanity-checked at compile time via tests) ---

    pub fn hydrogen_1() -> Self {
        Self::try_new(1, 0).expect("H-1")
    }

    pub fn carbon_14() -> Self {
        Self::try_new(6, 8).expect("C-14")
    }

    pub fn uranium_235() -> Self {
        Self::try_new(92, 143).expect("U-235")
    }

    pub fn uranium_238() -> Self {
        Self::try_new(92, 146).expect("U-238")
    }

    pub fn plutonium_239() -> Self {
        Self::try_new(94, 145).expect("Pu-239")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn u235() {
        let u = Nuclide::uranium_235();
        assert_eq!(u.z(), 92);
        assert_eq!(u.n(), 143);
        assert_eq!(u.mass_number(), 235);
        assert_eq!(u.notation(), "U-235");
    }

    #[test]
    fn rejects_bad_z() {
        assert!(Nuclide::try_new(0, 1).is_err());
        assert!(Nuclide::try_new(119, 1).is_err());
    }
}
