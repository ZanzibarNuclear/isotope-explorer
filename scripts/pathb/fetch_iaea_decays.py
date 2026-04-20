#!/usr/bin/env python3
"""
Fetch ground-state nuclear data from the IAEA LiveChart API.

Source: https://www-nds.iaea.org/relnsd/v1/data
Returns CSV with ~3,400 ground-state nuclides including:
  - Z, N, symbol, half-life, decay modes + branching fractions
  - Spin/parity, atomic mass (micro-AMU), natural abundance (%)
  - Binding energy, mass excess, separation energies

This replaces both mendeleev (isotope properties) and radioactivedecay
(ICRP-107 decay chains) from the Path A pipeline.

Outputs: data/iaea_ground_states.json
"""

import csv
import io
import json
import sys
from datetime import date
from pathlib import Path
from urllib.request import Request, urlopen


IAEA_URL = "https://www-nds.iaea.org/relnsd/v1/data?fields=ground_states&nuclides=all"

# IAEA blocks the default Python urllib User-Agent
HEADERS = {"User-Agent": "IsotopeExplorer/1.0 (nuclear-data-extraction)"}

# Map IAEA decay mode strings to our canonical names.
# We only map modes that change Z/N in ways our simulation handles.
# Exotic modes (2P, 14C, SF, etc.) are mapped to None and skipped.
DECAY_MODE_MAP = {
    "A": "Alpha",
    "B-": "BetaMinus",
    "B+": "BetaPlus",
    "EC": "ElectronCapture",
    "EC+B+": "BetaPlus",       # combined EC + beta-plus; same daughter (Z-1, N+1)
    "IT": "IsomericTransition",
    # Delayed-particle modes: we treat these as the primary decay
    # (e.g., B-N = beta-minus followed by neutron emission).
    # The simulation doesn't model delayed particles, so we map to the
    # primary mode that changes Z. The daughter will be looked up from
    # the database rather than computed from the mode.
    "B-N": "BetaMinus",
    "B-2N": "BetaMinus",
    "B-3N": "BetaMinus",
    "B-4N": "BetaMinus",
    "B-5N": "BetaMinus",
    "B-A": "BetaMinus",
    "B+P": "BetaPlus",
    "B+2P": "BetaPlus",
    "B+A": "BetaPlus",
    "ECP": "ElectronCapture",
    "EC2P": "ElectronCapture",
    "ECA": "ElectronCapture",
    "ECSF": "ElectronCapture",
    # Modes we skip (exotic, rare, or not modeled):
    "P": None,           # proton emission
    "2P": None,          # two-proton emission
    "N": None,           # neutron emission
    "2N": None,          # two-neutron emission
    "SF": None,          # spontaneous fission
    "SF+EC+B+": None,    # combined SF + EC
    "2B-": None,         # double beta-minus (extremely rare)
    "2B+": None,         # double beta-plus
    "2EC": None,         # double electron capture
    "14C": None,         # cluster decay (carbon-14 emission)
    "24NE": None,        # cluster decay (neon-24 emission)
    "{+22}Ne": None,     # cluster decay
    "{+24}Ne": None,     # cluster decay
    "{+25}Ne": None,     # cluster decay
    "{+34}Si": None,     # cluster decay
    "34SI": None,        # cluster decay
    "Mg": None,          # cluster decay (magnesium emission)
    "B-7N": None,        # beta-minus delayed 7-neutron (extremely rare)
    "B-6N": None,        # beta-minus delayed 6-neutron
    "B-P": None,         # beta-minus delayed proton (very rare)
    "B-SF": None,        # beta-minus delayed fission
}


def parse_spin_parity(jp_str):
    """Parse IAEA jp string like '7/2-' or '0+' into (spin_float, parity_int)."""
    if not jp_str or jp_str.strip() == "":
        return None, None

    jp = jp_str.strip().strip("()")  # remove parentheses indicating uncertainty

    parity = None
    if jp.endswith("+"):
        parity = 1
        jp = jp[:-1]
    elif jp.endswith("-"):
        parity = -1
        jp = jp[:-1]

    spin = None
    if "/" in jp:
        try:
            num, den = jp.split("/")
            spin = float(num) / float(den)
        except (ValueError, ZeroDivisionError):
            pass
    else:
        try:
            spin = float(jp)
        except ValueError:
            pass

    return spin, parity


def parse_float(s):
    """Parse a string to float, returning None for empty/invalid."""
    if not s or s.strip() == "":
        return None
    try:
        return float(s.strip())
    except ValueError:
        return None


def parse_int(s):
    """Parse a string to int, returning None for empty/invalid."""
    if not s or s.strip() == "":
        return None
    try:
        return int(s.strip())
    except ValueError:
        return None


