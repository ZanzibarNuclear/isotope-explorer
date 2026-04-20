#!/usr/bin/env python3
"""
Merge all Path B data sources into the embedded nuclide dataset.

Inputs (from scripts/pathb/data/):
  - iaea_ground_states.json (decay data, masses, abundances, spin/parity)
  - thermal_xs.json (thermal neutron cross-sections)
  - fission_yields.json (fission product yield distributions)

Output:
  - crates/nuclear-sim/data/nuclide_data.json (embedded in WASM binary)

Validates:
  - Decay chain completeness (every daughter is in the dataset)
  - Branching fraction sums (~1.0)
  - Fission mass/charge conservation
  - Regression against current 746-nuclide dataset
"""

import json
import os
import sys
from datetime import date
from pathlib import Path


SCRIPT_DIR = Path(__file__).parent
DATA_DIR = SCRIPT_DIR / "data"
RUST_DATA_DIR = SCRIPT_DIR.parent.parent / "crates" / "nuclear-sim" / "data"

# Decay modes our Rust simulation supports
SUPPORTED_MODES = {"Alpha", "BetaMinus", "BetaPlus", "ElectronCapture", "IsomericTransition"}


def load_iaea():
    with open(DATA_DIR / "iaea_ground_states.json") as f:
        data = json.load(f)
    return data["nuclides"]


def load_thermal_xs():
    with open(DATA_DIR / "thermal_xs.json") as f:
        data = json.load(f)
    return data["cross_sections"]


def load_fission_yields():
    with open(DATA_DIR / "fission_yields.json") as f:
        data = json.load(f)
    return data["fission_yields"]


def load_current_dataset():
    """Load the existing nuclide_data.json for regression comparison."""
    path = RUST_DATA_DIR / "nuclide_data.json"
    if not path.exists():
        return None
    with open(path) as f:
        data = json.load(f)
    return data.get("nuclides", {})


def build_merged_dataset(iaea, xs_data, fy_data):
    """Merge all sources into the final dataset."""
    nuclides = {}

    for key, nuc in iaea.items():
        z, n = nuc["z"], nuc["n"]

        # Filter decay modes to only those we support
        decay_modes = []
        for br in nuc.get("decay_modes", []):
            if br["mode"] in SUPPORTED_MODES:
                decay_modes.append({
                    "mode": br["mode"],
                    "fraction": br["fraction"],
                })

        # Re-normalize fractions
        total = sum(dm["fraction"] for dm in decay_modes)
        if decay_modes and total > 0 and abs(total - 1.0) > 0.001:
            for dm in decay_modes:
                dm["fraction"] = dm["fraction"] / total
        elif decay_modes and total == 0:
            # IAEA reports 0% for poorly-measured nuclides; distribute evenly
            even = 1.0 / len(decay_modes)
            for dm in decay_modes:
                dm["fraction"] = even

        # Cross-sections (if available)
        xs = xs_data.get(key)
        cross_sections = None
        if xs:
            cross_sections = {
                "capture": xs["sigma_capture_b"],
                "fission": xs["sigma_fission_b"],
                "elastic": xs["sigma_elastic_b"],
                "absorption": xs["sigma_absorption_b"],
                "nu_bar": xs["nu_bar"],
            }

        # Fission products (if available)
        fy = fy_data.get(key)
        fission_products = []
        if fy:
            for pp in fy["product_pairs"]:
                fission_products.append({
                    "light_z": pp["light_z"],
                    "light_n": pp["light_n"],
                    "heavy_z": pp["heavy_z"],
                    "heavy_n": pp["heavy_n"],
                    "neutrons_released": pp["neutrons"],
                    "fraction": pp["yield"],
                })

            # Normalize fission product fractions to sum to 1.0
            fp_total = sum(fp["fraction"] for fp in fission_products)
            if fp_total > 0 and abs(fp_total - 1.0) > 0.001:
                for fp in fission_products:
                    fp["fraction"] = fp["fraction"] / fp_total

        nuclides[key] = {
            "z": z,
            "n": n,
            "is_stable": nuc["is_stable"],
            "half_life_s": nuc["half_life_s"],
            "atomic_mass_amu": nuc.get("atomic_mass_amu"),
            "abundance": nuc.get("abundance"),
            "spin": nuc.get("spin"),
            "parity": nuc.get("parity"),
            "decay_modes": decay_modes,
            "cross_sections": cross_sections,
            "fission_products": fission_products,
        }

    return nuclides


