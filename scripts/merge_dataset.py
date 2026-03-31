#!/usr/bin/env python3
"""
Phase 1, Step 4: Merge and curate the extracted datasets.

Combines:
  - mendeleev_isotopes.json (isotope properties, element metadata)
  - decay_data.json (decay modes, branching fractions, daughters)
  - neutron_data.json (cross-sections, fission yields)

Outputs:
  - data/merged_nuclides.json (full merged dataset, all nuclides)
  - ../crates/nuclear-sim/data/nuclide_data.json (curated subset for embedding)

Validates decay chain completeness: every daughter nuclide referenced in a decay
should also be present in the dataset (or flagged).
"""

import json
from collections import defaultdict
from pathlib import Path


SCRIPT_DIR = Path(__file__).parent
DATA_DIR = SCRIPT_DIR / "data"


# Decay modes we model in our Rust simulation
SUPPORTED_DECAY_MODES = {
    "Alpha", "BetaMinus", "BetaPlus", "ElectronCapture", "IsomericTransition",
}

# Which Z values are actinides
ACTINIDE_RANGE = range(89, 104)  # Ac through Lr

# Notable nuclides to always include
ALWAYS_INCLUDE = {
    (1, 0),    # H-1
    (1, 1),    # H-2
    (1, 2),    # H-3
    (2, 2),    # He-4 (alpha particle)
    (6, 6),    # C-12
    (6, 8),    # C-14
    (7, 7),    # N-14
    (8, 8),    # O-16
    (26, 30),  # Fe-56
    (27, 33),  # Co-60
    (28, 32),  # Ni-60
    (36, 56),  # Kr-92
    (38, 52),  # Sr-90
    (53, 78),  # I-131
    (54, 81),  # Xe-135
    (55, 82),  # Cs-137
    (56, 82),  # Ba-138
    (62, 87),  # Sm-149
    (82, 126), # Pb-208
    (83, 126), # Bi-209
    (90, 142), # Th-232
    (92, 141), # U-233
    (92, 143), # U-235
    (92, 146), # U-238
    (93, 144), # Np-237
    (94, 145), # Pu-239
    (94, 147), # Pu-241
    (95, 146), # Am-241
}


def load_mendeleev():
    with open(DATA_DIR / "mendeleev_isotopes.json") as f:
        data = json.load(f)

    by_zn = {}
    for iso in data["isotopes"]:
        key = (iso["z"], iso["n"])
        by_zn[key] = iso

    elements = {}
    for elem in data["elements"]:
        elements[elem["z"]] = elem

    return by_zn, elements


def load_decays():
    with open(DATA_DIR / "decay_data.json") as f:
        data = json.load(f)

    by_zn = {}
    for nuc in data["nuclides"]:
        key = (nuc["z"], nuc["n"])
        # Skip metastable states for now (use ground state)
        if nuc["is_metastable"]:
            continue
        by_zn[key] = nuc

    return by_zn


def load_neutron_data():
    with open(DATA_DIR / "neutron_data.json") as f:
        data = json.load(f)

    xs_by_zn = {}
    for xs in data["cross_sections"]:
        key = (xs["z"], xs["n"])
        xs_by_zn[key] = xs

    yields_by_zn = {}
    for key_str, fy in data["fission_yields"].items():
        key = (fy["z"], fy["n"])
        yields_by_zn[key] = fy

    return xs_by_zn, yields_by_zn


def merge_nuclide(zn, mendeleev_data, decay_data, xs_data, fy_data):
    """Merge data from all sources for a single nuclide (Z, N)."""
    z, n = zn
    a = z + n

    entry = {
        "z": z,
        "n": n,
        "a": a,
    }

    # From mendeleev: mass, abundance, stability
    md = mendeleev_data.get(zn)
    if md:
        entry["atomic_mass"] = md.get("atomic_mass")
        entry["abundance"] = md.get("abundance")
        entry["is_radioactive"] = md.get("is_radioactive", False)
        entry["half_life_sec"] = md.get("half_life_sec")
    else:
        entry["atomic_mass"] = None
        entry["abundance"] = None
        entry["is_radioactive"] = True  # assume radioactive if not in mendeleev
        entry["half_life_sec"] = None

    # From radioactivedecay: decay modes, branching fractions, daughters
    dd = decay_data.get(zn)
    if dd:
        # Prefer radioactivedecay half-life (ICRP-107 is authoritative for radioactive nuclides)
        if dd.get("half_life_sec") is not None:
            entry["half_life_sec"] = dd["half_life_sec"]
        if dd.get("is_stable"):
            entry["is_radioactive"] = False

        branches = []
        for br in dd.get("decay_branches", []):
            mode = br["mode"]
            branches.append({
                "mode": mode,
                "fraction": br["fraction"],
                "daughter_z": br.get("daughter_z"),
                "daughter_n": br.get("daughter_n"),
            })
        entry["decay_branches"] = branches
    else:
        entry["decay_branches"] = []

    # Determine stability
    entry["is_stable"] = not entry["is_radioactive"]

    # Fissile flag: has fission cross-section > 1 barn at thermal energies
    xs = xs_data.get(zn)
    if xs:
        entry["cross_sections"] = {
            "thermal": xs.get("thermal"),
            "fast_1MeV": xs.get("fast_1MeV"),
        }
        thermal_fission = xs.get("thermal", {}).get("fission", 0)
        entry["is_fissile"] = thermal_fission > 1.0
        entry["nu_bar_thermal"] = xs.get("nu_bar_thermal")
        entry["nu_bar_fast"] = xs.get("nu_bar_fast")
    else:
        entry["cross_sections"] = None
        entry["is_fissile"] = False
        entry["nu_bar_thermal"] = None
        entry["nu_bar_fast"] = None

    # Fission products
    fy = fy_data.get(zn)
    if fy:
        entry["fission_products"] = fy.get("product_pairs", [])
    else:
        entry["fission_products"] = []

    return entry


