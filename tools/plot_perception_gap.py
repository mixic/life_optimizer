#!/usr/bin/env python3
"""Draw the perception-gap figures from the CSV the model exports.

Copyright (C) 2026 MILAN NIKOLIC
SPDX-License-Identifier: GPL-3.0-or-later

Usage:
    cargo run -p perception_gap -- --export out/perception
    python tools/plot_perception_gap.py out/perception

# What these show

The gap itself is unremarkable: two latents that both saturate, so the difference grows
monotonically and is largest at the end of the horizon. The figures worth the ink are the other
two — when the gap *grows fastest*, and whether it can be estimated at all.

# Why it refuses to draw

The export is checked before anything is rendered. A plot from inconsistent data looks exactly
like a plot from consistent data, which is the failure this repository treats as the dangerous
kind. In particular a rank-1 observability row must come with an infinite condition number: if the
export claimed both, the identifiability conclusion below would be drawn from contradictory
numbers.
"""

import argparse
import csv
import os
import sys

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt

FOOTER = (
    "Simulation with Declared parameters and synthetic truth — a statement about the estimator's "
    "own assumptions, not a measurement of any engineer."
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


def footer(fig):
    fig.text(0.008, 0.006, FOOTER, fontsize=7, color="#666666", ha="left")


# --------------------------------------------------------------------------- checks


def check_gap(rows, horizon):
    by_adoption = {}
    for row in rows:
        by_adoption.setdefault(row["adoption"], []).append(int(row["month"]))
    if not by_adoption:
        raise CheckError("gap_profile.csv has no adoption shares")
    for adoption, months in by_adoption.items():
        if months != list(range(1, horizon + 1)):
            raise CheckError(f"adoption {adoption}: months are not 1..{horizon} in order")


def check_observability(rows):
    for row in rows:
        rank = int(row["rank"])
        condition = float(row["condition"])
        if rank == 1 and condition < 1e8:
            raise CheckError(
                f"a rank-1 design must carry an effectively infinite condition number, "
                f"got {condition} in {row}"
            )
        if rank == 2 and condition >= 1e8:
            raise CheckError(f"a rank-2 design cannot have an infinite condition number: {row}")


def check_estimators(rows):
    for row in rows:
        for key in ("kalman_rmse", "subtraction_rmse", "prior_only_rmse"):
            value = float(row[key])
            if value < 0 or value != value:
                raise CheckError(f"{key} is {value} in {row}")
        coverage_key = "kalman_coverage" if "kalman_coverage" in row else "coverage"
        coverage = float(row[coverage_key])
        if not 0.0 <= coverage <= 1.0:
            raise CheckError(f"coverage {coverage} is outside [0, 1] in {row}")


# -------------------------------------------------------------------------- figures


def figure_gap(rows, out_dir):
    adoptions = sorted({float(r["adoption"]) for r in rows})
    fig, (left, right) = plt.subplots(1, 2, figsize=(11.5, 4.8))

    colours = plt.cm.viridis([i / max(1, len(adoptions) - 1) for i in range(len(adoptions))])
    for colour, adoption in zip(colours, adoptions):
        series = sorted(
            (int(r["month"]), float(r["gap"])) for r in rows if float(r["adoption"]) == adoption
        )
        months = [m for m, _ in series]
        gaps = [g for _, g in series]
        left.plot(months, gaps, color=colour, lw=2,
                  label=f"adoption {adoption:.0%}" if adoption in (adoptions[0], adoptions[-1])
                  else None)
        growth = [b - a for a, b in zip(gaps, gaps[1:])]
        right.plot(months[1:], growth, color=colour, lw=2)

    left.set_title("The gap grows monotonically: largest at the horizon")
    left.set_xlabel("months of sustained adoption")
    left.set_ylabel("believed − true competence")
    left.legend(frameon=False, fontsize=8)
    left.grid(alpha=0.25)

    right.set_title("When it grows fastest: early, and only at high adoption")
    right.set_xlabel("months")
    right.set_ylabel("month-on-month change in the gap")
    right.grid(alpha=0.25)

    fig.suptitle("When is the gap largest, and when does it open fastest?", fontsize=12)
    footer(fig)
    fig.tight_layout(rect=(0, 0.03, 1, 0.93))
    path = os.path.join(out_dir, "perception-1-gap.png")
    fig.savefig(path, dpi=165)
    plt.close(fig)
    return path


def figure_identifiability(rows, out_dir):
    """The figure the research question turns on."""
    designs = sorted({r["design"] for r in rows})
    fig, ax = plt.subplots(figsize=(9.4, 5.2))

    for design, colour, marker in zip(designs, ["#1f4e79", "#c0504d"], ["o", "s"]):
        series = sorted((float(r["gamma"]), float(r["condition"]), int(r["rank"]))
                        for r in rows if r["design"] == design)
        xs = [g for g, _, _ in series]
        ys = [c for _, c, _ in series]
        ax.plot(xs, ys, color=colour, marker=marker, lw=2, label=design)
        for gamma, condition, rank in series:
            if rank < 2:
                ax.annotate("rank 1: no estimate exists",
                            xy=(gamma, condition), xytext=(gamma + 0.07, 1e5),
                            fontsize=8, color=colour,
                            arrowprops=dict(arrowstyle="->", color=colour, lw=1))

    ax.set_yscale("log")
    ax.set_xlabel("γ — how strongly confidence moves review time independently of competence")
    ax.set_ylabel("condition number of the observability matrix (log scale)")
    ax.set_title("Behavioural telemetry sees belief only because confidence contaminates it")
    ax.legend(frameon=False, fontsize=9)
    ax.grid(alpha=0.25, which="both")
    ax.annotate(
        "with self-report the design is\nwell conditioned at every γ",
        xy=(0.30, 1.85), xytext=(0.12, 6.0), fontsize=8, color="#1f4e79",
    )
    footer(fig)
    fig.tight_layout(rect=(0, 0.03, 1, 1))
    path = os.path.join(out_dir, "perception-2-identifiability.png")
    fig.savefig(path, dpi=165)
    plt.close(fig)
    return path


def figure_estimators(rows, out_dir):
    designs = sorted({r["design"] for r in rows})
    fig, axes = plt.subplots(1, len(designs), figsize=(11.5, 4.8), sharey=True)
    if len(designs) == 1:
        axes = [axes]

    for axis, design in zip(axes, designs):
        series = sorted(
            (
                float(r["gamma"]),
                float(r["kalman_rmse"]),
                float(r["subtraction_rmse"]),
                float(r["static_rmse"]),
                float(r["prior_only_rmse"]),
            )
            for r in rows
            if r["design"] == design and abs(float(r["adoption"]) - 0.5) < 1e-9
        )
        if not series:
            continue
        xs = [s[0] for s in series]
        axis.plot(xs, [s[1] for s in series], color="#1f4e79", marker="o", lw=2, label="Kalman filter")
        axis.plot(xs, [s[2] for s in series], color="#c0504d", marker="s", lw=1.6, label="subtraction")
        axis.plot(xs, [s[4] for s in series], color="#7f7f7f", ls="--", lw=1.6,
                  label="filter's model, no data")
        finite_static = [(x, s[3]) for x, s in zip(xs, series) if s[3] == s[3] and s[3] < 5]
        if finite_static:
            axis.plot([x for x, _ in finite_static], [v for _, v in finite_static],
                      color="#4f6228", marker="^", lw=1.4, label="static fit")
        axis.set_title(design)
        axis.set_xlabel("γ")
        axis.grid(alpha=0.25)
        axis.legend(frameon=False, fontsize=8)

    axes[0].set_ylabel("RMSE of the estimated gap")
    fig.suptitle("The filter must beat subtraction, and must beat its own model", fontsize=12)
    footer(fig)
    fig.tight_layout(rect=(0, 0.03, 1, 0.93))
    path = os.path.join(out_dir, "perception-3-estimators.png")
    fig.savefig(path, dpi=165)
    plt.close(fig)
    return path


def figure_calibration(rows, out_dir):
    """Where an unidentifiable state announces itself: the band, not the error."""
    fig, ax = plt.subplots(figsize=(9.0, 4.8))
    for design, colour, marker in zip(
        sorted({r["design"] for r in rows}), ["#1f4e79", "#c0504d"], ["o", "s"]
    ):
        for gamma, style in ((0.0, "-"), (0.3, "--")):
            series = sorted(
                (float(r["mismatch"]), float(r["coverage"]))
                for r in rows
                if r["design"] == design and abs(float(r["gamma"]) - gamma) < 1e-9
            )
            if not series:
                continue
            ax.plot([m for m, _ in series], [c for _, c in series], color=colour,
                    marker=marker, ls=style, lw=1.8,
                    label=f"{design}, γ = {gamma:.1f}")

    ax.axhline(0.68, color="#333333", lw=0.9, ls=":")
    ax.annotate("nominal 68% band", xy=(0.02, 0.68), xytext=(0.05, 0.705), fontsize=8,
                color="#333333")
    ax.set_xlabel("dynamics mismatch between the world and the filter's model")
    ax.set_ylabel("share of months inside the one-sigma band")
    ax.set_title("Unidentifiability shows up as miscalibration, not as error")
    ax.set_ylim(0.5, 0.85)
    ax.legend(frameon=False, fontsize=8)
    ax.grid(alpha=0.25)
    footer(fig)
    fig.tight_layout(rect=(0, 0.03, 1, 1))
    path = os.path.join(out_dir, "perception-4-calibration.png")
    fig.savefig(path, dpi=165)
    plt.close(fig)
    return path


def figure_headline(rows, out_dir):
    """The repository's own thesis applied to itself: error stays put, the band does not.

    This is the figure the correction deserves. A filter whose RMSE is flat and whose coverage
    drifts from 73% to 62% passes every conventional accuracy check and is still wrong. The two
    panels use the same cells, so the divergence between them is the finding.
    """
    fig, (left, right) = plt.subplots(1, 2, figsize=(11.5, 4.9), sharex=True)
    order = sorted({r["design"] for r in rows})
    colours = dict(zip(order, ["#1f4e79", "#c0504d"]))
    markers = dict(zip(order, ["o", "s"]))

    for design in order:
        series = sorted(
            (float(r["mismatch"]), float(r["kalman_rmse"]), float(r["coverage"]))
            for r in rows
            if r["design"] == design and abs(float(r["gamma"]) - 0.0) < 1e-9
        )
        xs = [s[0] for s in series]
        left.plot(xs, [s[1] for s in series], color=colours[design], marker=markers[design],
                  lw=2, label=design)
        right.plot(xs, [s[2] for s in series], color=colours[design], marker=markers[design],
                   lw=2, label=design)

    left.set_ylabel("RMSE of the gap estimate")
    left.set_title("The point estimate passes every check")
    left.set_xlabel("dynamics mismatch")
    left.grid(alpha=0.25)
    left.legend(frameon=False, fontsize=8)
    left.annotate("flat", xy=(0.5, 0.065), fontsize=9, color="#666666")

    right.axhline(0.68, color="#333333", lw=0.9, ls=":")
    right.annotate("nominal 68%", xy=(0.02, 0.683), xytext=(0.06, 0.70), fontsize=8,
                   color="#333333")
    right.set_ylabel("coverage of the one-sigma band")
    right.set_title("…and the uncertainty is quietly wrong")
    right.set_xlabel("dynamics mismatch")
    right.set_ylim(0.55, 0.78)
    right.grid(alpha=0.25)
    right.annotate("drifts from 0.73\nto 0.62", xy=(1.0, 0.62), xytext=(0.35, 0.60),
                   fontsize=9, color="#c0504d",
                   arrowprops=dict(arrowstyle="->", color="#c0504d", lw=1))

    fig.suptitle("Passing the point-estimate check does not mean the model is right", fontsize=12)
    footer(fig)
    fig.tight_layout(rect=(0, 0.03, 1, 0.93))
    path = os.path.join(out_dir, "perception-5-headline.png")
    fig.savefig(path, dpi=165)
    plt.close(fig)
    return path


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("export_dir", help="directory written by --export")
    parser.add_argument("--out", default="figures")
    parser.add_argument("--horizon", type=int, default=36)
    args = parser.parse_args()

    root = args.export_dir
    try:
        gap_rows = read_csv(os.path.join(root, "gap_profile.csv"))
        obs_rows = read_csv(os.path.join(root, "observability.csv"))
        est_rows = read_csv(os.path.join(root, "estimator_grid.csv"))
        cal_rows = read_csv(os.path.join(root, "calibration.csv"))

        check_gap(gap_rows, args.horizon)
        check_observability(obs_rows)
        check_estimators(est_rows)
        check_estimators(cal_rows)
    except CheckError as error:
        print(f"refusing to draw: {error}", file=sys.stderr)
        return 2

    os.makedirs(args.out, exist_ok=True)
    written = [
        figure_gap(gap_rows, args.out),
        figure_identifiability(obs_rows, args.out),
        figure_estimators(est_rows, args.out),
        figure_calibration(cal_rows, args.out),
        figure_headline(cal_rows, args.out),
    ]
    for path in written:
        print(f"wrote {path}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
