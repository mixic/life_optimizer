#!/usr/bin/env python3
"""Re-derive the deduction tables from the ESTV workbooks, independently of the importer.

`tools/import_estv_deductions.py` turns `estv_deductions.xlsx` and
`estv_deduction_scales.xlsx` into `src/estv_deductions_data.rs`. Until this
script existed, that generated file had only ever been read by its own
generator: every number in the deduction engine traced back to a translation
step nobody had independently repeated. `VALIDATION.md` §5 recorded this as
"not validated". This is the independent repetition.

The check is a bijection on *content*, not on names. The importer invents an
identifier for each row (`AG_ABZUG_F_R_FAHRKOSTEN_HAUPT...`), and a verifier
that compared identifiers would be comparing the generator to itself. So each
side is reduced to a multiset of tuples -- (canton, label, amount, percent,
minimum, maximum) for the rules, (canton, label, authority, points) for the
scales -- and the two multisets must be equal. That catches a wrong value, a
dropped row, a duplicated row and a row filed under the wrong canton, without
assuming anything about how the importer names things.

The one judgement the importer makes that content alone does not capture is
`RuleKind`: whether a row is a deduction, a threshold breakpoint, a tax relief
or a rate constant. That is re-derived here from the label alone, by a rule
written from the label classes the module documents, and any disagreement is
reported rather than silently accepted.

Usage:
    python tools/verify_estv_deductions_import.py            # verify
    python tools/verify_estv_deductions_import.py --emit      # also write the CSV

Exit status is 0 when everything matches and 1 otherwise.
"""

from __future__ import annotations

import csv
import glob
import os
import re
import sys

import openpyxl

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SOURCES = os.path.join(ROOT, "source-documents")
RUST = os.path.join(ROOT, "src", "estv_deductions_data.rs")
SNAPSHOT = os.path.join(SOURCES, "estv_deductions_expected.csv")

# The four exports each begin with a title row, the tax year and two blank
# rows before the header. Locating the header by its first cell rather than by
# a fixed offset means a differently padded export still parses.
RULE_HEADER = "Kantons-Id"
SCALE_HEADER = "Kantons-Id"


def workbook_rows(path: str, sheet: str, header_cell: str, width: int) -> list[tuple]:
    """Every data row of an ESTV export, as tuples, after its header row."""
    book = openpyxl.load_workbook(path, read_only=True, data_only=True)
    try:
        sheet_obj = book[sheet]
        rows: list[tuple] = []
        seen_header = False
        for row in sheet_obj.iter_rows(values_only=True):
            if row and row[0] == header_cell:
                seen_header = True
                continue
            if not seen_header or not row or row[0] is None:
                continue
            rows.append(tuple(row[:width]))
        return rows
    finally:
        book.close()


def decode_rust_string(literal: str) -> str:
    """Undo the escapes the importer applies to a label."""
    return literal.replace('\\"', '"').replace("\\\\", "\\")


NAME = r'"((?:[^"\\]|\\.)*)"'
NUMBER = r"(-?[0-9]+(?:\.[0-9]+)?(?:[eE][+-]?[0-9]+)?)"

RULE_PATTERN = re.compile(
    r"^pub const (\w+): DeductionRule = DeductionRule \{\s*"
    r'canton_code: "' + NAME[1:-1] + r'",\s*'
    r"name: " + NAME + r",\s*"
    r"kind: RuleKind::(\w+),\s*"
    r"amount: " + NUMBER + r",\s*"
    r"percent: " + NUMBER + r",\s*"
    r"minimum: " + NUMBER + r",\s*"
    r"maximum: " + NUMBER + r",\s*"
    r"\};",
    re.M,
)

POINT_PATTERN = re.compile(
    r"ScalePoint \{ threshold: " + NUMBER + r", amount: " + NUMBER + r" \}"
)

SCALE_ARRAY_PATTERN = re.compile(
    r"^pub const (\w+): &\[ScalePoint\] = &\[(.*?)^\];",
    re.M | re.S,
)

