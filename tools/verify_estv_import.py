#!/usr/bin/env python3
"""Re-derive the cantonal tax brackets from the ESTV workbooks and compare them to the Rust.

Copyright (C) 2026 MILAN NIKOLIC
SPDX-License-Identifier: GPL-3.0-or-later

Usage:
    python tools/verify_estv_import.py            # compare, print a report
    python tools/verify_estv_import.py --emit     # also write the expected-value CSV

# Why this exists

`VALIDATION.md` §5 said the cantonal base scales "are not checked against anything outside the
repository". That was wrong about the sourcing -- `src/estv_scales_data.rs` is generated from the
official ESTV Tarife workbooks committed in `source-documents/` -- but right about the checking:
nothing independently re-derived the constants from those workbooks. A generated file that only
its own generator has ever read is sourced and unverified at the same time.

This script is the independent derivation. It reads the workbooks directly with openpyxl, applies
the tariff arithmetic itself (each row says "for the next CHF n, additionally r%", so the threshold
is the running total), and compares the result against the constants in the generated Rust.

It deliberately does not import `tools/import_estv_scales.py`: a verification that reuses the code
under test verifies nothing.

# What it does not cover

Tier-four cases the source does not make comparable: flat-rate cantons, which have no bands, and
formula cantons such as Basel-Landschaft, which publish algebra rather than brackets. Those are
reported as skipped with the reason, not silently omitted.
"""

from __future__ import annotations

import csv
import glob
import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SOURCES = os.path.join(ROOT, "source-documents")
RUST = os.path.join(ROOT, "src", "estv_scales_data.rs")

# A generated bracket: FederalBracket { threshold: 7000.0, rate: 0.02 }
BRACKET = re.compile(r"threshold:\s*(-?[\d.]+),\s*rate:\s*(-?[\d.]+)")


def rust_brackets(source: str) -> dict[str, list[tuple[float, float]]]:
    """Every `pub const XX_SINGLE_BRACKETS` / `_MARRIED_BRACKETS` block, keyed by name."""
    out: dict[str, list[tuple[float, float]]] = {}
    pattern = re.compile(
        r"pub const ([A-Z]{2})_(SINGLE|MARRIED)_BRACKETS\s*:\s*&\[FederalBracket\]\s*=\s*&\[(.*?)\];",
        re.S,
    )
    for match in pattern.finditer(source):
        code, status, body = match.group(1), match.group(2), match.group(3)
        out[f"{code}_{status}"] = [
            (float(threshold), float(rate)) for threshold, rate in BRACKET.findall(body)
        ]
    return out


def workbook_brackets(path: str, married: bool) -> list[tuple[float, float]] | None:
    """The cantonal bracket ladder, or `None` when the workbook does not publish one.

    Each row states the width of the next band and the rate that applies to it, so the threshold
    is the running sum of the widths before it. `rate` is the percentage on that band, converted
    to the decimal the Rust constants use.
    """
    import openpyxl

    # Two schemas, and misreading one as the other produces false mismatches rather than
    # catching real ones. Some exports state a band *width* -- "for the next CHF n, additionally
    # r%" -- so the threshold is a running total. Others state the taxable income at which the
    # rate changes, with a "Grundbetrag" that keeps the tax continuous. Detect from the header.
    #
    # Two subject conventions also coexist: cantons that split the tariff by Steuersubjekt
    # ("Alleinstehend ohne Kinder" against "Verheiratet / Alleinstehend mit Kindern"), and cantons
    # that publish one tariff labelled "Alle".

    book = openpyxl.load_workbook(path, data_only=True)
    sheet = book[book.sheetnames[0]]

    threshold_mode = False
    ladders: dict[str, list[tuple[float, float]]] = {}
    running: dict[str, float] = {}
    for row in sheet.iter_rows(min_row=1, values_only=True):
        cells = ["" if cell is None else str(cell).strip() for cell in row]
        if len(cells) < 7:
            continue
        if cells[2] == "Steuerart":
            threshold_mode = "Steuerbares Einkommen" in cells[5] or "Steuerbares" in cells[5]
            continue
        steuerart, subject, hoheit, value, rate = cells[2], cells[3], cells[4], cells[5], cells[6]
        if steuerart != "Einkommen" or hoheit != "Kanton":
            continue
        try:
            number = float(value)
            percent = float(rate)
        except ValueError:
            continue
        if threshold_mode:
            threshold = number
        else:
            threshold = running.get(subject, 0.0)
            running[subject] = threshold + number
        ladders.setdefault(subject, []).append((round(threshold, 6), round(percent / 100.0, 6)))

    if not ladders:
        return None

    def pick(predicate) -> list[tuple[float, float]] | None:
        for label, ladder in ladders.items():
            if predicate(label):
                return ladder
        return None

    chosen = pick(lambda label: "Verheiratet" in label) if married else pick(
        lambda label: "Alleinstehend" in label and "mit Kindern" not in label
    )
    if chosen is None:
        chosen = pick(lambda label: label == "Alle")
    return chosen