def fetch_and_parse():
    """Fetch IAEA ground-state data and return parsed nuclide records."""
    print(f"Fetching from {IAEA_URL} ...")
    req = Request(IAEA_URL, headers=HEADERS)
    with urlopen(req) as resp:
        raw = resp.read().decode("utf-8")

    reader = csv.DictReader(io.StringIO(raw))
    nuclides = {}
    skipped = 0
    unmapped_modes = {}

    for row in reader:
        z = parse_int(row["z"])
        n = parse_int(row["n"])
        if z is None or n is None:
            skipped += 1
            continue
        # Skip the free neutron (z=0)
        if z == 0:
            continue

        half_life_sec = parse_float(row.get("half_life_sec", ""))
        half_life_str = (row.get("half_life", "") or "").strip()
        is_stable = half_life_str == "STABLE"

        # Atomic mass: IAEA gives it in micro-AMU, convert to AMU
        atomic_mass_micro = parse_float(row.get("atomic_mass", ""))
        atomic_mass_amu = atomic_mass_micro / 1e6 if atomic_mass_micro is not None else None

        # Natural abundance: IAEA gives percentage, convert to fraction
        abundance_pct = parse_float(row.get("abundance", ""))
        abundance = abundance_pct / 100.0 if abundance_pct is not None else None

        # Spin/parity
        spin, parity = parse_spin_parity(row.get("jp", ""))

        # Binding energy per nucleon (keV)
        binding = parse_float(row.get("binding", ""))

        # Decay modes (up to 3)
        decay_branches = []
        for i in range(1, 4):
            mode_str = (row.get(f"decay_{i}", "") or "").strip()
            frac_str = (row.get(f"decay_{i}_%", "") or "").strip()
            if not mode_str:
                continue

            if mode_str not in DECAY_MODE_MAP:
                unmapped_modes[mode_str] = unmapped_modes.get(mode_str, 0) + 1
                continue

            canonical = DECAY_MODE_MAP[mode_str]
            if canonical is None:
                # Exotic mode we intentionally skip
                continue

            fraction = parse_float(frac_str)
            if fraction is not None:
                fraction = fraction / 100.0  # IAEA gives percentages
            else:
                fraction = 0.0

            decay_branches.append({
                "mode": canonical,
                "fraction": fraction,
                "iaea_mode": mode_str,  # keep original for debugging
            })

        # Normalize fractions if they don't sum to ~1.0
        total = sum(b["fraction"] for b in decay_branches)
        if decay_branches and total > 0 and abs(total - 1.0) > 0.01:
            for b in decay_branches:
                b["fraction"] = b["fraction"] / total

        key = f"{z}_{n}"
        nuclides[key] = {
            "z": z,
            "n": n,
            "symbol": (row.get("symbol", "") or "").strip(),
            "is_stable": is_stable,
            "half_life_s": half_life_sec if not is_stable else None,
            "atomic_mass_amu": atomic_mass_amu,
            "abundance": abundance,
            "spin": spin,
            "parity": parity,
            "binding_per_nucleon_kev": binding,
            "decay_modes": decay_branches,
        }

    if unmapped_modes:
        print(f"\nUnmapped decay modes (skipped):")
        for mode, count in sorted(unmapped_modes.items(), key=lambda x: -x[1]):
            print(f"  {mode}: {count}")

    return nuclides


def print_summary(nuclides):
    """Print summary statistics about the parsed dataset."""
    total = len(nuclides)
    stable = sum(1 for n in nuclides.values() if n["is_stable"])
    has_decay = sum(1 for n in nuclides.values() if n["decay_modes"])
    has_mass = sum(1 for n in nuclides.values() if n["atomic_mass_amu"] is not None)
    has_abundance = sum(1 for n in nuclides.values() if n["abundance"] is not None)
    has_spin = sum(1 for n in nuclides.values() if n["spin"] is not None)

    print(f"\n=== IAEA Ground State Data Summary ===")
    print(f"Total nuclides:       {total}")
    print(f"Stable:               {stable}")
    print(f"With decay modes:     {has_decay}")
    print(f"With atomic mass:     {has_mass}")
    print(f"With abundance:       {has_abundance}")
    print(f"With spin:            {has_spin}")

    # Check a few known values
    u235 = nuclides.get("92_143")
    if u235:
        print(f"\nU-235 check:")
        print(f"  mass = {u235['atomic_mass_amu']:.6f} AMU (expect ~235.043930)")
        print(f"  half_life = {u235['half_life_s']:.3e} s (expect ~2.22e16)")
        print(f"  decay = {u235['decay_modes']}")


def main():
    nuclides = fetch_and_parse()
    print_summary(nuclides)

    out_dir = Path(__file__).parent / "data"
    out_dir.mkdir(exist_ok=True)

    output = {
        "source": "IAEA LiveChart of Nuclides (Nuclear Data Services)",
        "url": IAEA_URL,
        "fetch_date": str(date.today()),
        "nuclide_count": len(nuclides),
        "nuclides": nuclides,
    }

    out_path = out_dir / "iaea_ground_states.json"
    with open(out_path, "w") as f:
        json.dump(output, f, indent=2)

    size_kb = out_path.stat().st_size / 1024
    print(f"\nWrote {out_path} ({size_kb:.0f} KB)")

    return 0


if __name__ == "__main__":
    sys.exit(main())