def select_curated_set(mendeleev_data, decay_data, xs_data, fy_data):
    """Select the ~200-500 nuclide subset for embedding in the WASM binary."""
    selected = set()

    # 1. Always-include list
    selected.update(ALWAYS_INCLUDE)

    # 2. All actinides present in either source
    all_zn = set(mendeleev_data.keys()) | set(decay_data.keys())
    for z, n in all_zn:
        if z in ACTINIDE_RANGE:
            selected.add((z, n))

    # 3. All nuclides with cross-section data
    selected.update(xs_data.keys())

    # 4. All nuclides with fission yield data (as parent or product)
    selected.update(fy_data.keys())
    for fy in fy_data.values():
        for pp in fy.get("product_pairs", []):
            selected.add((pp["light_z"], pp["light_a"] - pp["light_z"]))
            selected.add((pp["heavy_z"], pp["heavy_a"] - pp["heavy_z"]))

    # 5. Stable isotopes of common elements (for fission product chains)
    common_elements = list(range(1, 63))  # H through Sm
    for z, n in all_zn:
        if z in common_elements:
            md = mendeleev_data.get((z, n))
            if md and not md.get("is_radioactive", True):
                selected.add((z, n))

    # 6. Follow decay chains: ensure every daughter is included
    iterations = 0
    while iterations < 20:
        new_daughters = set()
        for zn in selected:
            dd = decay_data.get(zn)
            if not dd:
                continue
            for br in dd.get("decay_branches", []):
                dz = br.get("daughter_z")
                dn = br.get("daughter_n")
                if dz is not None and dn is not None:
                    daughter = (dz, dn)
                    if daughter not in selected:
                        new_daughters.add(daughter)

        if not new_daughters:
            break
        selected.update(new_daughters)
        iterations += 1

    # Filter to only nuclides that exist in at least one data source
    valid = set()
    for zn in selected:
        if zn in mendeleev_data or zn in decay_data:
            valid.add(zn)
        else:
            # Check if it's a valid nuclide (Z >= 1, N >= 0)
            z, n = zn
            if z >= 1 and n >= 0:
                valid.add(zn)

    return valid


def validate_decay_chains(dataset):
    """Check that all daughter nuclides in decay branches exist in the dataset."""
    by_zn = {(d["z"], d["n"]): d for d in dataset}
    missing = []

    for entry in dataset:
        for br in entry.get("decay_branches", []):
            dz = br.get("daughter_z")
            dn = br.get("daughter_n")
            if dz is not None and dn is not None:
                if (dz, dn) not in by_zn:
                    missing.append({
                        "parent": f"Z={entry['z']}, N={entry['n']}",
                        "daughter": f"Z={dz}, N={dn}",
                        "mode": br["mode"],
                    })

    return missing


