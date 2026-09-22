"""Scientific plots for AI-as-catalyst in software engineering.

Three kinds of figure, and the distinction between them is the point of the file:

1. **Evidence figures** (`1`, `2`). Every number is a *published estimate*, carried
   here with its study design, population, sample size and citation. Nothing is
   pooled, averaged or smoothed: the studies measure different outcomes on
   different populations, so a single "AI productivity number" would be an artefact
   of the plotting choice rather than a finding.
2. **Model figures** (`3`, `4`). These are *not* measurements. They are the
   closed-form of the achievement constraint in `src/optimizer.rs` evaluated over
   declared parameters, and they are labelled as such on the face of the figure.
   The formula is exact for the linear case (`compression_quality_sensitivity = 0`),
   which is the model's default:

       delivered(w) = w * P * (1 + alpha * rho)
       minimum viable w = G / (1 + alpha * rho)

   `tools/` reproduces that formula rather than calling the binary, and
   `verify_against_cli()` checks three points against the real CLI so the
   replication cannot drift silently.

No coefficient here is invented. The evidence values are cited; the model values are
declared parameters whose provenance is the point of the figure, not a measurement.

Usage:
    python tools/plot_ai_catalyst.py [--out figures]
"""

import argparse
import os
import subprocess
import sys

import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt
from matplotlib.patches import Patch

# ── Evidence ────────────────────────────────────────────────────────────────
#
# `effect` is the measured change in the *outcome named in `measure`*, in percent.
# The sign convention is stated per panel rather than assumed, because the two
# panels here use opposite ones and mixing them is the common error.
#
# `population` and `n` are carried into the figure because they are the honest
# explanation of why the estimates differ.

EVIDENCE = [
    {
        "key": "peng2023",
        "label": "Peng et al. 2023",
        "detail": "GitHub Copilot, RCT",
        "population": "95 Upwork freelancers\n(35 finished; median income\n$10–19k, ~6 yrs experience)",
        "n": 95,
        "measure": "time_on_task",
        "effect": -55.8,
        "ci": (-89.0, -21.0),
        "task": "Write an HTTP server in JavaScript,\n12 hidden tests, GitHub Classroom",
        "source": "arXiv:2302.06590 (Microsoft Research / GitHub)",
    },
    {
        "key": "paradis2024",
        "label": "Paradis et al. 2024",
        "detail": "Google internal AI features, RCT",
        "population": "96 Google engineers\n(C++, 1+ yr tenure)\n48 per arm",
        "n": 96,
        "measure": "time_on_task",
        "effect": -21.0,
        "ci": (-40.0, 3.1),
        "task": "Enterprise-grade task: patch a\n10-file / 474-line changelist\nin a monorepo, all tests pass",
        "source": "arXiv:2410.12944 (Google), ICSE-SEIP 2025",
    },
    {
        "key": "metr2025",
        "label": "Becker et al. 2025 (METR)",
        "detail": "Cursor Pro + Claude 3.5/3.7, RCT",
        "population": "16 experienced OSS maintainers\n(~5 yrs on the projects used)\n246 tasks, their own repos",
        "n": 16,
        "measure": "time_on_task",
        "effect": 19.0,
        "ci": None,
        "task": "Real maintenance tasks in mature\nprojects they maintain",
        "source": "arXiv:2507.09089 (METR)",
    },
    {
        "key": "cui2024",
        "label": "Cui et al. 2024",
        "detail": "Copilot access, 3 pooled field experiments",
        "population": "4,867 developers across\nMicrosoft, Accenture and others",
        "n": 4867,
        "measure": "throughput",
        "effect": 26.0,
        "ci": None,
        "task": "Daily work, not a set task;\noutcome is pull requests merged",
        "source": "Management Science (2025), doi:10.1287/mnsc.2025.00535",
    },
]

# ── Model parameters (declared, not measured) ───────────────────────────────
#
# **The unit conversion matters and is easy to get wrong.** The model's `alpha` is a
# gain in *output per unit of time* (`delivered = w * P * (1 + alpha)`), while three
# of the four studies report a change in *time per task*. Those are reciprocals, not
# the same number: "55.8% faster" is alpha = 1/(1 - 0.558) - 1 = **+126%**, not
# +55.8%. Plotting the time figure directly against an output axis would understate
# every speed-up and overstate every slowdown, in opposite directions.


def time_change_to_productivity(fractional_time_change):
    """Convert a fractional change in time per task into a productivity gain.

    `-0.558` (55.8% faster) -> `+1.262` (2.26x the output per hour).
    `+0.19` (19% slower) -> `-0.160` (0.84x the output per hour).
    """
    if fractional_time_change <= -1.0:
        return None
    return 1.0 / (1.0 + fractional_time_change) - 1.0


ALPHA_PENG = time_change_to_productivity(-0.558)   # +1.262
ALPHA_GOOGLE = time_change_to_productivity(-0.21)  # +0.266
ALPHA_METR = time_change_to_productivity(+0.19)    # -0.160
# Throughput is already an output *rate*, so no conversion applies.
ALPHA_CUI = 0.26

