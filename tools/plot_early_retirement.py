#!/usr/bin/env python3
"""Draw the early-retirement risk figures from the CSV the CLI exports.

Copyright (C) 2026 MILAN NIKOLIC
SPDX-License-Identifier: GPL-3.0-or-later

Usage:
    python tools/plot_early_retirement.py out/early-retirement \
        --out figures/early-retirement-1-risk.png

# What the figure is for

The question is "how much is enough", and the answer is a curve rather than a number: the
probability of running out falls with the retirement age, and the median balance left at
death does not fall with it. Drawing both against the same axis is what makes the trade
visible — retiring later buys safety with years of work, not with a bigger pot.

# Why it refuses to draw

The export is checked before anything is drawn: the ages must be strictly increasing and
the shortfall probability must lie in the unit interval. A figure drawn from an
inconsistent export looks exactly like one drawn from a consistent export, which is the
failure mode this repository treats as the dangerous kind.
"""

import argparse
import csv
import os
import sys

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt


class CheckError(Exception):
    """A check failed. Raised so nothing can be drawn after it."""


def read_csv(path):
    if not os.path.exists(path):
        raise CheckError(f"missing input: {path}")
    with open(path, newline="", encoding="utf-8") as handle:
        rows = list(csv.DictReader(handle))
    if not rows:
        raise CheckError(f"{path} has no rows")
    return rows


def number(row, key):
    try:
        return float(row[key])
    except (KeyError, TypeError, ValueError) as error:
        raise CheckError(f"column {key!r} is not numeric in row {row!r}") from error


def verify(rows):
    ages = [number(row, "age") for row in rows]
    if ages != sorted(ages) or len(set(ages)) != len(ages):
        raise CheckError(f"ages are not strictly increasing: {ages}")
    for row in rows:
        value = number(row, "shortfall_probability")
        if not (0.0 <= value <= 1.0):
            raise CheckError(
                f"age {row['age']}: shortfall probability {value} is not a probability"
            )
    shorts = [number(row, "shortfall_probability") for row in rows]
    notes = [
        f"{len(rows)} retirement ages from {int(ages[0])} to {int(ages[-1])}",
        f"shortfall probability runs {min(shorts):.3f} to {max(shorts):.3f}",
    ]
    # The claim the caption makes, checked rather than asserted: later retirement must not
    # raise the shortfall probability. If it did, the model's central recommendation would
    # be backwards and the figure would be advertising it.
    for previous, current in zip(rows, rows[1:]):
        if number(current, "shortfall_probability") > number(previous, "shortfall_probability") + 1e-9:
            raise CheckError(
                f"shortfall probability rises from age {previous['age']} to {current['age']}, "
                "so retiring later would be worse"
            )
    notes.append("shortfall probability is non-increasing in the retirement age")
    return notes


def draw(rows, out_path, notes):
    ages = [number(row, "age") for row in rows]
    shortfall = [number(row, "shortfall_probability") * 100.0 for row in rows]
    median_end = [number(row, "median_terminal_wealth") for row in rows]
    capital = [number(row, "capital") for row in rows]

    fig, axes = plt.subplots(1, 3, figsize=(16.5, 5.2))
    fig.suptitle(
        "Early retirement: the least that is enough, and what it costs in risk\n"
        "declared parameters on all sides — read the shapes, not the levels",
        fontsize=13,
        y=0.98,
    )

    ax = axes[0]
    ax.plot(ages, shortfall, "o-", color="#b4462f", linewidth=2.0, markersize=5)
    ax.axhline(10.0, color="#666666", linestyle="--", linewidth=1.0)
    ax.annotate("declared 10% target", (ages[0], 11.5), fontsize=8.5, color="#444444")
    ax.set_xlabel("retirement age")
    ax.set_ylabel("probability the pot runs out (%)")
    ax.set_title("1. Retiring later buys safety", fontsize=10.5)
    ax.grid(alpha=0.25)
    ax.set_ylim(-4, max(shortfall) * 1.12)

    ax = axes[1]
    ax.plot(ages, median_end, "o-", color="#1f6f9c", linewidth=2.0, markersize=5)
    ax.set_xlabel("retirement age")
    ax.set_ylabel("median unconsumed wealth at death (CHF)")
    ax.set_title("2. The median balance left barely moves", fontsize=10.5)
    ax.grid(alpha=0.25)

    ax = axes[2]
    ax.plot(ages, capital, "o-", color="#2f7d4f", linewidth=2.0, markersize=5)
    ax.set_xlabel("retirement age")
    ax.set_ylabel("total capital (CHF)")
    ax.set_title("3. The pot barely moves; the annuity does the work", fontsize=10.5)
    ax.grid(alpha=0.25)

    caption = (
        "Source: life-optimizer early-retirement --export. Every statutory figure is "
        "sourced and every fund- or canton-specific one is declared;\n"
        "the conversion rate's early-withdrawal reduction has NO statutory schedule, so a "
        "plan built on the default is a plan built on a guess.\n"
        "Panel 2 is the point of the exercise: a plan that funds the consumption and still "
        "leaves a large balance has saved too much, not wisely."
    )
    fig.text(0.5, 0.015, caption, ha="center", fontsize=8.2, color="#333333")

    for note in notes:
        print(f"  check: {note}")

    fig.tight_layout(rect=(0, 0.06, 1, 0.93))
    os.makedirs(os.path.dirname(out_path) or ".", exist_ok=True)
    fig.savefig(out_path, dpi=190)
    print(f"wrote {out_path}")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", help="the --export directory")
    parser.add_argument("--out", default="figures/early-retirement-1-risk.png")
    args = parser.parse_args()

    try:
        rows = read_csv(os.path.join(args.directory, "early-retirement-risk.csv"))
        notes = verify(rows)
    except CheckError as error:
        print(f"REFUSING TO DRAW: {error}", file=sys.stderr)
        return 1

    draw(rows, args.out, notes)
    return 0


if __name__ == "__main__":
    sys.exit(main())