def generate_rust_json(dataset, elements):
    """
    Generate the JSON structure optimized for Rust consumption.
    Keyed by "Z_N" string for easy HashMap loading.
    """
    rust_data = {
        "metadata": {
            "sources": [
                "mendeleev (isotope properties, atomic masses)",
                "radioactivedecay/ICRP-107 (decay modes, branching fractions)",
                "ENDF/B-VIII.0 reference values (cross-sections, fission yields)",
            ],
            "nuclide_count": len(dataset),
            "generated_by": "scripts/merge_dataset.py",
        },
        "nuclides": {},
    }

    for entry in sorted(dataset, key=lambda d: (d["z"], d["n"])):
        key = f"{entry['z']}_{entry['n']}"

        # Convert decay branches to Rust-friendly format
        decay_modes = []
        for br in entry.get("decay_branches", []):
            if br["mode"] in SUPPORTED_DECAY_MODES:
                decay_modes.append({
                    "mode": br["mode"],
                    "fraction": br["fraction"],
                })

        # Normalize fractions for supported modes only
        total = sum(dm["fraction"] for dm in decay_modes)
        if total > 0 and abs(total - 1.0) > 0.001:
            for dm in decay_modes:
                dm["fraction"] = dm["fraction"] / total

        # Convert fission products
        fission_products = []
        for fp in entry.get("fission_products", []):
            fission_products.append({
                "light_z": fp["light_z"],
                "light_n": fp["light_a"] - fp["light_z"],
                "heavy_z": fp["heavy_z"],
                "heavy_n": fp["heavy_a"] - fp["heavy_z"],
                "neutrons_released": fp["neutrons"],
                "fraction": fp["yield"],
            })

        # Normalize fission product fractions
        fp_total = sum(fp["fraction"] for fp in fission_products)
        if fp_total > 0 and abs(fp_total - 1.0) > 0.001:
            for fp in fission_products:
                fp["fraction"] = fp["fraction"] / fp_total

        nuclide = {
            "z": entry["z"],
            "n": entry["n"],
            "is_stable": entry.get("is_stable", False),
            "half_life_s": entry.get("half_life_sec"),
            "decay_modes": decay_modes,
            "is_fissile": entry.get("is_fissile", False),
            "fission_products": fission_products,
            "atomic_mass": entry.get("atomic_mass"),
            "abundance": entry.get("abundance"),
        }

        rust_data["nuclides"][key] = nuclide

    return rust_data


def main():
    print("Loading extracted datasets...")
    mendeleev_data, elements = load_mendeleev()
    decay_data = load_decays()
    xs_data, fy_data = load_neutron_data()

    print(f"  mendeleev: {len(mendeleev_data)} isotopes, {len(elements)} elements")
    print(f"  radioactivedecay: {len(decay_data)} nuclides")
    print(f"  neutron cross-sections: {len(xs_data)} isotopes")
    print(f"  fission yields: {len(fy_data)} fissile isotopes")

    # Select curated subset
    print("\nSelecting curated nuclide set...")
    curated_zns = select_curated_set(mendeleev_data, decay_data, xs_data, fy_data)
    print(f"  Selected {len(curated_zns)} nuclides")

    # Merge data for each selected nuclide
    print("\nMerging data...")
    dataset = []
    for zn in sorted(curated_zns):
        entry = merge_nuclide(zn, mendeleev_data, decay_data, xs_data, fy_data)
        dataset.append(entry)

    # Stats
    stable = sum(1 for d in dataset if d.get("is_stable"))
    radioactive = sum(1 for d in dataset if not d.get("is_stable"))
    fissile = sum(1 for d in dataset if d.get("is_fissile"))
    with_decays = sum(1 for d in dataset if d.get("decay_branches"))
    with_xs = sum(1 for d in dataset if d.get("cross_sections"))
    with_fp = sum(1 for d in dataset if d.get("fission_products"))
    print(f"\n  {len(dataset)} total nuclides")
    print(f"  {stable} stable, {radioactive} radioactive")
    print(f"  {fissile} fissile")
    print(f"  {with_decays} with decay mode data")
    print(f"  {with_xs} with cross-section data")
    print(f"  {with_fp} with fission product data")

    # Validate decay chains
    print("\nValidating decay chain completeness...")
    missing = validate_decay_chains(dataset)
    if missing:
        print(f"  WARNING: {len(missing)} missing daughter nuclides:")
        for m in missing[:20]:
            print(f"    {m['parent']} --{m['mode']}--> {m['daughter']}")
        if len(missing) > 20:
            print(f"    ... and {len(missing) - 20} more")
    else:
        print("  All decay chains complete!")

    # Write full merged dataset
    merged_path = DATA_DIR / "merged_nuclides.json"
    with open(merged_path, "w") as f:
        json.dump({"nuclide_count": len(dataset), "nuclides": dataset}, f, indent=2)
    print(f"\nWrote full merged dataset: {merged_path}")

    # Generate Rust-optimized JSON
    rust_json = generate_rust_json(dataset, elements)

    rust_data_dir = SCRIPT_DIR.parent / "crates" / "nuclear-sim" / "data"
    rust_data_dir.mkdir(exist_ok=True)
    rust_path = rust_data_dir / "nuclide_data.json"
    with open(rust_path, "w") as f:
        json.dump(rust_json, f, indent=2)
    print(f"Wrote Rust dataset: {rust_path} ({len(rust_json['nuclides'])} nuclides)")

    # Size report
    import os
    size_kb = os.path.getsize(rust_path) / 1024
    print(f"  JSON size: {size_kb:.1f} KB")


if __name__ == "__main__":
    main()
