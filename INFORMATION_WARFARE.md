# Information Warfare Between Blocs

A formal model of legitimation, credibility as a depletable stock, and why the
verification that would prevent a war is a public good nobody buys.

**Status of this document.** Every result below is a theorem *about the model in §2*,
proved from the assumptions listed in §2.7. None of them is a measurement, and none of
them is evidence about any particular war. §1 says exactly what a proof of this kind can
and cannot establish, and §5 keeps the historical record separate from the mathematics.
The repository's rule applies unchanged: parameters are illustrative unless a source is
named, and no number appears here without a provenance.

---

## 1. What a proof can and cannot establish here

This document proves things. That is worth being careful about, because the phrase
"mathematically proven" is doing a lot of work in public argument and most of what it is
used for is not true.

A proof here establishes that **if** the world had the structure in §2, **then** certain
consequences follow necessarily — that a threshold exists, that credibility is monotone
in sustained lying, that a network amplifies a lie by exactly `1/(1−ρ)`, that verification
is under-provided. Those are implications of a construction. They are not facts about
Iraq in 2003 or Yugoslavia in 1999.

What the mathematics can do, and does here:

- **Force consistency.** A verbal theory of propaganda can hold three claims that cannot
  all be true at once. A model cannot: once the objects are defined, the consequences are
  not negotiable.
- **Separate the mechanism from the arithmetic.** Theorem 2 says credibility is
  *monotone* in sustained lying. That is a structural claim and it survives any
  parameter choice. The *size* of the lie budget depends entirely on invented constants.
- **Name the conditions.** The most useful outputs below are not magnitudes but
  conditions: the deterred band of Theorem 1 where no lie can work at all; the threshold
  `λ = ρ(Σ)` of Theorem 3 where an information war becomes uncontrollable; the time-cost
  boundary of Theorem 6.
- **Make the theory falsifiable.** §4 states what would refute it. A model that cannot be
  wrong is not doing any work.

What it cannot do:

- **It cannot identify a liar.** The model is symmetric by construction: every bloc in it
  has the same instruments. If the model had a villain, the proof would be worthless.
- **It cannot settle a historical dispute.** Whether a particular claim was a lie, an
  error, or an honest reading of bad intelligence is a documentary question. §5 reports
  what the record establishes and, where the record does not settle it, says so.
- **It cannot calibrate itself.** The constants in §6 have no measured counterparts, and
  §6 says why: nobody can observe the counterfactual in which the same bloc told the truth.

The honest summary: **the theorems are true, the parameters are invented, and the mapping
from one to the other is the only part that could be wrong.** This is the same division
`AI_AS_CATALYST.md` draws between published evidence and model output, and it is drawn
here for the same reason.

---

## 2. The model

### 2.1 Actors and timing

There are `n` blocs, `N = {1, …, n}`, and discrete years `t = 0, 1, 2, …`. Each year:

1. Every bloc chooses an **assertion flow**: for each other bloc `j`, a mass of
   unsupported claims `x_ij(t) ≥ 0` aimed at `j`. Supported claims are free, harmless,
   and not modelled — they need no credibility because they survive checking.
2. The audience updates **beliefs** about each bloc (§2.3).
3. Detection resolves and **credibility** updates (§2.4).
4. Each bloc `i` decides whether to use force against each `j`, comparing the attack
   payoff to zero (§2.5).
5. Power, tension and the economic layer evolve exactly as `multipolar_sim` already
   evolves them.

The information layer is *upstream* of the existing simulator and replaces one thing in
it: the regional war, which is currently an exogenous Poisson draw, becomes endogenous.

### 2.2 The Council as the audience

Belief is held by an audience, and the audience that decides whether force is *authorised*
is a voting body. Model it as a set of members `C` with a quota and a set of veto holders
— the UN Security Council is the concrete instance: 15 members, 9 votes required, and any
of the five permanent members able to block.

Member `v` supports (or abstains in favour) with probability `q_v`, and

```
q_v = F_v(b_j)                                        (1)
```

where `b_j ∈ [0,1]` is the believed culpability of the prospective target and `F_v` is
nondecreasing. Voters are treated as independent conditional on `b_j` — an assumption,
listed as **A6**, and the one that makes the pivotality result below exact rather than
merely suggestive.

Let `W` be the collection of **winning coalitions** — the subsets of `C` whose support
carries the vote, which under a veto means "at least 9 votes and no veto holder opposed".
The probability of authorisation is

```
P(b_j) = Σ_{S ∈ W}  Π_{v ∈ S} q_v  Π_{v ∉ S} (1 − q_v).        (2)
```

### 2.3 Belief

For each bloc `j`, the audience holds a belief `b_j(t) ∈ [0,1]` about `j`'s true
culpability `θ_j ∈ [0,1]`. The true value is a fact of the world and is not chosen by
anyone in the model. Belief evolves by

```
b_j(t+1) = (1 − λ)·b_j(t) + λ·θ_j + s_j(t) + Σ_{m ≠ j} σ_mj · b_m(t),      (3)
```

where

- `λ ∈ (0,1]` is the **verification rate**: how fast an unaided audience reverts to the
  fact. Inspections, courts, a free press, the target's own rebuttal, and the sheer
  persistence of reality all live in this one number.
- `s_j(t) ≥ 0` is the total **belief injection** aimed at `j`, defined as
  `s_j(t) = Σ_{i≠j} x_ij(t)·k_i(t)`, where `k_i` is the injector's credibility. An
  assertion from a source with no standing moves nobody; an assertion of zero mass is no
  assertion. This product form is **A3**.
- `σ_mj ≥ 0` is **cross-contamination**: how much suspicion attaching to `m` transfers to
  `j`. Zero diagonal.

In matrix form with `b, θ, s ∈ R^n` and `Σ = (σ_mj)`,

```
b(t+1) = A·b(t) + λ·θ + s(t),      A = (1 − λ)·I + Σ.         (4)
```

### 2.4 Credibility

`k_i(t) ∈ [0, κ]` is bloc `i`'s standing as a source — what makes its next claim believed.
It is a **stock**, not a flow, and that is the whole point: a lie is paid for out of a
reserve that refills slowly and is spent suddenly.

Effort `x_i(t) = Σ_j x_ij(t)` attracts scrutiny. Detection is a Bernoulli draw

```
D_i(t) ~ Bernoulli(π(x_i(t))),     π(x) = 1 − e^(−μx),   μ > 0,        (5)
```

so `π(0) = 0`, `π` is strictly increasing and concave. Credibility then follows

```
k_i(t+1) = clip[ k_i(t) + r_i·(1 − k_i(t)/κ) − χ·D_i(t)·x_i(t) ],    (6)
```

clipped to `[0, κ]`, with `r_i > 0` the annual regeneration of standing (institutions
rebuilding themselves) and `χ > 0` the damage a unit of exposed falsehood does.

Equation (6) is the model's most consequential assumption, and it is worth stating what it
commits to: **damage is proportional to the size of the exposed falsehood**, not to the
number of times something was said. A large exposed lie is a larger scandal than a small
one. §3.7 shows this is exactly what generates the instalment result, and states the
condition under which the result reverses.

### 2.5 Legitimation and the attack decision

Believed culpability is not sought for its own sake. It is sought because it lowers the
price of force. Bloc `i`'s payoff from attacking `j` is

```
V_ij(b_j) = G_ij  +  γ·b_j  −  C_ij·(1 − Λ(b_j)),              (7)
```

- `G_ij` — the **spoils**: territory, resources, strategic position, domestic diversion.
- `C_ij` — the full cost of force if it is *unlegitimated*: censure, sanctions, coalition
  collapse, domestic opposition, the target's allies.
- `Λ : [0,1] → [0, Λ_max]`, continuous and nondecreasing, `Λ(0) = 0` — the **legitimation
  function**: the fraction of that cost which believed culpability removes. It is the
  production function of the whole industry.
- `γ ≥ 0` — the second channel, by which believed culpability raises the spoils directly:
  partners join, the target's friends abstain, the operation is cheaper to sustain. The
  base model takes `γ = 0`; every threshold result holds for `γ ≥ 0` and only the closed
  form changes.

Bloc `i` uses force against `j` when `V_ij(b_j) ≥ 0`.

### 2.6 World-economy loss

A war between `i` and `j` costs the world

```
L_ij = ℓ_d(p_i, p_j)  +  ℓ_t(E_ij)  +  ℓ_τ(Δτ),                (8)
```

direct destruction, trade-and-finance network severance, and the deadweight of the arms
race the war feeds. `E_ij ∈ [0,1]` is the pair's interdependence, which
`multipolar_sim`'s `economy.rs` already computes from sourced energy-flow shares.
Assume `ℓ_t` is strictly increasing and convex in `E_ij` — severing a link costs more the
more there was to sever, and the second severance costs more than the first.

The aggressor bears a share `ς_i ∈ (0,1)` of `L_ij` and the rest falls on everyone else.

### 2.7 The assumptions, listed

