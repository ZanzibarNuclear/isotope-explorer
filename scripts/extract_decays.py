#!/usr/bin/env python3
"""
Phase 1, Step 2: Extract decay data from radioactivedecay (ICRP-107 dataset).

For each nuclide: decay modes, branching fractions, daughter nuclides, half-lives.

Outputs: data/decay_data.json
"""

import json
import math
import re
from pathlib import Path

import radioactivedecay as rd


# Map radioactivedecay decay mode strings to our canonical names
DECAY_MODE_MAP = {
    "α": "Alpha",
    "β-": "BetaMinus",
    "β+": "BetaPlus",
    "EC": "ElectronCapture",
    "IT": "IsomericTransition",
    "SF": "SpontaneousFission",
    "p": "ProtonEmission",
    "n": "NeutronEmission",
    "2β-": "DoubleBetaMinus",
    "2β+": "DoubleBetaPlus",
    "β-n": "BetaMinusNeutron",
    "β-2n": "BetaMinus2Neutron",
    "β+p": "BetaPlusProton",
    "β+α": "BetaPlusAlpha",
    "β-α": "BetaMinusAlpha",
    "2p": "DoubleProtonEmission",
    "2n": "DoubleNeutronEmission",
    "14C": "ClusterDecay14C",
    "24Ne": "ClusterDecay24Ne",
}


def parse_nuclide_name(name: str):
    """Parse a radioactivedecay nuclide name like 'U-235' or 'Tc-99m' into (Z, A, is_metastable)."""
    # Handle metastable states
    is_meta = name.endswith("m") or name.endswith("n")  # n = second metastable
    clean = re.sub(r"[mn]$", "", name)

    match = re.match(r"([A-Za-z]+)-(\d+)", clean)
    if not match:
        return None

    symbol = match.group(1)
    a = int(match.group(2))

    z = symbol_to_z(symbol)
    if z is None:
        return None

    return {"z": z, "n": a - z, "a": a, "symbol": symbol, "is_metastable": is_meta}


# Element symbol -> Z lookup
_SYMBOLS = [
    "", "H", "He", "Li", "Be", "B", "C", "N", "O", "F", "Ne",
    "Na", "Mg", "Al", "Si", "P", "S", "Cl", "Ar", "K", "Ca",
    "Sc", "Ti", "V", "Cr", "Mn", "Fe", "Co", "Ni", "Cu", "Zn",
    "Ga", "Ge", "As", "Se", "Br", "Kr", "Rb", "Sr", "Y", "Zr",
    "Nb", "Mo", "Tc", "Ru", "Rh", "Pd", "Ag", "Cd", "In", "Sn",
    "Sb", "Te", "I", "Xe", "Cs", "Ba", "La", "Ce", "Pr", "Nd",
    "Pm", "Sm", "Eu", "Gd", "Tb", "Dy", "Ho", "Er", "Tm", "Yb",
    "Lu", "Hf", "Ta", "W", "Re", "Os", "Ir", "Pt", "Au", "Hg",
    "Tl", "Pb", "Bi", "Po", "At", "Rn", "Fr", "Ra", "Ac", "Th",
    "Pa", "U", "Np", "Pu", "Am", "Cm", "Bk", "Cf", "Es", "Fm",
    "Md", "No", "Lr", "Rf", "Db", "Sg", "Bh", "Hs", "Mt", "Ds",
    "Rg", "Cn", "Nh", "Fl", "Mc", "Lv", "Ts", "Og",
]

_SYMBOL_TO_Z = {s: i for i, s in enumerate(_SYMBOLS) if s}


def symbol_to_z(symbol: str) -> int | None:
    return _SYMBOL_TO_Z.get(symbol)


def extract_decay_data():
    """Extract decay data for all nuclides in the ICRP-107 dataset."""
    dd = rd.DEFAULTDATA
    nuclide_names = list(dd.nuclide_dict.keys())
    print(f"Found {len(nuclide_names)} nuclides in {dd.dataset_name} dataset")

    nuclides = []
    skipped = 0

    for name in nuclide_names:
        try:
            nuc = rd.Nuclide(str(name))
        except Exception:
            skipped += 1
            continue

        parsed = parse_nuclide_name(str(name))
        if parsed is None:
            skipped += 1
            continue

        # Get half-life
        half_life_s = nuc.half_life("s")
        if isinstance(half_life_s, str):
            half_life_s = None
            is_stable = True
        elif math.isinf(half_life_s):
            half_life_s = None
            is_stable = True
        else:
            is_stable = False

        # Get decay modes, branching fractions, and progeny
        modes = nuc.decay_modes()
        branching = nuc.branching_fractions()
        progeny = nuc.progeny()

        decay_branches = []
        for i, (mode_str, fraction) in enumerate(zip(modes, branching)):
            canonical_mode = DECAY_MODE_MAP.get(mode_str, mode_str)

            daughter_name = progeny[i] if i < len(progeny) else None
            daughter_parsed = None
            if daughter_name and daughter_name not in ("SF", "", None):
                daughter_parsed = parse_nuclide_name(str(daughter_name))

            branch = {
                "mode": canonical_mode,
                "fraction": float(fraction),
                "daughter": str(daughter_name) if daughter_name else None,
                "daughter_z": daughter_parsed["z"] if daughter_parsed else None,
                "daughter_n": daughter_parsed["n"] if daughter_parsed else None,
            }
            decay_branches.append(branch)

        entry = {
            "name": str(name),
            "z": parsed["z"],
            "n": parsed["n"],
            "a": parsed["a"],
            "is_metastable": parsed["is_metastable"],
            "is_stable": is_stable,
            "half_life_sec": float(half_life_s) if half_life_s is not None else None,
            "decay_branches": decay_branches,
        }
        nuclides.append(entry)

    print(f"Extracted {len(nuclides)} nuclides, skipped {skipped}")
    return nuclides


def main():
    out_dir = Path(__file__).parent / "data"
    out_dir.mkdir(exist_ok=True)

    print("Extracting decay data from radioactivedecay (ICRP-107)...")
    nuclides = extract_decay_data()

    # Summary
    radioactive = sum(1 for n in nuclides if not n["is_stable"])
    with_decays = sum(1 for n in nuclides if n["decay_branches"])
    print(f"\n{radioactive} radioactive, {with_decays} with decay mode data")

    output = {
        "source": "radioactivedecay (ICRP-107)",
        "nuclide_count": len(nuclides),
        "nuclides": nuclides,
    }

    out_path = out_dir / "decay_data.json"
    with open(out_path, "w") as f:
        json.dump(output, f, indent=2)

    print(f"\nWrote {out_path}")


if __name__ == "__main__":
    main()
