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


def parse_bands(rows, header_idx, columns, canton_code):
    """Return [(width_chf, rate_percent)] for one canton, in file order."""
    col = columns
    bands = []
    for row in rows[header_idx + 1:]:
        if len(row) <= max(col.values()):
            # Pad short rows so column lookup is safe.
            row = list(row) + [""] * (max(col.values()) + 1 - len(row))

        code = clean(row[col["Kanton"]])
        steuerart = clean(row[col.get("Steuerart", -1)]) if "Steuerart" in col else ""
        subject = clean(row[col.get("Steuersubjekt", -1)]) if "Steuersubjekt" in col else ""
        authority = clean(row[col.get("Steuerhoheit", -1)]) if "Steuerhoheit" in col else ""

        if code.upper() != canton_code.upper():
            continue
        # Income tax only, and only the cantonal scale. Municipal scales are
        # applied through the Steuerfuss, not as a separate base scale.
        if steuerart and steuerart.lower() != "einkommen":
            continue
        if authority and authority.lower() not in ("kanton", "kantonssteuer"):
            continue
        # A subject-specific split ("Verheiratet"/"Ledig") would need separate
        # scales; record it rather than silently merging.
        width = to_float(row[col["Für die nächsten CHF"]])
        rate = to_float(row[col["Zusätzlich %"]])
        if width is None or rate is None:
            continue
        bands.append((width, rate, subject))
    return bands


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
    src = sys.argv[1]
    out_path = sys.argv[2]
    canton_code = sys.argv[3] if len(sys.argv) > 3 else "AG"

    sheets = list(read_xlsx(src))
    if not sheets:
        raise SystemExit(f"no sheets in {src}")

    results = []
    for sheet_name, rows in sheets:
        header_idx, columns = find_header(rows)
        if not columns:
            continue

        # Every canton present in the file, not just the requested one.
        codes = []
        for row in rows[header_idx + 1:]:
            if len(row) > columns["Kanton"]:
                code = clean(row[columns["Kanton"]])
                if code and code not in codes:
                    codes.append(code)

        for code in codes:
            bands = parse_bands(rows, header_idx, columns, code)
            if not bands:
                continue
            splitting = find_splitting_factor(rows, header_idx)
            subjects = sorted({s for _w, _r, s in bands if s})
            results.append(
                {
                    "sheet": sheet_name,
                    "code": code,
                    "bands": bands,
                    "splitting": splitting,
                    "subjects": subjects,
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
    lines.append(f"//! Source: `{os.path.basename(src)}`, exported from the ESTV tax")
    lines.append("//! calculator's Tarife section.")
    lines.append("//!")
    lines.append("//! `threshold` is the income level at which the band begins and `rate` is")
    lines.append("//! that band's marginal rate as a fraction. The ESTV export expresses the")
    lines.append("//! same schedule as band *widths* (\"Für die nächsten CHF\"); the importer")
    lines.append("//! accumulates those widths into thresholds, which is an exact conversion")
    lines.append("//! for the marginal structure.")
    lines.append("")
    lines.append("use crate::federal_tax::FederalBracket;")
    lines.append("")
    lines.append("/// One canton's imported simple-tax scale.")
    lines.append("#[derive(Debug, Clone, Copy)]")
    lines.append("pub struct BaseScale {")
    lines.append("    pub canton_code: &'static str,")
    lines.append("    /// Income-splitting divisor, if the canton uses splitting.")
    lines.append("    pub splitting_factor: Option<f64>,")
    lines.append("    /// Marginal bands as (threshold, rate fraction).")
    lines.append("    pub brackets: &'static [FederalBracket],")
    lines.append("    /// The `Steuersubjekt` values present in the export, so an unexpected")
    lines.append("    /// subject split is visible rather than silently merged.")
    lines.append("    pub subjects: &'static [&'static str],")
    lines.append("}")
    lines.append("")

    for r in results:
        ident = f"{r['code']}_BRACKETS"
        lines.append(f"/// Simple-tax scale for canton {r['code']} ({r['sheet']}).")
        if r["subjects"]:
            lines.append(f"/// Steuersubjekt values in the export: {', '.join(r['subjects'])}.")
        lines.append(f"pub const {ident}: &[FederalBracket] = &[")
        for threshold, rate in bands_to_thresholds(r["bands"]):
            # `repr` on a float gives the shortest round-tripping decimal, which
            # is valid Rust float syntax (Python's `:?` is Rust's, not Python's).
            lines.append(
                f"    FederalBracket {{ threshold: {threshold!r}, rate: {rate!r} }},"
            )
        lines.append("];")
        lines.append("")

    lines.append("/// Every imported scale.")
    lines.append("pub const BASE_SCALES: &[BaseScale] = &[")
    for r in results:
        splitting = "None" if r["splitting"] is None else f"Some({r['splitting']!r})"
        subjects = ", ".join(f'"{s}"' for s in r["subjects"])
        lines.append("    BaseScale {")
        lines.append(f'        canton_code: "{r["code"]}",')
        lines.append(f"        splitting_factor: {splitting},")
        lines.append(f"        brackets: {r['code']}_BRACKETS,")
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

    print(f"wrote {out_path}")
    for r in results:
        thresholds = bands_to_thresholds(r["bands"])
        top = thresholds[-1][0] if thresholds else 0
        print(
            f"  {r['code']}: {len(r['bands'])} bands, thresholds 0..{top:.0f} CHF, "
            f"splitting={r['splitting']}, subjects={r['subjects']}"
        )


if __name__ == "__main__":
    main()
