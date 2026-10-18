#!/usr/bin/env python3
"""Draw the workflow figures from the CSV that `knowledge_work --export` writes.

Copyright (C) 2026 MILAN NIKOLIC
SPDX-License-Identifier: GPL-3.0-or-later

Usage:
    cargo run -p knowledge_work --release -- --export out/workflow
    python tools/plot_workflow.py out/workflow                  # figures beside the data
    python tools/plot_workflow.py out/workflow figures          # committed copies

# What these figures are for

The report is a set of thresholds, and a threshold is a claim about where something changes.
Four pictures carry the four claims: the operating envelope and its two edges, the cost of the
cycle-time commitment, the year a stable workflow stops being stable because the person
reviewing it changed, and the surface on which the direction question lives without an answer.

# Why it refuses to draw

The export is checked before anything is drawn. A figure drawn from an inconsistent export looks
exactly like one drawn from a consistent one, which is the failure mode this repository treats as
the dangerous kind. Two of the checks here are the load-bearing ones: a workflow marked stable
must have utilisation below one, and the acceptance shares must sum to one. A plotting script
that silently accepted a contradiction between those would have hidden exactly the defect the
models were built to expose.
"""

from __future__ import annotations

import csv
import math
import os
import sys
import textwrap

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt  # noqa: E402

STABLE = "#2166ac"
UNSTABLE = "#b2182b"
THRESHOLD = "#4d4d4d"
ACCENT = "#7f7f7f"
OPTION_COLOURS = ["#b2182b", "#2166ac", "#1a9850"]

DECLARED = (
    "Every parameter is declared, not calibrated: the curves show the model's structure, "
    "not an estimate of any workplace."
)


class CheckError(Exception):
    """The export failed a consistency check, so nothing may be drawn."""


def read_csv(path: str) -> list[dict[str, str]]:
    with open(path, newline="", encoding="utf-8") as handle:
        return list(csv.DictReader(handle))


def read_meta(directory: str) -> dict[str, float]:
    return {row["key"]: float(row["value"]) for row in read_csv(os.path.join(directory, "meta.csv"))}


def number(row: dict[str, str], key: str) -> float:
    return float(row[key])


def stamp(fig, note: str) -> None:
    width = max(80, int(fig.get_figwidth() * 13.5))
    text = textwrap.fill(" ".join(note.split()), width)
    fig.text(0.5, 0.006, text, ha="center", va="bottom", fontsize=7.2, color=ACCENT,
             linespacing=1.35)


def verify(directory: str, meta: dict[str, float]) -> dict[str, list]:
    envelope = read_csv(os.path.join(directory, "envelope.csv"))
    if len(envelope) < 5:
        raise CheckError("the envelope has too few points to draw a curve")
    capacities = [number(row, "capacity") for row in envelope]
    if capacities != sorted(capacities):
        raise CheckError("the envelope's capacity axis is not increasing")

    previous = -1.0
    for row in envelope:
        rho = number(row, "rho_generated")
        stable = row["stable"] == "1"
        if not math.isfinite(rho) or rho < 0.0:
            raise CheckError(f"utilisation {rho} at capacity {row['capacity']} is not a utilisation")
        # The load-bearing check: a workflow cannot be stable above saturation, and a figure
        # that drew it so would contradict its own vertical line.
        if stable and rho >= 1.0:
            raise CheckError(
                f"capacity {row['capacity']} is marked stable at utilisation {rho}"
            )
        if not stable and rho < 1.0:
            raise CheckError(
                f"capacity {row['capacity']} is marked unstable at utilisation {rho}, which is "
                "below saturation"
            )
        absorbable = number(row, "absorbable")
        if math.isfinite(absorbable):
            if not 0.0 <= absorbable <= 1.0:
                raise CheckError(f"absorbable share {absorbable} is not a share")
            if absorbable < previous - 1e-9:
                raise CheckError(
                    f"the absorbable share fell as capacity rose, at capacity {row['capacity']}"
                )
            previous = absorbable

    commitment = meta.get("critical_commitment", float("nan"))
    stability = meta.get("critical_stability", float("nan"))
    if not (math.isfinite(commitment) and math.isfinite(stability)):
        raise CheckError("meta.csv is missing one of the two thresholds")
    if commitment <= stability:
        raise CheckError(
            f"the commitment binds at {commitment} but stability fails at {stability}; a "
            "promise cannot outlive a mathematical impossibility"
        )

    deadline = read_csv(os.path.join(directory, "deadline.csv"))
    reachable = [number(row, "critical_capacity") for row in deadline
                 if math.isfinite(number(row, "critical_capacity"))]
    if len(reachable) < 3:
        raise CheckError("too few reachable deadlines to draw a curve")
    if reachable != sorted(reachable):
        raise CheckError("the critical capacity is not monotone in the deadline")
    if all(math.isfinite(number(row, "critical_capacity")) for row in deadline):
        raise CheckError(
            "no deadline was refused, so the refusal path has never been exercised"
        )

    trajectory = read_csv(os.path.join(directory, "trajectory.csv"))
    years = [int(number(row, "year")) for row in trajectory]
    if years != sorted(years) or len(set(years)) != len(years):
        raise CheckError("the trajectory's years are not strictly increasing")
    for row in trajectory:
        stable = row["stable"] == "1"
        cycle = number(row, "cycle_time")
        if stable and not math.isfinite(cycle):
            raise CheckError(
                f"year {row['year']} is stable but has no cycle time"
            )
        if not stable and math.isfinite(cycle):
            raise CheckError(
                f"year {row['year']} has no steady state yet reports a cycle time of {cycle}"
            )

    options = read_csv(os.path.join(directory, "options.csv"))
    shares = [number(row, "share") for row in options]
    if abs(sum(shares) - 1.0) > 1e-6:
        raise CheckError(
            f"the acceptance shares sum to {sum(shares)}, so the grid was not a simplex"
        )
    if len(options) < 3:
        raise CheckError("the direction panel needs at least three configurations")

    return {
        "envelope": envelope,
        "deadline": deadline,
        "trajectory": trajectory,
        "options": options,
    }


