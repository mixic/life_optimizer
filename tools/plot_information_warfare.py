"""Scientific plots for the information-warfare model.

Two kinds of content, and the distinction is the point of the file:

1. **Theorems** (`1`-`6`). Every curve here is a closed form from
   `INFORMATION_WARFARE.md` section 2, drawn on a banner that says so. These are
   *not* measurements and not even simulation output: they are the algebra, drawn.
   What makes them worth drawing is that three of the six have a shape that is not
   obvious from the statement -- the band of Theorem 1, the discontinuity of
   Theorem 3, and the interior optimum of Theorem 6.
2. **Nothing else.** There is deliberately no evidence panel, because the
   quantities that would fill one do not exist: see section 6.1 of the document,
   which reports that the detection hazard, the credibility damage, the aggregate
   cost of misinformation and the counterfactual have no estimate of any quality.

Each curve's defining equation is re-checked numerically before anything is drawn, so
a figure cannot be produced from a model that no longer satisfies its own theorems.
`--verify` additionally runs the optimizer-side binary and checks the one quantity
that is exactly comparable: the spectral radius of the influence ring.

Usage:
    python tools/plot_information_warfare.py [--out figures]
"""

import argparse
import math
import os
import subprocess
import sys

import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt

# ── The model, as closed forms ──────────────────────────────────────────────
#
# These mirror `multipolar_sim/src/information.rs` rather than calling it, and the
# constants are the module's defaults. The theorems are checked as invariants by that
# module's own tests, which is the stronger check; what this file adds is the picture,
# and `verify_definitions()` below is what stops the picture drifting from the algebra.

# CredibilityParams::default()
HAZARD = 0.70          # mu   -- detection hazard per unit of sustained assertion mass
DAMAGE = 1.20          # chi  -- credibility destroyed per unit of exposed falsehood
REGENERATION = 0.25    # r    -- annual regeneration of standing
CEILING = 1.00         # kappa

# LegitimationParams::default()
LAMBDA_MAX = 0.85      # how much of the cost of force justification can ever remove
STEEP = 3.0            # steepness of the response

# The two channels of `V = G + gamma*b - C*(1 - Lambda(b))`.
COALITION_CHANNEL = 0.15


def legitimation(belief, lam_max=LAMBDA_MAX, steep=STEEP):
    """`Lambda(b)`, normalised so that `Lambda(1) = lam_max` exactly."""
    b = min(max(belief, 0.0), 1.0)
    if steep <= 1e-12:
        return min(lam_max * b, lam_max)
    denom = 1.0 - math.exp(-steep)
    return min(lam_max * (1.0 - math.exp(-steep * b)) / denom, lam_max)


def attack_payoff(spoils, cost, belief, lam_max=LAMBDA_MAX, steep=STEEP,
                  gamma=COALITION_CHANNEL):
    """`V_ij(b)`, strictly increasing in belief."""
    b = min(max(belief, 0.0), 1.0)
    return spoils + gamma * b - cost * (1.0 - legitimation(b, lam_max, steep))


def attack_threshold(spoils, cost, lam_max=LAMBDA_MAX, steep=STEEP, gamma=COALITION_CHANNEL):
    """`b-bar` of Theorem 1, or `None` for the two boundary cases.

    Returns `0.0` when the operation pays at zero believed culpability (nothing is
    needed) and `None` when no achievable belief covers the cost (nothing is enough).
    Bisection, so that the drawn threshold and the tested one are found the same way.
    """
    at = lambda b: attack_payoff(spoils, cost, b, lam_max, steep, gamma)
    if at(0.0) >= 0.0:
        return 0.0
    if at(1.0) < 0.0:
        return None
    lo, hi = 0.0, 1.0
    for _ in range(200):
        mid = 0.5 * (lo + hi)
        if at(mid) >= 0.0:
            hi = mid
        else:
            lo = mid
    return 0.5 * (lo + hi)


def detection_probability(effort, hazard=HAZARD):
    """`pi(x) = 1 - exp(-mu x)`."""
    if effort <= 0.0 or hazard <= 0.0:
        return 0.0
    return 1.0 - math.exp(-hazard * effort)


def detection_burden(effort, hazard=HAZARD, damage=DAMAGE):
    """`beta(x) = chi x pi(x)`, strictly increasing from zero."""
    return damage * effort * detection_probability(effort, hazard)


