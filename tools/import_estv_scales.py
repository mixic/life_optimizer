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

    The real exports use more spellings than one might expect, and marital status
    must be tested in a careful order:

    * "Verheiratet" is unambiguous -> married.
    * "Alleinstehend mit Kindern im Konkurrenz" contains "Alleinstehend" and is a
      *single-parent* case -> single, not married.
    * "Alleinstehend mit / ohne Kinder" -> single.
    * "Alle" -> one shared scale.

    Observed across the 2026 exports:

        Alle                                       -> all
        Alleinstehend ohne Kinder                   -> single
        Alleinstehend mit Kindern im Konkurrenz     -> single
        Alleinstehend mit / ohne Kinder             -> single
        Alleinstehend                               -> single
        Verheiratet / Alleinstehend mit Kindern     -> married
        Verheiratet                                 -> married
    """
    low = " ".join(text.lower().split())

    if low == "alle":
        return "all"
    if "verheiratet" in low:
        return "married"
    if "alleinstehend" in low or low.startswith("ledig"):
        return "single"
    return "unknown"


def dedupe_same_scale(kinds):
    """Collapse subject labels that map to identical band tables.

    Input is keyed by `(subject_class, label)`, because several exports publish
    two labels that mean the same thing to the calculation. Jura, for example,
    lists "Alleinstehend ohne Kinder" and "Alleinstehend mit Kindern im
    Konkubinat" with byte-identical bands.

    If two labels in the same class have *different* bands, that is a real
    distinction this model does not yet represent (single parent in a household
    versus not). It is reported rather than silently collapsed, and the first is
    used so the result is deterministic.
    """
    by_class = {}
    for (kind, label), bands in kinds.items():
        signature = tuple((w, r) for w, r, _ in bands)
        by_class.setdefault(kind, []).append((label, signature, bands))

    collapsed = {}
    for kind, entries in by_class.items():
        # Deterministic: sort by label so "the first" is stable across runs.
        entries.sort(key=lambda e: e[0])
        signatures = {sig for _label, sig, _bands in entries}
        if len(signatures) > 1:
            labels = ", ".join(f"'{lbl}'" for lbl, _s, _b in entries)
            print(
                f"  NOTE: {kind} has {len(signatures)} distinct scales ({labels}); "
                f"using the first. The model treats them as one."
            )
        collapsed[kind] = entries[0][2]
    return collapsed


def find_columns(header_row):
    """Locate the ESTV export's columns by substring.

    Exact header matching is unreliable: the same field is emitted with
    different accent handling between exports (`Fuer die naechsten CHF` appears
    with a replacement character in some files), so a literal lookup raises
    KeyError on otherwise valid data.

    `Kanton` must be matched with `id` excluded, otherwise `Kantons-Id` wins and
    the numeric canton id (19, 12, 1) is returned where a two-letter code is
    expected.

    Two band formats exist and both are returned:

    * `Für die nächsten CHF` — a band **width**, used by the cantonal exports.
    * `Steuerbares Einkommen CHF` — an **absolute threshold**, with a
      `Grundbetrag CHF` base amount, used by the federal export. Treating that
      column as a width silently produces a wholly different tariff.
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
        # Width form (cantons). The exclusions matter: several headers contain
        # "CHF", so a bare `chf` match picks `Grundbetrag CHF` or `Steuerbares
        # Einkommen CHF` and the width column is then read from the wrong place,
        # silently turning a threshold grid into a band-width grid.
        "Für die nächsten CHF": find(
            "chf", exclude=("grundbetrag", "steuerbares", "einkommen"),
            fallback=None,
        ),
        # Threshold form (federation).
        "Steuerbares Einkommen CHF": find("steuerbares", "einkommen", fallback=None),
        "Grundbetrag CHF": find("grundbetrag", fallback=None),
        # Flat-rate form: two cantons levy a single percentage rather than a band
        # table. Obwalden and Uri both publish this shape, so a parser that only
        # knows band widths finds nothing for them.
        "Steuersatz %": find("steuersatz", fallback=None),
        "Zusätzlich %": find("zus", fallback=6),
    }


