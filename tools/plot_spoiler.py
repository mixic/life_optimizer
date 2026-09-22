"""Scientific plots for the third-party spoiler model.

Two kinds of content, and the distinction is the point of the file:

1. **Theorems** (`0`-`6`). Every curve is a closed form from `THIRD_PARTY_SPOILER.md`
   section 2, drawn on a banner that says so. These are not measurements and not
   simulation output: they are the algebra, drawn.
2. **Nothing else.** There is deliberately no evidence panel. What an evidence panel
   would need is the capture rate `g` -- the share of a war's efficiency loss that
   accrues to a third party -- and section 7 of the document reports that nobody has
   estimated it for any conflict.

Three of the panels show something the statements alone do not convey: that the attack
threshold's optimum *bifurcates* on the operating cost (T1), that fragmentation on its own
can make a coalition **more** reliable (T2), and that punishment only ever closes the
high-exposure tail (T6).

Each curve's defining equation is re-checked numerically before anything is drawn, so a
figure cannot be produced from a model that no longer satisfies its own theorems.

Usage:
    python tools/plot_spoiler.py [--out figures]
"""

import argparse
import math
import os
import sys

import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt

# ── The model, as closed forms ──────────────────────────────────────────────
#
# These mirror `multipolar_sim/src/spoiler.rs` rather than calling it. The theorems are
# asserted as invariants by that module's own tests, which is the stronger check; what
# this file adds is the picture, and `verify_definitions()` stops the picture drifting.

CAPTURE = 0.35         # g      -- share of the victims' loss the spoiler captures
LOSS = 0.45            # Lambda -- the harvest base
OPERATING_COST = 0.02  # epsilon
NATURAL_MARGIN = 0.60  # sigma_0
CURVATURE = 1.60       # a, the curvature of the grievance cost


def peace_margin(cc, dc):
    """`sigma = cc - dc`. Negative means the cooperative cell is not an equilibrium."""
    return cc - dc


def transfer_needed(margin, epsilon=OPERATING_COST):
    """`t*(sigma) = max(0, sigma) + epsilon`."""
    return max(0.0, margin) + epsilon


def harvest(capture=CAPTURE, loss=LOSS):
    """`g * Lambda`."""
    return capture * loss


def leverage_ratio(margin, capture=CAPTURE, loss=LOSS, epsilon=OPERATING_COST):
    """`R(sigma) = g*Lambda / (max(0, sigma) + epsilon)`."""
    return harvest(capture, loss) / transfer_needed(margin, epsilon)


def grievance_cost(margin, natural=NATURAL_MARGIN, curvature=CURVATURE):
    """`c_m(sigma) = a/2 * (natural - sigma)^2` for a margin below the natural one."""
    reduction = max(0.0, natural - margin)
    return 0.5 * curvature * reduction * reduction


def grievance_objective(margin, capture=CAPTURE, loss=LOSS, epsilon=OPERATING_COST,
                        natural=NATURAL_MARGIN, curvature=CURVATURE):
    """`Pi(sigma) = g*Lambda/(sigma + epsilon) - c_m(sigma)`."""
    return harvest(capture, loss) / (max(0.0, margin) + epsilon) - grievance_cost(
        margin, natural, curvature
    )


def critical_operating_cost(capture=CAPTURE, loss=LOSS, natural=NATURAL_MARGIN,
                            curvature=CURVATURE):
    """`epsilon_c = sqrt(g*Lambda / (a*sigma_0))`, the bifurcation point."""
    return math.sqrt(harvest(capture, loss) / (curvature * natural))


def grievance_optimum(epsilon, capture=CAPTURE, loss=LOSS, natural=NATURAL_MARGIN,
                      curvature=CURVATURE):
    """Where the spoiler stops: the corner `sigma = 0`, or an interior margin.

    Returns `(margin, at_edge)`. The theorem is a bifurcation -- the first draft got this
    wrong by asserting the interior case unconditionally.
    """
    if epsilon <= critical_operating_cost(capture, loss, natural, curvature):
        return 0.0, True
    condition = lambda m: curvature * (natural - m) - harvest(capture, loss) / (m + epsilon) ** 2
    lo, hi = 0.0, natural
    for _ in range(200):
        mid = 0.5 * (lo + hi)
        if condition(mid) > 0.0:
            lo = mid
        else:
            hi = mid
    return 0.5 * (lo + hi), False


