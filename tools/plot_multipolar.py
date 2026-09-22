#!/usr/bin/env python3
"""Draw the multipolar extrapolation from a `multipolar_sim --export` directory.

Usage:
    cargo run -p multipolar_sim --release -- --scenarios --export out/all
    python tools/plot_multipolar.py out/all                    # figures beside the data
    python tools/plot_multipolar.py out/all multipolar_sim/figures

The second argument sends the figures somewhere other than the data directory, which
is how the copies committed for `multipolar_sim/README.md` are made: the CSV stays
git-ignored, the pictures do not.

The figures are drawn from CSV only: this script never runs the simulation and never
invents a number. Everything it plots was produced by the simulator, and everything
the simulator produces is illustrative by construction -- the blocs, their growth
bias, the payoff coefficients and the pension elasticities are chosen to be plausible
in sign and rough magnitude, not fitted to anything. So the figures carry that note
on their face, and the reading they support is *comparative*: which bloc, which
channel, which assumption moves an outcome, and by how much relative to the others.

Three figures are written:

  fig1-extrapolation   the world, as the model extrapolates it
  fig2-europe-swiss     Europe's position, and the Swiss pension channel
  fig3-scenarios        the named worlds of --scenarios, if they were exported

The Switzerland panel is the pension channel, and only that. Switzerland is not a
bloc in this model -- it is a small open economy with no power share to extrapolate --
so what is drawn is what the model can actually say: how a world's cooperation level
translates into a pay-as-you-go pension promise through the two channels `pension.rs`
documents. What is *not* in the model is listed on the figure itself.
"""

from __future__ import annotations

import csv
import sys
import warnings
from pathlib import Path

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt  # noqa: E402
from matplotlib.patches import Patch  # noqa: E402

# A fixed colour per bloc, so the same actor keeps the same colour across figures --
# reading two panels side by side should not require re-learning the legend.
COLOURS = {
    "United States": "#1f77b4",
    "Europe": "#2ca02c",
    "Sinic": "#d62728",
    "Eurasian": "#9467bd",
    "Indo-Pacific": "#ff7f0e",
    "Africa": "#8c564b",
    "Gulf": "#e377c2",
    "Non-Aligned": "#7f7f7f",
}
FALLBACK = "#17becf"

CAVEAT = (
    "Invented parameters, not a forecast. Read the differences between worlds, "
    "not the levels in them."
)


def colour(name: str) -> str:
    return COLOURS.get(name, FALLBACK)


def read_rows(path: Path) -> list[dict]:
    with path.open(newline="", encoding="utf-8") as handle:
        return list(csv.DictReader(handle))


def require(directory: Path, name: str) -> Path | None:
    path = directory / name
    if not path.exists():
        return None
    return path


def share_bands(directory: Path) -> dict[str, dict[str, list[float]]]:
    """year -> per-bloc quantiles, transposed into bloc -> series."""
    path = require(directory, "shares-bands.csv")
    if path is None:
        return {}
    series: dict[str, dict[str, list[float]]] = {}
    for row in read_rows(path):
        bloc = row["bloc"]
        entry = series.setdefault(bloc, {"year": [], "p10": [], "p50": [], "p90": []})
        entry["year"].append(float(row["year"]))
        for key in ("p10", "p50", "p90"):
            entry[key].append(float(row[key]))
    return series


def world_bands(directory: Path) -> dict[str, list[float]]:
    path = require(directory, "world-bands.csv")
    if path is None:
        return {}
    rows = read_rows(path)
    out: dict[str, list[float]] = {"year": []}
    for column in rows[0].keys():
        if column != "year":
            out[column] = []
    for row in rows:
        for column in out:
            out[column].append(float(row[column]))
    return out


def bloc_summary(directory: Path) -> list[dict]:
    path = require(directory, "blocs-summary.csv")
    return read_rows(path) if path else []


def pension(directory: Path) -> list[dict]:
    path = require(directory, "pension-runs.csv")
    return read_rows(path) if path else []


def _footer(fig) -> None:
    fig.text(
        0.5,
        0.005,
        CAVEAT,
        ha="center",
        va="bottom",
        fontsize=8,
        style="italic",
        color="#555555",
    )