def column(rows: list[dict[str, str]], key: str) -> list[float]:
    return [number(row, key) for row in rows]


def draw_envelope(export: dict, meta: dict, out_path: str) -> None:
    rows = export["envelope"]
    capacity = column(rows, "capacity")
    rho = column(rows, "rho_generated")
    absorbable = column(rows, "absorbable")

    fig, axes = plt.subplots(1, 2, figsize=(11.6, 4.6))

    axis = axes[0]
    axis.plot(capacity, rho, color=UNSTABLE, linewidth=1.9)
    axis.axhline(1.0, color=THRESHOLD, linewidth=1.0, linestyle=(0, (4, 3)))
    axis.annotate("no steady state above 1", xy=(0.02, 1.0), xytext=(0, 5),
                  textcoords="offset points", fontsize=7.8, color=THRESHOLD)
    for key, label, colour, height in (
        ("critical_stability", "stability fails", UNSTABLE, 1.10),
        ("critical_commitment", "commitment fails", STABLE, 1.19),
    ):
        value = meta.get(key, float("nan"))
        if math.isfinite(value):
            axis.axvline(value, color=colour, linewidth=1.1, linestyle=(0, (2, 2)))
            axis.annotate(
                f"{label}\n{value:.3f}",
                xy=(value, height),
                xytext=(-4, 0),
                textcoords="offset points",
                fontsize=7.6,
                color=colour,
                ha="right",
                va="top",
            )
    axis.set_xlabel("capacity to judge (psychological model scale)")
    axis.set_ylabel("effective utilisation if all work is admitted")
    axis.set_title("(a) The envelope, and its two edges", fontsize=10)
    axis.set_xlim(0, 1)
    axis.set_ylim(0.97, 1.26)
    axis.grid(alpha=0.25, linewidth=0.6)

    axis = axes[1]
    axis.plot(capacity, absorbable, color=STABLE, linewidth=1.9)
    axis.axhline(1.0, color=THRESHOLD, linewidth=1.0, linestyle=(0, (4, 3)))
    axis.annotate("can take everything", xy=(0.30, 1.0), xytext=(0, -12),
                  textcoords="offset points", fontsize=7.8, color=THRESHOLD)
    axis.fill_between(capacity, 0, absorbable, color=STABLE, alpha=0.10)
    axis.set_xlabel("capacity to judge")
    axis.set_ylabel("share of generated work absorbable")
    axis.set_title("(b) What it can still take", fontsize=10)
    axis.set_xlim(0, 1)
    axis.set_ylim(0, 1.02)
    axis.grid(alpha=0.25, linewidth=0.6)

    fig.suptitle("The operating envelope: utilisation is not the only thing that can break",
                 fontsize=11.5)
    fig.tight_layout(rect=(0, 0.085, 1, 0.955))
    stamp(
        fig,
        "The two vertical lines are different failures. Below the commitment the promise is "
        "broken while the queue still converges; below stability the queue does not converge at "
        "all. " + DECLARED,
    )
    os.makedirs(os.path.dirname(out_path) or ".", exist_ok=True)
    fig.savefig(out_path, dpi=190)
    plt.close(fig)
    print(f"wrote {out_path}")


