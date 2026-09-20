"""Import an ESTV "Tarife" export into Rust base-scale data.

Input : an .xlsx exported from the ESTV tax calculator's "Tarife" section, e.g.
        `estv_scales_AG.xlsx`.
Output: a Rust module with one `BaseScale` per canton found in the file.

The ESTV format
---------------
Header row (German export):

    Kantons-Id | Kanton | Steuerart | Steuersubjekt | Steuerhoheit |
    Für die nächsten CHF | Zusätzlich %

Crucially, **"Für die nächsten CHF" is a band WIDTH, not a threshold**, and
"Zusätzlich %" is that band's marginal rate. The simple tax is therefore built by
accumulating CHF amounts:

    for each band, in order:
        chunk = min(remaining_income, band_width)
        simple_tax += round(chunk * band_rate_percent / 100)
        remaining_income -= chunk

An optional `Splittingfaktor` near the top means the canton uses income
splitting: taxable income is divided by the factor, the scale applied, and the
result multiplied back.

This is converted into the project's `FederalBracket` shape (a *threshold* plus a
rate as a fraction) by accumulating the widths. The conversion is exact for the
marginal structure, so `tax_with_scale` reproduces the same tax as the
accumulation loop — asserted by the generated tests.
"""

import os
import re
import sys
import importlib.util

# Load the sibling xlsx reader by path: `tools` is not an installed package.
_HERE = os.path.dirname(os.path.abspath(__file__))
_spec = importlib.util.spec_from_file_location(
    "xlsx_dump", os.path.join(_HERE, "xlsx_dump.py")
)
_xlsx_dump = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(_xlsx_dump)
read_xlsx = _xlsx_dump.read_xlsx


def clean(text):
    """Normalise header text: strip, collapse spaces, drop a UTF-8 BOM."""
    if text is None:
        return ""
    text = str(text).replace("\ufeff", "")
    return re.sub(r"\s+", " ", text).strip()


def to_float(text):
    """Parse a number that may use a comma decimal separator, else None."""
    text = clean(text)
    if text == "":
        return None
    try:
        return float(text.replace("'", "").replace(",", "."))
    except ValueError:
        return None


def find_header(rows):
    """Locate the header row and return (index, {column name: index})."""
    for i, row in enumerate(rows):
        cells = [clean(c) for c in row]
        if "Kanton" in cells and "Steuerhoheit" in cells:
            return i, {name: idx for idx, name in enumerate(cells) if name}
    return None, {}


def find_splitting_factor(rows, header_idx):
    """Read the Splittingfaktor from above the header, if present."""
    for row in rows[:header_idx]:
        cells = [clean(c) for c in row]
        for idx, cell in enumerate(cells):
            if cell.lower().startswith("splittingfaktor"):
                for later in cells[idx + 1:]:
                    value = to_float(later)
                    if value is not None:
                        return value
    return None


def classify_subject(text):
    """Map a `Steuersubjekt` value to 'single', 'married' or 'all'.

    The ESTV exports use a small vocabulary here:
      * "Alle"                                 -> one scale serves everyone
      * "Alleinstehend ohne Kinder"            -> single
      * "Verheiratet / Alleinstehend mit Kind" -> married

    These two spellings come from real exports and are documented in
    SWISS_TAX_DATA.md. Note the married label deliberately also covers single
    parents with children, which is how the cantons treat them.
    """
    low = text.lower()
    if "alle" == low.strip():
        return "all"
    if "alleinstehend ohne" in low or low.startswith("ledig"):
        return "single"
    if "verheiratet" in low:
        return "married"
    return "unknown"


