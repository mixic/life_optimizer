# Two Strategies: Buying the Peace, or Buying the War

*Companion to `THIRD_PARTY_SPOILER.md` and `INFORMATION_WARFARE.md`. Code in
`multipolar_sim/src/strategies.rs`, mode `--strategies`, figure
`figures/strategies-1-comparison.png`, plotted by `tools/plot_strategies.py`.*

---

## 1. The question, and why it needed its own framework

`THIRD_PARTY_SPOILER.md` shows that a third party can profit from a conflict between two
others, and that it is rarely made to pay. That raises a comparison the spoiler document
does not itself make: **what would the same influence have bought if it had been spent on
cooperation instead?** Trade, investment and security guarantees on one side; manufactured
grievance, subsidised defection and resource capture on the other.

The two are seldom compared, because they are studied in different literatures and
measured in different units. A comparison needs one unit, one budget and one clock. This
document supplies them, applies the result to three fragile regions — the Near East, the
Balkans and the Carpathians — and then runs the same seeded world under each strategy to
see whether the answer survives contact with a simulator that was not built for it.

**Read the differences, never the levels.** Every margin, rent, retaliation figure and
instrument price below is *declared*. A peace margin is a property of a counterfactual
payoff matrix; nobody has measured one and nobody can, so the levels are not findings. The
comparison between the strategies is the finding, and §6 moves each assumption in turn to
say where it stops holding.

---

## 2. The regions, as declared

Three dyads, chosen because they are the ones the question names and because they differ
in exactly the two ways the framework says should matter: what a conflict there releases,
and whether the victims can answer.

| Region | Dyad | `margin` σ₀ | `rent` Λ | `retaliation` | `trade_benefit` | P(peace) |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| **Near East** | Eurasian (Iran) – Gulf | −0.35 | 1.10 | 0.25 | 0.85 | 0.242 |
| **Balkans** | Europe – Eurasian (Serbia / Kosovo) | +0.25 | 0.35 | 0.45 | 0.45 | 0.691 |
| **Carpathians** | Europe – Eurasian (Ukraine / Russia) | −0.55 | 0.95 | 0.70 | 0.70 | 0.136 |

`margin` is the dyad's peace margin `σ = cc − dc`, inherited from
`THIRD_PARTY_SPOILER.md` §1: conflict is the equilibrium when it is negative, so a negative
row is a region already at war and a positive one is a peace a strategist would be
defending. `retaliation` is the victims' capacity to identify and punish a sponsor, and it
is Theorem 6 of the spoiler document reused rather than re-derived: the spoiler's take is
scaled by `1 − κ·e`.

**A region here is a dyad and a set of numbers, not a place.** The Balkans row is not a
claim that Serbia and Kosovo are one thing with Bosnia; it is a claim that the Europe–
Eurasian relationship in that theatre has a peace margin near the knife edge, a small rent,
and moderate retaliation. Nothing in the model resolves below that granularity, and §7 says
so.

**Provenance.** `margin` is marked illustrative *necessarily*: a counterfactual 2×2 payoff
margin is not the kind of quantity that has a measurement. `rent` is marked illustrative
*contingently*: resource rents are reported figures, and anchoring this field needs a number
rather than a re-derivation. The framework is built so that anchoring it does not disturb
anything else.

---

## 3. The model: two instruments, one variable

Both strategies move the **same** `σ`, which is what makes them comparable at all:

* **Cooperation** — trade access, investment, security guarantees — raises `cc`, and
  therefore `σ`, at a price.
* **Spoiling** — manufactured grievance, subsidised defection — lowers `σ` at a price, and
  the spoiler is paid out of the conflict that follows.

Because `P(peace) + P(conflict) = 1`, every unit of `σ` one strategy buys is a unit the
other cannot have. The state is stochastic (`P(conflict) = Φ(−σ/s)`), and both instruments
have diminishing returns (`Δσ = g·√spend`), with the spoiling instrument declared cheaper
per unit of `σ` — that is `THIRD_PARTY_SPOILER.md` Corollary 1.1, and §6 shows the verdict
does **not** turn on it.