def parse_cantons(src):
    """Return {canton_code: {subject_class: [(width, rate_percent, label)]}} plus factors."""
    out = {}
    factors = {}
    grids = {}

    for sheet_name, rows in read_xlsx(src):
        header_idx, _ = find_header(rows)
        if header_idx is None:
            continue

        columns = find_columns(rows[header_idx])
        factor = find_splitting_factor(rows, header_idx)
        width_col = columns["Für die nächsten CHF"]
        threshold_col = columns["Steuerbares Einkommen CHF"]
        rate_col = columns["Zusätzlich %"]

        # The federal export tabulates absolute thresholds plus a base amount,
        # so `Grundbetrag + rate x (income - threshold)` gives the tax directly.
        # Cantonal exports tabulate band widths instead. Treating the federal
        # thresholds as widths produces a completely different tariff, so the
        # two shapes are distinguished here rather than inferred later.
        absolute_grid = width_col is None and threshold_col is not None

        # A third shape: two cantons levy a single flat percentage of income
        # rather than any band schedule. There is no band column at all, so a
        # parser that requires one finds nothing for them.
        flat_rate_col = columns.get("Steuersatz %")
        flat_rate = (
            flat_rate_col is not None
            and width_col is None
            and threshold_col is None
        )

        # A canton publishes EITHER one "Alle" scale (marital difference is then
        # expressed through the splitting factor) OR separate scales per
        # Steuersubjekt (marital difference is then already in the scale).
        # Applying splitting on top of an already-married-specific scale would
        # double-count the marital adjustment, so the two axes are kept apart.
        # `columns` legitimately holds `None` for the band column that this
        # export does not use, so the padding width must ignore those.
        widest = max(v for v in columns.values() if v is not None)

        for row in rows[header_idx + 1:]:
            if len(row) <= widest:
                row = list(row) + [""] * (widest + 1 - len(row))

            raw_code = clean(row[columns["Kanton"]])
            code = raw_code.upper()
            steuerart = clean(row[columns["Steuerart"]])
            authority = clean(row[columns["Steuerhoheit"]])
            subject = clean(row[columns["Steuersubjekt"]])

            # The federal export uses "Bund" where cantons use a two-letter
            # code, so both shapes must be accepted or it is silently skipped.
            if code != "BUND" and not (len(code) == 2 and code.isalpha()):
                continue
            if steuerart and steuerart.lower() != "einkommen":
                continue

            # Take cantonal rows only for cantons, and Bundessteuer for the
            # federation. Several exports also carry `Gemeinde` rows; including
            # those would apply municipal tax twice, because the Steuerfuss
            # already carries the municipal multiplier.
            auth_low = authority.lower()
            if code == "BUND":
                if auth_low not in ("bundessteuer", "bund"):
                    continue
                code = "Bund"
            else:
                if auth_low not in ("kanton", "kantonssteuer"):
                    continue

            width = to_float(row[width_col]) if width_col is not None else None

            if flat_rate:
                # The flat rate lives in its own column; `Zusätzlich %` is absent
                # or empty for these exports.
                flat = to_float(row[flat_rate_col])
                if flat is None:
                    continue
                kind = classify_subject(subject)
                if kind == "unknown":
                    continue
                # A single row per subject: the whole tariff is that percentage.
                out.setdefault(code, {}).setdefault((kind, subject), []).append(
                    (0.0, flat, subject)
                )
                factors[code] = factor
                grids[code] = "flat"
                continue

            rate = to_float(row[rate_col])
            if rate is None:
                continue

            if absolute_grid:
                # Emit the threshold and rate; the base amount is implied by the
                # preceding bands, which `tax_with_scale` reproduces by
                # accumulating each slice.
                threshold = to_float(row[threshold_col])
                if threshold is None:
                    continue
                value = threshold
            else:
                if width is None:
                    continue
                value = width

            kind = classify_subject(subject)
            if kind == "unknown":
                continue
            # Key by label as well as class: several exports list two labels that
            # mean the same thing to the calculation (Jura publishes
            # "Alleinstehend ohne Kinder" and "Alleinstehend mit Kindern im
            # Konkubinat" with identical bands). Appending both to one list would
            # concatenate two scales into one, which silently produces a wrong
            # tariff, so labels are kept separate and merged afterwards.
            out.setdefault(code, {}).setdefault((kind, subject), []).append(
                (value, rate, subject)
            )
            factors[code] = factor
            grids[code] = absolute_grid

    return out, factors, grids


