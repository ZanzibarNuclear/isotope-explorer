//! Chemical element symbols by atomic number (IUPAC, Z = 1..=118).

/// Element symbol for `z`, or `None` if out of known range.
pub fn symbol(atomic_number: u16) -> Option<&'static str> {
    if atomic_number == 0 || atomic_number > ELEMENT_SYMBOLS.len() as u16 {
        return None;
    }
    Some(ELEMENT_SYMBOLS[(atomic_number - 1) as usize])
}

const ELEMENT_SYMBOLS: &[&str] = &[
    "H", "He", "Li", "Be", "B", "C", "N", "O", "F", "Ne", "Na", "Mg", "Al", "Si", "P", "S",
    "Cl", "Ar", "K", "Ca", "Sc", "Ti", "V", "Cr", "Mn", "Fe", "Co", "Ni", "Cu", "Zn", "Ga",
    "Ge", "As", "Se", "Br", "Kr", "Rb", "Sr", "Y", "Zr", "Nb", "Mo", "Tc", "Ru", "Rh", "Pd",
    "Ag", "Cd", "In", "Sn", "Sb", "Te", "I", "Xe", "Cs", "Ba", "La", "Ce", "Pr", "Nd", "Pm",
    "Sm", "Eu", "Gd", "Tb", "Dy", "Ho", "Er", "Tm", "Yb", "Lu", "Hf", "Ta", "W", "Re", "Os",
    "Ir", "Pt", "Au", "Hg", "Tl", "Pb", "Bi", "Po", "At", "Rn", "Fr", "Ra", "Ac", "Th", "Pa",
    "U", "Np", "Pu", "Am", "Cm", "Bk", "Cf", "Es", "Fm", "Md", "No", "Lr", "Rf", "Db", "Sg",
    "Bh", "Hs", "Mt", "Ds", "Rg", "Cn", "Nh", "Fl", "Mc", "Lv", "Ts", "Og",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hydrogen_and_oganesson() {
        assert_eq!(symbol(1), Some("H"));
        assert_eq!(symbol(118), Some("Og"));
        assert_eq!(symbol(0), None);
        assert_eq!(symbol(119), None);
    }
}