def weight_distribution(weights, support):
    """Poisson-binomial distribution of the total supporting weight, in integer units."""
    capacity = sum(max(0, int(round(w))) for w in weights)
    distribution = [0.0] * (capacity + 1)
    distribution[0] = 1.0
    reach = 0
    for weight, p in zip(weights, support):
        w = max(0, int(round(weight)))
        p = min(max(p, 0.0), 1.0)
        for total in range(reach + w, -1, -1):
            with_support = distribution[total - w] * p if total >= w else 0.0
            distribution[total] = distribution[total] * (1.0 - p) + with_support
        reach += w
    return distribution


def action_probability(weights, support, quota):
    """Probability the coalition reaches the quota."""
    distribution = weight_distribution(weights, support)
    return sum(p for total, p in enumerate(distribution) if total >= quota)


def split_independent(weights, support, index, parts):
    """Fragmentation on its own: each part decides for itself."""
    piece = weights[index] / parts
    w = list(weights[:index]) + [piece] * parts + list(weights[index + 1:])
    s = list(support[:index]) + [support[index]] * parts + list(support[index + 1:])
    return w, s


def split_estranged(weights, support, index, parts):
    """Divide and conquer proper: the parts can never both support.

    At most one part contributes, so the member's effective weight is capped at its
    largest part.
    """
    capped = weights[index] / parts
    w = list(weights)
    w[index] = capped
    return w, list(support)


def individual_threshold(cost_per_punisher, benefit):
    """`phi* = K/B`, the threshold each victim uses privately."""
    return cost_per_punisher / benefit if benefit > 0 else math.inf


def max_punishment(capacity_per_surplus, surplus, loss):
    """`K_max = k*(S - Lambda)`: what the victims can still inflict after their own loss."""
    return capacity_per_surplus * max(0.0, surplus - loss)


def attribution_threshold(harvest_value, capacity, victims):
    """`phi-dagger = g*Lambda / (n * K_max)`."""
    denominator = capacity * victims
    return harvest_value / denominator if denominator > 0 else math.inf


def profitable_until(total_cost, harvest_value):
    """`e-bar = 1 - T/H`."""
    return 1.0 - total_cost / harvest_value if harvest_value > 0 else 0.0


def deterred_from(harvest_value, punishment_capacity):
    """`e-dagger = H / (phi n k S + H)`."""
    return harvest_value / (punishment_capacity + harvest_value)


def blowback_optimum(benefit_scale=1.0, damage=4.0, discount=0.10,
                     hazard_curvature=1.20, support_curvature=0.80):
    """`s*` from `B'(s) = h'(s)H/(1+r) + c'(s)`."""
    derivative = lambda s: (
        benefit_scale / (2.0 * math.sqrt(max(s, 1e-12)))
        - hazard_curvature * s * damage / (1.0 + discount)
        - support_curvature * s
    )
    lo, hi = 1e-9, 1.0
    while derivative(hi) > 0.0 and hi < 1e9:
        hi *= 2.0
    for _ in range(200):
        mid = 0.5 * (lo + hi)
        if derivative(mid) > 0.0:
            lo = mid
        else:
            hi = mid
    return 0.5 * (lo + hi)