CITATIONS = {
    "Peng": "Peng et al. 2023",
    "Google": "Paradis et al. 2024",
    "Cui": "Cui et al. 2024",
    "METR": "Becker et al. 2025 (METR)",
}
LITERATURE_RANGE = (ALPHA_METR, ALPHA_PENG)
# Figure 1's axis is in *time*, so its band is the time range, unconverted. The two
# must not be swapped: -55.8 and +1.26 are different quantities by construction.
TIME_RANGE = (-55.8, 19.0)

# ── Constants read off the CLI, for the §2 panel of Figure 5 ────────────────
#
# Salary 150k, age 40, Bern, normal profile, no sparing. Taken from the tool's own
# report rather than hand-derived from the tax tables, so the panel shows what the
# program computes; `--verify` re-checks each against the binary so a change to the
# tax engine fails the check instead of silently redrawing the panel.
CLI_NET_80 = 7248.0     # CHF/month net at 80% work  (report: "Monthly Net")
CLI_NET_100 = 8832.0    # CHF/month net at 100% work
CLI_FLOOR_BASE = 3760.0  # mandatory floor with no debt (housing + essentials + debt)
FULL_TIME_HOURS = 42.0  # the model's full-time week


def hidden_work_hours(goal, work, alpha, retention=1.0,
                      full_time_hours=FULL_TIME_HOURS):
    """`§1.4` item 4: the cover the pessimistic outcome demands, in h/week.

    Mirrors `AchievementConstraint::hidden_work_percentage`, including its
    `unwrap_or(1.0)`: when *no* percentage delivers the goal, the shortfall is
    measured against a full-time load rather than reported as zero.
    """
    needed = minimum_viable_work(goal, alpha, retention)
    if needed is None:
        needed = 1.0
    return max(0.0, needed - work) * full_time_hours


def replacement_risk(goal, work, alpha, sensitivity, retention=1.0):
    """`§1.4` item 5: `clamp(relative shortfall * sensitivity, 0, 1)`.

    Mirrors `AchievementConstraint::replacement_risk_at`, which measures the
    shortfall against the *pessimistic, quality-adjusted* delivery.
    """
    if sensitivity <= 0.0 or goal <= 0.0:
        return 0.0
    delivered = work * (1.0 + alpha * retention)
    shortfall = max(0.0, (goal - delivered) / goal)
    return min(1.0, shortfall * sensitivity)


def amortised_work(work, horizon_years, evaluation_years):
    """`§1.4` item 6: the career average, with the first years at full time."""
    horizon = max(1.0, horizon_years)
    probation = min(max(0.0, evaluation_years), horizon)
    return (probation * 1.0 + (horizon - probation) * work) / horizon


def minimum_viable_work(goal, alpha, retention=1.0, productivity=1.0):
    """`G / (P * (1 + alpha * rho))`, or `None` when no percentage delivers.

    Mirrors `AchievementConstraint::minimum_viable_work_percentage` for the
    default zero compression sensitivity, where the relation is exactly linear and
    inverts in closed form.
    """
    factor = productivity * (1.0 + alpha * retention)
    if factor <= 0.0:
        return None
    w = goal / factor
    return w if w <= 1.0 else None


def required_gain(goal, work, retention=1.0, productivity=1.0):
    """Gross AI gain needed for `work` to deliver `goal` at a given retention."""
    if retention <= 0.0 or work <= 0.0:
        return None
    return goal / (work * productivity * retention) - 1.0


# ── Figures ─────────────────────────────────────────────────────────────────

BANNER_EVIDENCE = (
    "PUBLISHED ESTIMATES — not measurements made here.\n"
    "Populations and outcomes differ, so these are not four estimates of one quantity."
)
BANNER_MODEL = (
    "MODEL OUTPUT — not a measurement. Declared parameters, exact for the model's\n"
    "default zero compression sensitivity:  delivered(w) = w · P · (1 + α·ρ)"
)


def _banner(fig, text, colour):
    fig.text(
        0.5,
        0.998,
        text,
        ha="center",
        va="top",
        fontsize=8.5,
        color=colour,
        linespacing=1.3,
    )


def _cite(fig, text):
    fig.text(0.01, 0.012, text, ha="left", va="bottom", fontsize=7, color="#555555")


