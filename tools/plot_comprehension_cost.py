#!/usr/bin/env python3
"""Draw the comprehension-cost figures from the CSV the model exports.

Copyright (C) 2026 MILAN NIKOLIC
SPDX-License-Identifier: GPL-3.0-or-later

Usage:
    cargo run -p comprehension_cost -- --sweep --export out/comprehension
    python tools/plot_comprehension_cost.py out/comprehension

# What these figures are for, and what they are not

They are not evidence about software engineering. Every number comes from a model whose
comprehension-side parameters are **Declared** — invented, plausible in sign, unmeasured.
The figures therefore show how the model behaves, which is a statement about its
assumptions, not about the world.

What they do show, and what makes them worth drawing, is the *structure* of the argument:
that the result turns on one mechanism rather than another, and that the model is capable
of reporting the opposite conclusion when that mechanism is removed. The last figure is
the important one.

# Why it refuses to draw

The export is checked before anything is rendered, and a failed check raises rather than
skipping the figure. A plot drawn from an inconsistent export looks exactly like one drawn
from a consistent export, which is the failure this repository treats as the dangerous
kind. The control figure additionally refuses to exist unless the control *holds*: if the
model cannot report a win with the asymmetry removed, then the negative result elsewhere
is a construction artefact and should not be shown as a finding.
"""

import argparse
import csv
import os
import sys

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt

# Every figure carries this on its face. A reader who crops it off has been misled by the
# presentation, and this repository writes its limits into the artefact rather than a
# footnote.
FOOTER = (
    "Model output with Declared parameters — an illustration of the model's structure, "
    "not a measurement of any developer."
)


class CheckError(Exception):
    """A check failed. Raised so nothing is drawn after it."""


def read_csv(path):
    if not os.path.exists(path):
        raise CheckError(f"missing input: {path}")
    with open(path, newline="", encoding="utf-8") as handle:
        rows = list(csv.DictReader(handle))
    if not rows:
        raise CheckError(f"empty input: {path}")
    return rows


def numbers(rows, key, path):
    out = []
    for row in rows:
        try:
            out.append(float(row[key]))
        except (TypeError, ValueError):
            raise CheckError(f"{path}: column {key!r} is not numeric in {row}")
    return out


def optional_int(text):
    text = (text or "").strip()
    if not text:
        return None
    try:
        return int(text)
    except ValueError:
        raise CheckError(f"crossover month {text!r} is not an integer or empty")


# --------------------------------------------------------------------------- checks


def check_phase(rows, horizon):
    path = "phase.csv"
    expected = (
        {0.00, 0.15, 0.33},
        {6.0, 9.0, 12.0, 18.0, 24.0},
        {0.00, 0.10, 0.20, 0.50},
    )
    got = (
        {round(float(r["savings"]), 2) for r in rows},
        {round(float(r["half_life_ai"]), 0) for r in rows},
        {round(float(r["epsilon"]), 2) for r in rows},
    )
    for index, (want, have) in enumerate(zip(expected, got)):
        if want != have:
            raise CheckError(f"{path}: axis {index} is {sorted(have)}, expected {sorted(want)}")
    if len(rows) != 3 * 5 * 4:
        raise CheckError(f"{path}: {len(rows)} cells, expected 60")
    for row in rows:
        month = optional_int(row["crossover_month"])
        if month is not None and not (1 <= month <= horizon):
            raise CheckError(f"{path}: crossover month {month} outside 1..{horizon}")
        if optional_int(row["crossover_month"]) is None and float(row["benefit_end"]) < 0:
            raise CheckError(
                f"{path}: a cell with no crossover must not end negative: {row}"
            )


def check_central(rows, horizon):
    path = "central.csv"
    months = [int(float(r["month"])) for r in rows]
    if months != list(range(1, horizon + 1)):
        raise CheckError(f"{path}: months are not 1..{horizon} in order")
    # The export writes four decimal places, so three cumulative values in the hundreds
    # carry up to about 3e-4 of rounding between them. The tolerance is set to that and
    # no wider: it has to admit the rounding without admitting a benefit computed from a
    # different run, which would be off by whole hours.
    for row in rows:
        gap = (float(row["human_cumulative"]) - float(row["ai_cumulative"])) - float(
            row["benefit"]
        )
        if abs(gap) > 1e-3:
            raise CheckError(f"{path}: benefit does not equal the cumulative difference: {row}")
        for key in ("human_comprehension", "ai_comprehension"):
            value = float(row[key])
            if not 0.0 <= value <= 1.0:
                raise CheckError(f"{path}: {key} is {value}, outside [0, 1]")