def figure_extrapolation(directory: Path, out: Path) -> bool:
    shares = share_bands(directory)
    world = world_bands(directory)
    summary = bloc_summary(directory)
    if not shares or not world or not summary:
        print("note: no per-year bundle here, so fig1 was skipped")
        return False

    fig, axes = plt.subplots(2, 2, figsize=(15, 9.5))
    fig.suptitle(
        "The multipolar extrapolation, 50 years out\n"
        "median with the 10th-90th percentile spread across runs",
        fontsize=14,
        fontweight="bold",
    )

    # --- power shares --------------------------------------------------------
    ax = axes[0][0]
    ordered = sorted(shares.items(), key=lambda kv: -kv[1]["p50"][-1])
    for name, series in ordered:
        years = series["year"]
        ax.fill_between(
            years, series["p10"], series["p90"], color=colour(name), alpha=0.13, lw=0
        )
        ax.plot(years, series["p50"], color=colour(name), lw=2, label=name)
    ax.set_title("Power share, by bloc")
    ax.set_xlabel("years elapsed")
    ax.set_ylabel("share of world power")
    ax.yaxis.set_major_formatter(lambda value, _: f"{value:.0%}")
    ax.grid(alpha=0.25)
    ax.legend(fontsize=8, ncol=2, loc="upper right", framealpha=0.9)

    # --- cooperation and tension --------------------------------------------
    ax = axes[0][1]
    years = world["year"]
    ax.fill_between(
        years,
        world["cooperation_p10"],
        world["cooperation_p90"],
        color="#2ca02c",
        alpha=0.15,
        lw=0,
    )
    ax.plot(years, world["cooperation_p50"], color="#2ca02c", lw=2, label="cooperation")
    ax.set_ylim(0, 1)
    ax.set_title("How cooperative the system is")
    ax.set_xlabel("years elapsed")
    ax.set_ylabel("cooperation rate")
    ax.grid(alpha=0.25)

    twin = ax.twinx()
    twin.fill_between(
        years,
        world["tension_p10"],
        world["tension_p90"],
        color="#d62728",
        alpha=0.10,
        lw=0,
    )
    twin.plot(
        years,
        world["tension_p50"],
        color="#d62728",
        lw=2,
        linestyle="--",
        label="tension",
    )
    twin.set_ylabel("accumulated tension", color="#d62728")
    twin.tick_params(axis="y", colors="#d62728")
    handles = [
        Patch(facecolor="#2ca02c", alpha=0.4, label="cooperation (median, band)"),
        Patch(facecolor="#d62728", alpha=0.25, label="tension (median, band)"),
    ]
    ax.legend(handles=handles, fontsize=8, loc="lower right")

    # --- where the shares end, and how often --------------------------------
    ax = axes[1][0]
    names = [row["bloc"] for row in summary]
    lows = [float(row["end_p10"]) for row in summary]
    medians = [float(row["end_p50"]) for row in summary]
    highs = [float(row["end_p90"]) for row in summary]
    positions = range(len(names))
    ax.barh(
        list(positions),
        medians,
        color=[colour(name) for name in names],
        alpha=0.85,
    )
    ax.errorbar(
        medians,
        list(positions),
        xerr=[
            [m - low for m, low in zip(medians, lows)],
            [high - m for m, high in zip(medians, highs)],
        ],
        fmt="none",
        ecolor="#333333",
        elinewidth=1.2,
        capsize=3,
    )
    ax.set_yticks(list(positions))
    ax.set_yticklabels(names, fontsize=9)
    ax.invert_yaxis()
    ax.set_title("Where power ends up at year 50 (bar = median, whiskers = p10-p90)")
    ax.set_xlabel("share of world power")
    ax.xaxis.set_major_formatter(lambda value, _: f"{value:.0%}")
    ax.grid(alpha=0.25, axis="x")
    for index, row in enumerate(summary):
        if float(row["dominance_probability"]) > 0:
            ax.text(
                float(row["end_p90"]) + 0.005,
                index,
                f"P(dominant) {float(row['dominance_probability']):.0%}",
                va="center",
                fontsize=8,
                color="#333333",
            )

    # --- the pension channel -------------------------------------------------
    ax = axes[1][1]
    rows = pension(directory)
    security = [float(row["security_index"]) for row in rows]
    ax.hist(security, bins=30, color="#4c72b0", alpha=0.85)
    ax.axvline(1.0, color="#333333", linestyle="--", lw=1)
    ax.text(
        1.0,
        ax.get_ylim()[1] * 0.92,
        "  1.0 = the fully cooperative reference",
        fontsize=8,
        color="#333333",
    )
    ax.set_title(
        "Pension security index at year 50\n(1.0 = a fully cooperative world, 0 = nothing)"
    )
    ax.set_xlabel("security index")
    ax.set_ylabel("runs")

    fig.tight_layout(rect=(0, 0.02, 1, 0.95))
    _footer(fig)
    fig.savefig(out, dpi=140)
    plt.close(fig)
    print(f"wrote {out}")
    return True