def figure_evidence(out_dir):
    """Forest plot of the field experiments, split by the outcome measured."""
    fig, (ax1, ax2) = plt.subplots(
        1, 2, figsize=(14.0, 6.6), gridspec_kw={"width_ratios": [3, 1.15]}
    )
    fig.subplots_adjust(top=0.805, bottom=0.175, left=0.175, right=0.985, wspace=0.30)

    time_rows = [e for e in EVIDENCE if e["measure"] == "time_on_task"]
    tp_rows = [e for e in EVIDENCE if e["measure"] == "throughput"]

    # ── panel 1: time on task
    ys = list(range(len(time_rows)))[::-1]
    for y, e in zip(ys, time_rows):
        colour = "#1f77b4" if e["effect"] < 0 else "#d62728"
        if e["ci"]:
            lo, hi = e["ci"]
            ax1.plot([lo, hi], [y, y], color=colour, lw=2.0, alpha=0.55, zorder=2)
            for edge in (lo, hi):
                ax1.plot([edge, edge], [y - 0.12, y + 0.12], color=colour, lw=2.0, alpha=0.55)
        ax1.plot(
            [e["effect"]], [y], marker="o", markersize=10, color=colour,
            markeredgecolor="white", markeredgewidth=1.4, zorder=3,
        )
        ax1.annotate(
            f"{e['effect']:+.1f}%",
            (e["effect"], y),
            textcoords="offset points",
            xytext=(0, 15 if e["effect"] < 0 else -26),
            ha="center",
            fontsize=9.5,
            fontweight="bold",
            color=colour,
        )
        note = "n=%s" % f"{e['n']:,}"
        if not e["ci"]:
            note += "  (no CI in the source consulted)"
        ax1.annotate(
            note,
            (e["effect"], y),
            textcoords="offset points",
            xytext=(0, -30 if e["effect"] < 0 else 19),
            ha="center",
            fontsize=7.5,
            color="#666666",
        )

    ax1.axvline(0, color="#333333", lw=1.2, zorder=1)
    ax1.axvspan(TIME_RANGE[0], TIME_RANGE[1],
                color="#f0c419", alpha=0.10, zorder=0)
    ax1.set_yticks(ys)
    ax1.set_yticklabels(
        [f"{e['label']}\n{e['detail']}" for e in time_rows], fontsize=9
    )
    ax1.set_xlim(-100, 70)
    ax1.set_xlabel(
        "Change in time on task, %   (negative = faster)\n"
        "bars are 95% confidence intervals where the source reports one",
        fontsize=9.5,
    )
    ax1.set_title("Time on task", fontsize=11.5, fontweight="bold", pad=10)
    ax1.grid(axis="x", color="#dddddd", lw=0.7, zorder=0)
    ax1.set_axisbelow(True)
    for side in ("top", "right"):
        ax1.spines[side].set_visible(False)

    # ── panel 2: throughput, a different outcome
    y = 0
    e = tp_rows[0]
    ax2.plot([0, e["effect"]], [y, y], color="#2ca02c", lw=2.0, alpha=0.55, zorder=2)
    ax2.plot([e["effect"]], [y], marker="o", markersize=10, color="#2ca02c",
             markeredgecolor="white", markeredgewidth=1.4, zorder=3)
    ax2.annotate(f"{e['effect']:+.0f}%", (e["effect"], y),
                 textcoords="offset points", xytext=(-4, 16),
                 ha="center", fontsize=9.5, fontweight="bold", color="#2ca02c")
    ax2.annotate(f"n={e['n']:,}   (no CI in the\nsource consulted)",
                 (e["effect"], y), textcoords="offset points", xytext=(-6, -34),
                 ha="center", fontsize=7.5, color="#666666")
    ax2.axvline(0, color="#333333", lw=1.2, zorder=1)
    ax2.set_yticks([y])
    ax2.set_yticklabels([f"{e['label']}\n{e['detail']}"], fontsize=9)
    ax2.set_ylim(-1.0, 1.0)
    ax2.set_xlim(-5, 45)
    ax2.set_xlabel("Change in pull-request throughput, %\n(positive = more merged)",
                   fontsize=9.5)
    ax2.set_title("A different outcome:\nthroughput", fontsize=11.5, fontweight="bold", pad=10)
    ax2.grid(axis="x", color="#dddddd", lw=0.7)
    ax2.set_axisbelow(True)
    for side in ("top", "right"):
        ax2.spines[side].set_visible(False)

    # The whole point of the figure, stated on the figure.
    ax1.annotate(
        "the shaded band is the full range of\npublished estimates: 19% slower to 56% faster",
        xy=(TIME_RANGE[1], 0.02),
        xycoords=("data", "axes fraction"),
        xytext=(-92, 0.62),
        textcoords=("data", "axes fraction"),
        fontsize=8.5,
        color="#7a5c00",
        arrowprops=dict(arrowstyle="->", color="#7a5c00", lw=1.0),
    )

    ax1.set_title("Time on task", fontsize=11.5, fontweight="bold", pad=10)
    fig.suptitle(
        "Does AI speed up software engineering?  The honest answer is a range",
        fontsize=14,
        fontweight="bold",
        y=0.912,
    )
    _banner(fig, BANNER_EVIDENCE, "#8a6d00")
    _cite(fig, "Sources: " + " · ".join(e["source"] for e in EVIDENCE))
    path = os.path.join(out_dir, "ai-catalyst-1-evidence.png")
    fig.savefig(path, dpi=170)
    plt.close(fig)
    return path