def check_channels(rows):
    path = "channels.csv"
    if len(rows) != 5:
        raise CheckError(f"{path}: {len(rows)} rows, expected 5")
    previous = None
    for index, row in enumerate(rows):
        benefit = float(row["benefit"])
        if index == 0:
            if (row["delta"] or "").strip():
                raise CheckError(f"{path}: the first row must have no delta")
        else:
            delta = float(row["delta"])
            if abs((benefit - previous) - delta) > 1e-3:
                raise CheckError(f"{path}: delta does not match the benefit step in {row}")
        previous = benefit


def check_sensitivity(rows):
    path = "sensitivity.csv"
    if len(rows) < 10:
        raise CheckError(f"{path}: only {len(rows)} parameters")
    swings = [float(r["swing"]) for r in rows]
    for row in rows:
        low, high, swing = float(row["low"]), float(row["high"]), float(row["swing"])
        if abs(abs(high - low) - swing) > 1e-3:
            raise CheckError(f"{path}: swing does not equal |high - low| in {row}")
    if swings != sorted(swings, reverse=True):
        raise CheckError(f"{path}: rows are not ranked by swing")


def check_breakeven(rows):
    path = "breakeven.csv"
    c_ai = [float(r["c_ai"]) for r in rows]
    if c_ai != sorted(c_ai):
        raise CheckError(f"{path}: c_ai is not increasing")


def check_paths(rows, horizon):
    path = "paths.csv"
    if not rows:
        raise CheckError(f"{path}: no paths")
    for row in rows:
        month = optional_int(row["crossover_month"])
        if month is not None and not (1 <= month <= horizon):
            raise CheckError(f"{path}: crossover month {month} outside 1..{horizon}")


def check_control(rows):
    """The control must hold, or the whole set of figures is withdrawn."""
    path = "control.csv"
    if len(rows) != 3:
        raise CheckError(f"{path}: {len(rows)} rows, expected 3")
    for row in rows:
        benefit = float(row["benefit_end"])
        month = optional_int(row["crossover_month"])
        savings = float(row["savings"])
        if benefit < 0.0 or month is not None:
            raise CheckError(
                f"{path}: with the comprehension asymmetry removed the model must not report "
                f"a loss, but savings {savings} gives benefit {benefit:+.1f} h and "
                f"crossover {month}. The negative result elsewhere is then a construction "
                f"artefact and these figures must not be drawn."
            )


# -------------------------------------------------------------------------- figures


def footer(fig):
    fig.text(0.008, 0.006, FOOTER, fontsize=7, color="#666666", ha="left")


def figure_crossover(central, out_dir):
    months = [int(float(r["month"])) for r in central]
    human = [float(r["human_cumulative"]) for r in central]
    ai = [float(r["ai_cumulative"]) for r in central]
    benefit = [float(r["benefit"]) for r in central]
    crossing = next((m for m, b in zip(months, benefit) if b < 0), None)

    fig, (left, right) = plt.subplots(1, 2, figsize=(11, 4.6))

    left.plot(months, human, color="#1f4e79", lw=2, label="human author")
    left.plot(months, ai, color="#c0504d", lw=2, label="AI author")
    left.set_xlabel("month")
    left.set_ylabel("cumulative hours")
    left.set_title("Cumulative cost of the same 10 modules")
    left.legend(frameon=False)
    left.grid(alpha=0.25)

    right.axhline(0, color="#333333", lw=1)
    right.plot(months, benefit, color="#4f6228", lw=2)
    right.fill_between(months, benefit, 0, where=[b < 0 for b in benefit],
                       color="#c0504d", alpha=0.25, interpolate=True)
    right.set_xlabel("month")
    right.set_ylabel("human hours − AI hours")
    right.set_title("Cumulative benefit of AI authoring")
    right.grid(alpha=0.25)
    if crossing is not None:
        right.axvline(crossing, color="#c0504d", ls="--", lw=1)
        right.annotate(
            f"crosses in month {crossing}",
            xy=(crossing, min(benefit)),
            xytext=(crossing + 3, min(benefit) * 0.55),
            fontsize=9,
            color="#c0504d",
        )

    fig.suptitle(
        "The central cell: AI authoring saves hours early and loses them later",
        fontsize=12,
    )
    footer(fig)
    fig.tight_layout(rect=(0, 0.03, 1, 0.95))
    path = os.path.join(out_dir, "comprehension-1-crossover.png")
    fig.savefig(path, dpi=170)
    plt.close(fig)
    return path