Everything proved below is conditional on these. They are stated as assumptions rather
than derived because each is a modelling choice, and each is a place a critic should push.

| # | Assumption | What breaks if it fails |
| --- | --- | --- |
| A1 | `Λ` is continuous, nondecreasing, `Λ(0) = 0`, `Λ(1) = Λ_max > 0` | Thresholds still exist but are sets, not points; the deterred band may vanish |
| A2 | `V_ij` strictly increasing in `b_j` | The attack set is no longer an upper interval; the monotone comparative statics fail |
| A3 | Injection is the product of mass and credibility, `s = x·k` | Theorem 3's exact multiplier becomes an inequality; Theorems 2 and 6 are unaffected |
| A4 | `π` strictly increasing and concave; damage linear in exposed mass | Theorem 6 reverses if damage is per-incident rather than per-unit-mass (§3.7) |
| A5 | `ℓ_t` increasing and convex in `E`; `ς_i < 1` | Theorem 5's externality gap closes if the aggressor internalises the whole loss |
| A6 | Council votes independent conditional on `b` | Pivotality is no longer exactly `∂P/∂q_v`; the ranking result survives, the equality does not |
| A7 | `Σ ≥ 0`, `A ≥ 0`, cross-contamination nonnegative | Theorem 3 needs nonnegative matrices for Perron–Frobenius |
| A8 | The injection objective is concave in `s` | Theorem 5(iv) becomes an inequality on marginal returns only, not on the maximisers |

---

## 3. Results

### 3.1 Theorem 0 — what disinformation buys at the Council is pivotality, not votes

**Theorem 0.** Under (1)–(2) and A6, and with `P` as in (2):

**(i)** `P` is nondecreasing in each `q_v` and hence in `b_j`.

**(ii)** For every member `v`,

```
∂P/∂q_v = Pr[ v is pivotal ],
```

the probability that `v`'s vote changes the outcome, holding every other member's vote at
its own independent draw.

**(iii)** In the veto structure of the UN Security Council — 9 votes of 15 required, and
any permanent member able to block — a permanent member is pivotal in *strictly* more
configurations than an elected member, whenever all support probabilities are equal and
`0 < q < 1`. Formally, with `Piv(v)` the pivotality event,

```
Piv(elected) ⊆ Piv(permanent)      pointwise, for the same realisation of others' votes.
```

*Proof.* (i) Each summand in (2) is a product of terms each nondecreasing in `q_v` and
nonnegative, so `P` is nondecreasing; and `q_v = F_v(b_j)` nondecreasing gives the second
claim.

(ii) Condition on all votes but `v`'s, and write `p_yes`, `p_no` for the probability the
resolution passes given `v` votes yes, respectively no. Then `P = q_v·p_yes + (1−q_v)·p_no`,
so `∂P/∂q_v = p_yes − p_no`, which is exactly the probability that the two outcomes differ —
that is, that `v` is pivotal.

(iii) Refer to the event "the resolution passes on the votes of the other fourteen". For an
elected member `v`: a "no" from `v` is not a veto, so `v` is pivotal iff the others supply
**exactly eight** votes in favour and no permanent member among them votes against. For a
permanent member `u`: a "no" from `u` is a veto and fails the resolution outright, so `u` is
pivotal iff a "yes" from `u` passes it — that is, iff the others supply **at least eight**
votes in favour with no other permanent member opposed. The elected member's condition is
the `= 8` case of the permanent member's `≥ 8` case, and the containment is strict whenever
`Pr[≥ 9 others in favour and no other veto] > 0`, which holds for `0 < q < 1`. ∎

**Why this is the first result and not a footnote.** It says where disinformation is
aimed. The marginal return to effort spent on member `v` is

```
∂P/∂x_v  =  Pr[v pivotal] · (∂q_v/∂b) · (∂b/∂x_v),            (9)
```

and by (iii) and (A3) it is the product of three things: how **pivotal** the member is,
how **persuadable** it is, and how much **credibility** the injector can spend on it. It is
not a function of the member's power. Two consequences that are not obvious and that the
model forces:

1. **The most powerful member is often the wrong target.** A veto holder that would vote
   against regardless has pivotality zero; a member that would vote in favour regardless
   has pivotality zero. Effort aimed at either is wasted, which means the return is
   concentrated on the *swing* voter — and by (iii), on the swing *veto holder*, because
   the veto holder's pivotality set strictly contains the elected member's.
2. **A vote-winning coalition is not the objective.** Raising `q_v` on a member inside
   every winning coalition buys nothing. What is bought is the change in the probability
   of a *different* outcome, which is why defection and abstention are worth as much
   narrative effort as support.

### 3.2 Theorem 1 — the attack threshold, and the band where no lie can work

**Theorem 1.** Under A1, A2 and (7):

**(i) The attack set is an upper interval.** There is a threshold `b̄_ij ∈ [0,1] ∪ {−∞, +∞}`
such that `i` uses force against `j` if and only if `b_j ≥ b̄_ij`.

**(ii) The threshold falls in the spoils and rises in the cost.** On the interior region,
`∂b̄_ij/∂G_ij ≤ 0` and `∂b̄_ij/∂C_ij ≥ 0`.

**(iii) There is a deterred band where disinformation is worthless.** If
`V_ij(1) < 0` — the cost of force is so large that even full legitimation does not cover it —
then **no belief justifies the attack, and no quantity of disinformation can produce one.**

**(iv) There is a band where no justification is needed.** If `V_ij(0) ≥ 0` then `i`
attacks at `b_j = 0`: the operation pays for itself and the narrative is decoration.

**(v)** For the closed form with `γ = 0` and `Λ` strictly increasing, `Λ_max = 1`:

```
b̄_ij = Λ⁻¹( 1 − G_ij/C_ij ),     defined when 0 < 1 − G_ij/C_ij < 1.       (10)
```

*Proof.* (i) `V_ij` is continuous and strictly increasing in `b_j` by A2, so the superlevel
set `{b : V_ij(b) ≥ 0}` is an interval closed above; its infimum is `b̄_ij`. (ii)
Differentiate `V_ij(b̄) = 0` implicitly:
`∂V/∂G + (∂V/∂b)·(∂b̄/∂G) = 0`, so `∂b̄/∂G = −1/(∂V/∂b) ≤ 0` since `∂V/∂b > 0`; and
`∂V/∂C = −(1−Λ) < 0` on the interior, so `∂b̄/∂C = (1−Λ)/(∂V/∂b) ≥ 0`. (iii) If
`V_ij(1) < 0` then `V_ij(b) < 0` for all `b ≤ 1` by monotonicity; disinformation moves
`b` within `[0,1]` and so cannot cross the threshold. (iv) `V_ij(0) ≥ 0` and monotonicity.
(v) Substitute `γ = 0` and solve `Λ(b) = 1 − G/C`, which is in `(0,1)` exactly when
`0 < G < C`; apply `Λ⁻¹`. ∎

**Corollary 1.1 (a lie cannot create a war that does not pay).** The set of dyads in which
disinformation can change the outcome is exactly

```
B = { (i,j) : V_ij(0) < 0 ≤ V_ij(1) }.                        (11)
```

Outside `B` the narrative is either unnecessary (iv) or useless (iii). *This is the single
most testable claim in the document*, and it is uncomfortable for the intuitive account:
the model does not say that lies cause wars, it says that **lies select which of the wars
that already pay become feasible.** The intuitive account and this one differ observably —
§4 states the test.

**Corollary 1.2 (the second channel widens the band).** With `γ > 0`, the threshold also
falls in the target's *strategic value*, and the band `B` strictly contains the `γ = 0`
band. Legitimation and spoils are substitutes in the production of force: the cheaper a
war is, the less evidence a government needs to sell it.

### 3.3 Theorem 2 — credibility is monotone in sustained lying, and the lie budget is finite

The first result is exact, pathwise, and needs no distributional assumptions beyond (5).

**Theorem 2a (monotone damage).** Fix any realisation of the uniform sequence `(U_t)` and
define `D_t(x) = 1[ U_t < π(x) ]`. Let `k_t(x)` follow (6) with a constant sustained effort
`x`. If `0 ≤ x ≤ x′` then

```
k_t(x) ≥ k_t(x′)      for every t, pointwise in the realisation,
```

and consequently the same inequality holds for every expectation, every time average, and
every monotone functional of the path. Credibility is nonincreasing in sustained lie mass.

*Proof.* `π` is nondecreasing, so `D_t(x) ≤ D_t(x′)`. Define

```
φ_d(z) = clip[ z·(1 − r/κ) + r − d ].
```

`φ` is nondecreasing in `z` (slope `1 − r/κ`, and the clip is monotone) and nonincreasing
in `d`. The damage `d_t(x) = χ·D_t(x)·x` is nondecreasing in `x`, because both factors are.
Induct: if `k_t(x) ≥ k_t(x′)` then
`k_{t+1}(x) = φ_{d_t(x)}(k_t(x)) ≥ φ_{d_t(x′)}(k_t(x)) ≥ φ_{d_t(x′)}(k_t(x′)) = k_{t+1}(x′)`,
using monotonicity in `z` for the first inequality and monotonicity in `d`, together with
`d_t(x) ≤ d_t(x′)`, for the second. ∎

