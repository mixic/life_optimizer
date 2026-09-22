"""Generate Rust Steuerfuss constants from the ESTV workbooks.

Input : source-documents/steuerfuesse-np-<start>-<end>.xlsx  (natural persons)
Output: src/canton_steuerfuss_data.rs

Only the *natural persons* workbook is read. The legal-persons workbook
(`steuerfuesse-jp-...xlsx`) is also in `source-documents/` but no script consumes
it: corporate tax does not bear on a personal work-life decision, and the sheet
finder below is pinned to the `NP <year>` sheet names. It is kept for
completeness, not because anything imports it.

The ESTV sheets list, for each cantonal capital, the cantonal / municipal /
church multipliers applied to the *simple* tax ("Vielfaches der einfachen
Ansätze"). Those multipliers are the second level of the model in
`src/cantons.rs`; the simple tax scales themselves are a separate publication.

Deliberately not parsed here: any cell that is a footnote marker ("3)", "4)")
or a percentage-of-something-else figure ("88% 7)", "147.5%9)"). Those cells do
not mean "multiply the simple tax by this", so treating them as multipliers
would be wrong. They are emitted as `None` with their raw text preserved, so a
reader sees the exception rather than a substituted number.
"""

import re
import sys
import datetime
import importlib.util
import os

# Load the sibling xlsx reader by path: `tools` is a plain directory, not an
# installed package, so a normal import would depend on the caller's cwd.
_here = os.path.dirname(os.path.abspath(__file__))
_spec = importlib.util.spec_from_file_location(
    "xlsx_dump", os.path.join(_here, "xlsx_dump.py")
)
_xlsx_dump = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(_xlsx_dump)
read_xlsx = _xlsx_dump.read_xlsx

# The ESTV workbooks live in `source-documents/` at the repository root. A bare
# filename on the command line resolves against that directory, so the documented
# command works from any working directory; an explicit path still wins.
_SOURCES = os.path.join(os.path.dirname(_here), "source-documents")


def source_path(name):
    """Absolute path for a source workbook, or `name` if it already resolves."""
    if os.path.isabs(name) or os.path.exists(name):
        return name
    return os.path.join(_SOURCES, name)

# Cantonal capital (as spelled in the ESTV sheet) -> official canton code.
CAPITAL_TO_CODE = {
    "Zürich": "ZH",
    "Bern": "BE",
    "Luzern": "LU",
    "Altdorf": "UR",
    "Schwyz": "SZ",
    "Sarnen": "OW",
    "Stans": "NW",
    "Glarus": "GL",
    "Zug": "ZG",
    "Fribourg": "FR",
    "Solothurn": "SO",
    "Basel": "BS",
    "Liestal": "BL",
    "Schaffhausen": "SH",
    "Herisau": "AR",
    "Appenzell": "AI",
    "St. Gallen": "SG",
    "Chur": "GR",
    "Aarau": "AG",
    "Frauenfeld": "TG",
    "Bellinzona": "TI",
    "Lausanne": "VD",
    "Sion": "VS",
    "Neuchâtel": "NE",
    "Genève": "GE",
    "Delémont": "JU",
}

# The capital cell names the *city*, but the registry is keyed by the canton's
# official name, which differs in language for several cantons.
CANTO_NAME_OVERRIDE = {
    "Genève": "Geneva",
    "Luzern": "Lucerne",
    "Neuchâtel": "Neuchâtel",
    "Zürich": "Zürich",
}

NUMBER_RE = re.compile(r"^-?\d+(?:[.,]\d+)?$")


def clean_capital_label(raw):
    """Reduce a capital cell to its bare name.

    Real cells include trailing annotations that must not defeat the lookup:
    ``'Chur (GR)'``, ``'Sion (VS)*'``, ``'Altdorf (UR) '``. Everything from the
    first parenthesis onward, plus stray footnote asterisks, is dropped.
    """
    text = "" if raw is None else str(raw)
    text = text.split("(")[0]
    text = text.replace("*", " ").replace("\u2020", " ")
    return " ".join(text.split()).strip()


def parse_multiplier(raw):
    """Return (is_blank, value, raw_text).

    Three outcomes, kept distinct on purpose:

    * `is_blank=True`  -- the cell was empty or a bare dash: the source says the
      multiplier does not apply (e.g. Ticino has no church tax). There is
      nothing to preserve and nothing is wrong.
    * `value is not None` -- a plain multiplier, possibly written with a comma
      decimal separator as the German-locale sheets do.
    * `value is None, is_blank=False` -- the cell held something that is *not* a
      plain multiplier: a footnote marker like `3)` or a figure expressed
      relative to another tax like `88% 7)`. The raw text is preserved so the
      exception stays visible.

    Collapsing the first and third cases would make "no church tax here"
    indistinguishable from "we could not read this number" — the precise
    confusion this whole module exists to avoid.
    """
    text = "" if raw is None else str(raw).strip()
    if text == "" or text == "-":
        return True, None, text
    if NUMBER_RE.match(text):
        return False, float(text.replace(",", ".")), text
    return False, None, text