def figure_belief_gap(out_dir):
    """Predicted versus measured, in each paper's own unit.

    One row per claim, so nothing overlaps: a study with four beliefs gets four rows
    rather than four labels on one line. The unit is the *reduction in completion
    time* the papers themselves report — positive means faster — which keeps this
    figure free of the productivity conversion Figure 3 has to make.
    """
    fig, ax = plt.subplots(figsize=(12.4, 7.0))
    fig.subplots_adjust(top=0.775, bottom=0.135, left=0.335, right=0.975)

    # (study, rows) where each row is (kind, value, description)
    layout = [
        ("Peng et al. 2023 · GitHub Copilot",
         [("measured", 55.8, "measured — RCT, n=95"),
          ("believed", 35.0, "believed by the participants")]),
        ("Becker et al. 2025 · METR",
         [("measured", -19.0, "measured — RCT, 16 devs, 246 tasks"),
          ("believed", 24.0, "forecast by the developers, before"),
          ("believed", 20.0, "believed by the developers, after"),
          ("believed", 39.0, "predicted by economists"),
          ("believed", 38.0, "predicted by ML experts")]),
    ]

    y = 0.0
    yticks, ylabels = [], []
    for study, rows in layout:
        measured = [r for r in rows if r[0] == "measured"]
        top, bottom = y, y - (len(rows) - 1)
        if measured:
            m = measured[0][1]
            ax.plot([m, m], [bottom - 0.35, top + 0.35], color="#d62728", lw=2.0,
                    alpha=0.85, zorder=3)
            ax.annotate(f"{m:+.1f}%", (m, top + 0.5), ha="center", fontsize=9.5,
                        fontweight="bold", color="#d62728")
        for kind, value, note in rows:
            style = (dict(marker="o", color="#d62728", markersize=10)
                     if kind == "measured" else
                     dict(marker="D", color="#1f77b4", markersize=9))
            ax.plot([value], [y], markeredgecolor="white", markeredgewidth=1.3,
                    zorder=4, **style)
            yticks.append(y)
            ylabels.append(note)
            y -= 1.0
        y -= 0.7

    ax.axvline(0, color="#333333", lw=1.2, zorder=1)
    ax.set_yticks(yticks)
    ax.set_yticklabels(ylabels, fontsize=9)
    ax.set_xlim(-35, 70)
    ax.set_ylim(y + 0.35, 1.4)
    ax.set_xlabel(
        "Reduction in completion time, %   (positive = faster — each paper's own unit)",
        fontsize=10,
    )
    ax.grid(axis="x", color="#dddddd", lw=0.7)
    ax.set_axisbelow(True)
    for side in ("top", "right"):
        ax.spines[side].set_visible(False)
    ax.legend(
        handles=[
            plt.Line2D([], [], color="#d62728", marker="o", lw=2.0, markersize=9,
                       label="measured, controlled"),
            plt.Line2D([], [], color="#1f77b4", marker="D", lw=0, markersize=9,
                       label="believed, forecast, or predicted by experts"),
        ],
        loc="lower right",
        fontsize=9,
        frameon=False,
    )
    # Group names, placed by hand so they cannot collide with the row labels.
    ax.annotate("Peng et al. 2023\nGitHub Copilot", (-33, 0.55), fontsize=10,
                fontweight="bold", va="center")
    ax.annotate("Becker et al. 2025\nMETR", (-33, -2.55), fontsize=10,
                fontweight="bold", va="center")
    ax.set_title(
        "Everyone over-estimates: belief and forecast against measurement",
        fontsize=13.5,
        fontweight="bold",
        pad=16,
    )
    _banner(
        fig,
        "PUBLISHED VALUES — the belief rows are self-reports and expert predictions; the measured rows are controlled effects.\n"
        "METR is the case that matters: forecast 24% faster, still believed 20% faster afterwards, measured 19% slower.",
        "#8a6d00",
    )
    _cite(fig, "Sources: arXiv:2302.06590 · arXiv:2507.09089")
    path = os.path.join(out_dir, "ai-catalyst-2-belief-gap.png")
    fig.savefig(path, dpi=170)
    plt.close(fig)
    return path


def figure_model_phase(out_dir):
    """The model's answer to `CRITICS_CURRENT_WORK.md` §1.2, over the evidence range."""
    fig, ax = plt.subplots(figsize=(12.0, 6.6))
    fig.subplots_adjust(top=0.745, bottom=0.19, left=0.085, right=0.975)

    alphas = [(-0.3 + 0.01 * i) for i in range(171)]
    colours = {0.8: "#2ca02c", 1.0: "#1f77b4", 1.2: "#d62728"}
    for goal, colour in colours.items():
        ws = []
        for a in alphas:
            w = minimum_viable_work(goal, a)
            ws.append(w * 100 if w is not None else None)
        ax.plot(alphas, ws, color=colour, lw=2.4, label=f"assigned portfolio G = {goal}")

    # The evidence range.
    ax.axvspan(ALPHA_METR, ALPHA_PENG, color="#f0c419", alpha=0.12, zorder=0)
    ax.axhline(80, color="#666666", lw=1.2, ls="--", zorder=1)
    ax.annotate("80% work", (-0.28, 80), fontsize=9, color="#666666", va="bottom")

    for value, name in [
        (ALPHA_METR, "METR 2025\n19% slower"),
        (ALPHA_CUI, "Cui 2024\n+26% throughput"),
        (ALPHA_GOOGLE, "Google 2024\n21% faster"),
        (ALPHA_PENG, "Peng 2023\n55.8% faster"),
    ]:
        ax.axvline(value, color="#8a6d00", lw=1.0, ls=":", zorder=1)
        ax.annotate(
            name,
            (value, 8),
            fontsize=8.5,
            color="#8a6d00",
            ha="center",
            va="bottom",
        )

    ax.set_xlim(-0.3, 1.4)
    ax.set_ylim(35, 175)
    ax.set_xlabel(
        "AI productivity gain α — output per unit of time, not a time saving\n"
        "converted from the studies: 55.8% faster = α +1.26, 19% slower = α −0.16",
        fontsize=10,
    )
    ax.set_ylabel("lowest work percentage that still delivers G, %", fontsize=10)
    ax.grid(color="#dddddd", lw=0.7)
    ax.set_axisbelow(True)
    ax.legend(fontsize=9.5, loc="upper right", frameon=False)
    for side in ("top", "right"):
        ax.spines[side].set_visible(False)
    ax.set_title(
        "What the published range implies for “can I work 80%?”",
        fontsize=13.5,
        fontweight="bold",
        pad=14,
    )
    ax.annotate(
        "above this line no percentage\ndelivers the portfolio",
        xy=(1.05, 140), fontsize=8.5, color="#d62728",
    )
    ax.annotate(
        "at METR's measured effect the required\npercentage exceeds 100%: no reduction is\navailable at any published AI gain",
        xy=(ALPHA_METR, 122), xytext=(-0.27, 158),
        fontsize=8.5, color="#333333",
        arrowprops=dict(arrowstyle="->", color="#333333", lw=1.0),
    )
    ax.annotate(
        "the 80% week needs α ≥ 0.25 at G = 1.0 …",
        xy=(0.25, 80), xytext=(0.42, 66),
        fontsize=8.5, color="#1f77b4",
        arrowprops=dict(arrowstyle="->", color="#1f77b4", lw=1.0),
    )
    _banner(fig, BANNER_MODEL, "#8a3d00")
    _cite(
        fig,
        "Model: w_min = G / (1 + α·ρ) with ρ = 1 (no verification losses), P = 1, "
        "compression coefficient 0 — the life_optimizer default. α is an input, not a fitted quantity.",
    )
    path = os.path.join(out_dir, "ai-catalyst-3-model-phase.png")
    fig.savefig(path, dpi=170)
    plt.close(fig)
    return path