**Theorem 2b (the break-even lie, and what it depends on).** Let `β(x) = χ·x·π(x)` be the
**detection burden** — expected credibility damage per year at sustained effort `x`. Then
`β` is strictly increasing on `[0, ∞)` with `β(0) = 0` and `β(x) → ∞`. There is therefore a
unique **break-even lie mass** `x†` with

```
β(x†) = r,        equivalently     x†·(1 − e^(−μx†)) = r/χ,              (12)
```

and

```
∂x†/∂r > 0,        ∂x†/∂μ < 0,        ∂x†/∂χ < 0.                        (13)
```

Below `x†` the credibility stock has a positive reversion target; above it, effort is
self-consuming.

*Proof.* `β(x) = χx(1 − e^(−μx))`: `β(0) = 0`; `β′(x) = χ[(1 − e^(−μx)) + μx e^(−μx)] > 0`
for `x > 0`; `β(x) ≥ χx(1 − e^(−μx)) → ∞`. So `β` is a strictly increasing bijection
`[0,∞) → [0,∞)` and `x† = β⁻¹(r)` is unique. The comparative statics are the implicit
function theorem applied to `χx†(1 − e^(−μx†)) = r`, whose left side is increasing in each
of `χ, x†, μ` separately: `∂/∂χ > 0`, `∂/∂μ > 0`, `∂/∂x† > 0`. Hence
`∂x†/∂r = 1/(∂β/∂x†) > 0`, `∂x†/∂μ = −(∂β/∂μ)/(∂β/∂x†) < 0`,
`∂x†/∂χ = −(∂β/∂χ)/(∂β/∂x†) < 0`. ∎

**What (13) says in one line, and why it is a policy result.** The lie mass a state can
sustain is **increasing in how fast its institutions regenerate standing** and
**decreasing in the detection hazard and in the damage per exposed lie**. Verification
capacity, in this model, does not stop lies by being believed; it stops them by making the
liar's own credibility the currency. The cheapest way to raise the price of a lie is to
raise `μ`, and `μ` is a property of the *audience's* institutions, not of any bloc's.

**Corollary 2.1 (the lie budget buys finitely many wars).** Suppose a war requires a
belief shift of at least `Δ` per year for `T` years, delivered at effort `x`, so that
`x·k(t) ≥ Δ` throughout. Since `k` reverts toward `κ(1 − β(x)/r)` and that target is
strictly decreasing in `x` while `x` itself must rise to meet `Δ`, the sustainable number
of consecutive legitimated wars is finite for every parameterisation, and it is decreasing
in `μ` and `χ` and increasing in `r`. *The reserve is spent, not replenished, by success.*

### 3.4 Theorem 3 — a lie's reach is a property of the network, and there is a phase transition

**Theorem 3 (network amplification).** Let `A = (1 − λ)I + Σ` with `Σ ≥ 0`, and assume `A`
is irreducible.

**(i) The stability threshold is exact.**
```
ρ(A) = (1 − λ) + ρ(Σ),        hence        ρ(A) < 1  ⟺  λ > ρ(Σ).       (14)
```

**(ii) If `λ > ρ(Σ)`** the belief system (4) has the unique steady state
`b^∞ = (I − A)⁻¹(λθ + s̄)` for a constant injection `s̄`, and `b(t) → b^∞` geometrically
from any `b(0)`.

**(iii) The amplification is exactly `1/(1 − ρ)` in the Perron direction.** Since `A ≥ 0`
is irreducible, Perron–Frobenius gives a strictly positive eigenvector `v` with `Av = ρv`.
Then

```
(I − A)⁻¹ v = v / (1 − ρ),                                    (15)
```

so a sustained injection aligned with `v` moves long-run belief by `1/(1 − ρ)` times its
own size, and `(I − A)⁻¹ = Σ_{u≥0} A^u ⪰ 0` elementwise.

**(iv) If `λ ≤ ρ(Σ)`** there is no bounded interior steady state: belief has no
equilibrium and runs away along the Perron direction.

*Proof.* (i) Irreducibility is inherited. Let `v > 0` satisfy `Σv = ρ(Σ)v`. Then
`Av = (1 − λ)v + ρ(Σ)v = ((1 − λ) + ρ(Σ))v`, so `(1−λ)+ρ(Σ)` is an eigenvalue and
`ρ(A) ≥ (1−λ)+ρ(Σ)`. Conversely, Perron–Frobenius gives a nonnegative `w ≠ 0` with
`Aw = ρ(A)w`; then `Σw = (ρ(A) − (1−λ))w`, so `ρ(A) − (1−λ)` is an eigenvalue of `Σ` and
hence `≤ ρ(Σ)`. Equality follows.

(ii) `ρ(A) < 1` makes `I − A` invertible with `(I − A)⁻¹ = Σ_u A^u`, the Neumann series
converging in any submultiplicative norm because `‖A^u‖ ≤ ‖A‖^u → 0`. The fixed point is
unique and the convergence is geometric at rate `ρ(A)`.

(iii) Apply `Σ_u A^u` to the eigenvector: `Σ_u A^u v = Σ_u ρ^u v = v/(1−ρ)`. Nonnegativity
is termwise.

(iv) If `ρ(A) ≥ 1` the series does not converge and `(I − A)` is singular or its inverse
has a nonpositive entry; along `v` the recursion multiplies by `ρ(A)^t`, which does not
decay. ∎

**Corollary 3.1 (verification buys stability, contamination destroys it).** The system is
stable if and only if the verification rate exceeds the spectral radius of the
contamination network. There is therefore a **critical verification rate `λ* = ρ(Σ)`**, and
an information environment below it does not have more false belief — it has *no stable
belief at all*. Measured against a fixed `λ`, crossing `ρ(Σ)` produces a discontinuity: an
arbitrarily small increase in cross-contamination can make an arbitrarily large difference
to what the audience ends up believing.

**Corollary 3.2 (targeting the network beats targeting the claim).** Since
`∂b^∞/∂s̄ = (I − A)⁻¹`, the marginal effect of an injection aimed at `j` is column `j` of
that inverse, not the unit vector. A small network of mutually reinforcing outlets that
cite each other is worth `1/(1 − ρ(Σ))` times the same assertions issued independently —
which is a formal reason why coordination, not volume, is the operative variable in
information warfare.

**Remark.** The verification rate and the credibility stock are two different defences and
the model keeps them separate. `λ` is the audience's and cannot be bought by the target;
`k` is the speaker's and is spent by the speaker. A target of a lie cannot raise its
attacker's `μ`; it can only raise the *audience's* `λ` — which is the public good of
§3.5, and which is why the next result matters more than the previous one.

### 3.5 Theorem 4 — the verification that would prevent this is a public good, so it is under-provided

Verification is now a choice. Each bloc `i` invests `v_i ≥ 0` in it — independent media
support, inspections, forensic capacity, open-source intelligence, courts — at convex cost
`c_i(v_i)`, and the system's verification rate is

```
λ(v) = λ₀ + Λ·Σ_j v_j,       Λ > 0.                            (16)
```

Bloc `i`'s payoff from the information environment is

```
u_i(λ) = Φ_i(λ) − Ψ_i(λ),                                      (17)
```

where `Φ_i` is the protection it gets from *others* being disbelieved and `Ψ_i` is the
legitimation capacity it *loses* when its own claims are disbelieved. Both are increasing
in `λ`. Bloc `i` is a **net aggressor** if `Ψ_i′ > Φ_i′` and a **net target** if the
reverse.

**Theorem 4.** Let `P = {i : u_i′ > 0}` be the **net targets** — the blocs for which
verification is a good — and `A = {i : u_i′ < 0}` the **net aggressors**, and take
investment nonnegative.

**(i) Every net aggressor invests nothing.** If `u_g′ < 0` then `v_g = 0`; its best
response is the corner.

**(ii) Among the blocs it protects, verification is strictly under-provided.** Let `v^P`
maximise the targets' *own* joint surplus `Σ_{j∈P} [u_j(λ(v)) − c_j(v_j)]`. Then for every
`i ∈ P`,

```
v_i^NE  ≤  v_i^P,        strictly whenever |P| ≥ 2,                        (18)
```

and the shortfall is exactly `Λ·Σ_{j∈P, j≠i} u_j′(λ) > 0` — the free-rider term. Each
exposed bloc funds only its own protection.

**(iii) The comparison against world welfare has no fixed sign.** A world planner maximises
`Σ_j [u_j(λ(v)) − c_j(v_j)]`, aggressors included, and for them verification is a **bad**.
Its total marginal valuation is `Σ_j u_j′`, which can be negative. When it is,

```
Σ_j u_j′ < 0   ⟹   the world planner wants zero verification
                    while the exposed blocs are already funding some,       (19)
```