### 3.1 The asymmetry the comparison turns on

The two strategies do not have the same *time shape*, and this is the single most
consequential modelling choice in the framework:

* **Cooperation is a flow with upkeep.** A trade preference, a development programme, a
  security guarantee: each is paid for every year it is wanted and stops paying the year it
  stops being maintained. Its return continues exactly as long as the peace does — and the
  peace is what is being bought. Net annual margin: `B·P(peace) − x`.
* **Spoiling is a stock with a setup cost and a depleting prize.** The grievance is
  manufactured and the defection subsidised until `σ < 0`; after that the conflict sustains
  itself, because `σ < 0` *is* the equilibrium, so no upkeep is needed. But the prize is a
  region in conflict, and a region in conflict is being consumed: `Λ(t) = Λ₀·e^{−γt}`.

So the spoiler's prize is bounded by `Λ/γ` however long it waits, while the cooperator's
return grows with the horizon. That is where the framework's answer comes from.

### 3.2 Assumptions, listed

| # | Assumption | What it costs if wrong |
| --- | --- | --- |
| A1 | Cooperation requires annual upkeep and stops paying when the peace ends | Reverses the horizon result if cooperation is a one-off purchase |
| A2 | Conflict needs a setup payment, then sustains itself, but depletes the prize at `γ` | Reverses the horizon result if the prize is inexhaustible |
| A3 | Both instruments move `σ` with diminishing returns, `g·√spend` | Changes the optimal budget, not the sign of the comparison |
| A4 | `P(conflict) = Φ(−σ/s)`: the state is stochastic | None; it is what makes this a distribution |
| A5 | Retaliation discounts the spoiler's take but does not remove it | The punishment channel is Theorem 6, reused |
| A6 | Both strategies face the same `σ`, the same dispersion and the same horizon | The comparison is only meaningful under a common budget and clock |

**A1 and A2 are the load-bearing ones**, and §6 moves them.

---

## 4. Results

Measured at the default plan: intensity 0.60 of the optimal programme, discount 4% a year,
a 50-year horizon.

### 4.1 Region by region

| Region | cooperative spend | spoiling spend | P(peace) | P(war) | worth: coop | worth: spoil | better buy | critical rent |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | --- | ---: |
| Near East | 0.020 | 0.185 | 0.297 | 0.930 | 5.210 | **5.929** | SPOILING | 0.972 |
| Balkans | 0.005 | 0.183 | 0.720 | 0.606 | **7.132** | 0.870 | COOPERATION | 1.808 |
| Carpathians | 0.007 | 0.059 | 0.158 | 0.938 | 2.324 | **3.234** | SPOILING | 0.690 |

Totals: cooperation **14.666**, spoiling **10.033**. The framework prefers cooperation on
the declared parameters, by 4.633.

**The critical rent is the framework's cleanest single number**: the rent above which
spoiling beats cooperation in a region. The Near East (declared rent 1.10 against a critical
0.972) and the Carpathians (0.95 against 0.690) are above it and spoilable. The Balkans
(0.35 against 1.808) is far below it and not worth spoiling — **even though it is the only
one of the three where the peace already holds**, which is the point: a low-rent, defensible
region is safe for two independent reasons and the framework can tell them apart.

### 4.2 The horizon, and the result that replaced a guess

**There is a crossing at 20.9 years.** Below it the spoiler is worth more; above it the
cooperator is. The mechanism is §3.1: the spoiler's prize is capped by depletion while the
cooperator's is not.

The first version of this framework claimed something stronger and different — that the two
arms always cross, and that a *less* patient strategist has a *shorter* spoiling window.
The second half of that is false, and the numbers said so:

| discount rate | cooperation | spoiling | prefers | crossing |
| ---: | ---: | ---: | --- | --- |
| 0.00 | 32.822 | 13.097 | COOPERATION | 16.9 yr |
| 0.02 | 21.040 | 11.348 | COOPERATION | 18.6 yr |
| 0.04 | 14.666 | 10.033 | COOPERATION | 20.9 yr |
| 0.08 | 8.673 | 8.202 | COOPERATION | 33.4 yr |
| 0.15 | 5.028 | 6.327 | **SPOILING** | none |
| 0.30 | 2.845 | 4.453 | **SPOILING** | none |

Raising the discount rate pushes the crossing **later** (16.9 → 20.9 → 33.4 years), and past
a **critical discount rate of 0.092 a year** there is no crossing at any horizon and
spoiling dominates outright. The direction is the opposite of the intuition that a deferred
return suffers from impatience, and the reason is A1: in this structure it is the
**cooperator** who is paid in instalments and the **spoiler** who is paid early. Impatience
does not shorten the spoiler's window; it removes cooperation's advantage altogether.

The correction is recorded in `strategies.rs` at `critical_discount_rate`, and the test that
failed on the old claim is now the test for the new one.

### 4.3 Does a strategic region cancel the result?

The three regions do not point the same way, and the aggregate does not hide it: two of
three prefer spoiling, and the total prefers cooperation because the Balkans' cooperative
margin is large. That is worth stating rather than smoothing: **the framework's verdict is a
sum, not a consensus**, and a plan that spoils in two regions and cooperates in one is
exactly what the per-region table recommends.

---

## 5. The same world, run three ways

60 runs of 50 years, the same seed in all three arms, the region layer drawn from a stream
of its own so that switching it on cannot move the world's numbers. Every statistic is
differenced **run by run**, which is what gives the comparison an error bar: run *n* of every
arm faces the same shocks and the same region realisations, so the difference is taken with
the common noise already cancelled.

| statistic | spoiling − cooperation | std err | t |
| --- | ---: | ---: | ---: |
| cooperation index | +0.0279 | 0.0032 | +8.8 |
| conflict-trap years | −0.0393 | 0.0082 | −4.8 |
| Pareto-efficiency loss | −0.0375 | 0.0042 | −8.9 |
| pension security index | +0.0155 | 0.0018 | +8.8 |
| **regions in conflict** | **+0.651** | 0.0124 | **+52.3** |
| **rent captured** | **+10.444** | 0.0367 | **+284.4** |

### 5.1 What these aggregates do not show, and why that is not a result about strategies

Read the cooperation row before believing it. In this simulator, accumulated tension erodes
the payoff of mutual competition (`conflict_wear`), so a tenser system leaves the pure
dilemma *sooner* and the measured cooperation rate can **rise**. Both arms add conflict
relative to a baseline that models none, so both push that index up — and the spoiling arm
pushes it up **further** than the cooperative one. That is a property of the tension channel
this simulator happens to have, the same one `--scenarios` documents for a crisis shock, and
**not** a finding that spoiling produces cooperation.

The same caveat covers conflict-trap years, Pareto loss and the pension index, because all
four are read off the same tension-eroded payoff matrix. So the two statistics that actually
discriminate here are:

* **`regions in conflict`**, which counts the modelled mechanism directly: 1.82 per year under
  cooperation against 2.47 under spoiling, out of three declared regions.
* **`rent captured`**, which is zero by construction unless the arm is the spoiling one — a
  region releases its rent whoever is standing there, but capturing it is the strategy.

The world cannot adjudicate the strategy question on the aggregate, and this mode does not
pretend it does. The analytic comparison in §4 is where the question is answered; the Monte
Carlo is where the answer is checked against a world that was not built to make it look good.

---

## 6. Where the verdict stops holding

Each row moves one declared assumption and re-derives the whole comparison.