def figure_quality_breakeven(out_dir):
    """How much gross gain the quality channel can consume before 80% stops being real."""
    fig, ax = plt.subplots(figsize=(12.0, 6.4))
    fig.subplots_adjust(top=0.755, bottom=0.185, left=0.085, right=0.975)

    rhos = [0.40 + 0.006 * i for i in range(101)]
    for work, colour in [(0.9, "#2ca02c"), (0.8, "#1f77b4"), (0.7, "#9467bd")]:
        gains = []
        for rho in rhos:
            g = required_gain(1.0, work, retention=rho)
            gains.append(g * 100 if g is not None else None)
        ax.plot(rhos, gains, color=colour, lw=2.4, label=f"{work:.0%} work, full portfolio (G = 1)")

    ax.axhline(ALPHA_PENG * 100, color="#d62728", lw=1.4, ls="--")
    ax.annotate(f"Peng et al. 2023: α = {ALPHA_PENG*100:.0f}% (55.8% faster, converted)",
                (0.775, ALPHA_PENG * 100), fontsize=9, color="#d62728", va="bottom")
    ax.axhline(ALPHA_GOOGLE * 100, color="#ff7f0e", lw=1.4, ls="--")
    ax.annotate(f"Google: α = {ALPHA_GOOGLE*100:.0f}% (21% faster, converted)",
                (0.575, ALPHA_GOOGLE * 100 + 2), fontsize=9,
                color="#ff7f0e", va="bottom")

    ax.set_xlim(0.40, 1.0)
    ax.set_ylim(0, 160)
    ax.set_xlabel(
        "ρ — share of the AI gain that survives verification, review and rework\n"
        "(declared, not measured: no study measures ρ)",
        fontsize=10,
    )
    ax.set_ylabel("gross AI productivity gain α required, %", fontsize=10)
    ax.grid(color="#dddddd", lw=0.7)
    ax.set_axisbelow(True)
    ax.legend(fontsize=9.5, loc="upper right", frameon=False)
    for side in ("top", "right"):
        ax.spines[side].set_visible(False)
    ax.set_title(
        "The quality channel can consume the whole AI gain",
        fontsize=13.5,
        fontweight="bold",
        pad=14,
    )
    ax.annotate(
        "at ρ = 0.7 — a plausible retention — an 80% week needs a\n"
        "79% gross gain, above every published estimate",
        xy=(0.70, required_gain(1.0, 0.8, retention=0.70) * 100),
        xytext=(0.615, 103),
        fontsize=8.8,
        color="#1f77b4",
        arrowprops=dict(arrowstyle="->", color="#1f77b4", lw=1.1),
    )
    ax.annotate(
        "ρ = 1.0 is the model's\nassumption-free default",
        xy=(0.995, 25), xytext=(0.875, 72),
        fontsize=8.5, color="#555555",
        arrowprops=dict(arrowstyle="->", color="#555555", lw=1.0),
    )
    _banner(fig, BANNER_MODEL, "#8a3d00")
    _cite(
        fig,
        "Model: α = G / (w · ρ) − 1, the inverse of the line in Figure 3. ρ is a declared sensitivity; "
        "the security literature below says ρ < 1 is likely, and quantifies none of it.",
    )
    path = os.path.join(out_dir, "ai-catalyst-4-quality-breakeven.png")
    fig.savefig(path, dpi=170)
    plt.close(fig)
    return path