def find_columns(header_row):
    """Locate the ESTV export's columns by substring.

    Exact header matching is unreliable: the same field is emitted with
    different accent handling between exports (`Fuer die naechsten CHF` appears
    with a replacement character in some files), so a literal lookup raises
    KeyError on otherwise valid data.

    `Kanton` must be matched with `id` excluded, otherwise `Kantons-Id` wins and
    the numeric canton id (19, 12, 1) is returned where a two-letter code is
    expected.
    """
    names = [clean(c) for c in header_row]

    def find(*needles, fallback=None, exclude=()):
        for idx, name in enumerate(names):
            low = name.lower()
            if any(x.lower() in low for x in exclude):
                continue
            if all(n.lower() in low for n in needles):
                return idx
        return fallback

    return {
        "Kanton": find("kanton", exclude=("id",), fallback=1),
        "Steuerart": find("steuerart", fallback=2),
        "Steuersubjekt": find("steuersubjekt", fallback=3),
        "Steuerhoheit": find("steuerhoheit", fallback=4),
        "Für die nächsten CHF": find("chf", fallback=5),
        "Zusätzlich %": find("zus", fallback=6),
    }


def parse_cantons(src):
    """Return {canton_code: {subject_class: [(width, rate_percent, label)]}} plus factors."""
    out = {}
    factors = {}

    for sheet_name, rows in read_xlsx(src):
        header_idx, _ = find_header(rows)
        if header_idx is None:
            continue

        columns = find_columns(rows[header_idx])
        factor = find_splitting_factor(rows, header_idx)
        width_col = columns["Für die nächsten CHF"]
        rate_col = columns["Zusätzlich %"]

        # A canton publishes EITHER one "Alle" scale (marital difference is then
        # expressed through the splitting factor) OR separate scales per
        # Steuersubjekt (marital difference is then already in the scale).
        # Applying splitting on top of an already-married-specific scale would
        # double-count the marital adjustment, so the two axes are kept apart.
        for row in rows[header_idx + 1:]:
            if len(row) <= max(columns.values()):
                row = list(row) + [""] * (max(columns.values()) + 1 - len(row))

            code = clean(row[columns["Kanton"]]).upper()
            steuerart = clean(row[columns["Steuerart"]])
            authority = clean(row[columns["Steuerhoheit"]])
            subject = clean(row[columns["Steuersubjekt"]])

            if not code or len(code) != 2:
                continue
            if steuerart and steuerart.lower() != "einkommen":
                continue
            if authority and authority.lower() not in ("kanton", "kantonssteuer"):
                continue

            width = to_float(row[width_col])
            rate = to_float(row[rate_col])
            if width is None or rate is None:
                continue

            kind = classify_subject(subject)
            out.setdefault(code, {}).setdefault(kind, []).append((width, rate, subject))
            factors[code] = factor

    return out, factors


def bands_to_thresholds(bands):
    """Convert band widths into (threshold, rate_fraction) pairs.

    A width band means: the first `w1` CHF are taxed at `r1`, the next `w2` at
    `r2`, and so on. That is exactly a threshold schedule, so the widths
    accumulate into thresholds.
    """
    out = []
    threshold = 0.0
    for width, rate_percent, _subject in bands:
        out.append((threshold, rate_percent / 100.0))
        threshold += width
    return out