def figure_europe_swiss(directory: Path, out: Path) -> bool:
    shares = share_bands(directory)
    rows = pension(directory)
    exposure_path = require(directory, "exposure.csv")
    leverage_path = require(directory, "monetary-leverage.csv")
    if not shares or not rows or exposure_path is None or leverage_path is None:
        print("note: this directory has no world bundle, so fig2 was skipped")
        return False
    exposure = {row["bloc"]: row for row in read_rows(exposure_path)}
    leverage = {
        (row["holder"], row["subject"]): float(row["leverage"])
        for row in read_rows(leverage_path)
    }

    fig = plt.figure(figsize=(15, 10))
    fig.suptitle(
        "Europe and Switzerland in this model\n"
        "Europe is a bloc; Switzerland is not, and can only be read through the "
        "pension channel",
        fontsize=14,
        fontweight="bold",
    )
    grid = fig.add_gridspec(3, 3, hspace=0.62, wspace=0.28)

    # --- Europe against the poles -------------------------------------------
    ax = fig.add_subplot(grid[0, :2])
    for name in ("United States", "Europe", "Sinic", "Indo-Pacific"):
        series = shares.get(name)
        if not series:
            continue
        ax.fill_between(
            series["year"],
            series["p10"],
            series["p90"],
            color=colour(name),
            alpha=0.15,
            lw=0,
        )
        ax.plot(series["year"], series["p50"], color=colour(name), lw=2.4, label=name)
    ax.set_title("Europe against the United States and the Asian poles")
    ax.set_xlabel("years elapsed")
    ax.set_ylabel("share of world power")
    ax.yaxis.set_major_formatter(lambda value, _: f"{value:.0%}")
    ax.grid(alpha=0.25)
    ax.legend(fontsize=9)

    # --- what Europe is exposed to ------------------------------------------
    ax = fig.add_subplot(grid[0, 2])
    europe = exposure.get("Europe", {})
    labels = [
        "energy\nimports",
        "energy\ndisruption",
        "reserve\ncurrency",
        "US leverage\nover it",
    ]
    values = [
        float(europe.get("import_dependence", 0)),
        float(europe.get("disruption_exposure", 0)),
        float(europe.get("reserve_share", 0)),
        leverage.get(("United States", "Europe"), 0.0),
    ]
    ax.bar(labels, values, color=["#2ca02c", "#2ca02c", "#1f77b4", "#d62728"], alpha=0.85)
    ax.set_title("Europe's structural position")
    ax.set_ylim(0, 1)
    ax.tick_params(axis="x", labelsize=8)
    for index, value in enumerate(values):
        ax.text(index, value + 0.02, f"{value:.2f}", ha="center", fontsize=8)
    ax.grid(alpha=0.25, axis="y")

    # --- the Swiss pension channel ------------------------------------------
    panels = [
        ("replacement_rate", "PAYG replacement rate", "#4c72b0"),
        ("support_ratio", "contributor-to-retiree ratio", "#55a868"),
        ("portfolio_return", "portfolio real return", "#c44e52"),
    ]
    for index, (key, label, shade) in enumerate(panels):
        ax = fig.add_subplot(grid[1, index])
        values = [float(row[key]) for row in rows]
        ax.hist(values, bins=30, color=shade, alpha=0.85)
        ax.set_title(f"Swiss pension channel:\n{label}", fontsize=9.5)
        ax.set_ylabel("runs", fontsize=9)
        ax.tick_params(labelsize=8, labelrotation=30)
        ax.xaxis.set_major_locator(plt.MaxNLocator(5))

    ax = fig.add_subplot(grid[2, 0])
    security = [float(row["security_index"]) for row in rows]
    ax.hist(security, bins=30, color="#8172b2", alpha=0.85)
    ax.axvline(1.0, color="#333333", linestyle="--", lw=1)
    ax.set_title("Swiss pension channel:\nsecurity index", fontsize=9.5)
    ax.set_xlabel("1.0 = a fully cooperative world", fontsize=8)
    ax.set_ylabel("runs", fontsize=9)
    ax.tick_params(labelsize=8)

    ax = fig.add_subplot(grid[2, 1:])
    ax.axis("off")
    ax.text(
        0.0,
        1.0,
        "Switzerland is not a bloc here, so this is the pension channel and nothing\n"
        "else. The elasticities from geopolitics to pensions are invented; the AHV's\n"
        "real replacement and contribution rates are not, but they do not enter here.\n"
        "\n"
        "NOT in the model: the SNB and safe-haven flows, pharmaceutical and financial\n"
        "exports, Swiss-EU bilateral agreements, Swiss energy imports specifically,\n"
        "and any fiscal response. Europe's row above is a bloc; Switzerland's is a\n"
        "channel, and the two should not be read as the same kind of object.",
        va="top",
        ha="left",
        fontsize=9,
        color="#333333",
        bbox=dict(boxstyle="round", facecolor="#f5f5f5", edgecolor="#cccccc"),
    )

    _footer(fig)
    # The lower-right cell is a text panel with no axes, which `tight_layout` reports
    # itself unable to handle -- and constrained layout, the suggested alternative,
    # spreads the three rows far enough apart to look broken. The result here was
    # checked by eye, so the warning is silenced rather than acted on.
    with warnings.catch_warnings():
        warnings.filterwarnings("ignore", message=".*not compatible with tight_layout.*")
        fig.tight_layout(rect=(0, 0.02, 1, 0.93))
    fig.savefig(out, dpi=140)
    plt.close(fig)
    print(f"wrote {out}")
    return True


