"""Import the ESTV deduction tables into Rust data.

Inputs (the ESTV tax calculator's deduction exports):

  `estv_deductions.xlsx`        one rule per canton per deduction *category*
  `estv_deduction_scales.xlsx`  phase-out tables for the means-tested deductions

Output: `src/estv_deductions_data.rs`, with provenance and a typed rule shape.

The two tables are different things, and conflating them is the main trap
---------------------------------------------------------------------

`estv_deductions.xlsx` (618 rows, 112 distinct `Abzug` values, 27 jurisdictions:
26 cantons + `Bund`) carries, per row:

    Betrag    a fixed amount (CHF)
    Prozent   a percentage
    Minimum   a floor
    Maximum   a ceiling

and the two are *not* alternatives. `AR Pauschalabzug übrige Berufskosten` is
`Betrag=700, Prozent=10, Minimum=0, Maximum=2400`, i.e. "700 plus 10%, capped at
2400" — a shape that cannot be expressed by picking one column.

The same file also mixes in rows that are **not deductions at all**:

    Schwellwert für Abzug AHV-/IV-Rentner              (a threshold)
    Familien Sozialabzug Schwellwert Austritt 1        (a phase-out breakpoint)
    Steuerermässigung pro Kind                         (a relief, not a deduction)
    Maximaler technischer Zinssatz Einmaleinlage ...   (a rate used in a formula)

Summing every row would therefore both double-count and treat parameters as
deductions. The generator classifies by name and records the classification, so
the *data* module never has to guess later.

`estv_deduction_scales.xlsx` (466 rows, 15 distinct names, all `Steuerhoheit =
Kanton`) is a step function per (canton, name): `Reineinkommen` -> `Abzug CHF`.
The deduction is the amount at the highest threshold not exceeding income, and it
phases *down* as income rises — e.g. Fribourg's `Abzug für bescheidenes
Einkommen, Ledige ohne Kind` runs 4100 at 0, 3900 at 20301, ... down to 0 at
40301. These are the means-tested deductions the first table lists the *caps* for.

Nothing here is invented: every emitted figure is a cell from one of the two
workbooks, and every row carries the file and year it came from.
"""

import collections
import datetime
import importlib.util
import json
import os
import re
import sys

# Load the sibling xlsx reader by path: `tools` is not an installed package.
_HERE = os.path.dirname(os.path.abspath(__file__))
_spec = importlib.util.spec_from_file_location(
    "xlsx_dump", os.path.join(_HERE, "xlsx_dump.py")
)
_xlsx_dump = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(_xlsx_dump)
read_xlsx = _xlsx_dump.read_xlsx

# Reuse the licence header so regeneration cannot strip it.
_spec2 = importlib.util.spec_from_file_location(
    "add_license_header", os.path.join(_HERE, "add_license_header.py")
)
_lic = importlib.util.module_from_spec(_spec2)
_spec2.loader.exec_module(_lic)

# The ESTV workbooks live in `source-documents/` at the repository root. Resolving
# a bare filename against that directory keeps the documented command runnable from
# any working directory, which a plain relative default did not.
_SOURCES = os.path.join(os.path.dirname(_HERE), "source-documents")


def source_path(name):
    """Absolute path for a source document, or `name` if it already resolves."""
    if os.path.isabs(name) or os.path.exists(name):
        return name
    return os.path.join(_SOURCES, name)


def clean(text):
    """Normalise cell text: strip, collapse whitespace, drop a UTF-8 BOM."""
    if text is None:
        return ""
    text = str(text).replace("\ufeff", "")
    return re.sub(r"\s+", " ", text).strip()


def to_float(text):
    """Parse a number that may use a comma decimal separator, else None."""
    text = clean(text)
    if text == "":
        return None
    # Thousands separators are stripped, not treated as decimals: a right-single
    # quote or an ASCII apostrophe appears as a separator in these workbooks, and
    # reading it as a decimal point would shift a CHF 106,000 cap to CHF 106.
    normalised = text.replace("'", "").replace("\u2019", "")
    if "," in normalised and "." not in normalised:
        normalised = normalised.replace(",", ".")
    else:
        normalised = normalised.replace(",", "")
    try:
        return float(normalised)
    except ValueError:
        return None


def find_header(rows, must_have):
    """Locate the header row containing every name in `must_have`."""
    for i, row in enumerate(rows):
        cells = [clean(c) for c in row]
        if all(any(name == c for c in cells) for name in must_have):
            return i, {clean(c): idx for idx, c in enumerate(cells) if clean(c)}
    return None, {}