def main() -> int:
    with open(RUST, encoding="utf-8") as handle:
        generated = rust_brackets(handle.read())

    emit = "--emit" in sys.argv
    rows: list[dict[str, str]] = []
    compared = 0
    mismatched = 0
    skipped: list[str] = []

    for path in sorted(glob.glob(os.path.join(SOURCES, "estv_scales_*.xlsx"))):
        code = os.path.basename(path).split("_")[-1].split(".")[0].upper()
        if code == "BUND":
            continue
        for married in (False, True):
            status = "MARRIED" if married else "SINGLE"
            key = f"{code}_{status}"
            expected = workbook_brackets(path, married)
            if expected is None:
                skipped.append(f"{key} (no band tariff in the source)")
                continue
            actual = generated.get(key)
            if actual is None:
                skipped.append(f"{key} (no generated constant)")
                continue
            compared += 1
            # Compare numerically, not by representation: 0.018 and 0.018000000000000002 are the
            # same rate written twice, and a verifier that reports them as a mismatch would bury
            # the discrepancies that are real. The rate tolerance is 1e-6 rather than exact
            # because Jura's imported rates carry two extra digits (1.666702% against 1.6667%),
            # which is CHF 0.02 of tax on CHF 100,000 -- immaterial, and recorded here rather
            # than hidden by a loosened bound with no explanation.
            same = len(actual) == len(expected) and all(
                abs(a_threshold - e_threshold) < 1e-9 and abs(a_rate - e_rate) < 1e-6
                for (a_threshold, a_rate), (e_threshold, e_rate) in zip(actual, expected)
            )
            if not same:
                mismatched += 1
                print(f"MISMATCH {key}")
                print(f"  source   {expected[:6]}{' ...' if len(expected) > 6 else ''}")
                print(f"  rust     {actual[:6]}{' ...' if len(actual) > 6 else ''}")
            for index, (threshold, rate) in enumerate(expected):
                rows.append(
                    {
                        "constant": key,
                        "index": str(index),
                        "threshold": f"{threshold:.6f}",
                        "rate": f"{rate:.6f}",
                    }
                )

    print(f"\ncompared {compared} bracket ladders; {mismatched} mismatch(es)")
    if skipped:
        print(f"skipped {len(skipped)}:")
        for entry in skipped:
            print(f"  {entry}")

    if emit:
        target = os.path.join(SOURCES, "estv_scales_expected.csv")
        with open(target, "w", encoding="utf-8", newline="") as handle:
            writer = csv.DictWriter(
                handle, fieldnames=["constant", "index", "threshold", "rate"]
            )
            writer.writeheader()
            writer.writerows(rows)
        print(f"wrote {target} ({len(rows)} bracket rows)")

    return 1 if mismatched else 0


if __name__ == "__main__":
    sys.exit(main())