def figure_channels(rows, out_dir):
    labels = [r["configuration"] for r in rows]
    benefits = [float(r["benefit"]) for r in rows]
    deltas = [None] + [float(r["delta"]) for r in rows[1:]]

    fig, ax = plt.subplots(figsize=(9.5, 4.8))
    positions = range(len(labels))
    for index, (label, benefit, delta) in enumerate(zip(labels, benefits, deltas)):
        if delta is None:
            ax.barh(index, benefit, color="#1f4e79", height=0.6)
            ax.text(benefit + 4, index, f"{benefit:+.0f} h", va="center", fontsize=9)
            continue
        colour = "#c0504d" if delta < 0 else "#4f6228"
        ax.barh(index, delta, left=benefit - delta, color=colour, height=0.6)
        # Inside the bar. Placing the label off the end pushed it off the axes on the
        # longest bar, where it collided with the category labels.
        ax.text(benefit - delta / 2.0, index, f"{delta:+.0f} h", va="center",
                ha="center", fontsize=9, color="white", fontweight="bold")

    ax.axvline(0, color="#333333", lw=1)
    ax.set_yticks(list(positions))
    ax.set_yticklabels(labels, fontsize=9)
    ax.invert_yaxis()
    ax.set_xlabel("cumulative benefit at month 36 (hours)")
    ax.set_title("Where the difference comes from, one mechanism at a time")
    ax.grid(axis="x", alpha=0.25)
    footer(fig)
    fig.tight_layout(rect=(0, 0.03, 1, 1))
    path = os.path.join(out_dir, "comprehension-2-channels.png")
    fig.savefig(path, dpi=170)
    plt.close(fig)
    return path


def figure_phase(rows, horizon, out_dir):
    savings = sorted({round(float(r["savings"]), 2) for r in rows})
    half_lives = sorted({round(float(r["half_life_ai"]), 0) for r in rows})
    epsilons = sorted({round(float(r["epsilon"]), 2) for r in rows})

    fig, axes = plt.subplots(2, len(savings), figsize=(12, 6.4))
    if len(savings) == 1:
        axes = [[axes[0]], [axes[1]]]

    for column, level in enumerate(savings):
        benefit = [[float("nan")] * len(epsilons) for _ in half_lives]
        month = [[float("nan")] * len(epsilons) for _ in half_lives]
        for row in rows:
            if round(float(row["savings"]), 2) != level:
                continue
            i = half_lives.index(round(float(row["half_life_ai"]), 0))
            j = epsilons.index(round(float(row["epsilon"]), 2))
            benefit[i][j] = float(row["benefit_end"])
            parsed = optional_int(row["crossover_month"])
            if parsed is not None:
                month[i][j] = parsed

        top = axes[0][column]
        # Every cell in this grid is negative, so a diverging map centred on zero would
        # spend half its palette on values that never occur. Sequential instead, and the
        # scale is the data's own range so the differences are visible: darker = a
        # larger loss.
        flat = [value for row in benefit for value in row]
        low, high = min(flat), max(flat)
        if high - low < 1e-9:
            low, high = low - 1.0, high + 1.0
        mesh = top.imshow(benefit, cmap="Reds_r", vmin=low, vmax=high, aspect="auto")
        top.set_title(f"benefit at month {horizon}\nsavings = {int(level * 100)}%", fontsize=10)
        top.set_xticks(range(len(epsilons)))
        top.set_xticklabels([f"{e:.1f}" for e in epsilons], fontsize=8)
        top.set_yticks(range(len(half_lives)))
        top.set_yticklabels([f"{h:.0f}" for h in half_lives], fontsize=8)
        for i in range(len(half_lives)):
            for j in range(len(epsilons)):
                value = benefit[i][j]
                shade = (value - low) / (high - low)
                # Reds_r is dark at the low end, so dark cells need light text.
                colour = "white" if shade < 0.45 else "#222222"
                top.text(j, i, f"{value:+.0f}", ha="center", va="center", fontsize=8,
                         color=colour)
        fig.colorbar(mesh, ax=top, fraction=0.046, pad=0.04)

        bottom = axes[1][column]
        # Scaled to the observed months, not to the horizon: with a 1..36 scale every cell
        # of this grid renders as the same colour and the panel says nothing.
        present = [value for row in month for value in row if value == value]
        lo, hi = (min(present), max(present)) if present else (1.0, float(horizon))
        flat_panel = len(set(present)) <= 1
        if flat_panel:
            # One value everywhere is the finding, not an accident of the scale. A
            # colorbar over a padded range would imply variation that does not exist.
            bottom.imshow([[0.0] * len(epsilons) for _ in half_lives], cmap="Greys",
                          vmin=0.0, vmax=1.0, aspect="auto")
            bottom.text(
                0.5, -0.30, "no variation across the grid",
                transform=bottom.transAxes, ha="center", fontsize=8, color="#666666",
            )
        else:
            plot = bottom.imshow(month, cmap="YlGnBu", aspect="auto", vmin=lo, vmax=hi)
        bottom.set_title("month the benefit crosses zero", fontsize=10)
        bottom.set_xticks(range(len(epsilons)))
        bottom.set_xticklabels([f"{e:.1f}" for e in epsilons], fontsize=8)
        bottom.set_yticks(range(len(half_lives)))
        bottom.set_yticklabels([f"{h:.0f}" for h in half_lives], fontsize=8)
        for i in range(len(half_lives)):
            for j in range(len(epsilons)):
                value = month[i][j]
                if value != value:
                    text, colour = "—", "#222222"
                else:
                    text = f"{int(value)}"
                    # YlGnBu is dark at the high end; a flat panel is mid grey.
                    shade = 0.0 if flat_panel or hi == lo else (value - lo) / (hi - lo)
                    colour = "white" if shade > 0.6 else "#222222"
                bottom.text(j, i, text, ha="center", va="center", fontsize=8,
                            color=colour)
        if not flat_panel:
            fig.colorbar(plot, ax=bottom, fraction=0.046, pad=0.04)

    for row in axes:
        row[0].set_ylabel("AI retention half-life (months)", fontsize=9)
    for axis in axes[1]:
        axis.set_xlabel("AI defect multiplier", fontsize=9)

    fig.suptitle(
        "Phase diagram: the crossover barely moves with the parameters the hypothesis names",
        fontsize=12,
    )
    footer(fig)
    fig.tight_layout(rect=(0, 0.03, 1, 0.94))
    path = os.path.join(out_dir, "comprehension-3-phase.png")
    fig.savefig(path, dpi=170)
    plt.close(fig)
    return path