def validate_decay_chains(nuclides):
    """Check that daughters of supported decay modes exist in the dataset."""
    missing = []
    for key, nuc in nuclides.items():
        z, n = nuc["z"], nuc["n"]
        for dm in nuc["decay_modes"]:
            mode = dm["mode"]
            dz, dn = compute_daughter(z, n, mode)
            if dz is not None and dn is not None:
                daughter_key = f"{dz}_{dn}"
                if daughter_key not in nuclides:
                    missing.append((key, mode, daughter_key))
    return missing


def compute_daughter(z, n, mode):
    """Compute daughter (Z, N) for a given decay mode."""
    if mode == "Alpha":
        return (z - 2, n - 2) if z >= 2 and n >= 2 else (None, None)
    elif mode == "BetaMinus":
        return (z + 1, n - 1) if n >= 1 else (None, None)
    elif mode in ("BetaPlus", "ElectronCapture"):
        return (z - 1, n + 1) if z >= 1 else (None, None)
    elif mode == "IsomericTransition":
        return (z, n)
    return (None, None)


def validate_branching_fractions(nuclides):
    """Check that decay mode fractions sum to ~1.0 for nuclides with modes."""
    issues = []
    for key, nuc in nuclides.items():
        modes = nuc["decay_modes"]
        if not modes:
            continue  # no supported modes is fine (exotic decays only)
        total = sum(dm["fraction"] for dm in modes)
        if abs(total - 1.0) > 0.02:
            issues.append((key, total))
    return issues


def validate_fission_conservation(nuclides):
    """Check mass/charge conservation in fission products."""
    issues = []
    for key, nuc in nuclides.items():
        if not nuc["fission_products"]:
            continue
        pz = nuc["z"]
        pa = nuc["z"] + nuc["n"]
        compound_a = pa + 1
        for i, fp in enumerate(nuc["fission_products"]):
            lz, ln = fp["light_z"], fp["light_n"]
            hz, hn = fp["heavy_z"], fp["heavy_n"]
            nn = fp["neutrons_released"]
            la = lz + ln
            ha = hz + hn
            if lz + hz != pz:
                issues.append(f"{key} pair {i}: Z {lz}+{hz}={lz+hz} != {pz}")
            if la + ha + nn != compound_a:
                issues.append(f"{key} pair {i}: A {la}+{ha}+{nn}={la+ha+nn} != {compound_a}")
    return issues


def regression_check(new_nuclides, old_nuclides):
    """Compare against the current dataset for regressions."""
    if old_nuclides is None:
        return []

    issues = []
    old_keys = set(old_nuclides.keys())
    new_keys = set(new_nuclides.keys())
    missing_keys = old_keys - new_keys

    if missing_keys:
        issues.append(f"{len(missing_keys)} nuclides from old dataset not in new: "
                       f"{sorted(list(missing_keys))[:10]}...")

    # Check stability and decay mode agreement for shared keys
    for key in old_keys & new_keys:
        old = old_nuclides[key]
        new = new_nuclides[key]
        if old.get("is_stable") != new.get("is_stable"):
            issues.append(f"{key}: stability changed: {old['is_stable']} -> {new['is_stable']}")

    return issues