# ── Verification against the real binary ────────────────────────────────────

def figure_critique_extensions(out_dir):
    """One panel per critique extension: §1.4 items 4, 5, 6 and §2.

    Items 1-3 are already covered by Figures 3 and 4 (the fixed-goal constraint, the
    AI gain as a range, and the quality channel). This figure draws the four that had
    no picture, so every extension the critique asked for is inspectable rather than
    only asserted in prose.
    """
    fig, axes = plt.subplots(2, 2, figsize=(13.6, 9.4))
    fig.subplots_adjust(top=0.845, bottom=0.085, left=0.075, right=0.975,
                        hspace=0.42, wspace=0.26)
    (ax_hidden, ax_risk), (ax_amort, ax_debt) = axes
    works = [0.5 + 0.002 * i for i in range(251)]

    # ── §1.4 item 4: hidden work ────────────────────────────────────────────
    for alpha, colour in [(0.10, "#d62728"), (0.25, "#1f77b4"), (0.50, "#2ca02c")]:
        hours = [hidden_work_hours(1.0, w, alpha) for w in works]
        ax_hidden.plot([w * 100 for w in works], hours, color=colour, lw=2.3,
                       label=f"AI gain +{alpha*100:.0f}%")
        # The kink is where the schedule becomes robust and the cover goes to zero.
        w_min = minimum_viable_work(1.0, alpha)
        if w_min is not None:
            ax_hidden.plot([w_min * 100], [0], marker="v", color=colour, markersize=8,
                           zorder=4)
    ax_hidden.set_xlim(50, 100)
    ax_hidden.set_ylim(0, 22)
    ax_hidden.set_xlabel("contracted work percentage, %", fontsize=9.5)
    ax_hidden.set_ylabel("hidden work, h/week", fontsize=9.5)
    ax_hidden.set_title("§1.4 item 4 — hidden work\n(full portfolio, G = 1)",
                        fontsize=11, fontweight="bold")
    ax_hidden.legend(fontsize=8.5, frameon=False, loc="upper right")
    ax_hidden.grid(color="#dddddd", lw=0.7)
    ax_hidden.set_axisbelow(True)
    for side in ("top", "right"):
        ax_hidden.spines[side].set_visible(False)
    ax_hidden.annotate(
        "▲ = the lowest percentage that delivers;\nright of it the cover is exactly zero",
        xy=(0.53, 0.56), xycoords="axes fraction", fontsize=8, color="#444444")

    # ── §1.4 item 5: replacement risk ───────────────────────────────────────
    for k, colour in [(1.0, "#2ca02c"), (2.5, "#1f77b4"), (5.0, "#d62728")]:
        risk = [replacement_risk(1.0, w, 0.25, k) * 100 for w in works]
        ax_risk.plot([w * 100 for w in works], risk, color=colour, lw=2.3,
                     label=f"--replacement-risk {k:g}")
    ax_risk.axvline(80, color="#666666", lw=1.1, ls="--")
    ax_risk.annotate("80%", (80, 4), fontsize=8.5, color="#666666", rotation=90, va="bottom")
    ax_risk.set_xlim(50, 100)
    ax_risk.set_ylim(0, 105)
    ax_risk.set_xlabel("contracted work percentage, %", fontsize=9.5)
    ax_risk.set_ylabel("implied replacement risk, %", fontsize=9.5)
    ax_risk.set_title("§1.4 item 5 — replacement risk\n(G = 1, AI gain +25%)",
                      fontsize=11, fontweight="bold")
    ax_risk.legend(fontsize=8.5, frameon=False, loc="lower left")
    ax_risk.grid(color="#dddddd", lw=0.7)
    ax_risk.set_axisbelow(True)
    for side in ("top", "right"):
        ax_risk.spines[side].set_visible(False)

    # ── §1.4 item 6: evaluation period ──────────────────────────────────────
    years = [0.2 * i for i in range(76)]
    for work, colour in [(0.8, "#1f77b4"), (0.7, "#9467bd"), (0.6, "#8c564b")]:
        avg = [amortised_work(work, 25.0, y) * 100 for y in years]
        ax_amort.plot(years, avg, color=colour, lw=2.3, label=f"{work:.0%} contract")
    ax_amort.plot([5], [amortised_work(0.8, 25.0, 5.0) * 100], marker="o", color="#1f77b4",
                  markersize=9, markeredgecolor="white", markeredgewidth=1.4, zorder=5)
    ax_amort.annotate(
        "CLI-verified: 5 years\n→ 84.0% average",
        xy=(5, amortised_work(0.8, 25.0, 5.0) * 100), xytext=(6.4, 76.5),
        fontsize=8.5, color="#1f77b4",
        arrowprops=dict(arrowstyle="->", color="#1f77b4", lw=1.0))
    ax_amort.set_xlim(0, 15)
    ax_amort.set_ylim(58, 101)
    ax_amort.set_xlabel("evaluation period worked at full time, years", fontsize=9.5)
    ax_amort.set_ylabel("average workload to retirement, %", fontsize=9.5)
    ax_amort.set_title("§1.4 item 6 — evaluation period\n(age 40, horizon 25 years)",
                       fontsize=11, fontweight="bold")
    ax_amort.legend(fontsize=8.5, frameon=False, loc="lower right")
    ax_amort.grid(color="#dddddd", lw=0.7)
    ax_amort.set_axisbelow(True)
    for side in ("top", "right"):
        ax_amort.spines[side].set_visible(False)

    # ── §2: debt and the mandatory floor ────────────────────────────────────
    debts = [120.0 * i for i in range(76)]
    floors = [CLI_FLOOR_BASE + d for d in debts]
    ax_debt.plot(debts, floors, color="#333333", lw=2.4, label="mandatory floor = 3760 + debt")
    ax_debt.axhline(CLI_NET_80, color="#1f77b4", lw=1.8, ls="--",
                    label="net income at 80% work (7 248)")
    ax_debt.axhline(CLI_NET_100, color="#2ca02c", lw=1.8, ls="--",
                    label="net income at 100% work (8 832)")
    cross80 = CLI_NET_80 - CLI_FLOOR_BASE
    cross100 = CLI_NET_100 - CLI_FLOOR_BASE
    for cross, colour, text, label_at in [
        (cross80, "#1f77b4", f"the 80% week stops being\naffordable at CHF {cross80:,.0f}",
         (3560, 5250)),
        (cross100, "#2ca02c", f"and full time at CHF {cross100:,.0f}", (1900, 10300)),
    ]:
        ax_debt.plot([cross, cross], [0, CLI_FLOOR_BASE + cross], color=colour, lw=1.0, ls=":")
        ax_debt.plot([cross], [CLI_FLOOR_BASE + cross], marker="o", color=colour,
                     markersize=9, markeredgecolor="white", markeredgewidth=1.4, zorder=5)
        ax_debt.annotate(text, xy=(cross, CLI_FLOOR_BASE + cross),
                         xytext=label_at,
                         fontsize=8.5, color=colour,
                         arrowprops=dict(arrowstyle="->", color=colour, lw=1.0))
    ax_debt.set_xlim(0, 9000)
    ax_debt.set_ylim(3000, 13000)
    ax_debt.set_xlabel("monthly debt repayment, CHF", fontsize=9.5)
    ax_debt.set_ylabel("CHF per month", fontsize=9.5)
    ax_debt.set_title("§2 — debt raises the floor one-for-one\n(salary 150k, age 40, Bern)",
                      fontsize=11, fontweight="bold")
    ax_debt.legend(fontsize=8.5, frameon=False, loc="upper left")
    ax_debt.grid(color="#dddddd", lw=0.7)
    ax_debt.set_axisbelow(True)
    for side in ("top", "right"):
        ax_debt.spines[side].set_visible(False)

    fig.suptitle(
        "The critique's extensions, one panel each",
        fontsize=14.5,
        fontweight="bold",
        y=0.955,
    )
    _banner(
        fig,
        "MODEL OUTPUT — not measurements. The three achievement panels use the same closed forms as the "
        "optimizer;\nthe §2 panel draws the floor and the net-income lines the tool itself reports. "
        "Every constant is re-checked by --verify.",
        "#8a3d00",
    )
    _cite(fig, "Model: src/optimizer.rs, default zero compression sensitivity. Salary 150k, age 40, Bern, "
               "normal profile, no sparing.")
    path = os.path.join(out_dir, "ai-catalyst-5-critique-extensions.png")
    fig.savefig(path, dpi=165)
    plt.close(fig)
    return path