def figure_sensitivity(rows, out_dir):
    names = [r["parameter"] for r in rows]
    sweeps = [float(r["swing"]) for r in rows]
    shares = [s / sum(sweeps) * 100 for s in sweeps]

    fig, ax = plt.subplots(figsize=(9.5, 5.4))
    positions = range(len(names))
    colours = ["#c0504d" if "half_life_ai" in n or "defect" in n else "#1f4e79" for n in names]
    ax.barh(list(positions), shares, color=colours, height=0.66)
    for index, share in enumerate(shares):
        ax.text(share + 0.3, index, f"{share:.0f}%", va="center", fontsize=8)
    ax.set_yticks(list(positions))
    ax.set_yticklabels(names, fontsize=8)
    ax.invert_yaxis()
    ax.set_xlabel("share of the total one-at-a-time swing in benefit at month 36")
    ax.set_title("What the answer depends on")
    ax.grid(axis="x", alpha=0.25)
    ax.annotate(
        "the decay asymmetry — the parameter\nthe hypothesis is actually about",
        xy=(shares[names.index("half_life_ai (months)")],
            names.index("half_life_ai (months)")),
        xytext=(shares[names.index("half_life_ai (months)")] + 4,
                names.index("half_life_ai (months)") + 2.2),
        fontsize=8, color="#c0504d",
        arrowprops=dict(arrowstyle="->", color="#c0504d", lw=1),
    )
    footer(fig)
    fig.tight_layout(rect=(0, 0.03, 1, 1))
    path = os.path.join(out_dir, "comprehension-4-sensitivity.png")
    fig.savefig(path, dpi=170)
    plt.close(fig)
    return path


def figure_breakeven(breakeven, paths, out_dir):
    c_ai = [float(r["c_ai"]) for r in breakeven]
    benefit = [float(r["benefit"]) for r in breakeven]
    estimates = [float(r["benefit"]) for r in paths if float(r["benefit"]) == float(r["benefit"])]

    fig, (left, right) = plt.subplots(1, 2, figsize=(11, 4.6))

    left.axhline(0, color="#333333", lw=1)
    left.plot(c_ai, benefit, color="#1f4e79", lw=2, marker="o", ms=3)
    left.fill_between(c_ai, benefit, 0, where=[b < 0 for b in benefit],
                      color="#c0504d", alpha=0.2, interpolate=True)
    left.set_xlabel("comprehension an AI-authored module starts with")
    left.set_ylabel("benefit at month 36 (hours)")
    left.set_title("No initial comprehension is enough on its own")
    left.grid(alpha=0.25)

    right.hist(estimates, bins=24, color="#4f6228", alpha=0.75)
    right.axvline(0, color="#333333", lw=1)
    if estimates:
        right.axvline(sum(estimates) / len(estimates), color="#4f6228", ls="--", lw=1,
                      label=f"simulated mean {sum(estimates) / len(estimates):+.0f} h")
    right.set_xlabel("benefit at month 36 (hours), 300 seeded paths")
    right.set_ylabel("paths")
    right.set_title("Lumpy arrivals widen the outcome")
    right.legend(frameon=False, fontsize=8)
    right.grid(alpha=0.25)

    fig.suptitle("Break-even and spread", fontsize=12)
    footer(fig)
    fig.tight_layout(rect=(0, 0.03, 1, 0.93))
    path = os.path.join(out_dir, "comprehension-5-breakeven.png")
    fig.savefig(path, dpi=170)
    plt.close(fig)
    return path