# --- Classification of `Abzug` names -----------------------------------------
#
# Explicit prefix rules, not fuzzy matching: a name that matches nothing is
# reported and emitted as `Other` rather than being silently dropped, so a new
# ESTV category shows up in the test output instead of vanishing.

# Rows that state a threshold or a phase-out breakpoint, used *by* a deduction.
PARAMETER_PREFIXES = (
    "Schwellwert",
    "Familien Sozialabzug Schwellwert",
    "Entlastungsabzug",  # "Entlastungsabzug X Schwellwert"
)

# Rows that reduce tax directly rather than reducing taxable income, and rate
# parameters used inside another rule's arithmetic.
NOT_A_DEDUCTION_PREFIXES = (
    "Steuerermässigung pro Kind",
    "Maximaler technischer Zinssatz",
    "Faktor PrivVers/Sparzinsen",
    "Faktor Versicherungsprämien",
    "Krankenkasse Durchschnittsprämie",
)


def normalise_canton(raw):
    """Normalise the export's canton cell to this project's convention.

    Cantons are two-letter uppercase codes; the federation is `Bund`, which is
    how `estv_scales_Bund.xlsx` spells it and how `cantons::Canton::from_code`
    does *not* -- so it must be matched as a string, and emitting `BUND` would
    quietly create a jurisdiction nothing else in the codebase recognises.
    """
    code = clean(raw).upper()
    return "Bund" if code == "BUND" else code


def classify(name):
    """Return one of: deduction, parameter, other.

    `deduction`    reduces taxable income and can be summed with its siblings.
    `parameter`    a threshold or breakpoint consumed by another rule.
    `other`        a tax relief or a rate constant; not a deductible amount.
    """
    for prefix in PARAMETER_PREFIXES:
        if name.startswith(prefix):
            return "parameter"
    for prefix in NOT_A_DEDUCTION_PREFIXES:
        if name.startswith(prefix):
            return "other"
    return "deduction"


def merge_split_rules(rules):
    """Merge rows that carry different parts of one rule.

    The export splits a single rule across rows when its parts are stated
    separately. Valais has two `Pauschalabzug Unterhaltskosten von vermieteten
    Liegenschaften mit Alter bis 20 Jahren` rows:

        Prozent=10, Maximum=0      "10% of the base"
        Prozent=0,  Maximum=15000  "capped at 15,000"

    which one rule, not two. Keeping them separate would mean a consumer picks
    whichever it finds first and silently drops the cap -- or the rate. Merging
    takes the non-zero value of each field, which for this export's shape gives
    `Prozent=10, Maximum=15000`.

    A field that is non-zero in *both* rows is a genuine conflict: it means two
    rows of the same name disagree, which must be visible rather than resolved by
    row order.
    """
    merged = collections.OrderedDict()
    for rule in rules:
        key = (rule["canton"], rule["name"])
        if key not in merged:
            merged[key] = dict(rule)
            continue
        existing = merged[key]
        for field in ("amount", "percent", "minimum", "maximum"):
            mine, theirs = rule[field], existing[field]
            if mine == 0.0:
                continue
            if theirs == 0.0 or theirs == mine:
                existing[field] = mine
            else:
                print(
                    f"  CONFLICT {rule['canton']} {rule['name']!r}: "
                    f"{field} is {theirs} and {mine} in two rows; keeping the first"
                )
        if rule["kind"] != existing["kind"]:
            print(
                f"  CONFLICT {rule['canton']} {rule['name']!r}: "
                f"classified as both {existing['kind']} and {rule['kind']}"
            )
    return list(merged.values())