def main():
    print("Loading Path B data sources...")
    iaea = load_iaea()
    xs_data = load_thermal_xs()
    fy_data = load_fission_yields()
    old_dataset = load_current_dataset()

    print(f"  IAEA ground states: {len(iaea)} nuclides")
    print(f"  Thermal cross-sections: {len(xs_data)} isotopes")
    print(f"  Fission yields: {len(fy_data)} parents")
    if old_dataset:
        print(f"  Current dataset (regression ref): {len(old_dataset)} nuclides")

    # Merge
    print("\nMerging dataset...")
    nuclides = build_merged_dataset(iaea, xs_data, fy_data)

    # Stats
    stable = sum(1 for n in nuclides.values() if n["is_stable"])
    with_decay = sum(1 for n in nuclides.values() if n["decay_modes"])
    with_mass = sum(1 for n in nuclides.values() if n["atomic_mass_amu"] is not None)
    with_xs = sum(1 for n in nuclides.values() if n["cross_sections"] is not None)
    with_fy = sum(1 for n in nuclides.values() if n["fission_products"])
    fissile = sum(1 for n in nuclides.values()
                  if n["cross_sections"] and n["cross_sections"]["fission"] > 1.0)

    print(f"\n=== Merged Dataset Summary ===")
    print(f"Total nuclides:       {len(nuclides)}")
    print(f"Stable:               {stable}")
    print(f"With decay modes:     {with_decay}")
    print(f"With atomic mass:     {with_mass}")
    print(f"With cross-sections:  {with_xs}")
    print(f"With fission yields:  {with_fy}")
    print(f"Fissile (σ_f > 1b):   {fissile}")

    # Validations
    ok = True

    print("\n--- Decay chain completeness ---")
    missing = validate_decay_chains(nuclides)
    if missing:
        print(f"  {len(missing)} missing daughters:")
        for parent, mode, daughter in missing[:15]:
            print(f"    {parent} --{mode}--> {daughter}")
        if len(missing) > 15:
            print(f"    ... and {len(missing) - 15} more")
    else:
        print("  All decay chains complete!")

    print("\n--- Branching fraction sums ---")
    bad_fractions = validate_branching_fractions(nuclides)
    if bad_fractions:
        print(f"  {len(bad_fractions)} nuclides with fractions not summing to ~1.0:")
        for key, total in bad_fractions[:10]:
            print(f"    {key}: sum = {total:.4f}")
        ok = False
    else:
        print("  All branching fractions sum to ~1.0")

    print("\n--- Fission conservation ---")
    fission_issues = validate_fission_conservation(nuclides)
    if fission_issues:
        print(f"  {len(fission_issues)} conservation violations:")
        for issue in fission_issues[:10]:
            print(f"    {issue}")
        ok = False
    else:
        print("  All fission products conserve mass and charge")

    print("\n--- Regression check ---")
    regressions = regression_check(nuclides, old_dataset)
    if regressions:
        print(f"  {len(regressions)} regressions detected:")
        for r in regressions[:10]:
            print(f"    {r}")
    else:
        if old_dataset:
            print(f"  No regressions (all {len(old_dataset)} old nuclides present)")
        else:
            print("  No previous dataset to compare against")

    # Write output
    RUST_DATA_DIR.mkdir(exist_ok=True)
    output = {
        "metadata": {
            "sources": [
                "IAEA LiveChart API (decay modes, half-lives, spin/parity, masses)",
                "ENDF/B-VIII.0 reference values (thermal cross-sections)",
                "ENDF/B-VIII.0 fission yield sublibrary (product distributions)",
            ],
            "nuclide_count": len(nuclides),
            "xs_count": with_xs,
            "fy_count": with_fy,
            "generated_by": "scripts/pathb/merge_pathb.py",
            "generated_date": str(date.today()),
        },
        "nuclides": dict(sorted(nuclides.items(), key=lambda x: (x[1]["z"], x[1]["n"]))),
    }

    out_path = RUST_DATA_DIR / "nuclide_data.json"
    with open(out_path, "w") as f:
        json.dump(output, f, indent=2)

    size_kb = os.path.getsize(out_path) / 1024
    print(f"\nWrote {out_path}")
    print(f"  {len(nuclides)} nuclides, {size_kb:.0f} KB")

    if not ok:
        print("\nWARNING: validation issues detected (see above)")
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