SCALE_ENTRY_PATTERN = re.compile(
    r"DeductionScale \{\s*"
    r'canton_code: "' + NAME[1:-1] + r'",\s*'
    r"name: " + NAME + r",\s*"
    r'authority: "' + NAME[1:-1] + r'",\s*'
    r"points: (\w+),\s*"
    r"\},",
    re.S,
)


def rust_rules(source: str) -> list[tuple[str, str, str, float, float, float, float]]:
    """Every `DeductionRule` constant, as content tuples."""
    out = []
    for match in RULE_PATTERN.finditer(source):
        identifier, canton, name, kind, amount, percent, minimum, maximum = match.groups()
        out.append(
            (
                identifier,
                canton,
                decode_rust_string(name),
                kind,
                float(amount),
                float(percent),
                float(minimum),
                float(maximum),
            )
        )
    return out


def rust_scales(source: str) -> list[tuple[str, str, str, str, tuple]]:
    """Every `DeductionScale`, joined to its point array."""
    arrays = {}
    for match in SCALE_ARRAY_PATTERN.finditer(source):
        identifier, body = match.group(1), match.group(2)
        arrays[identifier] = tuple(
            (float(threshold), float(amount))
            for threshold, amount in POINT_PATTERN.findall(body)
        )
    out = []
    for match in SCALE_ENTRY_PATTERN.finditer(source):
        canton, name, authority, points = match.groups()
        out.append(
            (
                canton,
                decode_rust_string(name),
                decode_rust_string(authority),
                points,
                arrays.get(points, ()),
            )
        )
    return out


def workbook_rules(path: str):
    """Every rule in `estv_deductions.xlsx`, merged per (canton, label).

    The export splits some rules across two rows, and
    `tools/import_estv_deductions.py` documents the merge: Vaud's
    "Pauschalabzug Unterhaltskosten von vermieteten Liegenschaften mit Alter bis
    20 Jahren" arrives as one row carrying `Prozent=10` and a second carrying
    `Maximum=15000`, and the rule is the conjunction of the two. A field
    non-zero in exactly one row is taken from that row. A field non-zero in both
    rows with *different* values is a conflict, returned as such rather than
    silently resolved.

    Returns the merged rules and any conflicts found while merging.
    """
    merged: dict[tuple[str, str], dict[str, float]] = {}
    conflicts: list[str] = []
    for row in workbook_rows(path, "Export", RULE_HEADER, 8):
        canton, _steuerart, label, amount, percent, minimum, maximum = row[1:]
        key = (str(canton), str(label))
        values = {
            "amount": float(amount or 0.0),
            "percent": float(percent or 0.0),
            "minimum": float(minimum or 0.0),
            "maximum": float(maximum or 0.0),
        }
        if key not in merged:
            merged[key] = dict(values)
            continue
        existing = merged[key]
        for field, value in values.items():
            if value == 0.0:
                continue
            if existing[field] == 0.0 or existing[field] == value:
                existing[field] = value
            else:
                conflicts.append(
                    f"CONFLICT {key[0]} / {key[1]!r}: {field} is "
                    f"{existing[field]} and {value} in two rows of the source"
                )
    out = [
        (
            canton,
            label,
            values["amount"],
            values["percent"],
            values["minimum"],
            values["maximum"],
        )
        for (canton, label), values in merged.items()
    ]
    return out, conflicts


def workbook_scales(path: str):
    """Every *income* means-tested scale in `estv_deduction_scales.xlsx`.

    The workbook also carries wealth (`Vermögen`) scales -- fourteen rows for
    two Fribourg abatements -- which the importer deliberately keeps out of the
    income model. They are excluded here for the same reason and counted, so the
    exclusion appears in the report instead of looking like a silent shortfall
    in coverage.
    """
    grouped: dict[tuple[str, str, str], list[tuple[float, float]]] = {}
    excluded = 0
    for row in workbook_rows(path, "Export", SCALE_HEADER, 7):
        canton, steuerart, name, authority, threshold, amount = row[1:]
        if str(steuerart).lower() != "einkommen":
            excluded += 1
            continue
        key = (str(canton), str(name), str(authority))
        grouped.setdefault(key, []).append((float(threshold or 0.0), float(amount or 0.0)))
    out = [
        (canton, name, authority, tuple(sorted(points)))
        for (canton, name, authority), points in grouped.items()
    ]
    return out, excluded