def verify_definitions():
    """Re-check each curve's defining equation before drawing it.

    Not a substitute for the Rust invariants -- those check the theorems. This stops the
    *picture* from being drawn from a model that has stopped satisfying its definitions.
    """
    results = []

    # Theorem 1: leverage rises as the margin closes and diverges at the edge.
    ratios = [leverage_ratio(m) for m in [1.0, 0.75, 0.5, 0.25, 0.1, 0.01]]
    results.append((
        "T1 leverage rises as the margin closes",
        all(b > a for a, b in zip(ratios, ratios[1:])),
        f"{ratios[0]:.3f} -> {ratios[-1]:.1f}",
    ))
    results.append((
        "T1 leverage diverges to harvest/epsilon at the edge",
        abs(leverage_ratio(0.0) - harvest() / OPERATING_COST) < 1e-12,
        f"R(0) = {leverage_ratio(0.0):.4f}",
    ))

    # Theorem 1(iii): the bifurcation.
    critical = critical_operating_cost()
    brazen, brazen_edge = grievance_optimum(0.5 * critical)
    subtle, subtle_edge = grievance_optimum(2.0 * critical)
    results.append((
        "T1 the optimum bifurcates at epsilon_c",
        brazen_edge and not subtle_edge and subtle > 0.0,
        f"eps_c = {critical:.4f}: corner below, interior {subtle:.3f} above",
    ))
    results.append((
        "T1 a dearer operation is a subtler one",
        all(
            grievance_optimum(e)[0] < grievance_optimum(e + 0.2)[0]
            for e in [0.5, 0.7, 1.0, 1.5]
        ),
        "the optimal margin rises in epsilon",
    ))

    # Theorem 2: the two ways of splitting point in opposite directions.
    weights, support, quota = [6.0, 3.0, 3.0, 2.0], [0.8] * 4, 9.0
    before = action_probability(weights, support, quota)
    indep = action_probability(*split_independent(weights, support, 0, 2), quota)
    estran = action_probability(*split_estranged(weights, support, 0, 2), quota)
    results.append((
        "T2 fragmentation alone can ARM the coalition",
        indep > before,
        f"{before:.4f} -> {indep:.4f} under an independent split",
    ))
    results.append((
        "T2 the estrangement disarms it",
        estran < before,
        f"{before:.4f} -> {estran:.4f} when the parts cannot both support",
    ))

    # Theorem 3: the attribution threshold is positive and binding.
    capacity = max_punishment(0.5, 4.0, 1.5)
    threshold = attribution_threshold(0.5, capacity, 5)
    results.append((
        "T3 there is a positive attribution threshold",
        threshold > 0.0 and individual_threshold(0.3, 0.6) > 0.0,
        f"phi-dagger = {threshold:.4f}, phi* = {individual_threshold(0.3, 0.6):.2f}",
    ))
    results.append((
        "T3 the induced conflict depletes the capacity to punish",
        capacity < 0.5 * 4.0,
        f"K_max falls from {0.5 * 4.0} to {capacity}",
    ))

    # Theorem 4: impatience funds the proxy harder.
    patient = blowback_optimum(discount=0.02)
    impatient = blowback_optimum(discount=0.50)
    results.append((
        "T4 the impatient sponsor supports harder",
        impatient > patient,
        f"{patient:.3f} at r = 0.02 vs {impatient:.3f} at r = 0.50",
    ))

    # Theorem 6: the two conditions are monotone in the same variable, opposite ways.
    h = 0.40 * 0.50
    bar = profitable_until(0.10, h)
    dagger_strong = deterred_from(h, 3.00)
    dagger_weak = deterred_from(h, 0.15)
    results.append((
        "T6 profit and punishment move oppositely in exposure",
        bar > 0.0 and dagger_strong < dagger_weak,
        f"e-bar = {bar:.3f}; e-dagger {dagger_strong:.3f} (strong) vs {dagger_weak:.3f} (weak)",
    ))
    results.append((
        "T6 the containment condition is a condition, not an identity",
        (0.10 / 0.10) <= (h / 0.15) and not ((0.10 / 0.10) <= (h / 3.00)),
        "it holds for weak victims and fails for strong ones",
    ))
    results.append((
        "T6 the deterred band never reaches zero exposure",
        dagger_strong > 0.0,
        "the most profitable spoilers are untouched by punishment",
    ))

    return results


# ── Figures ─────────────────────────────────────────────────────────────────

BANNER = (
    "MODEL OUTPUT -- not a measurement and not a simulation. Every curve is a closed form\n"
    "from THIRD_PARTY_SPOILER.md section 2, drawn with that section's declared defaults."
)


def _banner(fig, text, colour="#8a6d00"):
    fig.text(0.5, 0.998, text, ha="center", va="top", fontsize=8.5, color=colour,
             linespacing=1.3)


def _cite(fig, text):
    fig.text(0.01, 0.010, text, ha="left", va="bottom", fontsize=7, color="#555555")


def _clean(ax):
    for side in ("top", "right"):
        ax.spines[side].set_visible(False)
    ax.grid(color="#dddddd", lw=0.7)
    ax.set_axisbelow(True)