so a system dominated by a net aggressor presents the *appearance* of
over-verification. Both signs occur in the implementation's tests.

**(iv) Counter-verification is a best response too.** If `v_i` may be negative —
expenditure on *discrediting* the verifiers rather than on verifying — then a bloc with
`u_g′ < 0` sets `v_g < 0`, and verification and counter-verification are strategic
substitutes, since `∂²u_i/∂v_i∂v_j = Λ·u_i″(λ) ≤ 0` under concavity.

**(v)** The gap in (18) grows with trade integration, by Theorem 5.

*Proof.* (i) Differentiating, `∂u_g/∂v_g = Λ·u_g′(λ) < 0`, so the maximum over `v_g ≥ 0` is
at `0`.

(ii) The environment is a pure public good among the targets: `∂u_i/∂v_j = Λu_i′(λ) > 0`
for `j ≠ i` inside `P`, and `∂c_i/∂v_j = 0`. Nash equates *own* marginal benefit to
marginal cost, `Λu_i′ = c_i′`; the coalition equates the *targets'* sum, `ΛΣ_{j∈P}u_j′ =
c_i′`. Both sides are strictly decreasing in `v_i` (concavity of `u_i` in `λ`, convexity of
`c_i`), so the solution is monotone in the marginal benefit, giving (18); and
`Σ_{j∈P}u_j′ > u_i′` exactly when `|P| ≥ 2`, giving strictness.

(iii) A planner who counts the aggressors maximises a different function. Its aggregate
marginal benefit `ΛΣ_j u_j′` is negative precisely when the aggressors' stake in being
believed exceeds the targets' exposure, and the nonnegativity constraint on `v` then puts
every `v_i` at zero, which is *below* the equilibrium. The planner's preference is not the
model's recommendation — the module computes it, and the document reports it, because
suppressing it would mean reporting only the comparison that flatters the argument.

(iv) The cross-partial is `Λu_i″(λ)`, nonpositive under concavity, which is the definition
of strategic substitutes. (v) See Theorem 5(iv). ∎

**A correction worth recording.** The first version of this theorem claimed that the
equilibrium is below the world-welfare planner *for every bloc*, proved by the step
`Σ_j u_j′ ≥ u_i′`. That step is false: the sum includes the aggressors' negative terms, so
it can fall below a single target's valuation. Writing the test is what exposed it — bloc 1
in the implementation's own example invests `0.3` at equilibrium against the planner's
`0.2` — and the theorem is now stated in the form the proof actually supports. It is left
in the record rather than quietly amended because the failure mode is the interesting one:
**the public-good claim survives only among the blocs that want the good**, and a model
that counted the aggressor as a beneficiary of truth would have produced a confident and
backwards answer.

**This is the paper's central negative result.** It is not that lying is profitable —
Theorem 2 already says the liar pays. It is that **the defence against lying is a public
good among the blocs it protects, and it is bought in the quantity the least-exposed of
them wants**, which in a system that also contains net aggressors is very close to none. A
bloc that is lied about is paying for its own defence against a capability whose benefits
accrue to everyone who is not the liar — while the blocs that most benefit from the
capability being unchecked contribute nothing, and can make the world-welfare arithmetic
appear to say that verification has gone too far.

### 3.6 Theorem 5 — the world economy: integration raises the damage and can cut either way on the war rate

Write the pair's interdependence `E = E_ij`. Let the spoils rise with it,
`G(E)` with `G′ > 0` — there is more to take where there is more to take — and let the
aggressor's own cost rise with it too, because severing a link destroys the aggressor's own
trade with the target: `C(E) = C₀ + κE`.

**Theorem 5.** Under A5, with `V(b, E) = G(E) + γb − (C₀ + κE)(1 − Λ(b))`:

**(i) The world's exposure is unambiguously increasing in integration:**

```
∂L_ij/∂E = ∂ℓ_t/∂E > 0,        ∂²L_ij/∂E² > 0.                  (20)
```

**(ii) The aggressor's private incentive moves with the balance of two elasticities.**
With belief held at `b`, define the elasticities `η_G = E·G′/G` and
`η_C = E·κ/C`. Then

```
∂V/∂E = G′ − κ(1 − Λ(b)),      sign(∂V/∂E) = sign( η_G·G − η_C·C·(1 − Λ(b)) ),   (21)
```

so the pair is **pacified** by integration if the spoils elasticity is small relative to the
aggressor's own cost exposure, and **destabilised** by it otherwise.

**(iii) The externality is strict.** The change in world welfare from the war is

```
ΔW = V_i + V_j − L_ij − Σ_{m ∉ {i,j}} ℓ_m,                       (22)
```

whereas `i` attacks on `V_i ≥ 0` alone. Whenever `ς_i < 1` and `L_ij > 0` there are
parameter values with `V_i ≥ 0` and `ΔW < 0`; explicitly, take `G(E)` with
`G(E) ≥ (C₀+κE)(1−Λ(b))`, `γ = 0`, and `ℓ_t(E)` large enough that `L_ij > V_i + V_j`.

**(iv) The private return to lying exceeds the social one, and the wedge grows with
integration.** Let `i` choose the belief injection `s` at convex cost `c(s)`, and let it
bear a share `ς_i < 1` of `L`. The difference between the private and the world's marginal
return to `s` is

```
W(s, E) = (1 − ς_i) · L_ij(E) · ∂b/∂s,                          (23)
```

which is nonnegative, strictly positive wherever more injection still buys belief, and

```
∂W/∂E = (1 − ς_i) · ∂L/∂E · ∂b/∂s  > 0                          (24)
```

by (i). Under A8 the private optimum therefore lies weakly above the planner's, and the
gap does not shrink as integration deepens.

*Proof.* (i) Immediate from A5 (`ℓ_t` strictly increasing). (ii) Differentiate (7) with
`G = G(E)`, `C = C₀ + κE`: `∂V/∂E = G′(E) − κ(1 − Λ(b))`; multiplying by `E > 0` and
substituting the definitions of the elasticities gives the stated sign. (iii) Evaluate
`ΔW` at any `E` satisfying the two stated inequalities: `V_i ≥ 0` makes the attack occur,
and `L_ij > V_i + V_j ≥ V_i` makes `ΔW < V_j + V_i − L_ij < 0`. (iv) The two objectives
differ by `(1 − ς_i)L_ij` times the probability of the war, whose derivative in `s` is
`(1 − ς_i)L_ij·∂b/∂s`; differentiating that in `E` gives (24), using (i) and
`∂b/∂s ≥ 0`. ∎

**The sentence this theorem exists to produce:** *integration makes wars rarer or more
common depending on whether the spoils or the aggressor's own exposure grows faster — and
either way it makes the wars that do happen more destructive, while widening the gap
between what a bloc gains from lying and what the world loses.* A more integrated world is
not a safer world or a more dangerous one; it is a world where the same lie does more
damage, which is why §3.5's under-provision gets worse rather than better as trade deepens.

### 3.7 Theorem 6 — why lies arrive in instalments

Take the total assertion mass `M` fixed, delivered as `T` equal instalments of size `M/T`,
and let the cost of taking longer be `δT` with `δ > 0` (the target consolidates, the
window closes, attention moves on).

**Theorem 6.** Under A4, the expected total credibility damage is

```
Φ(T) = χ·M·( 1 − e^(−μM/T) ),                                    (24)
```

which is **strictly decreasing and strictly convex in `T`**, with `Φ(∞) = 0`. The bloc
minimises `J(T) = Φ(T) + δT`. Let `z = μM/T`. Then

**(i)** At an interior optimum `z*` satisfies

```
χ·z*²·e^(−z*) = δ·μ,                                            (25)
```

and an interior optimum exists **iff** `δ·μ < 4χ·e^(−2)`; at equality `z* = 2`; if the
inequality is reversed the optimum is the corner `T* → ∞` — the bloc never stops dribbling.

**(ii) Same total mass, more instalments, less damage.** For `T < T′`, `Φ(T) > Φ(T′)`.

**(iii) The result is an artefact of A4, and reverses under the opposite assumption.** If
damage is charged *per exposed incident* rather than per unit of exposed mass, the total is
`T·χ·π(M/T)`, which is **increasing** in `T`: the same argument then says concentrate, not
dribble. Which of the two is right is an empirical question about how scandals actually
cost their authors, and §6 says what would settle it.

*Proof.* (24): each of the `T` instalments of mass `m = M/T` draws detection with
probability `π(m)` and costs `χm` when exposed, so the expectation is `T·χ·m·π(m)
= χM(1 − e^(−μM/T))`. Writing `z = μM/T`, `Φ = χM(1 − e^(−z))` and `δT = δμM/z`, so
`J = χM(1 − e^(−z)) + δμM/z`. Then `dJ/dz = χMe^(−z) − δμM/z²`, whose sign is that of
`H(z) = χz²e^(−z) − δμ`. `H` is continuous with `H(0) = −δμ < 0`, rising on `(0,2)` and
falling on `(2,∞)`, with maximum `H(2) = 4χe^(−2) − δμ`. So a root in `(0,2)` exists iff
`δμ < 4χe^(−2)`, and it is unique there because `H` is strictly increasing on that interval
(`H′(z) = χe^(−z)(2z − z²) > 0` for `z ∈ (0,2)`). Otherwise `J′ < 0` for all `z` and the
infimum is at `z → 0`, i.e. `T → ∞`. (ii) `1 − e^(−μM/T)` is strictly decreasing in `T`.
(iii) `T·χ(1 − e^(−μM/T))` is strictly increasing in `T` since `π` is concave and
`π(0) = 0`. ∎

**Remark (why this is in a paper about war).** Theorem 6 explains an observed regularity in
how legitimating narratives are built: not as one large claim that can be checked and
killed, but as a sequence of smaller ones whose cumulative effect is the same and whose
individual cost of exposure is lower. Its policy implication is specific and follows from
(ii) rather than from any parameter: **monitoring calibrated to the size of a single claim
is structurally defeated by salami tactics; monitoring must accumulate across claims,
which means it needs `μ` to act on the running total, not the increment.** Finally, note
that (iii) is not a technicality: the whole result rests on how scandals scale, and a
model that hid that behind a smooth functional form would be claiming more than it knows.

---

## 4. What the model predicts, and how it could be wrong

A model with no refutable implications is decoration. These are the places this one could
be killed, in the order they would be cheapest to test.

| # | Prediction | What would refute it |
| --- | --- | --- |
| P1 | Disinformation-to-force cases cluster in the band `B` of Corollary 1.1 — the war does not pay on its own but would pay if justified. | Cases where force is initiated with a legitimating narrative in dyads where the operation was plainly profitable at `b = 0` (no justification needed), or in dyads where no achievable belief could have covered the cost. If narratives appear uniformly across the band, the threshold is not doing work. |
| P2 | A bloc caught in an exposed falsehood suffers a fall in `k`, and the fall is increasing in the size of the exposed claim. | Detection events with no measurable fall in the claimant's standing; or falls uncorrelated with claim size. Note the counter-evidence honestly: the "backfire effect" literature is contested, and §3.4's mechanism does not require backfire, only a reduction. |
| P3 | Verification is under-provided relative to what the **exposed** blocs would jointly choose, and the shortfall is larger where trade integration is deeper (Theorem 4(ii) and (v)). Note what is *not* claimed: against world welfare the sign is not fixed, so the test is against the targets' own benchmark. | Cross-country or cross-domain data showing exposed states funding verification at the rate their own joint exposure implies; or net-aggressor blocs funding verification generously. |
| P4 | A narrative's long-run effect is a network property: systems with higher `ρ(Σ)` show more amplification per unit of assertion, and there is a discontinuity at `λ = ρ(Σ)` (Corollary 3.1). | Amplification that scales with assertion *volume* rather than with network structure; or no threshold behaviour as `λ` varies. |
| P5 | Legitimating narratives about a given target arrive in many instalments rather than one large claim, and the cumulative total exceeds any single claim's mass (Theorem 6). | Narratives that arrive as a single large, exposure-risky claim. This is the cheapest test in the table — it is counting. |

A prediction that is *not* on this list because the model does not make it: that
disinformation causes wars. Corollary 1.1 says the model claims the weaker and more
interesting thing — that it determines which of the wars already worth fighting become
politically feasible — and a reader who takes a stronger claim from this document than the
theorem supports has been misled by the prose rather than by the mathematics.

---

## 5. The historical record

This section is deliberately separated from §3, and the separation is the point. The
theorems are about the model; the cases below are about the world; and no theorem was used
to establish any fact in this section. The full sourcing — every claim with its document,
date and URL, plus the cases that could not be sourced adequately — is in the companion
file `information_warfare_claims_reference.md`. What follows is the part of it the model
has to answer to.

### 5.1 The cases are not all the same kind of thing, and lumping them would overstate the record

Three subtypes appear, and only the first is the phenomenon this paper models.

| Subtype | Structure | Cases |
| --- | --- | --- |
| **Ex ante legitimation** | A claim is made *before or during* the escalation, to justify, initiate or sustain force; it is later discredited. **This is the model's subject.** | Gulf of Tonkin 1964; Iraq 2003 (aluminium tubes, mobile BW labs, Niger/"16 words", the stockpile claim); Kosovo 1999 (Račak, and the authorisation question); Libya 2011 |
| **Post hoc denial** | Force or an atrocity has occurred; the accused state denies it; formal findings contradict the denial. **A different game** — there is no authorising audience to persuade and no attack threshold to cross. | MH17 2014; Bucha 2022; Ghouta 2013; Douma 2018 |
| **Insufficiently sourced or off-model** | Either the primary record could not be reached, or no government used the claim to justify force. | Nayirah 1990 (no primary hearing record reached); the Kosovo "yellow house" allegations (investigative outcome not reached); Nord Stream 2022 (no state has formally attributed responsibility, and no force was justified by it) |

Including the second and third groups in a table about *legitimation* would inflate the
evidence base for the mechanism. They are reported separately or not at all, and the two
unpublishable rows are named rather than quietly dropped.

### 5.2 The ex ante cases, and who did the correcting

Field 3 is the one that matters for the model, because it distinguishes a **self-correction**
— the originating state's own institutions — from an **external contradiction**, which the
claimant had no part in.

| Case | The claim | Who discredited it | Self-corrected? | Action proceeded? |
| --- | --- | --- | --- | --- |
| Gulf of Tonkin, 4 Aug 1964 | A second North Vietnamese attack on USS *Maddox* | **NSA's own historian** (Hanyok, written 2001, declassified Dec 2005): "no attack happened that night" | **Yes** — but the NSA historian expressly cleared Johnson and his ministers, blaming the intelligence gatherers | Yes; Gulf of Tonkin Resolution, 7 Aug 1964 |
| Iraq 2003 — aluminium tubes | Tubes for gas-centrifuge enrichment | **US Senate Select Committee on Intelligence**, S. Rep. 108-301, 9 Jul 2004: assessment not supported; more plausibly rocket casings | **Yes** — the originator's own legislature, formally | Yes; invasion 20 Mar 2003 |
| Iraq 2003 — mobile BW labs | Mobile biological-weapons production | **Iraq Survey Group (Duelfer Report)**, 2004; the source's own 2011 admission that he lied (BBC, 15 Feb 2011); decisive exposure was journalistic (LA Times, 2005) | **Mixed** — a US inspection body's negative finding, plus a defector's confession | Yes |
| Iraq 2003 — Niger, the "16 words" | Iraq sought uranium from Africa | **CIA Director Tenet**, 11 Jul 2003: "These 16 words should never have been included"; the Niger documents were forgeries | **Yes** — the executive, on the record | Yes |
| Iraq 2003 — the stockpile claim | Iraq held WMD stockpiles | **ISG/Duelfer**, 6 Oct 2004: no stockpiles; capability had decayed since 1991. **UK Iraq Inquiry (Chilcot)**, 6 Jul 2016 | **Yes**, at four levels of the originators' own states | Yes |
| Kosovo 1999 — Račak, 15 Jan 1999 | Killings characterised by the OSCE mission head as a massacre and a crime against humanity; Yugoslav/Serbian authorities called the dead combatants | **Neither, cleanly.** The ICTY *indicted* over ~45 killings, but Milošević died before judgment, so **there is no final judicial finding**; the OSCE never retracted | **No** — and the correcting party would have been an international body, not a state | Yes; Operation Allied Force, 24 Mar 1999 |
| Kosovo 1999 — authorisation | Force was justified by implied authority from UNSCR 1199/1203 and humanitarian necessity | The resolutions' text **contains no authorisation of force** (not contested). "Illegal but legitimate" is an **independent commission's** characterisation, not a court's | **No** — no NATO state withdrew the legal basis | Yes, without explicit prior UNSC authorisation |
| Libya 2011 | UNSCR 1973 authorised civilian protection; the operation went further | **UK House of Commons Foreign Affairs Committee**, HC 119 (2016): the intervention rested on erroneous assumptions | **Yes** — the originating coalition's own legislature | Yes; NATO operations from 19 Mar 2011 |

### 5.3 What the record does to the model — and it is not a comfortable fit

Three patterns run across the table, and each one lands on a different assumption in §2.
They are stated here as challenges, not as confirmations.

**(a) The correction attaches to the claim, not to the decision.** In every ex ante case the
military action stood. The factual record was corrected — by the CIA Director, by a Senate
committee, by an inspectorate, by a national inquiry — and the war was not reversed, the
legal basis was not repudiated, and no apology was issued. When the WMD claims collapsed,
the stated justification moved to Saddam's intent, sanctions non-compliance and humanitarian
grounds rather than being conceded.

*What this does to the model:* Theorem 1 treats legitimacy as reducing `C`, the cost of
force, at the moment of decision. That is right as far as it goes. What the record shows is
that the cost is not **re-imposed** when the justification later collapses — the correction
arrives after the operation is a fact. So the model's `V(b)` is a decision rule, not a
settlement rule, and a model of the settlement would need a second payoff evaluated at the
time of correction. This document does not build that, and the omission favours the model's
own story: it makes lying cheaper than Theorem 1 implies.

**(b) The correction lands on the intelligence institutions, not on the political
leadership.** Hanyok cleared Johnson and his ministers and blamed the intelligence
gatherers. Tenet took responsibility on the CIA's behalf. The Senate committee examined the
*intelligence community's* prewar assessments. This is structural, and the sources
individually decline to convert it into a finding about political leadership.

*What this does to the model:* this is the most serious problem in the document. §2.4 makes
credibility `k_i` a property of **the bloc** — the claimant pays for its own lies. The record
says the claimant is often not the payer: the cost is borne by an intelligence service,
which is a *different institution with a different credibility stock*. The model needs
`k` indexed by **institution** rather than by bloc, with the political leadership able to
externalise the damage onto the agency that produced the estimate. That refinement is not
implemented here, and its absence biases Theorem 2 in the direction the theorem wants:
making the liar pay is the model's assumption, and the record is evidence that someone else
pays instead.

**(c) In no case examined was the claim formally withdrawn.** The closest instances are
partial and precisely bounded: Tenet's concession that the sixteen words "should never have
been included"; the ISG's negative finding. **This is a claim about the absence of an act**,
so it must be phrased as "no instance was found", not as "no instance exists" — but if it
holds up, it is the single largest empirical threat to this paper.

*What this does to the model:* Theorem 2 says a detected lie depletes the liar's credibility
stock by `χ` per unit of exposed falsehood. The record is consistent with detection being
real, frequent and institutional (ISG, SSCI, Chilcot), and with `χ` nonetheless being small
enough that the claimant's future claims are not visibly discounted. So the threat is not
that detection fails — it is that **detection succeeds and the liar does not pay.** If `χ ≈ 0`,
then `x† → ∞`, the lie budget is unbounded, and the "liar's budget" of Theorem 2 is not a
constraint at all. The mathematics would remain correct and the mechanism would be
empirically dead.

Two honest qualifications, in the other direction. First, standing can fall without a formal
withdrawal, and the absence of a retraction is not the absence of a cost; nobody has
measured the cost, which is exactly the gap §6 identifies. Second, the model's `k` is not
the claimant's reputation in general but its standing **as a source on this kind of claim**,
which is a narrower and more plausibly depletable object than "the government's standing" —
the record does not measure that either.

**(d) The one case that fits hardest is the one with the weakest record.** Kosovo is the
closest match to the model's mechanism: a contested incident used as the trigger for an
escalation, no Security Council authorisation for the force, and — in this paper's terms — an
authorising body whose pivotality structure (Russia and China holding vetoes, the elected
members pivotal at the quota) is exactly what Theorem 0 describes. It is also the case where
the central disputed facts could not be adjudicated from the sources reached, where the
forensic evidence split along national lines, and where the only formal finding is an
indictment that never reached judgment. **The case that best illustrates the model is the
one that can least carry the evidential weight**, and saying so is more useful than a
fourteenth table row.

---

## 6. What is not established, and what would have to be measured

### 6.1 Four objects this model needs, which cannot be parameterised at all

This is the most important section in the document, and it is not a list of caveats. It is a
result: a systematic search of the published record found **no estimate, of any quality, for
four of the quantities the model is built on**. Not "no good estimate" — none, and for two
of them none is constructible from the available data.

| Object in §2 | Status of the published evidence | Consequence |
| --- | --- | --- |
| `μ`, the detection hazard | **Not estimable.** The denominator — total false official claims uttered — is unobservable, and the fact-checking literature explicitly warns its archives cannot serve as that denominator. What exists is *conditional coverage over curated narrative sets*: 47% of 135 prominent 2022 US election-misinformation narratives were ever fact-checked, the median first fact-check arrived four days later by which time 79% of posts had been published, and fewer than 3% of those exposed to a later-flagged article read the corresponding fact-check | `π(x) = 1 − e^(−μx)` is a modelling convenience with no empirical anchor. Theorems 2 and 6 are conditional on a function nobody has measured |
| `χ`, the credibility damage per exposed lie | **Not measured.** No verified effect size exists, in the lab or the field, for the reputational penalty to a sender whose lie is *detected*. The neighbouring literature measures the **decision to lie** (mean standardised report 0.234 across 90 studies; fewer than 1 in 20 lie maximally) and interpersonal **detection accuracy** (54%), neither of which is the penalty after detection | This is Theorem 2's load-bearing parameter, and it is unmeasured — see §5.3(c): the historical record is consistent with `χ ≈ 0` |
| The aggregate cost of misinformation to an economy | **No peer-reviewed estimate exists for any jurisdiction.** Both circulating global figures are vendor or scenario products: $78bn for 2019 rests on an *assumed* "up to 0.05% of stock market value at risk" with no cited basis, and its $417bn successor concedes in its own limitations that it is "less a truly global cost than a projection based on a geographically and structurally biased sample" with "a structural risk of duplicating figures" | §2.6's `L_ij` can be calibrated at the level of a *war* (§6.2) but the misinformation channel into the world economy has no measured magnitude at all |
| The counterfactual | **Not observable by construction.** There is no measured no-war path | Every number in Theorem 5 is a conditional comparative static, never a forecast |

Two further absences worth naming, because the model leans on them: nobody has measured a
**dose–response for repeated deception**, and the **backfire effect** — belief moving *away*
from a correction — is best treated as zero rather than as a positive parameter. The original
finding rested on cells of n=33 and n=60; a replication across five experiments, 52 issues and
more than 10,100 subjects "found no corrections capable of triggering backfire". §2.3's
contamination term does not require backfire, and a model that assumed it would be building
on the weaker half of the literature.

**The honest summary of the model's epistemic position:** its six theorems are exact given
the assumptions, and **four of the quantities needed to take them to data do not exist.**
That is a different and more useful statement than "the parameters are illustrative", which
would suggest a calibration exercise nobody has run yet. Some of these are not merely
unrun; they are unconstructible from the record as it stands.

### 6.2 What could legitimately parameterise the model

Measured, with the sample and method disclosed, and — the condition that matters — an
estimand matching the model object. Each is a candidate for one coefficient, not a licence
to calibrate the whole system.

| §2 object | Anchor | Source, method |
| --- | --- | --- |
| `ℓ_d`, direct destruction | War-site output **≈−3% at onset, ≈−7% cumulative at five years** | IMF, *WEO* Apr 2026 / WP 2026/189; local-projection difference-in-differences, 194 countries, 170 onsets, 1946–2024 |
| `ℓ_d` persistence | GDP per capita **≈−28% at ten years**; consumption −25%; exports −58% | Novta & Pugacheva, IMF WP 2020/110; Jordà local projections, 188 countries, 50 onsets |
| `ℓ_t`, severance | Trade between geopolitically distant blocs **−12%**, announced FDI **−20%** | Gopinath, Gourinchas, Presbitero & Topalova, IMF WP 2024/076; PPML gravity, UNGA ideal-point blocs |
| Escalation response | Distortive policy interventions between distant blocs **+28.3% to +33.5%** | Airaudo, de Soyres, Richards & Santacreu, Fed IFDP 1408; PPML, UN Comtrade, Global Trade Alert |
| Bilateral sanction cost | Target **−7.4% of exports**, sender **−0.3%** — the "friendly fire" term | Crozet & Hinz, *Economic Policy* 2020; gravity plus general-equilibrium counterfactual |
| Bloc weights | West **42%**, East **27%**, Neutral **31%** of world GDP (PPP) | ECB, from den Besten et al.'s UNGA-voting index |
| Sender lying cost | Standardised lie **0.234**; fewer than 5% lie maximally; behaviour barely moves across a 500-fold stake range | Abeler, Nosenzo & Raymond, *Econometrica* 2019; meta-analysis, 90 studies, >44,000 participants |
| Receiver signal quality | **54%** of lie-truth judgements correct; coin-flip accuracy with about two-thirds subjective confidence | Bond & DePaulo 2006; Serra-Garcia & Gneezy, *AER* 2021 |
| Non-Bayesian updating (A6) | **19%** of prior-contradicting signals misread, **28%** effectively missed; pro-party source bias **+9pp** (s.e. 0.6pp) | Aydogan et al., *Quantitative Economics* 2025; Thaler, *AEJ:Micro* 2024 |
| Market impact of a false claim | Abnormal return **−1.1% to −1.8%** over (0,1), reversing within a week | Arcuri, Gandolfi & Russo 2023; event study, 149 items |

### 6.3 What could only bound a sensitivity range — including the one dispute the model actually resolves

Some of the material is a **model projection** rather than a measurement, or the same
conceptual quantity has estimators that disagree by a factor of three. These belong in a
sensitivity sweep, never in a baseline.

- **Global fragmentation cost:** 0.2–7% of global GDP (IMF SDN/2023/001, itself a survey of
  general-equilibrium simulations); up to −9% with +4pp inflation (ECB Occasional Paper 365,
  Baqaee–Farhi); −0.7% to −15.2% of welfare by scenario (ECB WP 2839). Scenario outputs, not
  measurements.
- **The war-cost spread:** −7% (IMF), −10% (*AER* 2026), −13% (NBER WP 34389), −30% (Kiel
  brief). Four different estimands. **The spread is the uncertainty band**, and a model must
  say which one its dependent variable corresponds to.
- **Conflict-trap magnitude:** Collier's ~15% income penalty against Bove, Elia & Smith's
  synthetic-control range of **−33% to +32%** across 27 cases, of which only 12 are
  significantly negative. Measurement-based work trends smaller; model-based work trends
  larger. Relatedly, the widely cited "about half of civil wars recur" is contested by a
  re-analysis finding **23%** recurrence within five years — a number that does not survive
  checking, and is cited here as an example of one.
- **Correction efficacy:** bound between zero and the laboratory estimate. The meta-analytic
  range is wide and lower than it looks: continued-influence *r* ≈ −.05; correction *r* = .14
  for real-world political misinformation against .48 for constructed stimuli; and one
  synthesis of 245 effect sizes (N=53,320) found attempts to debunk science-relevant
  misinformation **not successful on average** (*d* = 0.11, p = 0.142). Laboratory effect
  sizes must not be transferred to real claims.
- **Per-war budgetary figures** (Kosovo, Libya, Iraq) are point-in-time outlays. The Iraq case
  is the instructive one: the CRS appropriations figure is about **$806bn**, while the
  $3 trillion headline is a projection whose author conceded under oath that "the real numbers
  were $3 to $5 trillion… if we had used one of the larger numbers, we would have lost
  credibility." That is a documented instance of an advocacy figure being *chosen* for its
  credibility rather than its accuracy, and it is exactly the failure mode this repository's
  §7 rule exists to prevent.

**And the dispute the model resolves.** Theorem 5(ii) says integration pacifies a dyad if the
aggressor's own cost exposure grows faster than the spoils, and destabilises it otherwise.
That condition is not a curiosity of the model — **it is the open question in the empirical
literature**, and the literature has not settled it:

| Pacific | Not pacific, or null |
| --- | --- |
| Oneal & Russett 1999 (*JPR*), with the authors' own concession that among *all* dyads they find no relationship; Hegre, Oneal & Russett 2010, which argues earlier work "overstated the pacific benefit of interdependence" | Barbieri 1996 (*JPR*), 14,341 dyad-years, 1870–1938: extensive interdependence **increases** the likelihood of militarised disputes, curvilinearly |
| Gartzke 2007 (*AJPS*), the "capitalist peace", which attributes the effect commonly credited to regime type to economic variables | Keshk, Pollins & Reuveny 2004 (*J. Politics*): conflict inhibits trade, while interdependence's effect on conflict is **statistically insignificant** |

No verified effect size exists on either side. The model's contribution here is not to settle
the dispute but to say **what would settle it**: an estimate of the two elasticities of
Theorem 5(ii), one for the spoils and one for the aggressor's own exposure. That is a
measurable quantity and nobody has measured it.

### 6.4 What would have to be measured to change the position

In descending order of how much of the paper would move:

1. **The reputational penalty after detection (`χ`).** One credible estimate would settle
   whether Theorem 2 binds or is empirically dead. The natural design is a credibility index
   for a *claimant* — a state's or an agency's standing as a source — measured around
   documented correction events, with the ISG, SSCI and Chilcot cases as the treated
   observations. This is the paper's central open question.
2. **The detection hazard (`μ`) as a function of accumulated narrative mass rather than
   single-claim size.** Theorem 6's policy implication is that monitoring must accumulate;
   that prediction is testable with existing fact-check archives even though those archives
   cannot give a probability of detection.
3. **The two elasticities of Theorem 5(ii).** Then the pacifying-or-not dispute has an
   answer rather than two camps.
4. **`ρ(Σ)`, the spectral radius of the contamination network**, from citation or
   co-citation data among outlets. Then Corollary 3.1's threshold is measurable instead of
   suggestive.
5. **The verification investment `v_i` by state**, against the exposure that Theorem 4 says
   should determine it. If exposed states systematically under-invest, P3 survives.

Until at least (1) exists, the correct description of this paper is: **a consistent formal
account of a mechanism, whose central parameters are unmeasured, one of which the historical
record suggests may be near zero.**

---

## 7. Relation to existing work

The components are not new, and pretending otherwise would be the worst thing this
document could do. What is assembled here is a specific combination.

- **Credibility as a stock, spent by lying.** Sobel (1985) is the canonical formal model of
  a reputation for honesty, and the `k` of §2.4 is a discrete-time variant of it with
  detection risk made explicit. The novelty here is not the stock; it is putting the stock
  in front of a *legitimation production function* and asking what it buys.
- **Private information and the incentive to misrepresent.** Fearon (1995) makes the
  incentive to misrepresent one of the three rationalist causes of war. This document
  formalises a channel that account leaves implicit: misrepresentation aimed not at the
  bargaining table but at the *authorising audience*, and therefore at the cost of force
  rather than at the terms of settlement.
- **Audience costs.** Fearon (1994) and the subsequent literature make domestic audiences a
  source of credible commitment. In this model the audience is not a constraint the leader
  faces but a production input the leader buys — which is the mirror image of that
  literature and the place where it and this model could be made to argue.
- **Cheap talk and its limits.** Crawford and Sobel (1982) show when talk is informative;
  the model here departs by making talk costly — not per statement, but through the
  speaker's stock, which is why the cost is invisible in any single utterance and enormous
  in aggregate. That is the formal content behind "credibility is a stock".
- **Bayesian persuasion.** Kamenica and Gentzkow (2011) solve for the sender's optimal
  information design. Equation (3) is a crude linear reduction of that problem; a serious
  version would let the sender choose a signal structure rather than a scalar injection,
  and would change Theorem 3's multiplier from a matrix inverse to something
  experiment-dependent.
- **Voting power.** Banzhaf (1965), Penrose (1946), Shapley and Shubik (1954) and Coleman
  (1971) supply the pivotality machinery of Theorem 0. The contribution there is only the
  application: reading `∂P/∂q_v` as the *return to disinformation spent on `v`*.
- **The security dilemma.** Jervis (1978), and `MULTIPOLAR_GAME.md` §4 in this repository.
  Legitimation is the term that lets a bloc escape the dilemma's cost without escaping the
  dilemma — it changes the price of defection, not its structure.
- **Input–output and nonnegative matrices.** Leontief's inverse is the object in Theorem
  3(iii), and Perron–Frobenius (see Seneta) supplies the multiplier.
- **Trade and conflict.** Oneal & Russett (1999), Gartzke (2007) and Hegre, Oneal & Russett
  (2010) against Barbieri (1996) and Keshk, Pollins & Reuveny (2004). Theorem 5(ii) is
  offered as the condition that would decide between them, not as a contribution to either
  side; §6.3 sets out both camps with sources and notes that no verified effect size exists
  on either.
- **Correction and continued influence.** The backfire controversy — Nyhan & Reifler (2010)
  against Wood & Porter (2019) — and the meta-analytic literature on whether corrections
  work: Walter & Murphy (2018), Walter & Tukachinsky (2020), Chan & Albarracín (2023). The
  model takes the *low* end of these estimates as its baseline, which is the conservative
  choice for a paper whose mechanism needs corrections to bite.
- **Conflict economics.** The empirical counterpart of §2.6: the IMF's local-projection work
  on war's output cost (2020/110, 2026/189), Benmelech & Monteiro (2025), Federle et al.
  (2026), and the conflict-trap literature from the World Bank (2003) through Bove, Elia &
  Smith (2017). §6.2 and §6.3 separate what can parameterise the model from what can only
  bound it.

**The novelty claim, stated modestly:** treating **legitimation as a production function
whose capital is credibility and whose output is a reduction in the price of force**, then
deriving (a) where the narrative is aimed (Theorem 0), (b) when it is useless (Corollary
1.1), (c) why it is self-limiting (Theorem 2), (d) that its reach is a network property
with a phase transition (Theorem 3), and (e) that the defence against it is a public good
that integration makes more valuable and no one buys (Theorems 4–5). Each of (a)–(e) is a
proposition about a model, and the model is the claim.

It should be said plainly what this is not. It is not an explanation of why any particular
war happened, and §5.3 shows that the historical record does not fit the model as cleanly as
its author would like — specifically on whether the liar pays at all. A paper that reported
only the parts of the record that fit would be doing the thing this repository's `FutureWork.md`
§7 forbids.

---

## 8. Reproduction

```
cargo test -p multipolar_sim information::        # the theorems, as invariants
cargo run -p multipolar_sim -- --information      # endogenous war onset, and the sweep
```

`multipolar_sim/src/information.rs` implements §2 and its test module asserts the theorems
as invariants rather than as prose. Fifteen tests, one per result and a few for the boundary
cases, all in that module:

| Theorem | The test that pins it |
| --- | --- |
| 0(ii) pivotality is the derivative | `the_return_to_persuading_a_member_is_its_pivotality` — numerical derivative of the Council's own probability function against the exact enumeration |
| 0(iii) the veto concentrates pivotality | `the_veto_concentrates_pivotality_on_the_permanent_members` |
| 1(i)–(ii) the threshold and its comparative statics | `the_attack_threshold_falls_in_spoils_and_rises_in_cost` |
| 1(iii)–(iv), Corollary 1.1 | `outside_the_band_disinformation_changes_nothing` |
| 2a monotone damage | `credibility_is_monotone_in_sustained_lying_pathwise` — a *pathwise coupling* against a shared uniform sequence, which is stronger than a mean comparison and is what the proof actually asserts |
| 2b the break-even lie | `the_break_even_lie_moves_with_verification_and_regeneration` |
| 3(i) the spectral identity | `the_spectral_identity_holds_and_names_the_verification_threshold` |
| 3(iii) the Perron multiplier | `the_perron_amplification_is_exactly_one_over_one_minus_rho` — the steady state is solved and read back, not asserted |
| 3(iv), Corollary 3.1 the phase transition | `below_the_verification_threshold_belief_has_no_equilibrium` |
| 4 the public-good failure | `verification_is_under_provided_among_targets_and_aggressors_free_ride` — both signs of the world-welfare comparison |
| 5(i)–(ii) integration | `integration_raises_world_exposure_and_can_cut_either_way_privately` |
| 5(iii)–(iv) the externality | `the_private_return_to_lying_exceeds_the_social_one_and_the_gap_grows_with_integration` |
| 6 the instalment optimum | `the_instalment_trade_off_has_an_interior_optimum_below_a_critical_time_cost` |

Two of these tests found errors during writing, and both are recorded in the document
rather than quietly fixed: the false step in the original Theorem 4 (§3.5) and a power
iteration that returned a spectral radius of 1 for every matrix, which would have destroyed
Corollary 3.1. A third — the interaction between the damage functional form and the
instalment result — is stated as a limitation in §3.7(iii) rather than tested away.

**Backwards compatibility is a hard requirement and is tested.** With the information layer
off — the default — the simulator reproduces its published behaviour exactly, verified both
by a golden-value test and by running the pre-change and post-change binaries and diffing
their full output. The layer's only effect on the simulation is to replace the exogenous war
draw; nothing else in the Monte Carlo is touched.

**The reproduction does not include a plotting script, and that is a deliberate omission
rather than an oversight.** The results in §3 are exact consequences of §2, and a figure of
them would be a picture of algebra that the tests already check; the interesting quantities
are the four objects of §6.1, which have no data to plot. The mode that produces a table
worth reading is `--information`, and it prints its own caveats.

---

## References

Theory.

- Abeler, J., Nosenzo, D., and Raymond, C. (2019). Preferences for truth-telling. *Econometrica* 87(4), 1115–1153.
- Aydogan, I., Baillon, A., Kemel, E., and Li, C. (2025). Signal perception and belief updating. *Quantitative Economics* 16(1).
- Banzhaf, J. F. (1965). Weighted voting doesn't work: a mathematical analysis. *Rutgers Law Review* 19, 317–343.
- Bond, C. F., and DePaulo, B. M. (2006). Accuracy of deception judgments. *Personality and Social Psychology Review* 10(3), 214–234.
- Chan, M. S., and Albarracín, D. (2023). A meta-analysis of correction effects in science-relevant misinformation. *Nature Human Behaviour* 7, 1514–1525.
- Coleman, J. S. (1971). Control of collectives and the power of a collectivity to act. In *Social Choice*, ed. B. Lieberman.
- Crawford, V. P., and Sobel, J. (1982). Strategic information transmission. *Econometrica* 50(6), 1431–1451.
- Fearon, J. D. (1994). Domestic political audiences and the escalation of international disputes. *American Political Science Review* 88(3), 577–592.
- Fearon, J. D. (1995). Rationalist explanations for war. *International Organization* 49(3), 379–414.
- Gentzkow, M., and Shapiro, J. M. (2006). Media bias and reputation. *Journal of Political Economy* 114(2), 280–316.
- Jervis, R. (1978). Cooperation under the security dilemma. *World Politics* 30(2), 167–214.
- Kamenica, E., and Gentzkow, M. (2011). Bayesian persuasion. *American Economic Review* 101(6), 2590–2615.
- Leontief, W. (1941). *The Structure of American Economy, 1919–1929*. Harvard University Press.
- Nyhan, B., and Reifler, J. (2010). When corrections fail: the persistence of political misperceptions. *Political Behavior* 32(2), 303–330.
- Penrose, L. S. (1946). The elementary statistics of majority voting. *Journal of the Royal Statistical Society* 109(1), 53–57.
- Schelling, T. C. (1960). *The Strategy of Conflict*. Harvard University Press.
- Seneta, E. (1981). *Non-negative Matrices and Markov Chains*. Springer.
- Serra-Garcia, M., and Gneezy, U. (2021). Mistakes, overconfidence, and the effect of sharing on detecting lies. *American Economic Review* 111(10), 3160–3183.
- Shapley, L. S., and Shubik, M. (1954). A method for evaluating the distribution of power in a committee system. *American Political Science Review* 48(3), 787–792.
- Sobel, J. (1985). A theory of credibility. *Review of Economic Studies* 52(4), 557–573.
- Thaler, M. (2024). The fake news effect: experimentally identifying motivated reasoning using trust in news. *American Economic Journal: Microeconomics* 16(2).
- Walter, N., and Murphy, S. T. (2018). How to unring the bell: a meta-analytic approach to correction of misinformation. *Communication Monographs* 85(3), 423–441.
- Walter, N., and Tukachinsky, R. (2020). A meta-analytic examination of the continued influence of misinformation in the context of media. *Communication Research* 47(2), 155–177.
- Wood, T., and Porter, E. (2019). The elusive backfire effect: mass attitudes' steadfast factual adherence. *Political Behavior* 41, 135–163.

Empirical anchors, and the disputes they bound. Full sourcing for the historical cases is in
`information_warfare_claims_reference.md`.

- Attinasi, M. G., Boeckelmann, L., and Meunier, B. (2023). The economic costs of supply chain decoupling. ECB Working Paper 2839.
- Barbieri, K. (1996). Economic interdependence: a path to peace or a source of interstate conflict? *Journal of Peace Research* 33(1), 29–49.
- Benmelech, E., and Monteiro, A. (2025). The economic consequences of war. NBER Working Paper 34389.
- Bove, V., Elia, L., and Smith, R. P. (2017). On the heterogeneous consequences of civil war. *Oxford Economic Papers* 69(3), 550–568.
- Crozet, M., and Hinz, J. (2020). Friendly fire: the trade impact of the Russia sanctions and counter-sanctions. *Economic Policy* 35(101), 97–146.
- De Groot, O. J., Bozzoli, C., Alamir, A., and Brück, T. (2022). The global economic burden of violent conflict. *Journal of Peace Research* 59(2), 259–276.
- Federle, J., Meier, A., Müller, G. J., Mutschler, W., and Schularick, M. (2026). The price of war. *American Economic Review* 116(3), 791–827.
- Gartzke, E. (2007). The capitalist peace. *American Journal of Political Science* 51(1), 166–191.
- Gopinath, G., Gourinchas, P.-O., Presbitero, A., and Topalova, P. (2024). Changing global linkages: a new Cold War? IMF Working Paper 2024/076.
- Hegre, H., Oneal, J. R., and Russett, B. (2010). Trade does promote peace: new simultaneous estimates of the reciprocal effects of trade and conflict. *Journal of Peace Research* 47(6), 763–774.
- Keshk, O. M. G., Pollins, B. M., and Reuveny, R. (2004). Trade still follows the flag: the primacy of politics in a simultaneous model of interdependence and armed conflict. *Journal of Politics* 66(4), 1155–1179.
- Knack, S., and Keefer, P. (1997). Does social capital have an economic payoff? *Quarterly Journal of Economics* 112(4), 1251–1288.
- Novta, N., and Pugacheva, E. (2020). The macroeconomic costs of conflict. IMF Working Paper 2020/110.
- Oneal, J. R., and Russett, B. (1999). Assessing the liberal peace with alternative specifications: trade still reduces conflict. *Journal of Peace Research* 36(4), 423–442.
- Suhrke, A., and Samset, I. (2007). What's in a figure? Estimating recurrence of civil war. *International Peacekeeping* 14(2), 195–203.
- Wack, M., Duskin, K., and Hodel, T. (2024). Fact-checking the fact-checkers? Measuring the prevalence and reach of fact-checks. arXiv:2412.13280.
- World Bank (2003). *Breaking the Conflict Trap: Civil War and Development Policy*. Oxford University Press.