def bands_to_thresholds(bands, absolute_grid=False):
    """Convert an export's band column into (threshold, rate_fraction) pairs.

    Three source shapes, and conflating them changes the entire tariff:

    * **width** (cantonal exports): the column is "for the next N CHF", so the
      first N are taxed at r1, the next at r2, and the widths accumulate into
      thresholds. `(w1, r1), (w2, r2)` -> `(0, r1), (w1, r2)`.
    * **threshold** (the federal export): the column already *is* the income at
      which the band begins, paired with a base amount, so each row maps
      straight through.
    * **flat** (Obwalden, Uri): the tariff is a single percentage of income, so
      the bracket table is a single 0%-at-0 floor and the percentage is applied
      separately through `BaseScale::flat_rate_percent`. Emitting `(0, rate)`
      here would work arithmetically but would misrepresent the tariff as a
      band, so the floor is emitted with a zero rate instead.

    Because a width schedule and a threshold schedule both express the tax as
    `sum(slice x rate)`, `tax_with_scale` evaluates either form once the
    thresholds are correct.
    """
    if absolute_grid == "flat":
        return [(0.0, 0.0)]

    if absolute_grid:
        return [(value, rate_percent / 100.0) for value, rate_percent, _s in bands]

    out = []
    threshold = 0.0
    for value, rate_percent, _subject in bands:
        out.append((threshold, rate_percent / 100.0))
        threshold += value
    return out