def plot_theorems(out_dir):
    """Six panels, one per result."""
    fig, axes = plt.subplots(2, 3, figsize=(15.5, 8.8))
    fig.subplots_adjust(top=0.845, bottom=0.105, left=0.058, right=0.985,
                        wspace=0.30, hspace=0.42)
    ax0, ax1, ax2, ax3, ax4, ax6 = axes.ravel()

    # ---- Theorem 0: three readings of "conflict" --------------------------
    # Plotted on the (cc, dc) plane: the diagonal is sigma = 0, and the three readings
    # occupy different regions of it.
    span = [i / 100 * 4.0 for i in range(401)]
    ax0.plot(span, span, color="#333333", lw=1.6, label="$\\sigma = 0$")
    ax0.fill_between(span, span, 4.0, color="#2ca02c", alpha=0.10)
    ax0.fill_between(span, 0.0, span, color="#d62728", alpha=0.10)
    ax0.set_xlim(0, 4.0)
    ax0.set_ylim(0, 4.0)
    ax0.set_xlabel("payoff from mutual cooperation  $cc$", fontsize=9.5)
    ax0.set_ylabel("temptation to defect  $dc$", fontsize=9.5)
    ax0.text(2.6, 3.2, "PEACE\n$cc \\geq dc$:\nthe cooperative cell\nIS an equilibrium",
             fontsize=8.5, color="#1d6b1d", ha="center", va="center")
    ax0.text(1.1, 0.7, "CONFLICT\n$cc < dc$", fontsize=8.5, color="#8b1a1a",
             ha="center", va="center")
    # The two cases where the readings come apart.
    ax0.plot([2.4], [1.2], marker="o", markersize=10, color="#2ca02c",
             markeredgecolor="white", markeredgewidth=1.4, zorder=4)
    ax0.annotate("crossed interests,\nno conflict\n(coordination)",
                 xy=(2.4, 1.2), xytext=(2.75, 1.75), fontsize=7.5, color="#1d6b1d",
                 arrowprops=dict(arrowstyle="->", color="#1d6b1d", lw=0.9))
    ax0.plot([2.0], [2.0], marker="o", markersize=10, color="#ff7f0e",
             markeredgecolor="white", markeredgewidth=1.4, zorder=4)
    ax0.annotate("violence with NO\nfailure of equilibrium\n(mistrust)",
                 xy=(2.0, 2.0), xytext=(0.25, 2.45), fontsize=7.5, color="#a04a00",
                 arrowprops=dict(arrowstyle="->", color="#a04a00", lw=0.9))
    ax0.set_title("T0 · what a conflict is", fontsize=10.5, fontweight="bold", pad=8)
    ax0.legend(fontsize=8, frameon=False, loc="upper left")
    _clean(ax0)

    # ---- Theorem 1: leverage, and the bifurcation -------------------------
    margins = [i / 400 * 1.2 for i in range(401)]
    ax1.plot(margins, [leverage_ratio(m) for m in margins], color="#1f77b4", lw=2.4)
    ax1.set_yscale("log")
    ax1.set_xlim(0, 1.2)
    ax1.set_ylim(0.08, 12.0)
    ax1.set_yticks([0.1, 0.2, 0.5, 1.0, 2.0, 5.0, 10.0])
    ax1.set_yticklabels(["0.1", "0.2", "0.5", "1", "2", "5", "10"])
    ax1.set_xlabel("peace margin  $\\sigma$", fontsize=9.5)
    ax1.set_ylabel("return per unit paid  $R$  (log scale)", fontsize=9.5)
    critical = critical_operating_cost()
    for epsilon, colour, style in [(0.5 * critical, "#d62728", "-"),
                                   (2.0 * critical, "#2ca02c", "--")]:
        margin, _ = grievance_optimum(epsilon)
        ax1.axvline(margin, color=colour, lw=1.2, ls=style)
        ax1.plot([margin], [leverage_ratio(margin)], marker="o", markersize=8,
                 color=colour, markeredgecolor="white", markeredgewidth=1.3, zorder=4)
    ax1.annotate(
        "cheap operation ($\\epsilon < \\epsilon_c$):\nthe corner at $\\sigma$ = 0, where the\n"
        "return is harvest / $\\epsilon$",
        xy=(0.01, leverage_ratio(0.0)), xytext=(0.22, 3.2), fontsize=7.5, color="#d62728",
        arrowprops=dict(arrowstyle="->", color="#d62728", lw=0.9),
    )
    ax1.annotate(
        "dear operation: stops short\nat an interior margin\n($\\epsilon_c$ = %.2f)" % critical,
        xy=(grievance_optimum(2.0 * critical)[0],
            leverage_ratio(grievance_optimum(2.0 * critical)[0])),
        xytext=(0.62, 0.16), fontsize=7.5, color="#2ca02c",
        arrowprops=dict(arrowstyle="->", color="#2ca02c", lw=0.9),
    )
    ax1.set_title("T1 · the leverage ratio, and the bifurcation",
                  fontsize=10.5, fontweight="bold", pad=8)
    _clean(ax1)

    # ---- Theorem 2: the weapon is the enmity ------------------------------
    weights, support, quota = [6.0, 3.0, 3.0, 2.0], [0.8] * 4, 9.0
    quotas = [i / 100 * 20.0 for i in range(201)]
    before = [action_probability(weights, support, q) for q in quotas]
    indep = [action_probability(*split_independent(weights, support, 0, 2), q) for q in quotas]
    estran = [action_probability(*split_estranged(weights, support, 0, 2), q) for q in quotas]
    ax2.plot(quotas, before, color="#333333", lw=2.4, label="the coalition undivided")
    ax2.plot(quotas, indep, color="#d62728", lw=2.0, ls="--",
             label="split, deciding independently")
    ax2.plot(quotas, estran, color="#2ca02c", lw=2.0, ls="-.",
             label="split, and set against each other")
    ax2.axvline(9.0, color="#999999", lw=1.0, ls=":")
    ax2.annotate(
        "the red curve is ABOVE the black:\nfragmentation alone made the\ncoalition MORE reliable",
        xy=(11.5, 0.85), xytext=(0.6, 0.09), fontsize=7.5, color="#d62728",
        arrowprops=dict(arrowstyle="->", color="#d62728", lw=0.9),
    )
    ax2.set_xlabel("quota the coalition must reach  $q$", fontsize=9.5)
    ax2.set_ylabel("probability the coalition can act", fontsize=9.5)
    ax2.set_xlim(0, 20)
    ax2.set_ylim(0, 1.05)
    ax2.set_title("T2 · the enmity is the weapon, not the division",
                  fontsize=10.5, fontweight="bold", pad=8)
    ax2.legend(fontsize=7.5, frameon=False, loc="upper right")
    _clean(ax2)

    # ---- Theorem 3: impunity -------------------------------------------------
    losses = [i / 100 * 3.0 for i in range(301)]
    for surplus, colour in [(5.0, "#2ca02c"), (4.0, "#ff7f0e"), (2.5, "#d62728")]:
        values = []
        for loss in losses:
            capacity = max_punishment(0.5, surplus, loss)
            values.append(attribution_threshold(0.50, capacity, 5))
        ax3.plot(losses, [min(v, 3.0) for v in values], color=colour, lw=2.2,
                 label=f"victims' surplus S = {surplus}")
    ax3.axhline(0.20, color="#333333", lw=1.2, ls="--")
    ax3.annotate("observed attribution, $\\varphi$ = 0.20:\nabove the curve the spoiler\nis deterred; below it, guilty\nand untouched",
                 xy=(0.35, 0.22), xytext=(0.9, 0.62), fontsize=7.5, color="#333333",
                 arrowprops=dict(arrowstyle="->", color="#333333", lw=0.9))
    ax3.set_xlabel("the damage the spoiler inflicts  $\\Lambda$", fontsize=9.5)
    ax3.set_ylabel("attribution threshold  $\\varphi^{\\dagger}$", fontsize=9.5)
    ax3.set_xlim(0, 3.0)
    ax3.set_ylim(0, 1.6)
    ax3.set_title("T3 · and the threshold rises with the damage",
                  fontsize=10.5, fontweight="bold", pad=8)
    ax3.legend(fontsize=8, frameon=False, loc="upper left")
    _clean(ax3)

    # ---- Theorem 4: impatience funds the proxy -----------------------------
    discounts = [i / 200 * 1.0 for i in range(1, 201)]
    supports = [blowback_optimum(discount=r) for r in discounts]
    ax4.plot(discounts, supports, color="#1f77b4", lw=2.4)
    for r, colour in [(0.02, "#2ca02c"), (0.50, "#d62728")]:
        s = blowback_optimum(discount=r)
        ax4.plot([r], [s], marker="o", markersize=9, color=colour,
                 markeredgecolor="white", markeredgewidth=1.4, zorder=4)
    ax4.set_xlim(0, 1.0)
    ax4.set_ylim(0.19, 0.30)
    ax4.annotate("patient sponsor:\nsupports less, and\nthe tail is smaller",
                 xy=(0.02, blowback_optimum(discount=0.02)), xytext=(0.10, 0.212),
                 fontsize=7.5, color="#2ca02c",
                 arrowprops=dict(arrowstyle="->", color="#2ca02c", lw=0.9))
    ax4.annotate("impatient sponsor:\nsupports harder, and\nbuys the tail",
                 xy=(0.50, blowback_optimum(discount=0.50)), xytext=(0.36, 0.270),
                 fontsize=7.5, color="#d62728",
                 arrowprops=dict(arrowstyle="->", color="#d62728", lw=0.9))
    ax4.set_xlabel("the sponsor's discount rate  $r$", fontsize=9.5)
    ax4.set_ylabel("optimal support  $s^*$", fontsize=9.5)
    ax4.set_title("T4 · blowback is the cost of capital",
                  fontsize=10.5, fontweight="bold", pad=8)
    _clean(ax4)

    # ---- Theorem 6: impunity attaches to success ---------------------------
    exposures = [i / 400 for i in range(401)]
    h = 0.40 * 0.50
    total_cost = 0.10
    profit = [0.40 * (1.0 - e) * 0.50 - total_cost for e in exposures]
    ax6.plot(exposures, profit, color="#1f77b4", lw=2.4, label="profit  $\\Pi(e)$")
    ax6.axhline(0.0, color="#333333", lw=1.0)
    bar = profitable_until(total_cost, h)
    ax6.axvline(bar, color="#1f77b4", lw=1.2, ls="--")
    # The two thresholds are annotated at staggered heights, because at these parameter
    # values they sit close enough to collide otherwise.
    for capacity, colour, label, height, text_x in [
            (0.15, "#2ca02c", "weak victims", 0.150, 0.62),
            (3.00, "#d62728", "strong victims", 0.048, 0.11),
        ]:
        dagger = deterred_from(h, capacity)
        ax6.axvline(dagger, color=colour, lw=1.4, ls=":")
        if bar > dagger:
            ax6.axvspan(dagger, bar, color=colour, alpha=0.10)
        ax6.annotate(f"{label}:  $e^\\dagger$ = {dagger:.2f}",
                     xy=(dagger, 0.0), xytext=(text_x, height), fontsize=7.5,
                     color=colour, arrowprops=dict(arrowstyle="->", color=colour, lw=0.9))
    ax6.annotate("$\\bar e$ = %.2f: past this the\nprize is not worth the operation" % bar,
                 xy=(bar, -0.06), xytext=(bar + 0.07, -0.125), fontsize=7.5, color="#1f77b4",
                 arrowprops=dict(arrowstyle="->", color="#1f77b4", lw=0.9))
    ax6.annotate("punishment only ever\ncloses this tail: the best\nspoilers sit at $e \\approx$ 0",
                 xy=(0.015, 0.098), xytext=(0.03, 0.128), fontsize=7.5, color="#444444",
                 arrowprops=dict(arrowstyle="->", color="#888888", lw=0.9))
    ax6.set_xlabel("the spoiler's exposure to the victims  $e$", fontsize=9.5)
    ax6.set_ylabel("profit", fontsize=9.5)
    ax6.set_xlim(0, 1.0)
    ax6.set_ylim(-0.15, 0.215)
    ax6.set_title("T6 · impunity attaches to SUCCESSFUL spoiling",
                  fontsize=10.5, fontweight="bold", pad=8)
    _clean(ax6)

    fig.suptitle("The third-party spoiler: divide and conquer as a strategy",
                 fontsize=14.5, fontweight="bold", y=0.925)
    _banner(fig, BANNER)
    _cite(fig,
          "Defining equations re-checked numerically before drawing; see verify_definitions(). "
          "The theorems themselves are asserted as invariants in multipolar_sim/src/spoiler.rs.")
    path = os.path.join(out_dir, "spoiler-1-theorems.png")
    fig.savefig(path, dpi=160)
    plt.close(fig)
    return path


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--out", default="figures", help="output directory for the PNGs")
    args = ap.parse_args()

    repo_root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    out_dir = args.out if os.path.isabs(args.out) else os.path.join(repo_root, args.out)
    os.makedirs(out_dir, exist_ok=True)

    print("defining equations, re-checked before drawing:")
    ok = True
    for name, passed, note in verify_definitions():
        if not passed:
            ok = False
        print(f"  {'OK      ' if passed else 'MISMATCH'}  {name:52} {note}")
    if not ok:
        print("\n  refusing to draw a figure from a model that fails its own definitions")
        return 1

    written = plot_theorems(out_dir)
    print(f"\nwrote {os.path.relpath(written, repo_root)}")

    print("\nthe quantities the document says have no measured counterpart (section 7):")
    for name, why in [
        ("capture rate g", "no estimate of what share of a war's loss a third party takes"),
        ("abuse hazard h(s)", "proxies that did NOT turn on their sponsor are unrecorded"),
        ("coordination cost kappa", "bears on P4 and has no direct measurement"),
        ("the counterfactual", "'this war would not have happened' is not observable"),
    ]:
        print(f"  {name:26} {why}")
    return 0


if __name__ == "__main__":
    sys.exit(main())