def classify_from_label(label: str) -> str:
    """The kind a row has, judged from its label alone.

    `estv_deductions_data.rs` documents the classes it expects: thresholds are
    labelled `Schwellwert`, tax reliefs are `Steuerermässigung`, and the
    remaining parameters are stated ceilings and factors. This function is
    written from that documentation, not from the importer, so agreement
    between the two is evidence rather than a tautology.

    `Schwellwert` is matched anywhere in the label rather than only at its
    start: Lucerne and Schwyz write `Entlastungsabzug Alleinstehende
    Schwellwert` and Vaud writes `Familien Sozialabzug Schwellwert Austritt 1`,
    and their amounts (CHF 50,000; 80,000; 35,000; 70,000; 25,000; 126,300;
    164,100) are income thresholds, not reliefs.
    """
    if "Schwellwert" in label:
        return "Threshold"
    if label.startswith("Steuerermässigung"):
        return "Other"
    if label.startswith("Maximaler technischer Zinssatz"):
        return "Other"
    if label.startswith("Faktor "):
        return "Other"
    if label.startswith("Krankenkasse Durchschnittsprämie"):
        return "Other"
    return "Deduction"


def normalise_label(label: str) -> str:
    """Collapse runs of whitespace in a label before comparing.

    Several ESTV labels carry a double space (`Verheiratete mit  Beiträgen
    Säule 2/3a`). The importer collapses them, so the data file spells those
    labels with a single space, and a verbatim comparison would report every
    affected canton as a mismatch. Collapsing on *both* sides keeps the
    comparison honest about the numbers while acknowledging the one
    normalisation the importer performs. It is a whitespace normalisation and
    nothing more: case, accents and punctuation are still compared exactly.
    """
    return " ".join(label.split())


def keyed(items, key):
    out: dict = {}
    for item in items:
        out.setdefault(key(item), []).append(item)
    return out