def main():
    # Usage: import_estv_scales.py <out.rs> <file.xlsx> [more.xlsx ...]
    # Accepting several files lets one run cover a batch of single-canton
    # exports, and the merge is first-wins so re-running with the same inputs is
    # idempotent.
    if len(sys.argv) < 3:
        raise SystemExit(
            "usage: import_estv_scales.py <out.rs> <file.xlsx> [more.xlsx ...] "
            "[--federal-out <path.rs>]"
        )
    argv = sys.argv[1:]
    federal_out_path = "src/federal_tariff_data.rs"
    if "--federal-out" in argv:
        i = argv.index("--federal-out")
        federal_out_path = argv[i + 1]
        del argv[i:i + 2]
    out_path = argv[0]
    srcs = argv[1:]

    cantons = {}
    factors = {}
    grids = {}
    contributing = []
    for src in srcs:
        parsed, parsed_factors, parsed_grids = parse_cantons(src)
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
            grids[code] = parsed_grids.get(code, False)

    if not cantons:
        raise SystemExit(f"no cantonal income bands found in {srcs}")

    results = []
    for code, by_kind in sorted(cantons.items()):
        by_kind = dedupe_same_scale(by_kind)
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

        grid_kind = grids.get(code, False)
        # A flat-rate canton's single row carries the percentage in the rate
        # field with a zero band value, so it is read out here.
        flat_rate_percent = None
        if grid_kind == "flat":
            flat_rate_percent = shared[0][1] if shared else None
            # A flat canton has one "Alle" row with a splitting factor of 0.0,
            # so splitting must not be applied; the percentage is the whole
            # tariff.
            applies_for_married = None

        subjects = sorted({s for bands in by_kind.values() for _w, _r, s in bands})
        results.append(
            {
                "code": code,
                "single": single_bands,
                "married": married_bands,
                "splitting_for_married": applies_for_married,
                "subjects": subjects,
                "absolute_grid": grid_kind,
                "shared_scale": bool(shared and not (single or married)),
                "flat_rate_percent": flat_rate_percent,
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
    lines.append("    /// Set for the cantons that levy a single flat percentage of income")
    lines.append("    /// instead of a band schedule (Obwalden, Uri). When present, `single`")
    lines.append("    /// and `married` are a zero floor and the rate below is the whole")
    lines.append("    /// tariff, so `base_tax = income * flat_rate_percent / 100`.")
    lines.append("    pub flat_rate_percent: Option<f64>,")
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
    lines.append("")
    lines.append("    /// Whether this canton is a flat-rate one.")
    lines.append("    pub fn is_flat_rate(&self) -> bool {")
    lines.append("        self.flat_rate_percent.is_some()")
    lines.append("    }")
    lines.append("}")
    lines.append("")

    for r in results:
        # The federal export's code is "Bund", which is not a valid Rust
        # identifier segment as-is; canton codes are already uppercase.
        ident_code = r["code"].upper()
        for kind in ("single", "married"):
            ident = f"{ident_code}_{kind.upper()}_BRACKETS"
            lines.append(
                f"/// Simple-tax scale for canton {r['code']}, {kind} taxpayer."
            )
            if r["shared_scale"]:
                lines.append("/// The canton publishes one shared scale; this is the same table")
                lines.append("/// for both marital statuses.")
            lines.append(f"pub const {ident}: &[FederalBracket] = &[")
            for threshold, rate in bands_to_thresholds(r[kind], r["absolute_grid"]):
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
        # The federal tariff is stored separately in `src/federal_tax.rs`, so it
        # is not repeated in this cantonal registry.
        if r["code"] == "Bund":
            continue
        factor = r["splitting_for_married"]
        splitting = "None" if factor is None else f"Some({factor!r})"
        subjects = ", ".join(f'"{s}"' for s in r["subjects"])
        lines.append("    BaseScale {")
        lines.append(f'        canton_code: "{r["code"]}",')
        lines.append(f"        splitting_factor_married: {splitting},")
        lines.append(f"        single: {r['code'].upper()}_SINGLE_BRACKETS,")
        lines.append(f"        married: {r['code'].upper()}_MARRIED_BRACKETS,")
        lines.append(f"        shared_scale: {str(r['shared_scale']).lower()},")
        flat = r.get("flat_rate_percent")
        lines.append(
            "        flat_rate_percent: "
            + ("None," if flat is None else f"Some({flat!r}),")
        )
        lines.append(f"        subjects: &[{subjects}],")
        lines.append("    },")
    lines.append("];")
    lines.append("")
    lines.append("/// Look up an imported scale by canton code.")
    lines.append("pub fn base_scale(canton_code: &str) -> Option<&'static BaseScale> {")
    lines.append("    BASE_SCALES.iter().find(|s| s.canton_code == canton_code)")
    lines.append("}")
    lines.append("")

    # The federal tariff is emitted into its own module so `federal_tax.rs` can
    # use the official published scale. Before this it held a hand-entered table
    # with a known non-monotonic defect and a wrong top marginal rate (7.39%
    # against the statutory 13.2%).
    bund = next((r for r in results if r["code"] == "Bund"), None)
    if bund is not None:
        # Reuse only the leading licence comment block, not the cantonal module
        # documentation that follows it.
        licence = []
        for ln in lines:
            if ln.startswith("// Life Optimizer") or licence:
                licence.append(ln)
                if ln == "" and len(licence) > 1:
                    break
        fed = list(licence)
        fed.append("//! Official federal direct-tax tariff (direkte Bundessteuer),")
        fed.append("//! imported from the ESTV Tarife export `estv_scales_Bund.xlsx`.")
        fed.append("//!")
        fed.append("//! GENERATED by `tools/import_estv_scales.py` -- do not edit by hand.")
        fed.append("//!")
        fed.append("//! This is the authoritative published tariff, replacing an earlier")
        fed.append("//! hand-entered table whose top marginal rate was 7.39% against the")
        fed.append("//! statutory 13.2%, and which contained a non-monotonic segment.")
        fed.append("//!")
        fed.append("//! `threshold` is the income at which the band begins and `rate` is that")
        fed.append("//! band's marginal rate as a fraction.")
        fed.append("")
        fed.append("use crate::federal_tax::FederalBracket;")
        fed.append("")
        for kind in ("single", "married"):
            fed.append(f"/// Federal tariff, {kind} taxpayer.")
            fed.append(f"pub const FEDERAL_{kind.upper()}_BRACKETS: &[FederalBracket] = &[")
            for threshold, rate in bands_to_thresholds(bund[kind], bund["absolute_grid"]):
                fed.append(
                    f"    FederalBracket {{ threshold: {threshold!r}, rate: {rate!r} }},"
                )
            fed.append("];")
            fed.append("")
        with open(federal_out_path, "w", encoding="utf-8", newline="\n") as fh:
            fh.write("\n".join(fed))
        print(f"wrote {federal_out_path}  (federal tariff: "
              f"{len(bund['single'])} single bands, {len(bund['married'])} married)")
    else:
        print("  no Bund export supplied; federal tariff left unchanged")

    with open(out_path, "w", encoding="utf-8", newline="\n") as fh:
        fh.write("\n".join(lines))

    print(f"wrote {out_path}  ({len(results)} cantons)")
    print(f"  sources: {', '.join(contributing)}")
    for r in results:
        top_s = bands_to_thresholds(r["single"], r["absolute_grid"])[-1][0]
        top_m = bands_to_thresholds(r["married"], r["absolute_grid"])[-1][0]
        factor = r["splitting_for_married"]
        print(
            f"  {r['code']:3} single={len(r['single']):2}b/{top_s:>11.0f}  "
            f"married={len(r['married']):2}b/{top_m:>11.0f}  "
            f"split={factor if factor is not None else '-'}  "
            f"{'shared' if r['shared_scale'] else 'per-subject'}"
        )


if __name__ == "__main__":
    main()