def parse_rules(path):
    """Parse `estv_deductions.xlsx` into (year, [rule dicts])."""
    sheets = list(read_xlsx(path))
    if not sheets:
        raise SystemExit(f"{path}: no sheets")
    _sheet, rows = sheets[0]

    year = None
    for row in rows[:4]:
        for cell in row:
            value = to_float(cell)
            if value is not None and 1990 <= value <= 2100:
                year = int(value)
                break
        if year:
            break

    header_idx, cols = find_header(
        rows, ["Kanton", "Steuerart", "Abzug", "Betrag", "Prozent", "Maximum"]
    )
    if header_idx is None:
        raise SystemExit(f"{path}: header not found")

    def col(name, fallback=None):
        # `Kantons-Id` must not win the `Kanton` lookup: the numeric id is not a
        # canton code and would emit identifiers like `13_ABZUG_...`.
        for key, idx in cols.items():
            if "id" in key.lower() and name.lower() == "kanton":
                continue
            if name.lower() == key.lower():
                return idx
        return fallback

    idx = {
        "Kanton": col("Kanton"),
        "Steuerart": col("Steuerart"),
        "Abzug": col("Abzug"),
        "Betrag": col("Betrag"),
        "Prozent": col("Prozent"),
        "Minimum": col("Minimum"),
        "Maximum": col("Maximum"),
    }
    missing = [k for k, v in idx.items() if v is None]
    if missing:
        raise SystemExit(f"{path}: missing columns {missing}")

    out = []
    for row in rows[header_idx + 1:]:
        if len(row) <= max(idx.values()):
            continue
        canton = normalise_canton(row[idx["Kanton"]])
        steuerart = clean(row[idx["Steuerart"]])
        name = clean(row[idx["Abzug"]])
        if not canton or not name:
            continue
        # Only income tax: the wealth rules belong to a model this project does
        # not have, and mixing them in would silently inflate income deductions.
        if steuerart and steuerart.lower() != "einkommen":
            continue
        out.append(
            {
                "canton": canton,
                "name": name,
                "kind": classify(name),
                "amount": to_float(row[idx["Betrag"]]) or 0.0,
                "percent": to_float(row[idx["Prozent"]]) or 0.0,
                "minimum": to_float(row[idx["Minimum"]]) or 0.0,
                "maximum": to_float(row[idx["Maximum"]]) or 0.0,
            }
        )
    merged = merge_split_rules(out)
    if len(merged) != len(out):
        print(f"  merged {len(out) - len(merged)} row(s) that the export split")
    return year, merged


def parse_scales(path):
    """Parse `estv_deduction_scales.xlsx` into (year, [scale dicts])."""
    sheets = list(read_xlsx(path))
    if not sheets:
        raise SystemExit(f"{path}: no sheets")
    _sheet, rows = sheets[0]

    year = None
    for row in rows[:4]:
        for cell in row:
            value = to_float(cell)
            if value is not None and 1990 <= value <= 2100:
                year = int(value)
                break
        if year:
            break

    header_idx, cols = find_header(
        rows, ["Kanton", "Steuerart", "Name", "Steuerhoheit", "Abzug CHF"]
    )
    if header_idx is None:
        raise SystemExit(f"{path}: header not found")

    def find(*needles, exclude=()):
        for key, index in cols.items():
            low = key.lower()
            if any(x in low for x in exclude):
                continue
            if all(n.lower() in low for n in needles):
                return index
        return None

    # `Kanton` must be matched with `id` excluded, or `Kantons-Id` wins and the
    # numeric canton id (10, 13, 1) is returned where a two-letter code is
    # expected. That produced identifiers like `10_SCALE_...` on the first run.
    c_canton = find("kanton", exclude=("id",))
    c_art = find("steuerart")
    c_name = find("name")
    c_hoheit = find("steuerhoheit")
    c_threshold = find("reineinkommen")
    c_amount = find("abzug")
    if None in (c_canton, c_art, c_name, c_hoheit, c_threshold, c_amount):
        raise SystemExit(f"{path}: missing columns")

    # Group by (canton, tax type, name, authority) and collect the step points.
    grouped = collections.OrderedDict()
    for row in rows[header_idx + 1:]:
        if len(row) <= max(c_canton, c_art, c_name, c_hoheit, c_threshold, c_amount):
            continue
        canton = normalise_canton(row[c_canton])
        art = clean(row[c_art])
        name = clean(row[c_name])
        hoheit = clean(row[c_hoheit])
        threshold = to_float(row[c_threshold])
        amount = to_float(row[c_amount])
        if not canton or not name or threshold is None or amount is None:
            continue
        if art and art.lower() != "einkommen":
            # The wealth scales use the same shape; kept out of the income model
            # for the same reason as above, and reported by the caller.
            continue
        key = (canton, name, hoheit)
        grouped.setdefault(key, []).append((threshold, amount))

    out = []
    for (canton, name, hoheit), points in grouped.items():
        points.sort(key=lambda p: p[0])
        out.append(
            {
                "canton": canton,
                "name": name,
                "authority": hoheit,
                "points": points,
            }
        )
    return year, out


def rust_str(text):
    """Emit a Rust string literal (double-quoted; `repr` would use single)."""
    return json.dumps(str(text), ensure_ascii=False)


