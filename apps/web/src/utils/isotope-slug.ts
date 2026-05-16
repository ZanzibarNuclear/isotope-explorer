import { PERIODIC_TABLE } from "../data/periodic-table-layout";

const SYMBOL_TO_Z = new Map(
  PERIODIC_TABLE.map((el) => [el.symbol.toLowerCase(), el.z]),
);

const Z_TO_SYMBOL = new Map(PERIODIC_TABLE.map((el) => [el.z, el.symbol]));

const SLUG_RE = /^([a-z]+)-(\d+)(m|n)?$/i;

export interface ParsedIsotopeSlug {
  z: number;
  n: number;
  /** Canonical path segment, e.g. `c-14`. */
  slug: string;
}

/** Build a URL slug from proton and neutron numbers, e.g. `c-14`. */
export function toIsotopeSlug(z: number, n: number): string | null {
  const symbol = Z_TO_SYMBOL.get(z);
  if (!symbol) return null;
  return `${symbol.toLowerCase()}-${z + n}`;
}

/** Parse a path slug like `c-14` or `u-235` into Z and N. */
export function parseIsotopeSlug(
  slug: string,
): ParsedIsotopeSlug | null {
  const trimmed = slug.trim();
  if (!trimmed) return null;

  const match = trimmed.match(SLUG_RE);
  if (!match) return null;

  const symbolKey = match[1].toLowerCase();
  const z = SYMBOL_TO_Z.get(symbolKey);
  if (z === undefined) return null;

  const massNumber = Number.parseInt(match[2], 10);
  if (!Number.isFinite(massNumber) || massNumber < 1) return null;

  const n = massNumber - z;
  if (n < 0) return null;

  const canonicalSymbol = Z_TO_SYMBOL.get(z);
  if (!canonicalSymbol) return null;

  const metastableSuffix = match[3]?.toLowerCase() ?? "";
  const canonicalSlug = `${canonicalSymbol.toLowerCase()}-${massNumber}${metastableSuffix}`;

  return { z, n, slug: canonicalSlug };
}