def draw_deadline(export: dict, meta: dict, out_path: str) -> None:
    rows = export["deadline"]
    deadlines = column(rows, "deadline_days")
    critical = column(rows, "critical_capacity")

    fig, axis = plt.subplots(figsize=(8.6, 4.8))
    reachable_x = [d for d, c in zip(deadlines, critical) if math.isfinite(c)]
    reachable_y = [c for c in critical if math.isfinite(c)]
    axis.plot(reachable_x, reachable_y, color=STABLE, linewidth=2.0, marker="o", markersize=4)
    axis.fill_between(reachable_x, 0, reachable_y, color=STABLE, alpha=0.10)

    if len(reachable_x) < len(deadlines):
        edge = min(reachable_x)
        axis.axvspan(0, edge, color=UNSTABLE, alpha=0.10)
        axis.annotate(
            "unmeetable\nat any capacity",
            xy=(edge * 0.5, 0.45),
            ha="center",
            fontsize=8.2,
            color=UNSTABLE,
        )
    axis.annotate("inside the envelope\nhere", xy=(max(reachable_x) * 0.7, 0.55),
                  fontsize=8.2, color=STABLE, ha="center")
    axis.set_xlabel("cycle-time commitment (days)")
    axis.set_ylabel("capacity to judge required")
    axis.set_title("A tighter promise costs capacity, until it costs everything", fontsize=11.5)
    axis.set_ylim(0, 1.02)
    axis.grid(alpha=0.25, linewidth=0.6)
    fig.tight_layout(rect=(0, 0.10, 1, 1))
    stamp(
        fig,
        "The capacity below which the workflow can no longer absorb its own output. The shaded "
        "region is where no capacity is enough, which the model refuses to report as a "
        "threshold. " + DECLARED,
    )
    os.makedirs(os.path.dirname(out_path) or ".", exist_ok=True)
    fig.savefig(out_path, dpi=190)
    plt.close(fig)
    print(f"wrote {out_path}")


def draw_trajectory(export: dict, meta: dict, out_path: str) -> None:
    rows = export["trajectory"]
    years = column(rows, "year")
    capacity = column(rows, "capacity")
    rho = column(rows, "rho")
    stable = [row["stable"] == "1" for row in rows]
    on_time = [row["on_time"] == "1" for row in rows]

    fig, axes = plt.subplots(2, 1, figsize=(10.4, 6.4), sharex=True)

    axis = axes[0]
    axis.plot(years, capacity, color=STABLE, linewidth=1.9)
    axis.set_ylabel("capacity to judge")
    axis.set_title("(a) The person", fontsize=10)
    axis.set_ylim(0, 1.0)
    axis.grid(alpha=0.25, linewidth=0.6)

    axis = axes[1]
    axis.plot(years, rho, color=UNSTABLE, linewidth=1.9)
    axis.axhline(1.0, color=THRESHOLD, linewidth=1.0, linestyle=(0, (4, 3)))
    first_over = next((y for y, ok in zip(years, on_time) if not ok), None)
    first_lost = next((y for y, ok in zip(years, stable) if not ok), None)
    low = float(min(rho))
    high = float(max(rho))
    for value, label, colour, at in (
        (first_over, "commitment breached", "#c98a00", high * 0.998),
        (first_lost, "no steady state", UNSTABLE, low * 1.002),
    ):
        if value is not None:
            axis.axvline(value, color=colour, linewidth=1.0, linestyle=(0, (2, 2)))
            axis.annotate(
                f"{label} (year {int(value)})",
                xy=(value, at),
                xytext=(4, 0),
                textcoords="offset points",
                fontsize=7.8,
                color=colour,
                va="center",
            )
    axis.set_ylabel("effective utilisation")
    axis.set_xlabel("year")
    axis.set_title("(b) The workflow", fontsize=10)
    axis.grid(alpha=0.25, linewidth=0.6)

    fig.suptitle("Nothing about the workflow changed; the capacity to catch defects did",
                 fontsize=11.5)
    fig.tight_layout(rect=(0, 0.075, 1, 0.95))
    stamp(
        fig,
        "The workflow is identical in every year. The person's capacity falls because assistance "
        "is solved for rather than chosen, and the review stage's stability is a property of the "
        "person. " + DECLARED,
    )
    os.makedirs(os.path.dirname(out_path) or ".", exist_ok=True)
    fig.savefig(out_path, dpi=190)
    plt.close(fig)
    print(f"wrote {out_path}")


