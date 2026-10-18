#!/usr/bin/env python3
"""Draw the psychological-layer figures from the CSV `multipolar_sim --psyche` exports.

Copyright (C) 2026 MILAN NIKOLIC
SPDX-License-Identifier: GPL-3.0-or-later

Usage:
    cargo run -p multipolar_sim --release -- --psyche --export out/psyche
    python tools/plot_psyche.py out/psyche                    # figures beside the data
    python tools/plot_psyche.py out/psyche multipolar_sim/figures

The second argument sends the figures somewhere other than the data directory, which is how
the copies committed for the document are made: the CSV stays git-ignored, the pictures do not.

# What these figures are for

The document's claims are shapes rather than levels. Assistance is solved for rather than
chosen; capacity is bought with practice and spent by disuse; the residual defect rate has a
floor that effort cannot remove; and the runway buys years of judgement rather than a different
destination. Every one of those is a curve, and a table of thirty numbers does not show a curve.

# Why it refuses to draw

The export is checked first, and nothing is drawn if a check fails. A figure drawn from an
inconsistent export looks exactly like one drawn from a consistent one, which is the failure
mode this repository treats as the dangerous kind. The checks here are weak on purpose -- they
test the export's internal consistency, not its truth. There is no truth to test: every
parameter in it is declared, as `psyche_provenance.txt` says beside the data.

Five figures are written:

  psychology-1-inversion     what tracking a tempo does to one cohort, against holding still
  psychology-2-thresholds    the three structural thresholds the model actually has
  psychology-3-cohort        the distribution behind the means, because a mean hides a split
  psychology-4-runway        the runway and the tempo, measured in years rather than end states
  psychology-5-phase         where maximum assistance stops being the best policy
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
from matplotlib.colors import LinearSegmentedColormap  # noqa: E402

# One colour per arm, kept the same in every figure: a reader should not have to re-learn which
# line is which between panels.
TRACKED = "#b2182b"
HELD = "#2166ac"
FLOOR = "#4d4d4d"
ACCENT = "#7f7f7f"

# The note that goes on every figure. Kept in one place so it cannot drift between them, and
# worded as a statement about what the numbers are rather than a disclaimer nobody reads.
DECLARED = (
    "Every parameter is declared, not calibrated: the curves show the model's structure, "
    "not an estimate of anything in the world."
)


class CheckError(Exception):
    """An export failed a consistency check. Raised so that nothing can be drawn after it."""


def read_csv(path: str) -> list[dict[str, str]]:
    with open(path, newline="", encoding="utf-8") as handle:
        return list(csv.DictReader(handle))


def number(row: dict[str, str], key: str) -> float:
    """Read a float from a row, accepting `NaN` only where it is written deliberately."""
    return float(row[key])


def read_meta(directory: str) -> dict[str, float]:
    meta: dict[str, float] = {}
    for row in read_csv(os.path.join(directory, "psyche_meta.csv")):
        meta[row["key"]] = float(row["value"])
    return meta


def verify(directory: str, meta: dict[str, float]) -> dict[str, list]:
    """Read the export and check that it is internally consistent before anything is drawn."""
    trajectories = read_csv(os.path.join(directory, "psyche_trajectories.csv"))
    if not trajectories:
        raise CheckError("psyche_trajectories.csv is empty")
    years = [int(number(row, "year")) for row in trajectories]
    if years != sorted(years) or len(set(years)) != len(years):
        raise CheckError("trajectory years are not strictly increasing")
    if years[0] != 0:
        raise CheckError(f"trajectories do not start at year 0 but at {years[0]}")
    if len(years) != int(meta["horizon_years"]):
        raise CheckError(
            f"{len(years)} trajectory rows for a {int(meta['horizon_years'])}-year horizon"
        )

    for row in trajectories:
        for key in (
            "track_capacity",
            "hold_capacity",
            "track_autonomy",
            "hold_autonomy",
            "track_defect",
            "hold_defect",
        ):
            value = number(row, key)
            if not math.isfinite(value) or not 0.0 <= value <= 1.0:
                raise CheckError(f"{key} at year {row['year']} is outside [0, 1]: {value}")
        for key in ("track_assistance", "hold_assistance"):
            value = number(row, key)
            if not 0.0 <= value <= 1.0:
                raise CheckError(f"{key} at year {row['year']} is outside [0, 1]: {value}")

    overhead = {
        arm: number(trajectories[0], f"{arm}_assistance") for arm in ("track", "hold")
    }
    for row in trajectories:
        if number(row, "hold_assistance") != overhead["hold"]:
            raise CheckError(
                "the held arm's assistance is not constant; it is supposed to be the "
                f"counterfactual that holds still, but it moved at year {row['year']}"
            )

    cohort = read_csv(os.path.join(directory, "psyche_cohort.csv"))
    arms = {row["arm"] for row in cohort}
    if arms != {"tracking", "held"}:
        raise CheckError(f"cohort arms are {sorted(arms)}, not the pair that is compared")
    for row in cohort:
        autonomy = number(row, "final_autonomy")
        if not 0.0 <= autonomy <= 1.0:
            raise CheckError(f"final autonomy {autonomy} is outside [0, 1]")

    runway = read_csv(os.path.join(directory, "psyche_runway.csv"))
    lags = [int(number(row, "runway_years")) for row in runway]
    if lags != sorted(lags):
        raise CheckError("runway column is not increasing; the axis would be scrambled")
    bought = number(runway[-1], "mean_autonomous_years") - number(
        runway[0], "mean_autonomous_years"
    )
    if bought <= 0.0:
        raise CheckError(
            "a longer runway bought no years of autonomy, which contradicts the sentence the "
            "runway panel is drawn to support"
        )

    tempo = read_csv(os.path.join(directory, "psyche_tempo.csv"))
    rates = [number(row, "tempo_growth") for row in tempo]
    if rates != sorted(rates):
        raise CheckError("tempo column is not increasing")

    thresholds = read_csv(os.path.join(directory, "psyche_thresholds.csv"))
    shares = sorted({number(row, "training_share") for row in thresholds})
    if len(shares) < 2:
        raise CheckError("the threshold panel needs at least two training shares to compare")
    for row in thresholds:
        value = number(row, "sigma_star")
        if math.isfinite(value) and not 0.0 <= value <= 1.0:
            raise CheckError(
                f"deskilling threshold {value} is outside [0, 1] and cannot be a coefficient"
            )

    defects = read_csv(os.path.join(directory, "psyche_defects.csv"))
    for row in defects:
        rate = number(row, "residual_defect_rate")
        floor = number(row, "floor")
        if not 0.0 <= floor <= 1.0 or rate < floor - 1e-9:
            raise CheckError(
                f"residual defect rate {rate} is below its own floor {floor}, so the floor "
                "is not a floor"
            )

    coherence = read_csv(os.path.join(directory, "psyche_coherence.csv"))
    curve = [row for row in coherence if row["section"] == "curve"]
    if number(curve[-1], "value") <= number(curve[0], "value"):
        raise CheckError("emotional coherence does not rise with the human share")

    phase = read_csv(os.path.join(directory, "psyche_phase.csv"))
    for row in phase:
        level = number(row, "optimal_assistance")
        if not 0.0 <= level <= 1.0:
            raise CheckError(f"optimal assistance {level} is outside [0, 1]")

    return {
        "trajectories": trajectories,
        "cohort": cohort,
        "runway": runway,
        "tempo": tempo,
        "thresholds": thresholds,
        "defects": defects,
        "coherence": curve,
        "coherence_targets": [row for row in coherence if row["section"] != "curve"],
        "phase": phase,
    }


def column(rows: list[dict[str, str]], key: str) -> list[float]:
    return [number(row, key) for row in rows]


def stamp(fig, note: str) -> None:
    """Put the provenance note under the figure, where a legend would otherwise float free.

    Wrapped to the figure's own width rather than hand-wrapped at each call site: a note that
    fits one figure and is clipped on a narrower one is the kind of defect that survives review
    because it is only visible in the rendering.
    """
    width = max(80, int(fig.get_figwidth() * 13.5))
    text = textwrap.fill(" ".join(note.split()), width)
    fig.text(
        0.5,
        0.006,
        text,
        ha="center",
        va="bottom",
        fontsize=7.2,
        color=ACCENT,
        linespacing=1.35,
    )


def draw_inversion(rows: list[dict[str, str]], meta: dict, out_path: str) -> None:
    """What tracking does, against the counterfactual in which assistance holds still."""
    years = column(rows, "year")
    floor = meta["autonomy_floor"]
    horizon = int(meta["horizon_years"])

    panels = [
        ("Assistance actually used", "assistance", None),
        ("Capacity to judge", "capacity", None),
        ("Self-knowledge", "self_model", None),
        ("Autonomy, with the declared floor", "autonomy", floor),
        ("Effective output (throughput × quality)", "effective_output", None),
        ("Residual defect rate", "defect", None),
    ]

    fig, axes = plt.subplots(2, 3, figsize=(12.6, 7.0), sharex=True)
    for axis, (title, key, extra) in zip(axes.flat, panels):
        axis.plot(years, column(rows, f"track_{key}"), color=TRACKED, linewidth=1.7, label="tracked")
        axis.plot(years, column(rows, f"hold_{key}"), color=HELD, linewidth=1.7, label="held")
        if key == "self_model":
            # The two arms coincide here, and a panel with one visible line looks like a bug
            # unless it says why. It is a limitation of the model, not an accident of plotting:
            # assistance buys throughput and costs capacity, but nothing in this channel connects
            # it to the inner work, which is bought only by the training share.
            axis.annotate(
                "both arms coincide:\nassistance does not enter\nthis channel in the model",
                xy=(0.5, 0.25),
                xycoords="axes fraction",
                ha="center",
                fontsize=7.8,
                color=FLOOR,
            )
        if extra is not None:
            axis.axhline(extra, color=FLOOR, linewidth=1.0, linestyle=(0, (4, 3)))
            axis.annotate(
                f"declared floor {extra:.2f}",
                xy=(horizon * 0.02, extra),
                xytext=(0, 4),
                textcoords="offset points",
                fontsize=7.5,
                color=FLOOR,
            )
        axis.set_title(title, fontsize=9.5)
        axis.grid(alpha=0.25, linewidth=0.6)
        axis.set_xlim(0, horizon - 1)

    axes.flat[0].set_ylabel("proportion / index")
    axes.flat[3].set_ylabel("proportion / index")
    for axis in axes[1]:
        axis.set_xlabel("year")
    axes.flat[0].legend(frameon=False, fontsize=8.5, loc="lower right")

    fig.suptitle(
        "Tracking a tempo removes the practice that builds the capacity to judge",
        fontsize=11.5,
    )
    fig.tight_layout(rect=(0, 0.085, 1, 0.955))
    stamp(
        fig,
        "Tracked: assistance solved for at each period so the person clears the bar. "
        "Held: the same people with assistance frozen at its initial level — a counterfactual, "
        "not a choice anyone has. "
        "Note that effective output rises under tracking: the model does not say the myopic "
        "objective is self-defeating, it says the objective is incomplete.\n" + DECLARED,
    )
    os.makedirs(os.path.dirname(out_path) or ".", exist_ok=True)
    fig.savefig(out_path, dpi=190)
    plt.close(fig)
    print(f"wrote {out_path}")


def draw_thresholds(export: dict, meta: dict, out_path: str) -> None:
    """The three structural thresholds, plus the defect floor that effort cannot remove."""
    thresholds = export["thresholds"]
    shares = sorted({number(row, "training_share") for row in thresholds})
    palette = ["#d73027", "#fc8d59", "#4575b4", "#313695"]

    fig, axes = plt.subplots(2, 2, figsize=(11.4, 7.2))

    # (a) The deskilling threshold: above this substitution coefficient, full assistance erodes.
    axis = axes[0][0]
    for share, colour in zip(shares, palette):
        points = [
            (number(row, "capacity"), number(row, "sigma_star"))
            for row in thresholds
            if number(row, "training_share") == share
        ]
        points = [(x, y) for x, y in points if math.isfinite(y)]
        if not points:
            continue
        xs, ys = zip(*points)
        axis.plot(xs, ys, color=colour, linewidth=1.7, label=f"training share {share:.2f}")
    axis.set_xlabel("capacity")
    axis.set_ylabel("substitution coefficient above which it deskills")
    axis.set_title("(a) Deskilling threshold", fontsize=10)
    axis.legend(frameon=False, fontsize=8)
    axis.grid(alpha=0.25, linewidth=0.6)
    axis.annotate(
        "where a curve reaches zero, every\nsubstitution deskills: at high capacity\n"
        "the build term is too small to\noffset disuse. The model therefore\n"
        "predicts an expert loses fastest --\na declared build shape, not an\nobservation.",
        xy=(0.03, 0.06),
        xycoords="axes fraction",
        ha="left",
        va="bottom",
        fontsize=7.0,
        color=FLOOR,
    )

    # (b) Emotional coherence, with the critical mixes marked.
    axis = axes[0][1]
    curve = export["coherence"]
    axis.plot(
        [number(row, "human_share") for row in curve],
        [number(row, "value") for row in curve],
        color=TRACKED,
        linewidth=1.8,
    )
    for row in export["coherence_targets"]:
        share = number(row, "human_share")
        target = number(row, "value")
        axis.plot([share, share], [0, target], color=ACCENT, linewidth=0.9, linestyle=(0, (2, 2)))
        axis.plot([0, share], [target, target], color=ACCENT, linewidth=0.9, linestyle=(0, (2, 2)))
        axis.plot([share], [target], marker="o", markersize=4, color=FLOOR)
        axis.annotate(
            f"{target:.2f}",
            xy=(share, target),
            xytext=(4, -9),
            textcoords="offset points",
            fontsize=7.5,
            color=FLOOR,
        )
    axis.set_xlabel("share of interaction that is with a person")
    axis.set_ylabel("emotional coherence")
    axis.set_title("(b) The emotional channel and its critical shares", fontsize=10)
    axis.grid(alpha=0.25, linewidth=0.6)

    # (c) The defect floor: flat at the bottom, and the flat part is what cannot be removed.
    axis = axes[1][0]
    defects = export["defects"]
    omegas = sorted({number(row, "shared_blind_spot") for row in defects})
    for omega, colour in zip(omegas, ["#1a9850", "#d9a01a", "#d73027"]):
        points = [
            row for row in defects if number(row, "shared_blind_spot") == omega
        ]
        xs = [number(row, "capacity") for row in points]
        ys = [number(row, "residual_defect_rate") for row in points]
        axis.plot(xs, ys, color=colour, linewidth=1.7, label=f"shared blind spot {omega:.2f}")
        axis.axhline(
            number(points[0], "floor"),
            color=colour,
            linewidth=0.9,
            linestyle=(0, (3, 3)),
            alpha=0.8,
        )
    axis.set_xlabel("capacity")
    axis.set_ylabel("residual defect rate")
    axis.set_title("(c) Defects the same machine wrote the tests for", fontsize=10)
    axis.legend(frameon=False, fontsize=8, title="correlated share", title_fontsize=8)
    axis.grid(alpha=0.25, linewidth=0.6)

    # (d) The phase boundary, as a line: where maximum assistance stops being optimal.
    axis = axes[1][1]
    phase = export["phase"]
    gains = sorted({number(row, "throughput_gain") for row in phase})
    boundary = []
    for gain in gains:
        points = sorted(
            (number(row, "substitution"), number(row, "optimal_assistance"))
            for row in phase
            if number(row, "throughput_gain") == gain
        )
        crossing = None
        for (s0, a0), (s1, a1) in zip(points, points[1:]):
            if a0 >= 0.995 and a1 < 0.995:
                crossing = (s0 + s1) / 2.0
                break
        if crossing is not None:
            boundary.append((gain, crossing))
    if boundary:
        axis.plot(
            [g for g, _ in boundary],
            [s for _, s in boundary],
            color=TRACKED,
            linewidth=1.9,
            marker="o",
            markersize=4,
        )
        axis.fill_between(
            [g for g, _ in boundary],
            0,
            [s for _, s in boundary],
            color=TRACKED,
            alpha=0.10,
        )
        axis.annotate(
            "below the boundary:\nhold back",
            xy=(boundary[0][0], boundary[0][1] / 2),
            xytext=(14, 6),
            textcoords="offset points",
            fontsize=8.5,
            color=TRACKED,
        )
        axis.annotate(
            "above it:\nassist maximally",
            xy=(boundary[-1][0], boundary[-1][1]),
            xytext=(-12, -26),
            textcoords="offset points",
            fontsize=8.5,
            ha="right",
            color=ACCENT,
        )
    axis.set_xlabel("throughput gain per unit of assistance")
    axis.set_ylabel("substitution coefficient at the boundary")
    axis.set_title("(d) Where full assistance stops being optimal", fontsize=10)
    axis.grid(alpha=0.25, linewidth=0.6)

    fig.suptitle("Three thresholds and one floor", fontsize=11.5)
    fig.tight_layout(rect=(0, 0.10, 1, 0.955))
    stamp(
        fig,
        "The threshold panels are computed from the same functions the tests check, so a wrong "
        "threshold is wrong in the picture as well as in the code. Panel (d) is drawn for a "
        "representative person at capacity 0.70.\n" + DECLARED,
    )
    os.makedirs(os.path.dirname(out_path) or ".", exist_ok=True)
    fig.savefig(out_path, dpi=190)
    plt.close(fig)
    print(f"wrote {out_path}")


def draw_cohort(export: dict, meta: dict, out_path: str) -> None:
    """The distribution behind the means, and the mechanism that produces it."""
    cohort = export["cohort"]
    floor = meta["autonomy_floor"]

    fig, axes = plt.subplots(1, 2, figsize=(11.6, 4.6))

    # (a) The distribution of the ethical variable, both arms.
    axis = axes[0]
    for arm, colour, label in (("tracking", TRACKED, "tracked"), ("held", HELD, "held")):
        values = sorted(number(row, "final_autonomy") for row in cohort if row["arm"] == arm)
        if not values:
            continue
        n = len(values)
        axis.step(
            values,
            [(index + 1) / n for index in range(n)],
            where="post",
            color=colour,
            linewidth=1.8,
            label=label,
        )
    axis.axvline(floor, color=FLOOR, linewidth=1.0, linestyle=(0, (4, 3)))
    share_led = sum(
        1
        for row in cohort
        if row["arm"] == "tracking" and number(row, "final_autonomy") < floor
    ) / max(1, sum(1 for row in cohort if row["arm"] == "tracking"))
    axis.annotate(
        f"declared floor {floor:.2f}\n{share_led:.0%} of the tracked cohort\nbelow it",
        xy=(floor, 0.5),
        xytext=(6, 0),
        textcoords="offset points",
        fontsize=7.8,
        color=FLOOR,
        va="center",
    )
    axis.set_xlabel("autonomy at the horizon")
    axis.set_ylabel("share of cohort at or below")
    axis.set_xlim(0, 1)
    axis.set_ylim(0, 1)
    axis.set_title("(a) Autonomy, whole distribution", fontsize=10)
    axis.grid(alpha=0.25, linewidth=0.6)
    axis.legend(frameon=False, fontsize=8.5, loc="lower right")

    # (b) Where the dispersion comes from: the bar the person's role settles at.
    axis = axes[1]
    for arm, colour, label in (("tracking", TRACKED, "tracked"), ("held", HELD, "held")):
        points = [
            (number(row, "expectation_ceiling"), number(row, "final_capacity"))
            for row in cohort
            if row["arm"] == arm
        ]
        if not points:
            continue
        axis.scatter(
            [x for x, _ in points],
            [y for _, y in points],
            s=3.2,
            color=colour,
            alpha=0.30,
            linewidths=0,
            label=label,
        )
    axis.set_xlabel("the bar the role settles at (a multiple of the pre-AI baseline)")
    axis.set_ylabel("capacity at the horizon")
    axis.set_title("(b) What makes two people end differently", fontsize=10)
    axis.grid(alpha=0.25, linewidth=0.6)
    axis.legend(frameon=False, fontsize=8.5, loc="lower left", markerscale=4)
    axis.annotate(
        "the held arm has no bar to differ on,\nso it has one outcome: capacity\n"
        "converges to the same point for all",
        xy=(0.30, 0.18),
        xycoords="axes fraction",
        ha="center",
        fontsize=7.6,
        color=FLOOR,
    )

    fig.suptitle(
        "The same people under two policies: dispersion is real, and it comes from the bar",
        fontsize=11.5,
    )
    fig.tight_layout(rect=(0, 0.095, 1, 0.945))
    stamp(
        fig,
        f"{int(meta['persons'])} people drawn once and run twice on the same seed, so the gap "
        "between the arms is the tempo's cost and not a difference between two groups.\n"
        + DECLARED,
    )
    os.makedirs(os.path.dirname(out_path) or ".", exist_ok=True)
    fig.savefig(out_path, dpi=190)
    plt.close(fig)
    print(f"wrote {out_path}")


def draw_runway(export: dict, meta: dict, out_path: str) -> None:
    """The runway and the tempo, measured in years of judgement rather than in end states."""
    runway = export["runway"]
    tempo = export["tempo"]

    fig, axes = plt.subplots(1, 2, figsize=(11.0, 4.4))

    axis = axes[0]
    lags = column(runway, "runway_years")
    years = column(runway, "mean_autonomous_years")
    axis.plot(lags, years, color=TRACKED, linewidth=1.9, marker="o", markersize=4)
    axis.set_xlabel("years between the machine arriving and expectations following")
    axis.set_ylabel("mean years spent judging")
    axis.set_title("(a) What the runway buys", fontsize=10)
    axis.grid(alpha=0.25, linewidth=0.6)
    bought = years[-1] - years[0]
    axis.annotate(
        f"+{bought:.1f} years of autonomy\nfor {int(lags[-1])} years of runway",
        xy=(0.05, 0.86),
        xycoords="axes fraction",
        fontsize=8.5,
        ha="left",
        color=TRACKED,
    )

    axis = axes[1]
    rates = column(tempo, "mean_autonomous_years")
    axis.plot(
        column(tempo, "tempo_growth"),
        rates,
        color=TRACKED,
        linewidth=1.9,
        marker="o",
        markersize=4,
    )
    axis.set_xlabel("machine tempo growth (per year)")
    axis.set_ylabel("mean years spent judging")
    axis.set_title("(b) What the tempo costs", fontsize=10)
    axis.grid(alpha=0.25, linewidth=0.6)
    axis.annotate(
        "years of judgement fall as the\nmachine sets a faster rhythm",
        xy=(column(tempo, "tempo_growth")[-1], rates[-1]),
        xytext=(-6, 14),
        textcoords="offset points",
        fontsize=8,
        ha="right",
        color=TRACKED,
    )

    fig.suptitle(
        "The runway changes when, not where — and it is the only lever anyone controls",
        fontsize=11.5,
    )
    fig.tight_layout(rect=(0, 0.095, 1, 0.945))
    stamp(
        fig,
        "Both panels count years above the declared autonomy floor. They do not move the "
        "horizon state, because the endpoint is set by the bar a role settles at rather than by "
        "the path taken to it.\n" + DECLARED,
    )
    os.makedirs(os.path.dirname(out_path) or ".", exist_ok=True)
    fig.savefig(out_path, dpi=190)
    plt.close(fig)
    print(f"wrote {out_path}")


def draw_phase(export: dict, meta: dict, out_path: str) -> None:
    """The whole (speed, substitutability) plane, so the argument's conditionality is visible."""
    phase = export["phase"]
    gains = sorted({number(row, "throughput_gain") for row in phase})
    substitutions = sorted({number(row, "substitution") for row in phase})

    grid = [[math.nan for _ in substitutions] for _ in gains]
    for row in phase:
        i = gains.index(number(row, "throughput_gain"))
        j = substitutions.index(number(row, "substitution"))
        grid[i][j] = number(row, "optimal_assistance")

    cmap = LinearSegmentedColormap.from_list("assist", ["#2166ac", "#f7f7f7", "#b2182b"])
    fig, axis = plt.subplots(figsize=(8.6, 5.2))
    mesh = axis.pcolormesh(
        substitutions,
        gains,
        grid,
        cmap=cmap,
        vmin=0.0,
        vmax=1.0,
        shading="nearest",
    )
    bar = fig.colorbar(mesh, ax=axis, pad=0.02)
    bar.set_label("assistance that maximises effective output")

    # The 0.995 contour is the boundary the argument turns on: below it, holding back wins.
    crossings = []
    for i, gain in enumerate(gains):
        row = grid[i]
        for j in range(len(row) - 1):
            if row[j] >= 0.995 and row[j + 1] < 0.995:
                crossings.append((substitutions[j], gain))
                break
    if crossings:
        xs, ys = zip(*crossings)
        axis.plot(xs, ys, color="#111111", linewidth=2.0, drawstyle="steps-post")
        axis.annotate(
            "hold back fits\nhere",
            xy=(0.30, 0.10),
            xytext=(0.10, 0.34),
            textcoords="axes fraction",
            fontsize=8.4,
            color="#111111",
            arrowprops=dict(arrowstyle="->", color="#111111", linewidth=1.0),
        )
        axis.text(
            0.025,
            0.975,
            "above and right of the line:\nmaximum assistance is optimal",
            transform=axis.transAxes,
            va="top",
            fontsize=8.4,
            color="white",
        )

    axis.set_xlabel("substitution coefficient (practice removed by assistance)")
    axis.set_ylabel("throughput gain per unit of assistance")
    axis.set_title(
        "Where maximum assistance is optimal, and where it is not", fontsize=11.5
    )
    fig.tight_layout(rect=(0, 0.12, 1, 1))
    stamp(
        fig,
        "Representative person at capacity 0.70 and self-knowledge 0.40, 30-year horizon. The "
        "default parameterisation sits in the upper right, where assisting fully wins. The "
        "figure is what makes the argument conditional rather than absolute. " + DECLARED,
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
        ("psychology-1-inversion.png", lambda path: draw_inversion(export["trajectories"], meta, path)),
        ("psychology-2-thresholds.png", lambda path: draw_thresholds(export, meta, path)),
        ("psychology-3-cohort.png", lambda path: draw_cohort(export, meta, path)),
        ("psychology-4-runway.png", lambda path: draw_runway(export, meta, path)),
        ("psychology-5-phase.png", lambda path: draw_phase(export, meta, path)),
    ]
    for name, draw in figures:
        draw(os.path.join(out_dir, name))
    return 0


if __name__ == "__main__":
    sys.exit(main())