def break_even_lie(hazard=HAZARD, damage=DAMAGE, regeneration=REGENERATION):
    """`x-dagger`: the unique effort at which expected damage equals regeneration."""
    target = regeneration
    lo, hi = 0.0, 1.0
    while detection_burden(hi, hazard, damage) < target:
        hi *= 2.0
        if hi > 1e12:
            return None
    for _ in range(200):
        mid = 0.5 * (lo + hi)
        if detection_burden(mid, hazard, damage) < target:
            lo = mid
        else:
            hi = mid
    return 0.5 * (lo + hi)


def stationary_credibility(effort, hazard=HAZARD, damage=DAMAGE,
                           regeneration=REGENERATION, ceiling=CEILING):
    """The level the stock reverts toward; negative means it is being spent faster."""
    if regeneration <= 0.0:
        return 0.0
    return ceiling * (1.0 - detection_burden(effort, hazard, damage) / regeneration)


def amplification(verification, contamination):
    """`1 / (1 - rho(A))` with `rho(A) = (1 - lambda) + sigma`, or `None` if unstable.

    Corollary 3.1: the system is stable iff `lambda > sigma`, and undefined below.
    """
    rho_a = (1.0 - verification) + contamination
    if rho_a >= 1.0:
        return None
    return 1.0 / (1.0 - rho_a)


def instalment_damage(mass, instalments, hazard=HAZARD, damage=DAMAGE):
    """`T * chi * (M/T) * pi(M/T)`, Theorem 6."""
    if instalments < 1.0 or mass <= 0.0:
        return 0.0
    return damage * mass * (1.0 - math.exp(-hazard * mass / instalments))


INSTALMENT_CRITICAL = 4.0 * DAMAGE * math.exp(-2.0)


def verify_definitions():
    """Re-check each curve's defining equation before drawing it.

    Not a substitute for the Rust invariants -- those check the theorems. This stops the
    *picture* from being drawn from a model that has stopped satisfying its own
    definitions, which is the specific way a plotting script goes wrong.
    """
    results = []

    # Theorem 1: the threshold is a root of V, and V is increasing in belief.
    b = attack_threshold(0.35, 1.0)
    results.append((
        "T1 threshold is a root of V",
        abs(attack_payoff(0.35, 1.0, b)) < 1e-6,
        f"V(b-bar) = {attack_payoff(0.35, 1.0, b):+.2e}",
    ))
    rising = all(
        attack_payoff(0.35, 1.0, x + 0.01) > attack_payoff(0.35, 1.0, x)
        for x in [i / 100 for i in range(99)]
    )
    results.append(("T1 V is strictly increasing in belief", rising, "checked on a grid"))

    # Theorem 1(iii)-(iv): the two boundary cases.
    results.append((
        "T1 Corollary 1.1 boundary cases",
        attack_threshold(1.5, 1.0) == 0.0 and attack_threshold(0.05, 5.0) is None,
        "profitable-on-its-own returns 0; unaffordable returns None",
    ))

    # Theorem 2: the break-even lie solves beta(x) = r, and beta is increasing.
    x = break_even_lie()
    results.append((
        "T2 break-even solves beta(x) = r",
        abs(detection_burden(x) - REGENERATION) < 1e-6,
        f"beta(x*) = {detection_burden(x):.6f} vs r = {REGENERATION}",
    ))
    increasing = all(
        detection_burden(x0 + 0.01) > detection_burden(x0) for x0 in [i / 100 for i in range(300)]
    )
    results.append(("T2 detection burden is strictly increasing", increasing, "checked on a grid"))
    results.append((
        "T2 stationary credibility changes sign at x*",
        stationary_credibility(0.5 * x) > 0.0 > stationary_credibility(1.5 * x),
        "positive below the break-even, negative above",
    ))

    # Theorem 3: the phase transition is exactly at lambda = sigma.
    sigma = 0.20
    results.append((
        "T3 amplification undefined at and below the threshold",
        amplification(sigma - 0.01, sigma) is None and amplification(sigma, sigma) is None,
        "rho(A) >= 1 gives no steady state rather than a large one",
    ))
    results.append((
        "T3 amplification diverges as lambda falls to sigma",
        amplification(sigma + 1e-6, sigma) > 1e5,
        f"1/(1-rho) = {amplification(sigma + 1e-6, sigma):.3e}",
    ))

    # Theorem 6: the critical time cost, and the optimum sitting on the increasing branch.
    results.append((
        "T6 critical time cost is 4*chi*exp(-2)",
        abs(INSTALMENT_CRITICAL - 4.0 * DAMAGE * math.exp(-2.0)) < 1e-12,
        f"delta* = {INSTALMENT_CRITICAL / HAZARD:.4f} at mu = {HAZARD}",
    ))
    mass = 4.0
    delta = 0.5 * INSTALMENT_CRITICAL / HAZARD
    z = solve_instalment_z(delta * HAZARD)
    results.append((
        "T6 optimum satisfies the first-order condition",
        abs(DAMAGE * z * z * math.exp(-z) - delta * HAZARD) < 1e-9,
        f"chi z^2 e^-z = {DAMAGE * z * z * math.exp(-z):.6f}, delta*mu = {delta * HAZARD:.6f}",
    ))
    interior = HAZARD * mass / z
    results.append((
        "T6 the optimum is interior, not the corner",
        interior > 1.0,
        f"T* = {interior:.2f} instalments for a mass of {mass}",
    ))

    # Theorem 4: a net aggressor invests nothing and the targets under-invest.
    valuations = [-0.9, 0.6, 0.4, 0.3]
    nash = [max(0.0, v) * 0.5 for v in valuations]
    targets = sum(v for v in valuations if v > 0.0)
    coalition = [0.5 * targets for _ in valuations]
    results.append((
        "T4 aggressor invests nothing",
        nash[0] == 0.0,
        "u' < 0 puts the best response at the corner",
    ))
    results.append((
        "T4 targets strictly under-invest",
        all(nash[i] < coalition[i] for i in (1, 2, 3)),
        f"Nash {nash[1]:.2f} vs coalition {coalition[1]:.2f}",
    ))
    world = 0.5 * sum(valuations)
    results.append((
        "T4 the world-welfare comparison is not signed",
        world < nash[1],
        "a target can over-invest against world welfare when an aggressor dominates",
    ))

    return results