def figure_control(control, channels_rows, out_dir):
    """The figure that makes the rest of them worth anything."""
    savings = [float(r["savings"]) for r in control]
    symmetric = [float(r["benefit_end"]) for r in control]
    declared = float(channels_rows[-1]["benefit"])

    fig, (left, right) = plt.subplots(1, 2, figsize=(11, 4.4))

    positions = range(len(savings))
    left.axhline(0, color="#333333", lw=1)
    left.bar(list(positions), symmetric, color="#4f6228", width=0.55)
    for index, value in enumerate(symmetric):
        left.text(index, value + 0.6, f"{value:+.0f} h", ha="center", fontsize=9)
    left.set_xticks(list(positions))
    left.set_xticklabels([f"{int(s * 100)}%" for s in savings])
    left.set_xlabel("authoring saving assumed")
    left.set_ylabel("benefit at month 36 (hours)")
    left.set_title("Asymmetry removed: AI wins, as it must")
    left.grid(axis="y", alpha=0.25)

    right.axhline(0, color="#333333", lw=1)
    right.bar([0, 1], [declared, symmetric[-1]], color=["#c0504d", "#4f6228"], width=0.5)
    right.set_xticks([0, 1])
    right.set_xticklabels(["as declared", "asymmetry\nremoved"], fontsize=9)
    right.set_ylabel("benefit at month 36 (hours)")
    right.set_title(f"The sign turns on the asymmetry alone ({savings[-1]:.0%} saving)")
    for index, value in enumerate([declared, symmetric[-1]]):
        right.text(index, value + (5 if value > 0 else -5), f"{value:+.0f} h",
                   ha="center", va="bottom" if value > 0 else "top", fontsize=9)
    right.grid(axis="y", alpha=0.25)

    fig.suptitle("The control: the model can report a win", fontsize=12)
    footer(fig)
    fig.tight_layout(rect=(0, 0.03, 1, 0.93))
    path = os.path.join(out_dir, "comprehension-6-control.png")
    fig.savefig(path, dpi=170)
    plt.close(fig)
    return path


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("export_dir", help="directory written by --export")
    parser.add_argument("--out", default="figures", help="where to write the PNGs")
    parser.add_argument("--horizon", type=int, default=36, help="horizon in months")
    args = parser.parse_args()

    root = args.export_dir
    try:
        central = read_csv(os.path.join(root, "central.csv"))
        channels_rows = read_csv(os.path.join(root, "channels.csv"))
        phase_rows = read_csv(os.path.join(root, "phase.csv"))
        sensitivity_rows = read_csv(os.path.join(root, "sensitivity.csv"))
        breakeven_rows = read_csv(os.path.join(root, "breakeven.csv"))
        path_rows = read_csv(os.path.join(root, "paths.csv"))
        control_rows = read_csv(os.path.join(root, "control.csv"))

        check_central(central, args.horizon)
        check_channels(channels_rows)
        check_phase(phase_rows, args.horizon)
        check_sensitivity(sensitivity_rows)
        check_breakeven(breakeven_rows)
        check_paths(path_rows, args.horizon)
        check_control(control_rows)
    except CheckError as error:
        print(f"refusing to draw: {error}", file=sys.stderr)
        return 2

    os.makedirs(args.out, exist_ok=True)
    written = [
        figure_crossover(central, args.out),
        figure_channels(channels_rows, args.out),
        figure_phase(phase_rows, args.horizon, args.out),
        figure_sensitivity(sensitivity_rows, args.out),
        figure_breakeven(breakeven_rows, path_rows, args.out),
        figure_control(control_rows, channels_rows, args.out),
    ]
    for path in written:
        print(f"wrote {path}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
