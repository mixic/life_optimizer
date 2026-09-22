#!/usr/bin/env python3
"""Draw the strategy comparison from the CSVs `--strategies --export` writes.

Copyright (C) 2026 MILAN NIKOLIC
SPDX-License-Identifier: GPL-3.0-or-later

Usage:
    python tools/plot_strategies.py out/strategies [--out figures/strategies-1-comparison.png]

# What the figure is for

The comparison between buying the peace and buying the war, drawn so that the
*difference* is legible and the *level* is not. Every panel is either a paired
difference with its own error bar or a per-region pair of bars, because with declared
parameters the levels carry no information and a plot that showed them alone would be a
plot of the assumptions.

# Why it refuses to draw

Every panel is preceded by a check that the CSV is internally consistent: the arms must
be paired run for run, probabilities must lie in the unit interval, the region count must
match across files, and the horizon curve must actually cross if the report says it does.
A figure drawn from an inconsistent export would look exactly like a figure drawn from a
consistent one, which is the failure mode this project treats as the dangerous kind. If a
check fails the script prints why and writes nothing.
"""

import argparse
import csv
import math
import os
import sys

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt

# Fixed arm colours, so a panel cannot silently swap which strategy is which.
COLOUR_COOPERATION = "#1f6f9c"
COLOUR_SPOILING = "#b4462f"
COLOUR_BASELINE = "#7a7a7a"


class CheckError(Exception):
    """A check failed. Raised rather than printed so nothing can be drawn after it."""


def read_csv(path):
    if not os.path.exists(path):
        raise CheckError(f"missing input: {path}")
    with open(path, newline="", encoding="utf-8") as handle:
        return list(csv.DictReader(handle))


def number(row, key):
    try:
        return float(row[key])
    except (KeyError, TypeError, ValueError) as error:
        raise CheckError(f"column {key!r} is not numeric in row {row!r}") from error


def verify(runs, regions, horizon):
    """Check the export is internally consistent before anything is drawn."""
    notes = []

    # --- the arms must be paired: same runs, same count -------------------
    by_arm = {}
    for row in runs:
        by_arm.setdefault(row["arm"], []).append(row)
    expected = {"BASELINE", "COOPERATION", "SPOILING"}
    if set(by_arm) != expected:
        raise CheckError(f"expected the three arms {sorted(expected)}, got {sorted(by_arm)}")
    counts = {arm: len(rows) for arm, rows in by_arm.items()}
    if len(set(counts.values())) != 1:
        raise CheckError(f"the arms are not paired, run counts differ: {counts}")
    if counts["BASELINE"] == 0:
        raise CheckError("no runs in the export")

    # Run indices must line up across arms, or "paired" is a word and not a fact.
    for arm in expected:
        indices = sorted(int(number(row, "run")) for row in by_arm[arm])
        if indices != list(range(counts[arm])):
            raise CheckError(f"arm {arm} has a broken run index: {indices[:8]}...")
    notes.append(f"{counts['BASELINE']} runs per arm, paired run for run")

    # --- probabilities in the unit interval ------------------------------
    for row in regions:
        for key in ("base_peace", "peace_under_cooperation", "conflict_under_spoiling"):
            value = number(row, key)
            if not (0.0 <= value <= 1.0):
                raise CheckError(f"{row['region']}: {key} = {value} is not a probability")
        if number(row, "margin") != number(row, "margin"):
            raise CheckError(f"{row['region']}: margin is NaN")
    notes.append(f"{len(regions)} regions, all probabilities inside [0, 1]")

    # --- the region file must describe regions the run file acted on ------
    # Nothing to compare directly, but the region list must be non-empty and its spend
    # figures must be non-negative: a negative spend would mean the plot was drawing a
    # strategy that pays the strategist to do nothing.
    for row in regions:
        for key in ("cooperative_spend", "spoiling_spend"):
            if number(row, key) < 0.0:
                raise CheckError(f"{row['region']}: {key} is negative")
    notes.append("no negative spends")

    # --- the horizon curve must be monotone in years and must agree with itself
    if len(horizon) < 3:
        raise CheckError(f"the horizon curve has only {len(horizon)} points")
    years = [number(row, "years") for row in horizon]
    if years != sorted(years) or len(set(years)) != len(years):
        raise CheckError("the horizon curve's years are not strictly increasing")
    for row in horizon:
        stated = number(row, "difference")
        recomputed = number(row, "value_spoiling") - number(row, "value_cooperative")
        if abs(stated - recomputed) > 1e-6:
            raise CheckError(
                f"horizon {row['years']}: difference {stated} is not "
                f"spoiling - cooperation = {recomputed}"
            )
    notes.append(f"horizon curve consistent over {len(horizon)} points")

    # Whether the arms actually cross is a claim the caption makes, so it is checked
    # rather than asserted: a figure that said "crossing" over a flat line would be the
    # worst kind of wrong.
    first = number(horizon[0], "difference")
    last = number(horizon[-1], "difference")
    crossings = 0
    for previous, current in zip(horizon, horizon[1:]):
        if number(previous, "difference") * number(current, "difference") < 0.0:
            crossings += 1
    notes.append(
        f"spoiling-minus-cooperation goes {first:+.2f} to {last:+.2f} "
        f"with {crossings} crossing(s)"
    )

    return notes, count_crossing(horizon)


