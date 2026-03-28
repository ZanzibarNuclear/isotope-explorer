//! A **nucleus** in this codebase is a concrete bound system: a [Nuclide] identity plus (later)
//! dynamical state for your simulation (excitation, velocity, etc.).

use crate::nuclide::Nuclide;

/// Nuclear composition and identity. Extend this struct when you add internal energy,
/// angular momentum labels, or reaction history.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Nucleus {
    nuclide: Nuclide,
}

impl Nucleus {
    pub fn new(nuclide: Nuclide) -> Self {
        Self { nuclide }
    }

    pub fn nuclide(&self) -> Nuclide {
        self.nuclide
    }

    pub fn proton_count(&self) -> u16 {
        self.nuclide.z()
    }

    pub fn neutron_count(&self) -> u16 {
        self.nuclide.n()
    }

    pub fn mass_number(&self) -> u16 {
        self.nuclide.mass_number()
    }

    pub fn notation(&self) -> String {
        self.nuclide.notation()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_u235() {
        let n = Nucleus::new(Nuclide::uranium_235());
        assert_eq!(n.proton_count(), 92);
        assert_eq!(n.neutron_count(), 143);
        assert_eq!(n.notation(), "U-235");
    }
}
