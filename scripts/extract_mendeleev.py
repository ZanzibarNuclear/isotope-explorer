#!/usr/bin/env python3
"""
Phase 1, Step 1: Extract isotope properties from mendeleev.

Pulls: Z, A, atomic_mass, abundance, half_life, spin, parity, is_radioactive
for all isotopes, plus element metadata for Z=1..118.

Outputs: data/mendeleev_isotopes.json
"""

import json
import math
from pathlib import Path

from mendeleev import element as get_element
from mendeleev.fetch import fetch_table


def extract_isotopes():
    """Extract isotope data from mendeleev's database."""
    # Fetch the full isotopes table via pandas
    isotopes_df = fetch_table("isotopes")
    elements_df = fetch_table("elements")

    print(f"Found {len(isotopes_df)} isotope records")
    print(f"Found {len(elements_df)} element records")

    isotopes = []
    for _, row in isotopes_df.iterrows():
        z = int(row["atomic_number"])
        a = int(row["mass_number"])
        n = a - z

        if z < 1 or z > 118 or n < 0:
            continue

        isotope = {
            "z": z,
            "n": n,
            "a": a,
            "atomic_mass": _safe_float(row.get("atomic_mass")),
            "abundance": _safe_float(row.get("abundance")),
            "half_life_sec": _parse_half_life(row),
            "spin": _safe_float(row.get("spin")),
            "parity": _safe_str(row.get("parity")),
            "is_radioactive": bool(row.get("is_radioactive", False)),
        }
        isotopes.append(isotope)

    # Extract element metadata
    elements = []
    for _, row in elements_df.iterrows():
        z = int(row["atomic_number"])
        elem = {
            "z": z,
            "symbol": row["symbol"],
            "name": row["name"],
            "group_id": _safe_int(row.get("group_id")),
            "period": _safe_int(row.get("period")),
            "block": _safe_str(row.get("block")),
        }
        elements.append(elem)

    return isotopes, elements


def _parse_half_life(row):
    """Extract half-life in seconds from mendeleev data."""
    hl = row.get("half_life")
    if hl is None or (isinstance(hl, float) and math.isnan(hl)):
        return None
    hl_unit = row.get("half_life_unit", "s")
    if hl_unit is None or (isinstance(hl_unit, float) and math.isnan(hl_unit)):
        hl_unit = "s"

    try:
        hl = float(hl)
    except (ValueError, TypeError):
        return None

    # Convert to seconds
    _YEAR = 365.25 * 86400.0
    unit_to_seconds = {
        # SI-prefix style (common in nuclear data)
        "s": 1.0,
        "ms": 1e-3,
        "us": 1e-6,
        "µs": 1e-6,
        "ns": 1e-9,
        "ps": 1e-12,
        "fs": 1e-15,
        "as": 1e-18,
        "zs": 1e-21,
        "ys": 1e-24,
        "m": 60.0,
        "h": 3600.0,
        "d": 86400.0,
        "y": _YEAR,
        "ky": _YEAR * 1e3,
        "My": _YEAR * 1e6,
        "Gy": _YEAR * 1e9,
        "Ty": _YEAR * 1e12,
        "Py": _YEAR * 1e15,
        "Ey": _YEAR * 1e18,
        "Yy": _YEAR * 1e24,
        # Mendeleev singular/long forms
        "sec": 1.0,
        "msec": 1e-3,
        "usec": 1e-6,
        "nsec": 1e-9,
        "psec": 1e-12,
        "fsec": 1e-15,
        "asec": 1e-18,
        "zsec": 1e-21,
        "ysec": 1e-24,
        "minute": 60.0,
        "min": 60.0,
        "hour": 3600.0,
        "day": 86400.0,
        "year": _YEAR,
        "kyear": _YEAR * 1e3,
        "Myear": _YEAR * 1e6,
        "Gyear": _YEAR * 1e9,
        "Tyear": _YEAR * 1e12,
        "Pyear": _YEAR * 1e15,
        "Eyear": _YEAR * 1e18,
        "Yyear": _YEAR * 1e24,
        "Zyear": _YEAR * 1e21,
    }

    factor = unit_to_seconds.get(str(hl_unit), None)
    if factor is None:
        print(f"  Warning: unknown half-life unit '{hl_unit}' for Z={row.get('atomic_number')}, A={row.get('mass_number')}")
        return None

    return hl * factor


def _safe_float(val):
    if val is None:
        return None
    try:
        f = float(val)
        return None if math.isnan(f) or math.isinf(f) else f
    except (ValueError, TypeError):
        return None


def _safe_int(val):
    if val is None:
        return None
    try:
        return int(val)
    except (ValueError, TypeError):
        return None


def _safe_str(val):
    if val is None:
        return None
    s = str(val)
    return None if s == "nan" else s


def main():
    out_dir = Path(__file__).parent / "data"
    out_dir.mkdir(exist_ok=True)

    print("Extracting isotope data from mendeleev...")
    isotopes, elements = extract_isotopes()

    # Summary stats
    stable = sum(1 for i in isotopes if not i["is_radioactive"])
    with_hl = sum(1 for i in isotopes if i["half_life_sec"] is not None)
    print(f"\nExtracted {len(isotopes)} isotopes ({stable} stable, {with_hl} with half-life data)")
    print(f"Extracted {len(elements)} elements")

    output = {
        "source": "mendeleev",
        "isotope_count": len(isotopes),
        "element_count": len(elements),
        "isotopes": isotopes,
        "elements": elements,
    }

    out_path = out_dir / "mendeleev_isotopes.json"
    with open(out_path, "w") as f:
        json.dump(output, f, indent=2)

    print(f"\nWrote {out_path}")


if __name__ == "__main__":
    main()