def draw_direction(export: dict, meta: dict, out_path: str) -> None:
    rows = export["options"]
    labels = [row["option"] for row in rows]
    shares = column(rows, "share")

    criteria = [
        ("throughput", "throughput", +1),
        ("cycle_time", "cycle time", -1),
        ("escaped_defects", "escaped defects", -1),
        ("autonomy_years", "autonomy years", +1),
        ("backlog", "backlog", -1),
    ]

    fig, axes = plt.subplots(1, 2, figsize=(12.4, 4.8))

    # (a) The surface: who wins under which priorities.
    axis = axes[0]
    order = sorted(range(len(rows)), key=lambda index: -shares[index])
    axis.barh(
        [labels[index] for index in order],
        [shares[index] * 100.0 for index in order],
        color=[OPTION_COLOURS[index % len(OPTION_COLOURS)] for index in order],
        alpha=0.85,
    )
    for position, index in enumerate(order):
        axis.annotate(f"{shares[index] * 100.0:.0f}%", xy=(shares[index] * 100.0, position),
                      xytext=(4, 0), textcoords="offset points", va="center", fontsize=8.5)
    axis.set_xlabel(f"share of {int(meta.get('weightings', 0))} weightings won")
    axis.set_title("(a) No configuration wins outright", fontsize=10)
    axis.set_xlim(0, 100)
    axis.grid(alpha=0.25, axis="x", linewidth=0.6)

    # (b) Parallel coordinates: what each configuration gives up.
    axis = axes[1]
    values: dict[str, list[float]] = {}
    for key, _, direction in criteria:
        raw = column(rows, key)
        low, high = min(raw), max(raw)
        if high > low:
            normalised = [
                (value - low) / (high - low) if direction > 0 else (high - value) / (high - low)
                for value in raw
            ]
        else:
            normalised = [0.5 for _ in raw]
        values[key] = normalised

    positions = list(range(len(criteria)))
    for index, label in enumerate(labels):
        axis.plot(
            positions,
            [values[key][index] for key, _, _ in criteria],
            color=OPTION_COLOURS[index % len(OPTION_COLOURS)],
            linewidth=1.9,
            marker="o",
            markersize=5,
            label=label,
        )
    axis.set_xticks(positions)
    axis.set_xticklabels([name for _, name, _ in criteria], fontsize=8.5)
    axis.set_ylabel("better →")
    axis.set_ylim(-0.05, 1.05)
    axis.set_title("(b) What each one trades away", fontsize=10)
    axis.legend(frameon=False, fontsize=7.5, loc="upper left", ncol=1)
    axis.grid(alpha=0.25, linewidth=0.6)

    fig.suptitle("Productivity toward what: the trade-off the framework refuses to resolve",
                 fontsize=11.5)
    fig.tight_layout(rect=(0, 0.085, 1, 0.95))
    stamp(
        fig,
        "Panel (a) counts, over every weighting of the five criteria, how often each "
        "configuration comes first. Panel (b) normalises each criterion across the three "
        "configurations, higher is better. The weights are the question, not the model's "
        "output. " + DECLARED,
    )
    os.makedirs(os.path.dirname(out_path) or ".", exist_ok=True)
    fig.savefig(out_path, dpi=190)
    plt.close(fig)
    print(f"wrote {out_path}")


def main() -> int:
    if len(sys.argv) < 2:
        print(__doc__, file=sys.stderr)
        return 2
    directory = sys.argv[1]
    out_dir = sys.argv[2] if len(sys.argv) > 2 else directory

    try:
        meta = read_meta(directory)
        export = verify(directory, meta)
    except (CheckError, KeyError, ValueError, FileNotFoundError) as error:
        print(f"REFUSING TO DRAW: {error}", file=sys.stderr)
        return 1

    figures = [
        ("workflow-1-envelope.png", draw_envelope),
        ("workflow-2-deadline.png", draw_deadline),
        ("workflow-3-trajectory.png", draw_trajectory),
        ("workflow-4-direction.png", draw_direction),
    ]
    for name, draw in figures:
        draw(export, meta, os.path.join(out_dir, name))
    return 0


if __name__ == "__main__":
    sys.exit(main())