| assumption varied | values | verdict |
| --- | --- | --- |
| **discount rate** | 0.00 → 0.30 | DECISIVE. Cooperation to 0.08; spoiling from 0.15 |
| **depletion rate** | 0.02 → 1.00 | DECISIVE. Spoiling at 0.02–0.06; cooperation from 0.12 |
| **victims' retaliation** | 0.00 → 2.00 | DECISIVE. Spoiling at 0.00; cooperation from 0.35 |
| **spoiling price (gain)** | 0.30 → 1.80 | **NOT DECISIVE.** Cooperation across the whole range |

Three of the four move the verdict, and one does not — and the one that does not is the
assumption the framework was most likely to be accused of stacking. Making the spoiling
instrument 40% cheaper per unit of `σ` (gain 0.30 against the cooperative 0.60) still leaves
cooperation ahead on the declared regions; making it *three times* cheaper (1.80) narrows the
margin from 5.751 to 3.771 and never flips it. **The framework's verdict does not rest on the
claim that propaganda is cheaper than investment.**

What it does rest on:

* **Depletion** (A2). At `γ = 0.02` spoiling wins with no crossing at any horizon; by
  `γ = 0.12` cooperation wins. The spoiler's whole case is that the prize is bounded, and how
  fast it is consumed is the parameter that decides whether the case holds.
* **Retaliation** (A5). At zero punishment capacity spoiling wins; from 0.35 upward
  cooperation does. This is the framework's answer to *why the pattern looks the way it
  does*: the same strategy that loses against a region that can answer wins against one that
  cannot, so the geography is partly explained by who can retaliate rather than by who has
  resources.
* **Patience** (§4.2). Impatience locks the spoiler in.

---

## 7. What is not established

1. **No level here is a finding.** Six declared numbers per region drive every figure. The
   framework is a way of pricing two strategies against each other, not a measurement of
   either.
2. **Nothing is calibrated to any real conflict.** The Near East row is not a claim about
   Iran, the Gulf, Israel or Palestine. Reading it as one would be reading the assumptions
   back out as though they were evidence.
3. **A region is not a place.** Three dyads at bloc granularity cannot represent factions,
   internal politics, or the difference between the Sahel and the Gulf.
4. **The world simulation cannot adjudicate the question**, for the documented reason in
   §5.1. It is a check, not an arbiter.
5. **The horizon result depends on A1 and A2 jointly.** If cooperation can be bought once, or
   if a conflict can be held open without consuming the prize, the crossing moves or vanishes.
   Both assumptions are declared rather than derived, and §6 moves them.
6. **The payout mechanism is absent from the world model.** The regional rent is priced
   analytically and reported per arm, but the world simulation has no channel by which a
   captured rent becomes power. So the Monte Carlo cannot confirm the analytic totals; it can
   only confirm the mechanism counts.
7. **What would change this.** A measured or bounded peace margin for any real dyad, a
   published estimate of the capturable share of a region's resource rent, or evidence on how
   fast a conflict actually consumes its own prize. The first is probably impossible; the
   second and third are not.

---

## 8. Reproduction

```sh
cargo run -p multipolar_sim -- --strategies --runs 60 --export out/strategies
cargo run -p multipolar_sim -- --strategies --strategy-sensitivity
python tools/plot_strategies.py out/strategies --out figures/strategies-1-comparison.png
```

The strategy layer is **off by default** and a disabled layer changes nothing — not a payoff,
not a random draw. A golden test pins the world's own stream, and the module's tests pin the
distribution, the annuity, the complementarity of the two instruments at a common margin, the
interiority of both optima, the crossing and its disappearance, the direction of the
impatience effect, the retaliation comparative static, and that an unresolvable region is
skipped without desynchronising the comparison.

`tools/plot_strategies.py` refuses to draw unless its five consistency checks pass: the three
arms paired run for run, all probabilities inside the unit interval, no negative spends, the
horizon curve agreeing with its own difference column, and the crossing actually present when
the caption says it is. It caught a real defect on first run — the export was rounded finely
enough for the check to fail by 1e-6 — and the export was fixed rather than the check.