def main() -> int:
    emit = "--emit" in sys.argv
    rules_path = os.path.join(SOURCES, "estv_deductions.xlsx")
    scales_path = os.path.join(SOURCES, "estv_deduction_scales.xlsx")
    for path in (rules_path, scales_path):
        if not os.path.exists(path):
            print(f"missing source workbook: {path}")
            return 1

    with open(RUST, encoding="utf-8") as handle:
        source = handle.read()

    problems = 0

    # ---- rules -----------------------------------------------------------
    rust = rust_rules(source)
    book, merge_conflicts = workbook_rules(rules_path)
    for message in merge_conflicts:
        problems += 1
        print(message)

    by_label_rust = keyed(rust, lambda r: (r[1], normalise_label(r[2])))
    by_label_book = keyed(book, lambda r: (r[0], normalise_label(r[1])))

    for key in sorted(set(by_label_rust) | set(by_label_book)):
        in_rust = by_label_rust.get(key, [])
        in_book = by_label_book.get(key, [])
        if len(in_rust) == len(in_book) == 1:
            r, b = in_rust[0], in_book[0]
            for index, field in ((4, "amount"), (5, "percent"), (6, "minimum"), (7, "maximum")):
                if abs(r[index] - b[index - 2]) > 1e-9:
                    problems += 1
                    print(
                        f"MISMATCH rule {key[0]} / {key[1]!r}: {field} "
                        f"{r[index]} in the data file, {b[index - 2]} in the workbook"
                    )
            continue
        if len(in_rust) > len(in_book):
            problems += 1
            print(
                f"MISSING in workbook: {key[0]} / {key[1]!r} has {len(in_rust)} "
                f"constant(s) and {len(in_book)} row(s)"
            )
        elif len(in_book) > len(in_rust):
            problems += 1
            print(
                f"NOT IMPORTED: {key[0]} / {key[1]!r} has {len(in_book)} row(s) "
                f"and {len(in_rust)} constant(s)"
            )
        else:
            # Duplicated on both sides with the same multiplicity: compare the
            # contents pairwise after sorting, since order is not meaningful.
            left = sorted(tuple(r[4:]) for r in in_rust)
            right = sorted(tuple(b[2:]) for b in in_book)
            if left != right:
                problems += 1
                print(f"MISMATCH rule {key[0]} / {key[1]!r}: {left} against {right}")

    # Kind is a judgement, so it is checked against an independently written
    # label rule and disagreements are reported individually.
    for identifier, _canton, name, kind, *_rest in rust:
        expected = classify_from_label(name)
        if kind != expected:
            problems += 1
            print(
                f"MISMATCH kind {identifier} ({name!r}): "
                f"RuleKind::{kind} in the data file, {expected} from the label"
            )

    # ---- scales ----------------------------------------------------------
    rust_scales_rows = rust_scales(source)
    book_scales_rows, wealth_rows = workbook_scales(scales_path)

    rust_by_name = keyed(
        rust_scales_rows, lambda s: (s[0], normalise_label(s[1]), s[2])
    )
    book_by_name = keyed(
        book_scales_rows, lambda s: (s[0], normalise_label(s[1]), s[2])
    )

    for key in sorted(set(rust_by_name) | set(book_by_name)):
        in_rust = rust_by_name.get(key, [])
        in_book = book_by_name.get(key, [])
        if not in_rust:
            problems += 1
            print(f"NOT IMPORTED: scale {key[0]} / {key[1]!r} ({key[2]})")
            continue
        if not in_book:
            problems += 1
            print(f"MISSING in workbook: scale {key[0]} / {key[1]!r} ({key[2]})")
            continue
        if len(in_rust) > 1 or len(in_book) > 1:
            problems += 1
            print(
                f"AMBIGUOUS: scale {key[0]} / {key[1]!r} ({key[2]}) appears "
                f"{len(in_rust)}x in the data file and {len(in_book)}x in the workbook"
            )
            continue
        expected = in_book[0][3]
        actual = in_rust[0][4]
        if actual != expected:
            problems += 1
            print(f"MISMATCH scale {key[0]} / {key[1]!r} ({key[2]})")
            print(f"  workbook {expected}")
            print(f"  data     {actual}")

    print(
        f"\ncompared {len(rust)} rule constants against {len(book)} merged workbook rules; "
        f"{len(rust_scales_rows)} scales against {len(book_scales_rows)} income scales "
        f"({wealth_rows} wealth rows out of scope); "
        f"{problems} problem(s)"
    )

    if emit:
        # Emit the *workbook's* values, not the data file's. A snapshot written
        # from the data file would let a Rust test compare the generated file
        # against itself; written from the workbook, the same test becomes a
        # genuine independent derivation.
        #
        # A fixed-width prefix and the label last, so a consumer can split with
        # a bounded split and let the label absorb its own commas. ESTV labels
        # contain commas (`Abzug für AHV/IV-Rentner, mit
        # Unterstützungspflicht`), and quoting them for a CSV reader would mean
        # shipping a CSV parser inside the Rust test that consumes this file.
        with open(SNAPSHOT, "w", encoding="utf-8", newline="\n") as handle:
            handle.write(
                "# Derivation: python tools/verify_estv_deductions_import.py --emit\n"
                "# Every value below is read from the ESTV workbooks and computed\n"
                "# here; nothing is copied out of src/estv_deductions_data.rs.\n"
                "# rule,canton,amount,percent,minimum,maximum,kind,label\n"
                "# scale,canton,authority,points,label\n"
                "# kind is derived from the label; points are threshold:amount\n"
                "# pairs separated by ';'. Labels keep the source's spacing.\n"
            )
            for canton, label, amount, percent, minimum, maximum in sorted(book):
                handle.write(
                    f"rule,{canton},{amount:.6f},{percent:.6f},{minimum:.6f},"
                    f"{maximum:.6f},{classify_from_label(label)},{label}\n"
                )
            for canton, name, authority, points in sorted(book_scales_rows):
                encoded = ";".join(f"{threshold:.6f}:{amount:.6f}" for threshold, amount in points)
                handle.write(f"scale,{canton},{authority},{encoded},{name}\n")
        print(f"wrote {SNAPSHOT}")

    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main())