def main():
    # Usage: import_estv_scales.py <out.rs> <file.xlsx> [more.xlsx ...]
    # Accepting several files lets one run cover a batch of single-canton
    # exports, and the merge is first-wins so re-running with the same inputs is
    # idempotent.
    if len(sys.argv) < 3:
        raise SystemExit(
            "usage: import_estv_scales.py <out.rs> <file.xlsx> [more.xlsx ...]"
        )
    out_path = sys.argv[1]
    srcs = sys.argv[2:]

    cantons = {}
    factors = {}
    contributing = []
    for src in srcs:
        parsed, parsed_factors = parse_cantons(src)
        if not parsed:
            print(f"  {os.path.basename(src)}: no cantonal income bands found")
            continue
        contributing.append(os.path.basename(src))
        for code, kinds in parsed.items():
            if code in cantons:
                print(f"  {os.path.basename(src)}: {code} already imported, keeping first")
                continue
            cantons[code] = kinds
            factors[code] = parsed_factors.get(code)

    if not cantons:
        raise SystemExit(f"no cantonal income bands found in {srcs}")

    results = []
    for code, by_kind in sorted(cantons.items()):
        single = by_kind.get("single")
        married = by_kind.get("married")
        shared = by_kind.get("all")

        if shared and not (single or married):
            # One scale for everyone: marital difference is expressed through the
            # splitting factor, applied only for married taxpayers.
            single_bands = married_bands = shared
            applies_for_married = factors.get(code)
        elif single and married:
            # Separate scales already encode the marital difference, so splitting
            # must NOT be applied on top or it would be counted twice.
            single_bands, married_bands = single, married
            applies_for_married = None
        elif single and not married:
            single_bands = married_bands = single
            applies_for_married = None
        elif married and not single:
            single_bands = married_bands = married
            applies_for_married = None
        else:
            print(f"  SKIP {code}: no usable Steuersubjekt classification")
            continue

        subjects = sorted({s for bands in by_kind.values() for _w, _r, s in bands})
        results.append(
            {
                "code": code,
                "single": single_bands,
                "married": married_bands,
                "splitting_for_married": applies_for_married,
                "subjects": subjects,
                "shared_scale": bool(shared and not (single or married)),
            }
        )

    if not results:
        raise SystemExit(f"no cantonal income bands found in {src}")

    lines = []
    lines.append("// Life Optimizer")
    lines.append("// Copyright (C) 2026 MILAN NIKOLIC")
    lines.append("//")
    lines.append("// This program is free software: you can redistribute it and/or modify")
    lines.append("// it under the terms of the GNU General Public License as published by")
    lines.append("// the Free Software Foundation, either version 3 of the License, or")
    lines.append("// (at your option) any later version.")
    lines.append("//")
    lines.append("// This program is distributed in the hope that it will be useful,")
    lines.append("// but WITHOUT ANY WARRANTY; without even the implied warranty of")
    lines.append("// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the")
    lines.append("// GNU General Public License for more details.")
    lines.append("//")
    lines.append("// You should have received a copy of the GNU General Public License")
    lines.append("// along with this program.  If not, see <https://www.gnu.org/licenses/>.")
    lines.append("")
    lines.append("//! Cantonal simple-tax scales imported from ESTV \"Tarife\" exports.")
    lines.append("//!")
    lines.append("//! GENERATED by `tools/import_estv_scales.py` -- do not edit by hand;")
    lines.append("//! re-run the importer instead.")
    lines.append("//!")
    lines.append(f"//! Source: ESTV tax calculator Tarife exports ({len(contributing)} files):")
    for name in contributing:
        lines.append(f"//!   * `{name}`")
    lines.append("//!")
    lines.append("//! `threshold` is the income level at which the band begins and `rate` is")
    lines.append("//! that band's marginal rate as a fraction. The ESTV export expresses the")
    lines.append("//! same schedule as band *widths* (\"Für die nächsten CHF\"); the importer")
    lines.append("//! accumulates those widths into thresholds, which is an exact conversion")
    lines.append("//! for the marginal structure.")
    lines.append("//!")
    lines.append("//! # Marital treatment is expressed in two different ways")
    lines.append("//!")
    lines.append("//! Cantons differ, and the two axes must not be applied together:")
    lines.append("//!")
    lines.append("//! * A canton publishing ONE scale (`Steuersubjekt = Alle`) expresses the")
    lines.append("//!   marital difference through the **splitting factor**, which applies")
    lines.append("//!   only to married taxpayers.")
    lines.append("//! * A canton publishing SEPARATE scales per `Steuersubjekt` has already")
    lines.append("//!   encoded the difference in the scale, so the splitting factor is")
    lines.append("//!   `None` here and must not be applied on top.")
    lines.append("")
    lines.append("use crate::federal_tax::FederalBracket;")
    lines.append("")
    lines.append("/// One canton's imported simple-tax scale.")
    lines.append("#[derive(Debug, Clone, Copy)]")
    lines.append("pub struct BaseScale {")
    lines.append("    pub canton_code: &'static str,")
    lines.append("    /// Income-splitting divisor for MARRIED taxpayers, applied only when the")
    lines.append("    /// canton publishes a single shared scale. `None` when marital status is")
    lines.append("    /// already encoded in the separate `single`/`married` scales.")
    lines.append("    pub splitting_factor_married: Option<f64>,")
    lines.append("    /// Marginal bands for a single taxpayer.")
    lines.append("    pub single: &'static [FederalBracket],")
    lines.append("    /// Marginal bands for a married taxpayer.")
    lines.append("    pub married: &'static [FederalBracket],")
    lines.append("    /// True when the canton publishes one scale for all taxpayers.")
    lines.append("    pub shared_scale: bool,")
    lines.append("    /// The `Steuersubjekt` values present in the export, so an unexpected")
    lines.append("    /// subject split is visible rather than silently merged.")
    lines.append("    pub subjects: &'static [&'static str],")
    lines.append("}")
    lines.append("")
    lines.append("impl BaseScale {")
    lines.append("    /// The bands that apply to the given marital status.")
    lines.append("    pub fn brackets(&self, married: bool) -> &'static [FederalBracket] {")
    lines.append("        if married { self.married } else { self.single }")
    lines.append("    }")
    lines.append("")
    lines.append("    /// The splitting factor to apply, which is only ever non-`None` for a")
    lines.append("    /// married taxpayer on a shared scale.")
    lines.append("    pub fn splitting_factor(&self, married: bool) -> Option<f64> {")
    lines.append("        if married { self.splitting_factor_married } else { None }")
    lines.append("    }")
    lines.append("}")
    lines.append("")

    for r in results:
        for kind in ("single", "married"):
            ident = f"{r['code']}_{kind.upper()}_BRACKETS"
            lines.append(
                f"/// Simple-tax scale for canton {r['code']}, {kind} taxpayer."
            )
            if r["shared_scale"]:
                lines.append("/// The canton publishes one shared scale; this is the same table")
                lines.append("/// for both marital statuses.")
            lines.append(f"pub const {ident}: &[FederalBracket] = &[")
            for threshold, rate in bands_to_thresholds(r[kind]):
                # `repr` on a float gives the shortest round-tripping decimal,
                # which is valid Rust float syntax.
                lines.append(
                    f"    FederalBracket {{ threshold: {threshold!r}, rate: {rate!r} }},"
                )
            lines.append("];")
            lines.append("")

    lines.append("/// Every imported scale.")
    lines.append("pub const BASE_SCALES: &[BaseScale] = &[")
    for r in results:
        factor = r["splitting_for_married"]
        splitting = "None" if factor is None else f"Some({factor!r})"
        subjects = ", ".join(f'"{s}"' for s in r["subjects"])
        lines.append("    BaseScale {")
        lines.append(f'        canton_code: "{r["code"]}",')
        lines.append(f"        splitting_factor_married: {splitting},")
        lines.append(f"        single: {r['code']}_SINGLE_BRACKETS,")
        lines.append(f"        married: {r['code']}_MARRIED_BRACKETS,")
        lines.append(f"        shared_scale: {str(r['shared_scale']).lower()},")
        lines.append(f"        subjects: &[{subjects}],")
        lines.append("    },")
    lines.append("];")
    lines.append("")
    lines.append("/// Look up an imported scale by canton code.")
    lines.append("pub fn base_scale(canton_code: &str) -> Option<&'static BaseScale> {")
    lines.append("    BASE_SCALES.iter().find(|s| s.canton_code == canton_code)")
    lines.append("}")
    lines.append("")

    with open(out_path, "w", encoding="utf-8", newline="\n") as fh:
        fh.write("\n".join(lines))

    print(f"wrote {out_path}  ({len(results)} cantons)")
    print(f"  sources: {', '.join(contributing)}")
    for r in results:
        top_s = bands_to_thresholds(r["single"])[-1][0]
        top_m = bands_to_thresholds(r["married"])[-1][0]
        factor = r["splitting_for_married"]
        print(
            f"  {r['code']:3} single={len(r['single']):2}b/{top_s:>11.0f}  "
            f"married={len(r['married']):2}b/{top_m:>11.0f}  "
            f"split={factor if factor is not None else '-'}  "
            f"{'shared' if r['shared_scale'] else 'per-subject'}"
        )


if __name__ == "__main__":
    main()