def figure_scenarios(directory: Path, out: Path) -> bool:
    world_path = directory / "scenarios-world.csv"
    shares_path = directory / "scenarios-shares.csv"
    if not world_path.exists() or not shares_path.exists():
        return False

    world = read_rows(world_path)
    shares = read_rows(shares_path)

    fig, axes = plt.subplots(1, 2, figsize=(16, 8), gridspec_kw={"width_ratios": [3, 2]})
    fig.suptitle(
        "Named worlds, the same shock draws\n"
        "only the named assumption differs between them",
        fontsize=14,
        fontweight="bold",
    )

    ax = axes[0]
    blocs = []
    for row in shares:
        if row["bloc"] not in blocs:
            blocs.append(row["bloc"])
    scenarios = []
    for row in world:
        if row["scenario"] not in scenarios:
            scenarios.append(row["scenario"])
    width = 0.8 / max(len(scenarios), 1)
    for offset, scenario in enumerate(scenarios):
        values = [
            next(
                (
                    float(row["end_share"])
                    for row in shares
                    if row["scenario"] == scenario and row["bloc"] == bloc
                ),
                0.0,
            )
            for bloc in blocs
        ]
        positions = [index + offset * width for index in range(len(blocs))]
        ax.bar(positions, values, width=width, label=scenario, alpha=0.9)
    ax.set_xticks([index + 0.4 - width / 2 for index in range(len(blocs))])
    ax.set_xticklabels(blocs, rotation=30, ha="right", fontsize=9)
    ax.set_ylabel("share of world power at year 50")
    ax.yaxis.set_major_formatter(lambda value, _: f"{value:.0%}")
    ax.set_title("Where power ends up in each world")
    ax.legend(fontsize=8)
    ax.grid(alpha=0.25, axis="y")

    ax = axes[1]
    ax.axis("off")
    header = f"{'world':<26}{'coop':>6}{'trap':>7}{'pension':>9}  leader"
    lines = [header, "-" * len(header)]
    for row in world:
        lines.append(
            f"{row['scenario'][:25]:<26}"
            f"{float(row['cooperation']):>6.3f}"
            f"{float(row['trap']):>7.1%}"
            f"{float(row['pension']):>9.3f}"
            f"  {row['leader']}"
        )
    lines.append("")
    lines.append("trap = share of years both tense and uncooperative")
    lines.append("pension = security index, 1.0 = fully cooperative")
    for index, row in enumerate(world):
        lines.append("")
        lines.append(f"{row['scenario']}:")
        premise = row["premise"]
        while premise:
            lines.append("  " + premise[:78])
            premise = premise[78:]
    ax.text(
        0.0,
        1.0,
        "\n".join(lines),
        va="top",
        ha="left",
        family="monospace",
        fontsize=8.5,
    )

    _footer(fig)
    fig.tight_layout(rect=(0, 0.02, 1, 0.94))
    fig.savefig(out, dpi=140)
    plt.close(fig)
    print(f"wrote {out}")
    return True


def main() -> int:
    if len(sys.argv) not in (2, 3):
        print(__doc__)
        return 2
    directory = Path(sys.argv[1])
    if not directory.is_dir():
        sys.exit(f"error: {directory} is not a directory")
    out = Path(sys.argv[2]) if len(sys.argv) == 3 else directory
    out.mkdir(parents=True, exist_ok=True)

    figure_extrapolation(directory, out / "fig1-extrapolation.png")
    figure_europe_swiss(directory, out / "fig2-europe-swiss.png")
    if not figure_scenarios(directory, out / "fig3-scenarios.png"):
        print("note: no scenarios-world.csv here, so fig3 was skipped")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