def main():
    if len(sys.argv) < 2:
        raise SystemExit(
            "usage: import_estv_deductions.py <out.rs> [--rules <file>] "
            "[--scales <file>]"
        )
    out_path = sys.argv[1]
    rules_path = source_path("estv_deductions.xlsx")
    scales_path = source_path("estv_deduction_scales.xlsx")
    argv = sys.argv[2:]
    if "--rules" in argv:
        rules_path = source_path(argv[argv.index("--rules") + 1])
    if "--scales" in argv:
        scales_path = source_path(argv[argv.index("--scales") + 1])

    rules_year, rules = parse_rules(rules_path)
    scales_year, scales = parse_scales(scales_path)

    kinds = collections.Counter(r["kind"] for r in rules)
    print(f"  {rules_path}: {len(rules)} rows, year {rules_year}")
    print(f"    {dict(kinds)}")
    print(f"  {scales_path}: {len(scales)} scales, year {scales_year}")

    unknown = sorted({r["name"] for r in rules if r["kind"] == "other"})
    if unknown:
        print(f"    classified as neither deduction nor threshold ({len(unknown)}):")
        for name in unknown:
            print(f"      {name}")

    lines = []
    lines.extend(_lic.HEADER_LINES)
    lines.append("")
    lines.append("//! Swiss income-tax deduction rules, imported from the ESTV tax")
    lines.append("//! calculator's deduction exports.")
    lines.append("//!")
    lines.append("//! GENERATED by `tools/import_estv_deductions.py` -- do not edit by")
    lines.append("//! hand; re-run the importer instead.")
    lines.append("//!")
    lines.append(f"//! Source: `{os.path.basename(rules_path)}` (rules) and")
    lines.append(f"//! `{os.path.basename(scales_path)}` (phase-out scales), both ESTV")
    lines.append(f"//! publications for tax year {rules_year}.")
    lines.append("//!")
    lines.append("//! # Two tables, two jobs")
    lines.append("//!")
    lines.append("//! [`DEDUCTION_RULES`] gives, per canton and per deduction, the amount")
    lines.append("//! and the bounds the ESTV calculator applies:")
    lines.append("//!")
    lines.append("//! ```text")
    lines.append("//! deduction = clamp(amount + percent/100 * base, minimum, maximum)")
    lines.append("//! ```")
    lines.append("//!")
    lines.append("//! Note that `amount` and `percent` are **not** alternatives: Aargau's")
    lines.append("//! \"Pauschalabzug übrige Berufskosten\" is `Betrag=700, Prozent=10,")
    lines.append("//! Maximum=2400`, i.e. 700 plus 10% capped at 2400.")
    lines.append("//!")
    lines.append("//! [`DEDUCTION_SCALES`] gives the means-tested deductions as step")
    lines.append("//! functions of net income. The amount is the one at the highest")
    lines.append("//! threshold not exceeding income; these phase *down* as income rises.")
    lines.append("//!")
    lines.append("//! # These tables also contain non-deductions")
    lines.append("//!")
    lines.append("//! The rules file mixes in thresholds (`Schwellwert ...`), tax")
    lines.append("//! reliefs (`Steuerermässigung pro Kind`) and rate constants")
    lines.append("//! (`Maximaler technischer Zinssatz ...`). Summing every row would")
    lines.append("//! double-count and would treat parameters as deductions, so each row")
    lines.append("//! carries a [`RuleKind`] recording what it is. A consumer must")
    lines.append("//! filter on `Deduction`.")
    lines.append("")
    lines.append("/// What a row in the rules file actually is.")
    lines.append("#[derive(Debug, Clone, Copy, PartialEq, Eq)]")
    lines.append("pub enum RuleKind {")
    lines.append("    /// Reduces taxable income; may be summed with its siblings.")
    lines.append("    Deduction,")
    lines.append("    /// A threshold or phase-out breakpoint consumed by another rule.")
    lines.append("    Threshold,")
    lines.append("    /// A tax relief or a rate constant: not a deductible amount.")
    lines.append("    Other,")
    lines.append("}")
    lines.append("")
    lines.append("/// One imported deduction rule for one canton (or `Bund`).")
    lines.append("#[derive(Debug, Clone, Copy)]")
    lines.append("pub struct DeductionRule {")
    lines.append("    /// Canton code, or `Bund` for the federal rule.")
    lines.append("    pub canton_code: &'static str,")
    lines.append("    /// The ESTV `Abzug` label, verbatim. Kept as the source spells it so a")
    lines.append("    /// rule can be found in the workbook by eye.")
    lines.append("    pub name: &'static str,")
    lines.append("    pub kind: RuleKind,")
    lines.append("    /// Fixed amount in CHF.")
    lines.append("    pub amount: f64,")
    lines.append("    /// Percentage of the base, as a percentage (10.0 means 10%).")
    lines.append("    pub percent: f64,")
    lines.append("    /// Lower bound in CHF.")
    lines.append("    pub minimum: f64,")
    lines.append("    /// Upper bound in CHF; `0.0` means the source states no ceiling.")
    lines.append("    pub maximum: f64,")
    lines.append("}")
    lines.append("")
    lines.append("/// One step of a means-tested deduction's phase-out table.")
    lines.append("#[derive(Debug, Clone, Copy)]")
    lines.append("pub struct ScalePoint {")
    lines.append("    /// Net income (`Reineinkommen`) at which this step begins.")
    lines.append("    pub threshold: f64,")
    lines.append("    /// The deduction amount from this threshold onwards.")
    lines.append("    pub amount: f64,")
    lines.append("}")
    lines.append("")
    lines.append("/// A means-tested deduction, expressed as a step function of net income.")
    lines.append("#[derive(Debug, Clone, Copy)]")
    lines.append("pub struct DeductionScale {")
    lines.append("    pub canton_code: &'static str,")
    lines.append("    pub name: &'static str,")
    lines.append("    /// `Steuerhoheit` from the source: which level of government levies it.")
    lines.append("    pub authority: &'static str,")
    lines.append("    pub points: &'static [ScalePoint],")
    lines.append("}")
    lines.append("")
    lines.append("impl DeductionScale {")
    lines.append("    /// The deduction at a given net income.")
    lines.append("    ///")
    lines.append("    /// Zero below the first threshold, then the amount at the highest")
    lines.append("    /// threshold not exceeding the income. The tables phase down to zero,")
    lines.append("    /// so this is a step function rather than an interpolation.")
    lines.append("    pub fn amount_at(&self, net_income: f64) -> f64 {")
    lines.append("        let mut amount = 0.0;")
    lines.append("        for point in self.points {")
    lines.append("            if net_income < point.threshold {")
    lines.append("                break;")
    lines.append("            }")
    lines.append("            amount = point.amount;")
    lines.append("        }")
    lines.append("        amount")
    lines.append("    }")
    lines.append("}")
    lines.append("")
    lines.append("impl DeductionRule {")
    lines.append("    /// The deduction this rule yields on a given base (normally net")
    lines.append("    /// income), before the means-tested scales are applied.")
    lines.append("    ///")
    lines.append("    /// `clamp(amount + percent/100 * base, minimum, maximum)`. A `maximum`")
    lines.append("    /// of `0.0` means the source states no ceiling, which is *not* a")
    lines.append("    /// ceiling of zero, so it is ignored.")
    lines.append("    pub fn apply(&self, base: f64) -> f64 {")
    lines.append("        let raw = self.amount + self.percent / 100.0 * base;")
    lines.append("        let mut value = raw.max(self.minimum);")
    lines.append("        if self.maximum > 0.0 {")
    lines.append("            value = value.min(self.maximum);")
    lines.append("        }")
    lines.append("        value.max(0.0)")
    lines.append("    }")
    lines.append("}")
    lines.append("")

    # --- rule constants, one per (canton, name) ------------------------------
    # Index so the registry can reference them as `&'static` slices.
    by_canton = collections.defaultdict(list)
    for rule in rules:
        by_canton[rule["canton"]].append(rule)

    def ident(canton, name):
        slug = re.sub(r"[^A-Za-z0-9]+", "_", name).strip("_").upper()
        return f"{canton.upper()}_{slug}"

    # Slugging two different source labels can collide -- Valais has both
    # "Pauschalabzug Unterhaltskosten von vermieteten Liegenschaften mit Alter bis
    # 20 Jahren" and "Pauschalabzug Unterhaltskosten von vermieteten
    # Liegenschaften mit Alter bis 20 Jahren" differing only in characters the
    # slug drops. Colliding constants would be a compile error, so a stable index
    # is appended to every duplicate after the first.
    used = collections.Counter()

    def unique_ident(canton, name):
        base = ident(canton, name)
        used[base] += 1
        if used[base] == 1:
            return base
        return f"{base}_{used[base]}"

    for canton in sorted(by_canton):
        for rule in sorted(by_canton[canton], key=lambda r: r["name"]):
            name = rule["name"]
            kind = {
                "deduction": "RuleKind::Deduction",
                "parameter": "RuleKind::Threshold",
                "other": "RuleKind::Other",
            }[rule["kind"]]
            lines.append(
                f"/// `{canton}`: {name}"
            )
            lines.append(f"pub const {ident(canton, name)}: DeductionRule = DeductionRule {{")
            lines.append(f"    canton_code: {rust_str(canton)},")
            lines.append(f"    name: {rust_str(name)},")
            lines.append(f"    kind: {kind},")
            lines.append(f"    amount: {rule['amount']!r},")
            lines.append(f"    percent: {rule['percent']!r},")
            lines.append(f"    minimum: {rule['minimum']!r},")
            lines.append(f"    maximum: {rule['maximum']!r},")
            lines.append("};")
            lines.append("")

    lines.append("/// Every imported deduction rule, all jurisdictions.")
    lines.append("pub const DEDUCTION_RULES: &[DeductionRule] = &[")
    for canton in sorted(by_canton):
        for rule in sorted(by_canton[canton], key=lambda r: r["name"]):
            lines.append(f"    {ident(canton, rule['name'])},")
    lines.append("];")
    lines.append("")

    # --- scales -------------------------------------------------------------
    for scale in sorted(scales, key=lambda s: (s["canton"], s["name"])):
        ident_name = f"{scale['canton'].upper()}_SCALE_" + re.sub(
            r"[^A-Za-z0-9]+", "_", scale["name"]
        ).strip("_").upper()
        lines.append(
            f"/// `{scale['canton']}`: {scale['name']} ({len(scale['points'])} steps)"
        )
        lines.append(f"pub const {ident_name}: &[ScalePoint] = &[")
        for threshold, amount in scale["points"]:
            lines.append(
                f"    ScalePoint {{ threshold: {threshold!r}, amount: {amount!r} }},"
            )
        lines.append("];")
        lines.append("")

    lines.append("/// Every imported means-tested deduction scale.")
    lines.append("pub const DEDUCTION_SCALES: &[DeductionScale] = &[")
    for scale in sorted(scales, key=lambda s: (s["canton"], s["name"])):
        ident_name = f"{scale['canton'].upper()}_SCALE_" + re.sub(
            r"[^A-Za-z0-9]+", "_", scale["name"]
        ).strip("_").upper()
        lines.append("    DeductionScale {")
        lines.append(f"        canton_code: {rust_str(scale['canton'])},")
        lines.append(f"        name: {rust_str(scale['name'])},")
        lines.append(f"        authority: {rust_str(scale['authority'])},")
        lines.append(f"        points: {ident_name},")
        lines.append("    },")
    lines.append("];")
    lines.append("")
    lines.append("/// Deduction rules for one jurisdiction (`Bund` or a canton code).")
    lines.append("///")
    lines.append("/// Returns a `Vec` rather than an opaque iterator: a captured `&str` in an")
    lines.append("/// `impl Iterator` return type makes the generated function's lifetime")
    lines.append("/// bound fail to compile, and the table is small enough that the")
    lines.append("/// allocation is irrelevant next to the clarity.")
    lines.append("pub fn rules_for(canton_code: &str) -> Vec<&'static DeductionRule> {")
    lines.append("    DEDUCTION_RULES")
    lines.append("        .iter()")
    lines.append("        .filter(|r| r.canton_code == canton_code)")
    lines.append("        .collect()")
    lines.append("}")
    lines.append("")
    lines.append("/// The means-tested scales for one canton.")
    lines.append("pub fn scales_for(canton_code: &str) -> Vec<&'static DeductionScale> {")
    lines.append("    DEDUCTION_SCALES")
    lines.append("        .iter()")
    lines.append("        .filter(|s| s.canton_code == canton_code)")
    lines.append("        .collect()")
    lines.append("}")
    lines.append("")

    with open(out_path, "w", encoding="utf-8", newline="\n") as fh:
        fh.write("\n".join(lines))
    print(f"wrote {out_path}  ({len(rules)} rules, {len(scales)} scales)")

    # Coverage summary, so a shrinking import is visible rather than silent.
    cantons = sorted({r["canton"] for r in rules})
    print(f"  jurisdictions: {len(cantons)} ({', '.join(cantons)})")
    per_canton = collections.Counter(r["canton"] for r in rules)
    thin = [c for c in cantons if per_canton[c] < 8]
    if thin:
        print(f"  NOTE: fewer than 8 rules for {', '.join(thin)}")


if __name__ == "__main__":
    main()