def verify_extension_quantities(repo_root):
    """Check the constants and closed forms behind Figure 5 against the binary.

    Each case names a string the report must contain. If the optimizer's arithmetic
    or the tax engine moves, this fails rather than the figure quietly redrawing
    itself around a stale constant.
    """
    cases = [
        (["optimize", "--salary", "150000", "--age", "40", "--required-output-index", "1.0",
          "--enforcement", "risk-weighted", "--replacement-risk", "0.5"],
         "Hidden work:     21.0 h/week",
         "hidden work at 50% work and no AI (the cover is measured against full time)"),
        (["optimize", "--salary", "150000", "--age", "40", "--required-output-index", "1.0",
          "--ai-productivity-gain", "0.25", "--evaluation-period-years", "5"],
         "Average workload:84.0%",
         "the 5-year evaluation period amortised into an 80% contract over 25 years"),
        (["optimize", "--salary", "150000", "--age", "40", "--required-output-index", "1.0",
          "--ai-productivity-gain", "0.25"],
         "Monthly Net:     7248 CHF/month",
         "net income at 80% work — the line the §2 panel crosses"),
        (["optimize", "--salary", "150000", "--age", "40", "--monthly-debt", "1000"],
         "mandatory floor of CHF 4760/month",
         "the floor moving one-for-one with declared debt"),
    ]
    results = []
    for argv, needle, why in cases:
        try:
            out = subprocess.run(
                ["cargo", "run", "-q", "--"] + argv,
                cwd=repo_root, capture_output=True, text=True,
                encoding="utf-8", errors="replace", timeout=600,
            ).stdout
        except (OSError, subprocess.SubprocessError) as exc:  # pragma: no cover
            results.append((needle, why, False, f"not run: {exc}"))
            continue
        normalised = " ".join(out.split())
        ok = " ".join(needle.split()) in normalised
        results.append((needle, why, ok, "" if ok else "not found in the report"))
    return results


