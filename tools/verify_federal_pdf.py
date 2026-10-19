#!/usr/bin/env python3
"""Read the official federal tariff PDF and compare it with the imported ladder.

`source-documents/Tarif Art. 42 - Abs. 1 - 2026-20XX_d.pdf` is the ESTV's
published direct-federal-tax tariff for married taxpayers from Steuerjahr 2026.
It is in **Einheits-Satz** form: for each taxable income it prints a rate and the
resulting simple tax, rather than the marginal bands `federal_tariff_data.rs`
holds. The two are meant to describe the same tax, so this script extracts the
PDF's rows and integrates the imported ladder for comparison.

# What it found

They do not agree, at the bottom of the table:

| Taxable income | PDF rate | PDF ``Einfache Steuer`` | Tax from the imported ladder |
|---:|---:|---:|---:|
| CHF 100 | 1.5500% | 1.55 | 0.00 |
| CHF 200 | 1.5500% | 3.10 | 0.00 |
| CHF 900 | 1.5500% | 13.95 | 0.00 |

The imported married ladder opens with a **zero** band up to CHF 29,700 -- its
first two points are ``(0, 0.0)`` and ``(29_700, 0.01)`` -- so it charges nothing
below that income. The PDF charges 1.55% from the first CHF 100.

Both cannot be right. The statutory tariff is in ``source-documents/SR 642.11.pdf``
(the DBG), but that file yields no text at all: it is the Identity-H case
described in ``SWISS_TAX_DATA.md``, and neither this script nor the repository's
other PDF tools can read it. So this script records the contradiction instead of
resolving it, which is the honest outcome: the export is official, the PDF is
official, and they disagree.

# Coverage, and why it is small

Only the first nine rows extract. The PDF lays each page out in three column
blocks, and beyond the first few rows the income tokens are emitted in a
different order from the rate and tax tokens, so the strict
``(income, rate, tax)`` triples this script looks for stop matching. Widening the
matcher without widening the evidence would risk inventing rows, so it does not.

The finding therefore rests on nine rows and is stated as such: the lowest nine
rows of the married federal tariff **disagree**, and the rest of the table was
not compared. The single taxpayer's ladder is not in this PDF at all -- it holds
only the Verheiratete table.

Usage:
    python tools/verify_federal_pdf.py            # report
    python tools/verify_federal_pdf.py --emit     # also write the snapshot

Exit status is 0 when the rows agree, 1 when they do not, so a caller that only
wants the report can still read ``MISMATCH`` lines.
"""

from __future__ import annotations

import csv
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

import pdf_text  # noqa: E402

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SOURCES = os.path.join(ROOT, "source-documents")
PDF = os.path.join(SOURCES, "Tarif Art. 42 - Abs. 1 - 2026-20XX_d.pdf")
LADDER_CSV = os.path.join(SOURCES, "estv_scales_expected.csv")
SNAPSHOT = os.path.join(SOURCES, "federal_pdf_expected.csv")

INTEGER = re.compile(r"^\d{1,3}(?:'\d{3})*$")
RATE = re.compile(r"^\d\.\d{4}$")
TAX = re.compile(r"^\d{1,3}(?:'\d{3})*\.\d{2}$")


def number(text: str) -> float:
    return float(text.replace("'", ""))


def pdf_rows(path: str) -> list[tuple[int, float, float]]:
    """(taxable income, Einheits-Satz in percent, einfache Steuer in CHF)."""
    with open(path, "rb") as handle:
        data = handle.read()
    tokens = [piece.strip() for piece in pdf_text.streams_to_text(
        pdf_text.inflate_streams(data)) if piece.strip()]

    rows: list[tuple[int, float, float]] = []
    index = 0
    while index < len(tokens) - 2:
        if (
            INTEGER.match(tokens[index])
            and RATE.match(tokens[index + 1])
            and TAX.match(tokens[index + 2])
        ):
            rows.append(
                (int(number(tokens[index])), float(tokens[index + 1]),
                 number(tokens[index + 2]))
            )
            index += 3
        else:
            index += 1
    return rows


def married_ladder(path: str) -> list[tuple[float, float]]:
    """The imported federal married ladder, from the workbook-derived snapshot."""
    if not os.path.exists(path):
        raise SystemExit(
            f"missing {path}; run `python tools/verify_estv_import.py --emit` first"
        )
    with open(path, encoding="utf-8") as handle:
        return [
            (float(row[2]), float(row[3]))
            for row in csv.reader(handle)
            if row and row[0] == "BUND_MARRIED"
        ]


def tax_from_ladder(ladder: list[tuple[float, float]], income: float) -> float:
    """Integrate the marginal bands up to `income`."""
    total = 0.0
    for index, (threshold, rate) in enumerate(ladder):
        if income <= threshold:
            break
        following = ladder[index + 1][0] if index + 1 < len(ladder) else float("inf")
        span = min(income, following) - threshold
        if span > 0.0:
            total += span * rate
    return total


def main() -> int:
    emit = "--emit" in sys.argv
    if not os.path.exists(PDF):
        print(f"missing source PDF: {PDF}")
        return 1

    rows = pdf_rows(PDF)
    ladder = married_ladder(LADDER_CSV)
    if not rows:
        print("no (income, rate, tax) triples extracted from the PDF")
        return 1

    mismatched = 0
    for income, rate, tax in rows:
        from_ladder = tax_from_ladder(ladder, float(income))
        # The PDF's own rate and simple tax agree exactly (tax = income x rate/100),
        # which is a self-consistency check on the extraction rather than on the tariff.
        self_consistent = abs(tax - income * rate / 100.0) < 0.005
        agrees = abs(from_ladder - tax) < 0.005
        if not self_consistent or not agrees:
            mismatched += 1
            print(
                f"MISMATCH income {income}: PDF rate {rate:.4f}% tax {tax:.2f}; "
                f"ladder {from_ladder:.2f}"
                + ("" if self_consistent else " (PDF row self-inconsistent)")
            )

    print(
        f"\nextracted {len(rows)} rows from the PDF, compared against "
        f"{len(ladder)} imported married bands; {mismatched} disagreement(s)"
    )
    if rows:
        print(f"  PDF income range: {min(r[0] for r in rows)} - {max(r[0] for r in rows)}")
    print(
        "  coverage: only the rows the layout allows to be read unambiguously; "
        "the rest of the table was not compared"
    )

    if emit:
        with open(SNAPSHOT, "w", encoding="utf-8", newline="\n") as handle:
            handle.write(
                "# Derivation: python tools/verify_federal_pdf.py --emit\n"
                "# income,rate_percent,tax_chf -- read from the ESTV federal tariff PDF\n"
                "# for married taxpayers from Steuerjahr 2026 (Einheits-Satz form)\n"
            )
            for income, rate, tax in rows:
                handle.write(f"{income},{rate:.4f},{tax:.2f}\n")
        print(f"wrote {SNAPSHOT}")

    return 1 if mismatched else 0


if __name__ == "__main__":
    sys.exit(main())