def find_year_sheets(rows_by_sheet, prefix="NP "):
    """Locate sheets named like 'NP 2026' and return {year: rows}."""
    out = {}
    for name, rows in rows_by_sheet.items():
        m = re.match(rf"^{re.escape(prefix)}(\d{{4}})$", name.strip())
        if m:
            out[int(m.group(1))] = rows
    return out


def extract_capital_rows(rows):
    """Pull (capital_label, canton_col, municipal_col) from one year's sheet."""
    results = []
    for row in rows:
        # Column 1 holds the capital name; columns 2 and 3 the multipliers.
        label = row[1] if len(row) > 1 else ""
        label = clean_capital_label(label)
        if label in CAPITAL_TO_CODE:
            cantonal = row[2] if len(row) > 2 else ""
            municipal = row[3] if len(row) > 3 else ""
            results.append((label, cantonal, municipal))
    return results


def main():
    xlsx_path = source_path(sys.argv[1])
    out_path = sys.argv[2]

    sheets = {name: rows for name, rows in read_xlsx(xlsx_path)}
    years = find_year_sheets(sheets)
    if not years:
        raise SystemExit(f"no 'NP <year>' sheets found in {xlsx_path}")

    # Pull every year we have, so a caller can choose a vintage.
    per_year = {}
    for year, rows in sorted(years.items()):
        entries = {}
        for label, cantonal, municipal in extract_capital_rows(rows):
            code = CAPITAL_TO_CODE[label]
            c_blank, c_val, c_raw = parse_multiplier(cantonal)
            m_blank, m_val, m_raw = parse_multiplier(municipal)
            entries[code] = {
                # The registry matches on the canton's official name, which for
                # Genève differs from the capital city's spelling.
                "capital": CANTO_NAME_OVERRIDE.get(label, label),
                "cantonal": c_val,
                "cantonal_blank": c_blank,
                "cantonal_raw": c_raw,
                "municipal": m_val,
                "municipal_blank": m_blank,
                "municipal_raw": m_raw,
            }
        per_year[year] = entries

    missing_by_year = {
        y: sorted(set(CAPITAL_TO_CODE.values()) - set(e))
        for y, e in per_year.items()
    }

    stamp = datetime.date.today().isoformat()

    def rust_str(text):
        """Emit a Rust string literal.

        Python `repr` would produce single quotes, which is not a valid Rust
        string and breaks immediately on cell text like `88% 7)` that contains
        an apostrophe. Double quotes plus escaping is the correct form.
        """
        escaped = (
            str(text)
            .replace("\\", "\\\\")
            .replace('"', '\\"')
            .replace("\n", "\\n")
            .replace("\r", "")
            .replace("\t", "\\t")
        )
        return f'"{escaped}"'

    lines = []
    # The GPL-3.0 header is emitted here rather than added afterwards by
    # tools/add_license_header.py, so regenerating the data never strips it.
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
    lines.append("//! Cantonal and capital-city Steuerfuss data, generated from ESTV")
    lines.append("//! workbooks.")
    lines.append("//!")
    lines.append(f"//! Generated on {stamp} by `tools/generate_steuerfuss.py` from")
    # The *basename*, not the resolved path: `source_path` may return an absolute
    # path, and baking a machine-specific one into a committed file would make the
    # header wrong for every other checkout. The sibling generated modules cite the
    # document by name for the same reason.
    lines.append(
        f"//! `{os.path.basename(xlsx_path)}`. Do not edit by hand -- "
        "re-run the generator."
    )
    lines.append("//!")
    lines.append("//! Source: Eidgenössische Steuerverwaltung (ESTV), \"Steuerfüsse in den")
    lines.append("//! Kantonshauptorten\", income and wealth taxes of natural persons.")
    lines.append("//!")
    lines.append("//! `multiplier` is the factor applied to the canton's *simple* tax:")
    lines.append("//! 2.975 means 297.5%. `None` means the source cell was a footnote")
    lines.append("//! marker or a percentage-of-another-tax figure rather than a plain")
    lines.append("//! multiplier; `raw` preserves what the sheet actually said.")
    lines.append("")
    lines.append("/// One canton's multipliers for one year, in a cantonal capital.")
    lines.append("#[derive(Debug, Clone, Copy, PartialEq)]")
    lines.append("pub struct SteuerfussRow {")
    lines.append("    pub canton_code: &'static str,")
    lines.append("    pub capital: &'static str,")
    lines.append("    pub year: u16,")
    lines.append("    /// Cantonal multiplier applied to the simple tax.")
    lines.append("    pub cantonal: Option<f64>,")
    lines.append("    /// Municipal multiplier of the capital city.")
    lines.append("    pub municipal: Option<f64>,")
    lines.append("    /// True when the source cell was empty or a dash: the multiplier does")
    lines.append("    /// not apply (e.g. no church tax in that canton). Distinct from an")
    lines.append("    /// unparsed value, which means the cell held something else.")
    lines.append("    pub cantonal_blank: bool,")
    lines.append("    pub municipal_blank: bool,")
    lines.append("    /// Raw source text when a figure was not a plain multiplier.")
    lines.append("    pub cantonal_raw: &'static str,")
    lines.append("    pub municipal_raw: &'static str,")
    lines.append("}")
    lines.append("")
    lines.append("/// All rows, ordered by year then canton code.")
    lines.append("pub const STEUERFUSS_ROWS: &[SteuerfussRow] = &[")
    for year in sorted(per_year, reverse=True):
        entries = per_year[year]
        for code in sorted(entries, key=lambda c: c):
            e = entries[code]

            def fmt(v):
                return "None" if v is None else f"Some({v!r})"

            lines.append("    SteuerfussRow {")
            lines.append(f"        canton_code: {rust_str(code)},")
            lines.append(f"        capital: {rust_str(e['capital'])},")
            lines.append(f"        year: {year},")
            lines.append(f"        cantonal: {fmt(e['cantonal'])},")
            lines.append(f"        municipal: {fmt(e['municipal'])},")
            lines.append(f"        cantonal_blank: {str(e['cantonal_blank']).lower()},")
            lines.append(f"        municipal_blank: {str(e['municipal_blank']).lower()},")
            lines.append(f"        cantonal_raw: {rust_str(e['cantonal_raw'])},")
            lines.append(f"        municipal_raw: {rust_str(e['municipal_raw'])},")
            lines.append("    },")
    lines.append("];")
    lines.append("")
    lines.append("/// Years covered, newest first.")
    lines.append(
        "pub const STEUERFUSS_YEARS: &[u16] = &["
        + ", ".join(str(y) for y in sorted(per_year, reverse=True))
        + "];"
    )
    lines.append("")
    lines.append("/// Look up one canton's row for a year.")
    lines.append("pub fn steuerfuss_row(canton_code: &str, year: u16) -> Option<&'static SteuerfussRow> {")
    lines.append("    STEUERFUSS_ROWS")
    lines.append("        .iter()")
    lines.append("        .find(|r| r.canton_code == canton_code && r.year == year)")
    lines.append("}")
    lines.append("")

    # Emit a match with one arm per canton code so callers can select on the
    # enum without a string lookup. Written for every canton present, with the
    # year each row came from, so the provenance stays visible at the call site.
    lines.append("/// Select a canton's row for a given year via a match on the code.")
    lines.append("///")
    lines.append("/// Generated so that every canton in the workbook is reachable; an")
    lines.append("/// unrecognised code returns `None` rather than a neighbouring canton.")
    lines.append("pub fn steuerfuss_for_code(canton_code: &str, year: u16) -> Option<&'static SteuerfussRow> {")
    lines.append("    match canton_code {")
    codes_seen = sorted({c for entries in per_year.values() for c in entries})
    for code in codes_seen:
        lines.append(f'        "{code}" => steuerfuss_row("{code}", year),')
    lines.append("        _ => None,")
    lines.append("    }")
    lines.append("}")
    lines.append("")

    with open(out_path, "w", encoding="utf-8", newline="\n") as fh:
        fh.write("\n".join(lines))

    total_rows = sum(len(v) for v in per_year.values())
    print(f"wrote {out_path}")
    print(f"  years: {', '.join(str(y) for y in sorted(per_year, reverse=True))}")
    print(f"  rows:  {total_rows}")
    for y in sorted(per_year, reverse=True):
        m = missing_by_year[y]
        if m:
            print(f"  WARNING {y}: no row for {', '.join(m)}")
    # Report the cells that were not plain multipliers, so they are visible.
    odd = [
        (y, r["capital"], r["cantonal_raw"], r["municipal_raw"])
        for y, entries in sorted(per_year.items(), reverse=True)
        for r in entries.values()
        if (r["cantonal_raw"] and r["cantonal"] is None)
        or (r["municipal_raw"] and r["municipal"] is None)
    ]
    print(f"  non-numeric cells preserved as raw: {len(odd)}")
    for y, cap, craw, mraw in odd[:12]:
        print(f"    {y} {cap}: cantonal={craw!r} municipal={mraw!r}")


if __name__ == "__main__":
    main()