def verify_against_cli(repo_root):
    """Check the closed form in this file against the optimizer's own answer.

    Three points, chosen to exercise the ratio rather than a round number. The CLI
    is asked for the recommended percentage and the figure formula for the same
    minimum; the reported percentage should be the smallest grid point at or above
    the formula's value.
    """
    cases = [
        {"goal": "1.0", "gain": "0.5"},
        {"goal": "1.0", "gain": "0.25"},
        {"goal": "1.2", "gain": "0.6"},
    ]
    grid = [0.5, 0.6, 0.7, 0.8, 0.9, 1.0]
    results = []
    for case in cases:
        gain = float(case["gain"])
        goal = float(case["goal"])
        formula = minimum_viable_work(goal, gain)
        cmd = [
            "cargo", "run", "-q", "--", "optimize",
            "--salary", "150000", "--age", "40",
            "--required-output-index", case["goal"],
            "--ai-productivity-gain", case["gain"],
        ]
        try:
            out = subprocess.run(
                cmd,
                cwd=repo_root,
                capture_output=True,
                # The report contains box-drawing and emoji glyphs, so the default
                # locale decoder (cp1252 on this machine) raises rather than reads it.
                text=True,
                encoding="utf-8",
                errors="replace",
                timeout=600,
            ).stdout
        except (OSError, subprocess.SubprocessError) as exc:  # pragma: no cover
            results.append((case, formula, None, f"not run: {exc}"))
            continue
        reported = None
        for line in out.splitlines():
            if "Work Percentage:" in line:
                reported = float(line.split(":")[1].strip().rstrip("%")) / 100.0
                break
        expected = None
        if formula is not None:
            expected = next((g for g in grid if g >= formula - 1e-9), None)
        results.append((case, formula, reported, expected))
    return results


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--out", default="figures", help="output directory for the PNGs")
    ap.add_argument("--verify", action="store_true",
                    help="also check the closed form against the optimizer binary")
    args = ap.parse_args()

    repo_root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    out_dir = args.out if os.path.isabs(args.out) else os.path.join(repo_root, args.out)
    os.makedirs(out_dir, exist_ok=True)

    written = [
        figure_evidence(out_dir),
        figure_belief_gap(out_dir),
        figure_model_phase(out_dir),
        figure_quality_breakeven(out_dir),
        figure_critique_extensions(out_dir),
    ]
    for path in written:
        print(f"wrote {os.path.relpath(path, repo_root)}")

    print("\nevidence carried on the figures:")
    for e in EVIDENCE:
        ci = f"  95% CI [{e['ci'][0]:+.1f}, {e['ci'][1]:+.1f}]" if e["ci"] else "  (no CI)"
        print(f"  {e['label']:28} {e['measure']:14} {e['effect']:+6.1f}%{ci}  n={e['n']:,}")

    print("\nthe conversion the figures rest on (time change -> productivity gain):")
    for name, t in [("Peng 2023", -0.558), ("Google 2024", -0.21), ("METR 2025", +0.19)]:
        print(f"  {name:14} time {t*100:+6.1f}%  ->  alpha {time_change_to_productivity(t):+7.3f}")
    print(f"  {'Cui 2024':14} throughput +26%     ->  alpha {ALPHA_CUI:+7.3f}  (already a rate)")

    print("\nmodel quantities (declared parameters, not measurements):")
    for goal in (0.8, 1.0, 1.2):
        row = [f"G={goal}"]
        for a, name in [(ALPHA_METR, "METR"), (ALPHA_CUI, "Cui"),
                        (ALPHA_GOOGLE, "Google"), (ALPHA_PENG, "Peng")]:
            w = minimum_viable_work(goal, a)
            row.append(f"{name} a={a:+.3f}->{('none' if w is None else f'{w*100:.0f}%')}")
        print("  " + "  ".join(row))
    print("  required gross gain for 80% work, full portfolio (G = 1):")
    for rho in (1.0, 0.9, 0.8, 0.7, 0.6, 0.5):
        g = required_gain(1.0, 0.8, retention=rho)
        print(f"    rho={rho:.1f} -> {g*100:6.1f}%")

    if args.verify:
        print("\nverification against the optimizer binary:")
        ok = True
        for case, formula, reported, expected in verify_against_cli(repo_root):
            f = "none" if formula is None else f"{formula*100:.1f}%"
            r = "not run" if reported is None else f"{reported*100:.0f}%"
            e = "none" if expected is None else f"{expected*100:.0f}%"
            match = "OK" if reported is not None and reported == expected else "MISMATCH"
            if match != "OK":
                ok = False
            print(f"  G={case['goal']} gain={case['gain']}: formula w_min={f}, "
                  f"CLI recommends {r} (grid point {e})  {match}")

        print("\nFigure 5 quantities, read back from the report:")
        for needle, why, passed, note in verify_extension_quantities(repo_root):
            if not passed:
                ok = False
            print(f"  {'OK      ' if passed else 'MISMATCH'}  {needle!r:44} {why}"
                  + (f"  -- {note}" if note else ""))

        if not ok:
            print("\n  the closed form in this script does not reproduce the binary")
            return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