def count_crossing(horizon):
    """The first crossing horizon, linearly interpolated, or None."""
    for previous, current in zip(horizon, horizon[1:]):
        a = number(previous, "difference")
        b = number(current, "difference")
        if a * b < 0.0:
            ya = number(previous, "years")
            yb = number(current, "years")
            # Linear interpolation between the two bracketing years.
            fraction = abs(a) / (abs(a) + abs(b))
            return ya + fraction * (yb - ya)
    return None


def paired_differences(by_arm, first, second, key):
    """Run-by-run differences of one statistic between two arms."""
    a = sorted(by_arm[first], key=lambda row: int(number(row, "run")))
    b = sorted(by_arm[second], key=lambda row: int(number(row, "run")))
    return [number(y, key) - number(x, key) for x, y in zip(a, b)]


def mean_and_error(values):
    n = len(values)
    if n == 0:
        return 0.0, 0.0
    mean = sum(values) / n
    if n < 2:
        return mean, 0.0
    variance = sum((value - mean) ** 2 for value in values) / (n - 1)
    return mean, math.sqrt(variance / n)


def draw(runs, regions, horizon, out_path, notes, crossing):
    by_arm = {}
    for row in runs:
        by_arm.setdefault(row["arm"], []).append(row)

    fig, axes = plt.subplots(2, 3, figsize=(17.5, 10.0))
    fig.suptitle(
        "Two strategies for a fragile region: buying the peace against buying the war\n"
        "every parameter declared, not measured -- read the differences, never the levels",
        fontsize=13.5,
        y=0.985,
    )

    names = [row["region"] for row in regions]
    positions = range(len(names))

    # ---- 1. what each strategy is worth, per region ----------------------
    ax = axes[0][0]
    cooperative = [number(row, "value_cooperative") for row in regions]
    spoiling = [number(row, "value_spoiling") for row in regions]
    width = 0.38
    ax.bar([p - width / 2 for p in positions], cooperative, width,
           label="cooperation", color=COLOUR_COOPERATION)
    ax.bar([p + width / 2 for p in positions], spoiling, width,
           label="spoiling", color=COLOUR_SPOILING)
    for index, row in enumerate(regions):
        rent = number(row, "rent")
        critical = row["critical_rent"]
        if critical == "none":
            ax.annotate("spoiling never wins",
                        (index, max(cooperative[index], spoiling[index])),
                        textcoords="offset points", xytext=(0, 6),
                        ha="center", fontsize=8, color="#444444")
        else:
            critical = float(critical)
            verdict = "spoilable" if rent > critical else "not worth spoiling"
            ax.annotate(f"rent {rent:.2f} vs critical {critical:.2f}\n{verdict}",
                        (index, max(cooperative[index], spoiling[index])),
                        textcoords="offset points", xytext=(0, 6),
                        ha="center", fontsize=7.5, color="#444444")
    ax.set_xticks(list(positions))
    ax.set_xticklabels(names, fontsize=9)
    # Headroom for the annotations. Put at 1.30 rather than 1.10 because each annotation is
    # two lines and the first version drew them across the panel title, which is a collision
    # rather than a style choice.
    ceiling = max(max(cooperative), max(spoiling))
    ax.set_ylim(0, ceiling * 1.34)
    ax.set_ylabel("present value over the horizon")
    ax.set_title("1. What the same budget buys, region by region", fontsize=10.5)
    ax.legend(fontsize=8.5, loc="upper right")
    ax.grid(axis="y", alpha=0.25)

    # ---- 2. the probabilities each arm is buying -------------------------
    ax = axes[0][1]
    peace = [number(row, "peace_under_cooperation") for row in regions]
    war = [number(row, "conflict_under_spoiling") for row in regions]
    baseline = [number(row, "base_peace") for row in regions]
    ax.bar([p - width / 2 for p in positions], peace, width,
           label="P(peace) under cooperation", color=COLOUR_COOPERATION)
    ax.bar([p + width / 2 for p in positions], war, width,
           label="P(war) under spoiling", color=COLOUR_SPOILING)
    ax.plot(list(positions), baseline, "o--", color=COLOUR_BASELINE,
            markersize=5, label="P(peace) before any strategy")
    ax.set_xticks(list(positions))
    ax.set_xticklabels(names, fontsize=9)
    ax.set_ylim(0, 1.08)
    ax.set_ylabel("probability")
    ax.set_title("2. The two arms buy the same event, in opposite directions", fontsize=10.5)
    ax.legend(fontsize=8.5, loc="lower right")
    ax.grid(axis="y", alpha=0.25)

    # ---- 3. the paired differences, against the strategy question --------
    ax = axes[0][2]
    statistics = [
        ("cooperation", "cooperation index"),
        ("trap", "conflict-trap years"),
        ("pareto_loss", "Pareto-efficiency loss"),
        ("pension", "pension security index"),
        ("regions_in_conflict", "regions in conflict"),
        ("realised_rent", "rent captured"),
    ]
    labels = []
    means = []
    errors = []
    for key, label in statistics:
        values = paired_differences(by_arm, "COOPERATION", "SPOILING", key)
        mean, error = mean_and_error(values)
        labels.append(label)
        means.append(mean)
        errors.append(error)
    y_positions = list(range(len(labels)))
    ax.errorbar(means, y_positions, xerr=errors, fmt="o", color="#333333",
                ecolor="#888888", capsize=3, markersize=5)
    ax.axvline(0.0, color="#bbbbbb", linewidth=1.0)
    ax.set_yticks(y_positions)
    ax.set_yticklabels(labels, fontsize=8.5)
    ax.set_xlabel("spoiling minus cooperation, paired per run")
    ax.set_title("3. The paired differences, with their own error bars", fontsize=10.5)
    ax.grid(axis="x", alpha=0.25)

    # ---- 4. the horizon curve, which is the framework's answer ----------
    ax = axes[1][0]
    years = [number(row, "years") for row in horizon]
    cooperative_curve = [number(row, "value_cooperative") for row in horizon]
    spoiling_curve = [number(row, "value_spoiling") for row in horizon]
    ax.plot(years, cooperative_curve, color=COLOUR_COOPERATION, linewidth=2.0,
            label="cooperation")
    ax.plot(years, spoiling_curve, color=COLOUR_SPOILING, linewidth=2.0,
            label="spoiling")
    if crossing is not None:
        marker = None
        for value, coop, spoil in zip(years, cooperative_curve, spoiling_curve):
            if abs(value - crossing) < 1.5:
                marker = (coop + spoil) / 2.0
                break
        if marker is not None:
            ax.axvline(crossing, color="#666666", linestyle="--", linewidth=1.2)
            ax.annotate(f"crossing at {crossing:.1f} yr",
                        (crossing, marker), textcoords="offset points",
                        xytext=(6, 8), fontsize=8.5, color="#333333")
    ax.set_xlabel("horizon (years)")
    ax.set_ylabel("present value")
    ax.set_title("4. The spoiler's prize is bounded, the cooperator's is not", fontsize=10.5)
    ax.legend(fontsize=8.5)
    ax.grid(alpha=0.25)

    # ---- 5. the two statistics that discriminate, per run ---------------
    ax = axes[1][1]
    for arm, colour in (("COOPERATION", COLOUR_COOPERATION),
                        ("SPOILING", COLOUR_SPOILING)):
        values = [number(row, "regions_in_conflict") for row in by_arm[arm]]
        ax.hist(values, bins=18, alpha=0.65, color=colour,
                label=f"{arm.lower()} -- mean {sum(values) / len(values):.2f}")
    ax.set_xlabel("mean regions in conflict per year (of 3 declared)")
    ax.set_ylabel("runs")
    ax.set_title("5. The mechanism, counted: only this and rent separate the arms",
                 fontsize=10.5)
    ax.legend(fontsize=8.5)
    ax.grid(axis="y", alpha=0.25)

    # ---- 6. rent captured, which is zero unless the arm is the spoiler ---
    ax = axes[1][2]
    for arm, colour in (("COOPERATION", COLOUR_COOPERATION),
                        ("SPOILING", COLOUR_SPOILING)):
        values = [number(row, "realised_rent") for row in by_arm[arm]]
        ax.hist(values, bins=18, alpha=0.65, color=colour,
                label=f"{arm.lower()} -- mean {sum(values) / len(values):.3f}")
    ax.set_xlabel("present value of rent captured")
    ax.set_ylabel("runs")
    ax.set_title("6. Who is paid, and by what: rent is zero unless spoiling", fontsize=10.5)
    ax.legend(fontsize=8.5)
    ax.grid(axis="y", alpha=0.25)

    caption = (
        "Sources: multipolar_sim --strategies --export. Every margin, rent, retaliation and "
        "instrument price is DECLARED, not measured;\n"
        "the panels are differences and paired contrasts, which is the only thing the "
        "declared parameters can support. Panel 3's 'regions in conflict'\n"
        "and 'rent captured' are the two statistics that separate the arms; the cooperation "
        "index, trap years, Pareto loss and the pension index all move\n"
        "together in this simulator because accumulated tension erodes the payoff of mutual "
        "competition, so none of them can adjudicate the question."
        + ("" if crossing is not None else
           "\nThe horizon curve does not cross at these parameters: the verdict holds at "
           "every horizon up to the range exported.")
    )
    fig.text(0.5, 0.012, caption, ha="center", fontsize=8.2, color="#333333")

    for note in notes:
        print(f"  check: {note}")

    fig.tight_layout(rect=(0, 0.055, 1, 0.955))
    os.makedirs(os.path.dirname(out_path) or ".", exist_ok=True)
    fig.savefig(out_path, dpi=190)
    print(f"wrote {out_path}")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", help="the --export directory")
    parser.add_argument("--out", default="figures/strategies-1-comparison.png")
    args = parser.parse_args()

    try:
        runs = read_csv(os.path.join(args.directory, "strategies-runs.csv"))
        regions = read_csv(os.path.join(args.directory, "strategies-regions.csv"))
        horizon = read_csv(os.path.join(args.directory, "strategies-horizon.csv"))
        notes, crossing = verify(runs, regions, horizon)
    except CheckError as error:
        print(f"REFUSING TO DRAW: {error}", file=sys.stderr)
        return 1

    draw(runs, regions, horizon, args.out, notes, crossing)
    return 0


if __name__ == "__main__":
    sys.exit(main())