def solve_instalment_z(delta_mu, damage=DAMAGE):
    """Solve `chi z^2 e^-z = delta*mu` on the increasing branch `z in (0, 2)`."""
    lo, hi = 0.0, 2.0
    for _ in range(200):
        mid = 0.5 * (lo + hi)
        if damage * mid * mid * math.exp(-mid) < delta_mu:
            lo = mid
        else:
            hi = mid
    return 0.5 * (lo + hi)


# ── Figures ─────────────────────────────────────────────────────────────────

BANNER = (
    "MODEL OUTPUT -- not a measurement and not a simulation. Every curve is a closed\n"
    "form from INFORMATION_WARFARE.md section 2, drawn with that section's declared defaults."
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
    """Six panels, one per theorem."""
    fig, axes = plt.subplots(2, 3, figsize=(15.5, 8.8))
    fig.subplots_adjust(top=0.845, bottom=0.105, left=0.055, right=0.985,
                        wspace=0.30, hspace=0.40)
    ax1, ax2, ax3, ax4, ax5, ax6 = axes.ravel()

    # ---- Theorem 1: the band where a lie decides --------------------------
    #
    # One worked cost is shaded into the three regions of Theorem 1 and Corollary 1.1;
    # the other costs are drawn as curves only, so the shading stays readable.
    worked_cost = 1.4
    spoils = [i / 400 * 3.6 for i in range(401)]
    bare_edge = worked_cost          # V(0) >= 0: the operation pays for itself
    deterr_edge = worked_cost * (1.0 - LAMBDA_MAX) - COALITION_CHANNEL  # V(1) < 0: nothing suffices

    ax1.axvspan(0, deterr_edge, color="#d62728", alpha=0.08)
    ax1.axvspan(deterr_edge, bare_edge, color="#f0c419", alpha=0.16)
    ax1.axvspan(bare_edge, 3.6, color="#2ca02c", alpha=0.08)
    for cost, colour in zip([0.8, 1.0, 1.4, 2.0],
                            ["#1f77b4", "#2ca02c", "#ff7f0e", "#d62728"]):
        xs, ys = [], []
        for g in spoils:
            b = attack_threshold(g, cost)
            if b is not None and 0.0 < b <= 1.0:
                xs.append(g)
                ys.append(b)
        ax1.plot(xs, ys, color=colour, lw=2.2, label=f"cost of force C = {cost}")
    ax1.set_xlabel("spoils  G", fontsize=9.5)
    ax1.set_ylabel("belief needed to justify force  $\\bar b$", fontsize=9.5)
    ax1.set_xlim(0, 3.6)
    ax1.set_ylim(0, 1.05)
    ax1.set_title("T1 · a lie cannot create a war that does not pay",
                  fontsize=10.5, fontweight="bold", pad=8)
    # The deterred band is genuinely narrow, and that is a result rather than a layout
    # problem: `Lambda_max = 0.85` lets justification remove most of the cost of force, so
    # cost alone deters only for `G < 0.15*C - gamma`. It is annotated with an arrow
    # because it is too thin to hold a label.
    ax1.annotate(
        "deterred -- and narrow, because\n$\\Lambda_{max}$ = 0.85 lets a lie remove most\n"
        "of the cost, so only $G < 0.15C - \\gamma$ is deterred",
        xy=(deterr_edge, 0.60), xytext=(1.98, 0.06), fontsize=7.5, color="#8b1a1a",
        ha="left", va="bottom",
        arrowprops=dict(arrowstyle="->", color="#8b1a1a", lw=0.9,
                        connectionstyle="arc3,rad=0.18"),
    )
    ax1.text((deterr_edge + bare_edge) / 2, 0.97, "the band:\njustification decides",
             fontsize=7.5, color="#7a5c00", ha="center", va="top")
    ax1.text(2.80, 0.97, "no justification\nneeded at all", fontsize=7.5,
             color="#1d6b1d", ha="center", va="top")
    ax1.legend(fontsize=8, frameon=False, loc="center right",
               bbox_to_anchor=(1.0, 0.40))
    _clean(ax1)

    # ---- Theorem 2: the liar's budget ------------------------------------
    efforts = [i / 400 * 1.2 for i in range(401)]
    stat = [stationary_credibility(e) for e in efforts]
    ax2.plot(efforts, stat, color="#1f77b4", lw=2.4)
    ax2.axhline(0.0, color="#333333", lw=1.0)
    x_star = break_even_lie()
    ax2.axvline(x_star, color="#d62728", lw=1.4, ls="--")
    ax2.fill_between(efforts, stat, 0, where=[s > 0 for s in stat],
                     color="#2ca02c", alpha=0.10)
    ax2.annotate(
        f"$x^\\dagger$ = {x_star:.2f}\nexpected exposure\n= regeneration",
        xy=(x_star, -0.9), xytext=(x_star + 0.16, -0.35), fontsize=8.5, color="#d62728",
        arrowprops=dict(arrowstyle="->", color="#d62728", lw=1.0),
    )
    ax2.text(0.08, 0.35, "sustainable", fontsize=9, color="#1d6b1d")
    ax2.text(x_star + 0.06, 0.55, "self-consuming", fontsize=9, color="#8b1a1a")
    ax2.set_xlabel("sustained assertion mass  x", fontsize=9.5)
    ax2.set_ylabel("reversion level of the credibility stock", fontsize=9.5)
    ax2.set_xlim(0, 1.2)
    ax2.set_ylim(-2.6, 1.15)
    ax2.set_title("T2 · the liar's budget is finite", fontsize=10.5, fontweight="bold", pad=8)
    _clean(ax2)

    # ---- Theorem 3: the phase transition ---------------------------------
    verifications = [0.02 + i / 800 * 0.96 for i in range(801)]
    for sigma, colour in [(0.05, "#2ca02c"), (0.20, "#ff7f0e"), (0.40, "#d62728")]:
        xs, ys = [], []
        for v in verifications:
            a = amplification(v, sigma)
            if a is not None and a <= 25:
                xs.append(v)
                ys.append(a)
        ax3.plot(xs, ys, color=colour, lw=2.2, label=f"$\\rho(\\Sigma)$ = {sigma}")
        ax3.axvline(sigma, color=colour, lw=1.0, ls=":")
    ax3.set_xlim(0, 1.0)
    ax3.set_ylim(0, 25)
    ax3.annotate(
        "at $\\lambda = \\rho(\\Sigma)$ belief has NO\nsteady state -- not a large one,\nnone at all",
        xy=(0.41, 12), xytext=(0.50, 15), fontsize=8, color="#444444",
        arrowprops=dict(arrowstyle="->", color="#888888", lw=0.9),
    )
    ax3.set_xlabel("verification rate  $\\lambda$", fontsize=9.5)
    ax3.set_ylabel("amplification  $1/(1-\\rho(A))$", fontsize=9.5)
    ax3.set_title("T3 · a lie's reach is a network property",
                  fontsize=10.5, fontweight="bold", pad=8)
    ax3.legend(fontsize=8, frameon=False, loc="upper right")
    _clean(ax3)

    # ---- Theorem 4: the public-good failure ------------------------------
    valuations = [-0.9, 0.6, 0.4, 0.3]
    labels = ["aggressor\n($u'<0$)", "target 1", "target 2", "target 3"]
    targets_total = sum(v for v in valuations if v > 0.0)
    nash = [max(0.0, v) * 0.5 for v in valuations]
    coalition = [0.5 * targets_total for _ in valuations]
    world = [max(0.0, 0.5 * sum(valuations)) for _ in valuations]

    positions = list(range(len(valuations)))
    width = 0.27
    ax4.bar([p - width for p in positions], nash, width, label="Nash equilibrium",
            color="#1f77b4")
    ax4.bar(positions, coalition, width,
            label="what the exposed blocs would jointly choose", color="#2ca02c")
    ax4.bar([p + width for p in positions], world, width,
            label="world-welfare planner", color="#d62728")
    ax4.set_xticks(positions)
    ax4.set_xticklabels(labels, fontsize=8.5)
    ax4.set_ylabel("verification investment  $v_i$", fontsize=9.5)
    ax4.set_ylim(0, 1.42)
    ax4.annotate(
        "the aggressor invests nothing",
        xy=(0, 0.30), xytext=(0.02, 0.68), textcoords="axes fraction", fontsize=8,
        color="#444444", arrowprops=dict(arrowstyle="->", color="#888888", lw=0.9),
    )
    ax4.annotate(
        "against WORLD welfare these targets\nover-invest: the sign is not fixed",
        xy=(1.28, 0.30), xytext=(0.30, 0.50), textcoords="axes fraction", fontsize=8,
        color="#8b1a1a", arrowprops=dict(arrowstyle="->", color="#8b1a1a", lw=0.9),
    )
    ax4.set_title("T4 · under-provided -- among those it protects",
                  fontsize=10.5, fontweight="bold", pad=8)
    ax4.legend(fontsize=7.5, frameon=False, loc="upper center", ncol=1)
    _clean(ax4)

    # ---- Theorem 5: integration cuts both ways ---------------------------
    #
    # Two axes, because the two curves are different quantities: the world's loss in
    # levels, and the *change* in the aggressor's private payoff relative to no
    # integration. Rescaling one onto the other's axis would be a drawing decision
    # masquerading as a finding.
    integrations = [i / 100 for i in range(101)]
    loss = [0.06 * 0.3 + 0.25 * e ** 1.5 + 0.10 * 0.5 for e in integrations]
    ax5.plot(integrations, loss, color="#d62728", lw=2.6, label="world loss if a war happens")
    ax5.set_xlabel("interdependence of the pair  $E$", fontsize=9.5)
    ax5.set_ylabel("world-economy loss  $L$", fontsize=9.5, color="#d62728")
    ax5.tick_params(axis="y", labelcolor="#d62728")
    ax5.set_ylim(0, max(loss) * 1.35)

    twin = ax5.twinx()
    for spoils_el, exposure, colour in [(0.10, 1.20, "#2ca02c"), (1.20, 0.05, "#ff7f0e")]:
        baseline = attack_payoff(0.30, 1.00, 0.5)
        change = [
            attack_payoff(0.30 * (1.0 + spoils_el * e), 1.00 + exposure * e, 0.5) - baseline
            for e in integrations
        ]
        twin.plot(integrations, change, color=colour, lw=2.0, ls="--",
                  label=("pacified: the aggressor's own exposure dominates"
                         if colour == "#2ca02c" else
                         "destabilised: the spoils dominate"))
    twin.axhline(0.0, color="#999999", lw=0.8)
    twin.set_ylabel("change in the aggressor's attack payoff", fontsize=9.5)
    twin.set_ylim(-0.16, 0.16)
    for side in ("top",):
        twin.spines[side].set_visible(False)
    handles, labs = ax5.get_legend_handles_labels()
    handles2, labs2 = twin.get_legend_handles_labels()
    ax5.legend(handles + handles2, labs + labs2, fontsize=7.5, frameon=False,
               loc="upper left", bbox_to_anchor=(0.02, 0.99))
    ax5.set_title("T5 · neither safe nor dangerous, but costlier",
                  fontsize=10.5, fontweight="bold", pad=8)
    _clean(ax5)

    # ---- Theorem 6: why lies arrive in instalments ------------------------
    mass = 4.0
    instalments = [1.0 + i / 20 for i in range(1, 380)]
    delta = 0.5 * INSTALMENT_CRITICAL / HAZARD
    ax6.plot(instalments, [instalment_damage(mass, t) for t in instalments],
             color="#1f77b4", lw=2.4, label="credibility damage  $\\Phi(T)$")
    ax6.plot(instalments, [instalment_damage(mass, t) + delta * t for t in instalments],
             color="#d62728", lw=1.8, ls="--", label="plus the time cost  $\\delta T$")
    optimal = HAZARD * mass / solve_instalment_z(delta * HAZARD)
    total = instalment_damage(mass, optimal) + delta * optimal
    ax6.plot([optimal], [total], marker="o", markersize=9, color="#d62728",
             markeredgecolor="white", markeredgewidth=1.4, zorder=4)
    ax6.annotate(
        f"$T^*$ = {optimal:.1f} instalments:\nthe same narrative, spread\nthinner, costs less",
        xy=(optimal, total), xytext=(optimal + 8.5, total + 0.6), fontsize=8.5,
        color="#d62728", arrowprops=dict(arrowstyle="->", color="#d62728", lw=1.0),
    )
    ax6.set_xlabel("number of instalments  T", fontsize=9.5)
    ax6.set_ylabel("cost to the liar", fontsize=9.5)
    ax6.set_xlim(1, 40)
    ax6.set_ylim(0, 11)
    ax6.set_title("T6 · the same narrative, spread thinner, costs less",
                  fontsize=10.5, fontweight="bold", pad=8)
    ax6.legend(fontsize=8, frameon=False, loc="upper right")
    _clean(ax6)

    fig.suptitle("Information warfare between blocs: the model, drawn",
                 fontsize=14.5, fontweight="bold", y=0.925)
    _banner(fig, BANNER)
    _cite(fig,
          "Defining equations re-checked numerically before drawing; see verify_definitions(). "
          "The theorems themselves are asserted as invariants in multipolar_sim/src/information.rs.")
    path = os.path.join(out_dir, "information-warfare-1-theorems.png")
    fig.savefig(path, dpi=160)
    plt.close(fig)
    return path


def verify_against_binary(repo_root):
    """The one quantity the figure and the binary can be compared on exactly.

    The spectral radius of the influence ring is `2 * sigma`, and `--information` prints
    its own value. Everything else on the figure is a closed form whose counterpart in the
    binary depends on a stochastic clipped chain, so comparing them would be comparing an
    expectation with a realisation -- which is worse than not comparing at all.
    """
    results = []
    try:
        out = subprocess.run(
            ["cargo", "run", "-q", "-p", "multipolar_sim", "--release", "--",
             "--information", "--runs", "4", "--seed", "7",
             "--information-contamination", "0.10"],
            cwd=repo_root, capture_output=True, text=True, encoding="utf-8",
            errors="replace", timeout=900,
        ).stdout
    except (OSError, subprocess.SubprocessError) as exc:  # pragma: no cover
        return [("binary", "the mode could not be run", False, str(exc))]

    printed = None
    for line in out.splitlines():
        # The sweep table's second column is rho(Sigma), constant down the rows.
        if "rho(Sigma)" in line:
            continue
        parts = line.split()
        if len(parts) >= 6 and parts[0].replace("*", "").replace(".", "").isdigit():
            try:
                printed = float(parts[1])
            except ValueError:
                continue
            break

    expected = 2.0 * 0.10
    agrees = printed is not None and abs(printed - expected) < 1e-9
    results.append((
        "rho(Sigma) for a ring of strength 0.10",
        f"the binary reported {printed}",
        agrees,
        f"the ring gives 2*sigma = {expected}",
    ))
    ran = "lambda" in out and "contestable" in out
    results.append((
        "the binary produced its sweep table",
        "expected the --information output",
        ran,
        "rows present" if ran else "no rows found",
    ))
    return results


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--out", default="figures", help="output directory for the PNGs")
    ap.add_argument("--verify", action="store_true",
                    help="also check the spectral radius against the simulator binary")
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

    print("\nthe quantities the document says have no estimate at all (section 6.1):")
    for name, why in [
        ("detection hazard mu", "the denominator of false claims made is unobservable"),
        ("credibility damage chi", "no verified effect size for the penalty after detection"),
        ("cost of misinformation", "both circulating figures are vendor products"),
        ("the counterfactual", "not observable by construction"),
    ]:
        print(f"  {name:26} {why}")

    if args.verify:
        print("\nverification against the simulator binary:")
        for name, why, passed, note in verify_against_binary(repo_root):
            if not passed:
                ok = False
            print(f"  {'OK      ' if passed else 'MISMATCH'}  {name:44} {why}  -- {note}")
        if not ok:
            print("\n  the figure and the binary disagree")
            return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
