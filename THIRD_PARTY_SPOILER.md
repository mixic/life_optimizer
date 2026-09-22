# The Third-Party Spoiler

Divide and conquer as a strategy: who profits from a conflict between others, why the
profiteer is so rarely made to pay, and what the Pareto-inferior Nash equilibrium is
actually for.

**Status.** Every result below is a theorem about the model in §2, proved from the
assumptions listed in §2.7. None is a measurement and none is evidence about any
particular war. §1 answers the definitional question — *what is a conflict?* — because the
answer turns out to determine what can be said about spoilers at all. §5 keeps the
historical record separate from the mathematics, and §7 says what has no measured
counterpart. The repository's rule applies unchanged: no number appears here without a
provenance.

---

## 1. What is a conflict?

The question sounds like semantics and is not. Three readings are in circulation, they come
apart, and which one is used determines whether a third party can be said to *cause* a war
or only to *profit from* one.

| Reading | Formal content | Verdict |
| --- | --- | --- |
| **Crossed interests** | There is an outcome both sides would prefer to the status quo | **Too weak to be a definition.** Almost every interaction has this, including pure coordination. It is not a pathology; it is the ordinary condition of trade |
| **An attack** | Someone uses force | **A symptom, not the thing.** An attack is what a conflict looks like when it materialises, and it can occur with no underlying conflict of interest at all (pure predation) |
| **A failure of equilibrium** | The cooperative outcome is **not a Nash equilibrium** | **This is the one that does the work** |

**Definition 1.** Let the interaction between two parties be the 2×2 game with payoffs
`(cc, cd, dc, dd)` in the convention `game.rs` already uses — mutual cooperation, cooperate
against defection, defect against cooperation, mutual defection — and define the **peace
margin**

```
σ  :=  cc − dc.
```

Then:

- **The interaction is at peace** iff `σ ≥ 0`: given that the other side cooperates, neither
  wants to defect, so `(C,C)` is a Nash equilibrium.
- **The interaction is in conflict** iff `σ < 0`: the cooperative cell is not an equilibrium.
  Each side defects not because it wants the other harmed but because defecting is its best
  response — the security dilemma, which `MULTIPOLAR_GAME.md` §4 and `game.rs`'s
  `Equilibrium::Dilemma` already describe.
- **The conflict has materialised** iff the realised play is `(D,D)` or a mixture. This is
  the outcome the simulator already measures with `Solution::efficiency_loss`.

**Theorem 0.** The three readings are pairwise independent.

**(i) Crossed interests without conflict.** A coordination game — `cc = 2, dc = 1, cd = 0,
dd = 0` — has a Pareto gap and `σ = 1 > 0`. The interests are crossed; there is no conflict.
*Proof:* `cc ≥ dc` and `cc ≥ cd`, so `(C,C)` is an equilibrium. ∎

**(ii) Conflict without crossed interests.** `cc = 0, dc = 1, cd = −1, dd = 0`. Here
`σ = −1 < 0`, so the interaction is in conflict, while mutual cooperation is *no better
jointly* than mutual defection — there is nothing for the two to be in conflict *about*.
*Proof:* `dc > cc` so `(C,C)` is not an equilibrium; and `cc + cc = dd + dd`, so no joint
surplus exists to divide. It is a conflict of position, not of interest. ∎

**(iii) An attack need not be a conflict.** If `cc < dd` and `cd < dd`, defection is
strictly dominant and the game is `Harmony`-in-reverse: there is no dilemma, only
predation. The "attack" reading would call it a conflict; Definition 1 calls it what it is —
an equilibrium in which one side is simply taking from the other, with no failure of
coordination to explain.

**Why this matters for the question asked.** Under reading one, virtually every pair of
blocs is "in conflict" and the word carries no information. Under reading two, a third party
can only be blamed for an act of violence it committed. **Under Definition 1, a third party
can be blamed for something precise and severe: having moved a dyad from `σ ≥ 0` to
`σ < 0`.** That is an act, it is observable in principle, and it is exactly what
divide-and-conquer consists of. The rest of this document is about the price of that act,
the revenue it generates, and why it goes unpunished.

**And the Pareto-inferior equilibrium.** `THEORY_OF_SPARING.md` §7d treats engineered
obsolescence as a 2×2 game whose Nash equilibrium is Pareto-inferior, and
`MULTIPOLAR_GAME.md` §4 treats the arms race the same way. Both describe the inferior
equilibrium as an *emergent failure* — something nobody chose. The central reframing of this
document is that **the Pareto-inferior equilibrium is also a product, and someone can be
paid to install it.** §3.7 draws the consequence: the simulator's existing aggregate
`efficiency_loss` is not only a welfare loss, it is a **revenue pool**, and its size is the
upper bound on what spoiling a given world is worth.

---

## 2. The model

### 2.1 Actors

Three roles, and the third is the subject:

- **Two parties**, `A` and `B`, playing the 2×2 game of §1 with the payoffs `game.rs`
  derives from their power gap, tension and interdependence.
- **A spoiler**, `C`, outside the dyad. `C` does not play the dyad's game. It has two
  instruments:

| Instrument | Effect | Cost |
| --- | --- | --- |
| **Manufacture grievance** (`m`) | Reduces the peace margin `σ`, by making each side believe the other's defection is more likely or less costly to answer | `c_m(·)`, increasing and convex |
| **Subsidise defection** (`t`) | Raises `dc` directly — arms, money, intelligence, guarantees | linear, unit cost 1 |

**Corollary 1.1 of §1 tells us these are perfect substitutes in their effect on `σ`**:
reducing `σ` by `δ` by either route leaves the dyad identically placed. The choice between
them is therefore *purely* a question of relative cost, which is why §3.1 can say something
sharp about which one a spoiler will reach for.

### 2.2 The harvest

If the dyad moves to `(D,D)`, the victims lose jointly what the simulator already calls the
efficiency loss. Write `Λ` for that quantity, normalised as `efficiency_loss` is, so
`Λ ∈ [0,1]`.

The spoiler captures a fraction `g ∈ [0,1]` of it. The decomposition matters and is the
subject of §3.7:

```
Λ   =   g·Λ        +     (1 − g)·Λ
        ─────            ─────────
        transferred      deadweight
        to the spoiler   lost to everyone
```

`g` has three sources, all of them ordinary economics: **terms of trade** (a war between two
suppliers raises the price the third gets), **relative position** (§3.5: destroying others'
power raises your share with no capability of your own), and **the war market itself** (arms,
finance, logistics, and the diplomatic price of ending it).

The spoiler's profit is

```
Π  =  g·Λ  −  t  −  c_m(Δσ)  −  c_d(d)  −  E[blowback],
```

where `d` is deniability investment (§3.3) and the last term is the blowback of §3.4.

### 2.3 The victims' deterrent, and why counting matters

If the victims retaliate, they do it jointly: sanctions, counter-coalitions, or the simple
threat of turning on the spoiler instead of each other. Their capacity to do so depends on
two things the spoiler can attack separately:

- **Their total weight** `P`, which a split does not change.
- **The number of parties `n` who must agree**, which a split does change.

Model the deterrent as

```
D(n, P)  =  P  −  κ·n(n − 1)/2,            κ ≥ 0,
```

so that coordination is charged on the pairwise links, and model retaliation through a
**quota rule**: the coalition acts iff the supporting weight reaches a threshold `q`.

Retaliation is funded out of the surplus the victims still have, so

```
K_max  =  k · S,        S = their available surplus,
```

which is the channel by which the induced conflict weakens the response to itself (§3.3).

### 2.4 The spoiler's own instrument, and its drift

The spoiler's proxy accumulates capability with support `s`. It turns on its sponsor with
hazard `h(s)`, increasing and convex, doing damage `H` if it does. The spoiler discounts at
rate `r`. Its payoff from the instrument is

```
Π_C(s)  =  B(s)  −  h(s)·H/(1 + r)  −  c(s),
```

with `B` concave and increasing (what the proxy delivers while it is pointed outward) and
`c` convex.

### 2.5 Exposure

Bloc `i`'s share of system power is `p_i = w_i / Σ_j w_j`. Its **exposure** to a conflict
between `A` and `B` is `e_i ∈ [0,1]`: how much of its own capability the conflict destroys.
For the spoiler, `e_C` is its entanglement with the victims — trade, finance, energy,
diaspora, geography.

### 2.6 Provenance

The dyad's payoffs come from `game.rs`, unchanged. `Λ` is that module's `efficiency_loss`,
unchanged. The spoiler's parameters — `g`, `κ`, `q`, `h`, `H`, `r`, `e` — are **declared
and illustrative**; §7 says which of them have measured counterparts and which do not.

### 2.7 Assumptions, listed

| # | Assumption | What breaks if it fails |
| --- | --- | --- |
| A1 | The passage indicator is nondecreasing in each member's realised contribution | Theorem 2 loses its monotonicity step |
| A2 | `c_m` increasing and convex; grievance can be manufactured at finite cost | Theorem 1(iii)'s interior optimum may not exist |
| A3 | The two instruments are perfect substitutes in their effect on `σ` | Corollary 1.1 becomes an inequality rather than an equivalence |
| A4 | Retaliation is funded from available surplus, `K_max = k·S` | Theorem 3(ii)'s self-weakening loop closes |
| A5 | Abuse hazard `h` increasing and convex; `B` concave | Theorem 4's comparative statics need the second-order signs |
| A6 | Power shares are relative and renormalised | Theorem 5(iii) — destruction as a substitute for growth — fails |
| A7 | Attribution is a probability `φ ∈ [0,1]` known to both sides | Theorem 3(iii)'s threshold becomes a set |

---

## 3. Results

### 3.1 Theorem 1 — the leverage ratio, and why propaganda beats subsidy

**Theorem 1 (leverage).** Let the peace margin be `σ` and let the spoiler face a fixed
operating cost `ε > 0`.

**(i)** The **minimum transfer** that creates conflict is

```
t*(σ)  =  max(0, σ) + ε,                                         (1)
```

so the **leverage ratio** — harvest per unit paid — is

```
R(σ)  =  g·Λ / (max(0, σ) + ε).                                  (2)
```

**(ii)** `R` is strictly decreasing in `σ` on `σ > 0`, and `lim_{σ→0⁺} R = gΛ/ε`.

**(iii) The optimum bifurcates on the cost of operating, and the first draft of this
theorem got it wrong.** Let `c_m(σ) = (a/2)(σ₀ − σ)²` for `σ ≤ σ₀`, so that moving a margin
is quadratic in the reduction achieved. Define the **critical operating cost**

```
ε_c  =  sqrt( g·Λ / (a·σ₀) ).                                    (3)
```

Then:

- **If `ε ≤ ε_c`, the optimum is the corner `σ* = 0`.** The spoiler drives the dyad all the
  way to the edge of conflict and leaves it there, paying nothing to push it over.
- **If `ε > ε_c`, the optimum is interior**, `σ* ∈ (0, σ₀)`, satisfying

```
a·(σ₀ − σ*)  =  g·Λ / (σ* + ε)²,                                 (4)
```

and **`dσ*/dε > 0`: a dearer operation is a subtler one.**

*Proof.* (i) and (ii) are above. For (iii), `Π(σ) = gΛ/(σ+ε) − c_m(σ)`, so
`Π′(σ) = −gΛ/(σ+ε)² + a(σ₀ − σ)`. At the corner `Π′(0) = aσ₀ − gΛ/ε²`, which is `≤ 0`
exactly when `ε ≤ ε_c`. At the other end `Π′(σ₀) = −gΛ/(σ₀+ε)² < 0` always. So when
`ε > ε_c` the derivative is positive at `0` and negative at `σ₀`, and by continuity there is
a root in between; `Π″ = 2gΛ/(σ+ε)³ − a`, so imposing `a ≥ 2gΛ/ε³` makes `Π` strictly
concave on the interval and that root the unique global maximum. For the comparative static
let `F(σ, ε) = a(σ₀−σ) − gΛ/(σ+ε)²`. Then `∂F/∂σ = Π″ < 0` and
`∂F/∂ε = 2gΛ/(σ+ε)³ > 0`, so `dσ*/dε = −(∂F/∂ε)/(∂F/∂σ) > 0`. ∎

**What the correction cost, and what survived.** The first draft claimed the optimum is
*always* interior, on the reasoning that the spoiler prefers to stop short and let the
dilemma fire. That reasoning is right about the *incentive* and wrong about the *arithmetic*:
the leverage term's derivative grows like `1/σ²` and beats a quadratic grievance cost as the
margin closes, so at a low enough operating cost the corner wins. Writing the test is what
exposed it — the bisection returned the corner, and the first-order condition it was supposed
to satisfy was off by a factor of four hundred. The corrected result is strictly better,
because it is a **bifurcation**: there are two kinds of spoiler, and which one you face is
decided by how expensive the operation is to run.

**Corollary 1.0 (brazen and subtle).** Cheap spoiling is brazen: it drives margins to zero
and leaves the dyads there. Expensive spoiling is subtle: it stops short, and the further
short the more it costs. That is a testable difference in the *style* of operations, and it
is not a difference in the spoiler's character.

**Corollary 1.1 (which instrument).** By A3 the two instruments move `σ` identically, so the
spoiler uses whichever is cheaper per unit of `σ` reduced. Narrative is cheaper exactly when
claims are cheap, and claims are cheap when the audience's verification rate is low. In the
notation of `INFORMATION_WARFARE.md` §2.3, that is a low `λ`. Therefore:

> **The cheaper the truth is to check, the more a spoiler has to spend on arms; the dearer
> it is to check, the more it will spend on stories.** Weak verification does not merely
> permit disinformation — it *substitutes propaganda for matériel* in the production of war.

This is where the two documents join. `INFORMATION_WARFARE.md` modelled the manufacture of
belief for its own sake; here belief is an *input* to a third party's production function,
and its price relative to guns decides the technology.

**Corollary 1.2 (target selection).** Since `R` falls in `σ`, the spoiler ranks dyads by
`σ` and picks the smallest. Three observable implications, and they are the ones worth
testing:

1. Spoiling concentrates on dyads with a **narrow** margin — recent or latent disputes,
   not settled ones. A settled border is expensive to reopen.
2. The spoiler prefers dyads where an **identity marker** already exists, because a marker
   that is hereditary, hard to verify, and already salient is the cheapest way to move `σ`.
   Identity is not the *cause* of these conflicts on this account; it is the cheapest
   available lever, which is a different and more uncomfortable claim.
3. The spoiler is indifferent to which side is "right", because `σ` is symmetric in the
   parties. Any model in which spoilers consistently favour one side is a model with
   something else in it — a genuine preference, a co-religionist constituency, or a
   domestic audience.

### 3.2 Theorem 2 — the weapon is the enmity, not the division

This theorem also had to be corrected, and the correction is the more interesting result.

**Theorem 2.** Let a coalition act iff its supporting weight reaches a quota `q`, with the
passage indicator nondecreasing in each member's contribution (A1). Split member `k` of
weight `w` into `parts` pieces of weight `w/parts` each, and consider the two ways the split
can be arranged.

**(i) Fragmentation alone has no fixed sign, and can arm the target.** If the parts decide
**independently**, expected supporting weight is preserved and the variance of the total
strictly falls. The passage probability therefore *falls* when the coalition's expected
weight sits below the quota — where the spread was the only thing that occasionally carried
it over — and **rises** when it sits above it, where the spread was the only thing that
occasionally held it under. **Dividing a rival is not automatically good for the divider.**

**(ii) Only the estrangement reliably disarms it.** If the spoiler additionally arranges that
the parts **can never both support**, then in any state at most one part contributes, so the
member's realised contribution is capped at `w/parts` and is weakly lower in every state. The
passage probability weakly falls, strictly wherever the undivided member's full weight was
pivotal with positive probability.

**(iii)** The gain is increasing in `w`: a larger member occupies more pivotal states, so the
largest member is the one worth dividing.

**(iv)** Splitting a member that was already unreliable and individually below the quota buys
nothing: that coalition could not act in the first place.

*Proof.* (i) Independence preserves the mean `Σ wᵢpᵢ` and strictly reduces the variance for
`0 < p < 1` and `parts ≥ 2`. The passage indicator is a step at `q`, so lowering the spread
of the total raises `Pr[total ≥ q]` when the mean exceeds `q` and lowers it when the mean
falls short. (ii) In any state the undivided body carries the whole of `w` if the member is in
favour; the estranged member carries at most its largest part, `w/parts`, because the parts
cannot both be in favour. Every other member is untouched, so the total is weakly lower in
every state, and A1 gives a weakly lower passage probability; it is strictly lower on the
states where the original member was pivotal and the total now falls below `q`. (iii) The
pivotal set grows with `w`, so the probability fall grows with it. (iv) If the passage
probability was already zero for the coalitions that matter, no division can reduce it
further. ∎

**Why this correction matters more than the theorem it replaced.** The first draft said that
dividing a member caps its contribution, full stop — and that is true only of the estranged
version. The independent version, which is what "a bloc splits in two" ordinarily means, does
something quite different: it turns a body of weight `w` into two bodies that each decide for
themselves. That is a reduction in correlation *without* a reduction in weight, and against a
coalition whose combined weight comfortably exceeds the quota it makes the coalition **more**
reliable than before, not less.

So the model says something sharper than "divide and conquer":

> **Fragmentation is not the weapon. What a coalition needs is the ability to act together,
> and independence is not enough to destroy it — independence removes the spread, while the
> estrangement removes the possibility.** A spoiler that merely splits a rival may hand it a
> more dependable coalition. What it must do instead is make the parts *unable to be on the
> same side*, which is to say it must manufacture an enmity, not a border.

That is the formal content of the observation this question started from. What is done to a
divided society is not the drawing of lines; it is the setting of the parts against each
other. On this model, line-drawing alone is as likely to help the target as to hurt it — and
the historical cases in §5 have to be read with that in mind, because a partition that
*worked* is evidence that estrangement was achieved, not that the partition did the work.

**Corollary 2.1 (the coordination-cost channel).** Under the deterrent
`D(n, P) = P − κ·n(n−1)/2` of §2.3, splitting one victim into two reduces `D` by exactly
`κ·n` — the new links — for every `κ > 0`, while leaving `P` unchanged. So where deterrence
requires coordination, division is *always* profitable, and the spoiler's return is
increasing in the cost of coordination. A world with cheap communication is a world with
less to gain from dividing.

### 3.3 Theorem 3 — impunity is derived, not assumed

The empirical question is why the conflict parties are punished and the sponsor is not.
Three mechanisms, each provable, and each *created by the spoiling itself*.

**Theorem 3.** Let there be `n ≥ 2` victims, attribution probability `φ`, punishment cost
`K` borne by each punisher, and shared deterrent benefit `B`.

**(i) Punishment is a public good among the victims.** Each victim punishes iff
`φ·B ≥ K`, giving the individual threshold

```
φ*  =  K / B.                                                    (4)
```

Each punisher captures only its own `B`, so the number who punish is below the number for
whom joint punishment would pay. The shortfall is the standard free-rider term and its size
is `(n − 1)·φ·B` per punisher.

**(ii) The induced conflict weakens the retaliation it provokes.** Under A4, the victims'
maximum credible punishment is `K_max = k·S`, and the induced conflict reduces their surplus
`S` by `Λ`. So

```
K_max  ⟶  k·(S − Λ),        and therefore        φ* falls in feasibility terms
                                                  while the ABSOLUTE deterrent falls,
```

because the same threshold now has to be met with less to spend. The victim's capacity to
answer falls by exactly `k·Λ` — **the spoiler's own damage is what disarms the response to
it.**

**(iii) There is an attribution threshold below which no punishment occurs at all.**
Combining: the spoiler is deterred only if `φ·n·K_max > g·Λ`. Given a `K_max` reduced by
(ii), this is a strictly harder condition, and for every parameterisation there is a
`φ† > 0` with no punishment in equilibrium whenever `φ < φ†` — **the spoiler is guilty and
nobody acts.**

**(iv) Deniability investment has a target, not a maximum.** With `c_d` convex and
decreasing in `φ`, the spoiler's optimum satisfies `−c_d′(φ) = n·K_max` on the region where
deterrence binds, and it is exactly zero where `φ < φ†` already holds. So **a spoiler facing
weak victims buys no deniability at all.**

*Proof.* (i) The punishment decision is individual and the benefit is collective, so the
private first-order condition (4) is below the social one; the gap is the sum of the other
victims' marginal benefits. (ii) Substitute `S − Λ` for `S` in `K_max = kS`; the reduction is
`kΛ`. (iii) Let `G(φ) = φ·n·k(S − Λ) − gΛ`; `G` is increasing and linear in `φ`, so
`φ† = gΛ / (n·k(S − Λ))` is the unique root, and `G(φ) < 0` — no punishment — for `φ < φ†`.
(iv) The spoiler minimises `c_d(φ) + 𝟙[φ ≥ φ†]·(expected punishment)`. Below `φ†` the second
term is zero, so the objective is strictly increasing in deniability effort and the optimum
is the corner `d = 0`. Above `φ†` the first-order condition
`−c_d′(φ) = n·K_max` applies. ∎

**This is the answer to the question the document is organised around.** The spoiler is not
punished because successful spoiling destroys both the **motive** to punish it — by
depleting the surplus that would have funded the response (ii) — and the **means** of
identifying it (iii), and because even an identified spoiler faces victims who each prefer
to let the others do the punishing (i). Impunity is not an institutional failure, a gap in
international law, or a moral lapse. **It is the equilibrium outcome of a successful
spoiling operation**, which is why it is so reliable.

**Corollary 3.1 (when a sponsor *is* punished).** Punishment requires `φ ≥ φ†`, i.e. one of:
high attribution (the operation is openly run by a state, as with the Contras), high
`n·k·(S − Λ)` (many victims who are rich and unexhausted), or low `gΛ` (little to gain).
Every one of these is a condition the spoiler can act on, which is why the observed pattern
is not "sponsors go free" but "sponsors who are *bad at it* go free less often".

### 3.4 Theorem 4 — blowback is the cost of capital, not a mistake

The proxy's loyalty is not a parameter the spoiler controls. Model it as §2.4 does.

**Theorem 4.** Let `B` be concave and increasing, `h` increasing and convex, `c` convex.

**(i)** The optimal support `s*` is interior and satisfies

```
B′(s*)  =  h′(s*)·H/(1 + r)  +  c′(s*).                          (5)
```

**(ii) A more patient spoiler supports less, and an impatient one supports more:**

```
ds*/dr  <  0.
```

**(iii)** If the hazard is *unknown* and the spoiler is ambiguity-averse — maximising the
worst case over a set of hazards — then support is **strictly lower** than under
expected-value maximisation at the mean hazard.

**(iv)** For every `H` there is a support level at which the strategy's ex-post payoff is
negative, and a spoiler who discounts at `r` under-weights precisely that region. **The
blowback is not a failure of the model; it is the model's prediction for a principal whose
cost of capital is high enough.**

*Proof.* (i) `Π_C′ = B′ − h′H/(1+r) − c′`; `Π_C″ = B″ − h″H/(1+r) − c″ < 0` under the stated
curvature, so the root of (5) is the unique maximum and it is interior because
`B′(0) > h′(0)H/(1+r) + c′(0)` for some parameter region and `B′` decreases while the
right-hand side increases. (ii) Put `G(s, r) = B′(s) − h′(s)H/(1+r) − c′(s)`. Then
`∂G/∂s = Π_C″ < 0` and `∂G/∂r = −h′(s)H/(1+r)² < 0` (since `h′ > 0`). Hence
`ds*/dr = −(∂G/∂r)/(∂G/∂s) = −(−)/(−) < 0`. (iii) Worst-case maximisation over a hazard set
`ℋ` gives the first-order condition with `h` replaced by `h̄ = sup ℋ`, and the solution is
monotone decreasing in the hazard used, so support falls relative to the mean. (iv) Ex post
payoff `B(s) − H − c(s)` is negative for `H > B(s) − c(s)`, which is attainable for any
`B` since `H` is unbounded. ∎

**Why this is worth a theorem.** The instinctive reading of the mujahideen-to-9/11 sequence
is that somebody blundered — that the sponsor failed to foresee. Part (ii) says the opposite:
**under-supporting the tail is what a rational impatient principal does**, and part (iv) says
the tail always exists. A spoiler is not gambling because it is foolish. It is accepting a
convex tail risk in exchange for a flow of present gain, at a rate set by its discount
rate. The policy implication is uncomfortable and follows from (ii) rather than from any
parameter: **there is no level of sophistication that makes proxy sponsorship safe — only
patience makes it safer.**

Note what (iii) does too. Ambiguity aversion *reduces* support, which means the blame for
blowback attaches less to ignorance than to **confidence**: a principal that thinks it knows
the proxy's hazard supports it harder than one that admits it does not.

### 3.5 Theorem 5 — the winner is the least exposed, not the strongest

Power shares are relative and renormalised each year, so for `i ≠ j`,

```
∂p_i / ∂w_j  <  0   and more precisely   p_i = w_i / Σ_k w_k.
```

**Theorem 5.** Let a conflict between `A` and `B` destroy a fraction of their power and let
`e_i` be bloc `i`'s exposure.

**(i)** Bloc `i`'s share gain is **decreasing in its own exposure** `e_i` and *independent of
its own power* in proportional terms.

**(ii)** With no destruction of `i`'s own power at all, `p_i` rises by the factor
`Σw / (Σw − δ)` **regardless of the size of `i`** — so the *proportional* gain from a war
between others is the same for every bystander.

**(iii) Destruction is a perfect substitute for growth.** `∂p_i/∂w_j < 0` for every `j ≠ i`,
so destroying a rival's capability raises `i`'s share by exactly as much as growing `i`'s own
would have — with no capability produced.

**(iv)** The strategic consequence: the profit-maximising spoiler is the bloc for which
`e_C` is least, and the set of such blocs is *not* the set of the strongest.

**(v)** Since the proportional gain is common to all bystanders and the cost `t*` does not
depend on the spoiler's size, the **rate of return is the same for every spoiler** — so the
strategy is selected by whichever bloc has the **worst outside option**, not by the largest.

*Proof.* (i)–(ii) `p_i = w_i/Σw`; reducing the denominator by `δ` raises every `p_i` by the
same factor `Σw/(Σw−δ)`, while reducing `w_i` itself lowers `p_i`. So the net gain is falling
in `e_i` and the pure-bystander gain is common. (iii) `∂p_i/∂w_j = −w_i/(Σw)² < 0` and
`∂p_i/∂w_i = (Σw − w_i)/(Σw)² > 0`; these are the two ways to raise `p_i`, and both are
available. (iv) `Π = gΛ − t*(σ) − E[blowback]`, with the first term increasing in `e_C`'s
reciprocal through `g` (an entangled spoiler's terms-of-trade gain is partly its own loss),
so `Π` is decreasing in `e_C`. (v) The proportional gain is common by (ii) and `t*` depends
only on `σ`, not on the spoiler, so `R` is common; the choice is therefore made by the
outside option. ∎

**The uncomfortable implication of (iii).** In a model where power is *relative*, hurting a
rival and growing yourself are substitutes, and the former requires no capability of your
own. That is the structural reason a bloc with a stagnant economy and a modest army can
still change the world's distribution of power. It does not need to build; it needs to
break.

**And (v) is a prediction rather than a platitude.** If the rate of return is common, spoiling
should be observed among blocs whose *growth* has stalled — declining incumbents and trapped
middle powers — and not among the frontier growers, who have a better use for the same
money. That is a checkable claim about who runs proxies, and it is not the claim that "the
strong do it".

### 3.6 Theorem 6 — profit and impunity are governed by the same variable

This is the document's central result. The first version of it **was false**, and the
correction is recorded here rather than quietly made, because the failure mode is
instructive.

**Theorem 6.** Let the spoiler's capture rate fall with its entanglement and the victims'
punishment capacity rise with it:

```
g(e)   =  g₀·(1 − e)          the harvest, falling in exposure
K_max  =  k·S·e               the punishment the victims can inflict
```

with `g₀ ∈ (0,1]`, and write `T = t*(σ) + ε + E[blowback]` for the spoiler's total cost of
operating, `H = g₀Λ` for the harvest at zero exposure, and `X = H − T` for the maximum
profit, which must be positive for the strategy to be worth considering.

**(i) Both conditions move with `e`, and in opposite directions.** Profit
`Π(e) = g₀(1−e)Λ − T` is strictly decreasing in `e`; the deterrent
`φ·n·K_max(e) = φ·n·k·S·e` is strictly increasing in it. So the profitable region is an
interval `[0, ē)` and the unpunishable region is an interval `[0, e†)`, both anchored at
zero exposure:

```
ē  =  1 − T/H,                    e†  =  H / (φ·n·k·S + H).            (6)
```

**(ii) At the profit-maximising exposure the punishment is exactly zero.** `Π` is maximised
at `e = 0`, where `K_max = 0`. **The spoilers who do best are never punished at all** — the
weak claim is a theorem.

**(iii) The strong claim — that the profitable region lies inside the unpunishable one —
holds if and only if `ē ≤ e†`, which is equivalent to**

```
X / T   ≤   H / (φ·n·k·S).                                            (7)
```

That is: the *containment needs a condition*. It holds when the maximum profit multiple is
no greater than the ratio of the harvest to the victims' punishment capacity — roughly,
**when the stakes are modest relative to what the victims can bring to bear.** It fails for
a rich prize and a weak, entangled set of victims.

**(iv) When the condition fails there is a band of exposures in which the spoiler would
profit and is deterred anyway** — `(e†, ē)`, nonempty exactly when the containment fails.
This is the mirror of impunity, and it is what the model has to say about the *effectiveness*
of punishment.

**(v) The substantive reading — punishment only ever closes the tail.** The most profitable
spoilers sit near `e = 0`, where `K_max = 0` and there is nothing to punish. Raising the
victims' capacity `φ·n·k·S` lowers `e†` and so widens the deterred band *downwards*, but it
cannot reach the region where the profit is largest. **Deterrence against spoilers is
structurally limited to the marginal ones**: it works on the operation that was only just
worth running, and never on the one that was worth running comfortably.

**(vi) The correction, and why it matters.** The first draft asserted the containment
unconditionally, on the reasoning that "low entanglement helps the harvest and hurts the
victims". That reasoning is sound and the conclusion does not follow from it: two monotone
functions of the same variable are not automatically nested, and whether they are depends on
their levels, not their slopes. Writing the test is what exposed it. The surviving result is
the one worth having, and it is sharper than the false one:

> **Impunity is not a property of spoiling. It is a property of spoiling *successfully*.**
> The relationship between profit and impunity is monotone but not total: whether the
> profitable region lies inside the region where punishment cannot bite is a condition on
> stakes and capacities, not an identity.

*Proof.* (i) Immediate from the definitions: `Π′(e) = −g₀Λ < 0` and `d(φnkSe)/de = φnkS > 0`.
Setting `Π(e) = 0` gives `ē = 1 − T/H`; setting `φnkSe = g₀(1−e)Λ = H(1−e)` and solving gives
`e† = H/(φnkS + H)`. (ii) `K_max(0) = 0`, so the deterrent vanishes where profit is largest.
(iii) The profitable region `[0, ē)` is contained in `[0, e†)` iff `ē ≤ e†`. Substituting:
`1 − T/H ≤ H/(φnkS + H)`. Multiply through by `H(φnkS + H) > 0`:
`(H − T)(φnkS + H) ≤ H²`, i.e. `X·φnkS + X·H ≤ H²`, i.e.
`X·φnkS ≤ H² − X·H = H(H − X) = H·T`. Divide by `T·φnkS > 0`: `X/T ≤ H/(φnkS)`, which is
(7). (iv) `ē > e†` is the strict complement of `ē ≤ e†`, and the band `(e†, ē)` is nonempty
exactly then; inside it `Π > 0` by (i) while `φnkSe > H(1−e)`, so the spoiler would profit
and does not act. (v) `∂e†/∂(φnkS) < 0` from the closed form, while `ē` does not depend on
`φnkS` at all, so raising the victims' capacity moves the *lower* edge of the band and leaves
the profit-maximising exposure untouched. (vi) Arithmetic. ∎

**Corollary 6.1.** Where the condition holds, impunity is complete and the observed
population of punished sponsors is selected on **incompetence** — on having chosen a prize too
rich or victims too capable for their own entanglement. Where it fails, punishment bites, but
only on the tail. The two regimes are distinguishable in data by whether punished and
unpunished sponsors differ in exposure, which is P5.

### 3.7 Corollary — the Pareto-inferior equilibrium, and who pays for it

The user-facing question was how this strategy "copes with" the Pareto-inferior Nash
equilibrium. The answer is that it does not cope with it; **it produces it, and bills for
it.**

Write the dyad's efficiency loss `Λ` as the simulator already computes it. Then:

```
Π_spoiler  =  g·Λ  −  t*  −  E[blowback]
World loss =  Λ  +  t*  +  E[blowback]  −  g·Λ
           =  (1 − g)·Λ  +  t*  +  E[blowback].
```

Three regimes, all reachable:

| `g` | What it means | World welfare |
| --- | --- | --- |
| `g = 1` | The spoiler captures the entire loss | Unchanged in aggregate — **a pure transfer**, and the war is a transaction |
| `0 < g < 1` | Part captured, part destroyed | Strictly worse for the world |
| `g = 0` | Nothing captured | Pure destruction; the spoiler would not do it unless it valued the harm itself |

The spoiler's private condition is `gΛ > t* + E[blowback]`, which **ignores the deadweight
entirely**. So the spoiler spoils in cases where the world is strictly worse off, and the
gap between the private and social condition is exactly `(1−g)Λ`. This is the same externality
as `INFORMATION_WARFARE.md` Theorem 5(iv), in a different costume: **the private return to
manufacturing conflict exceeds the social one, by the part of the damage nobody collects.**

**The reframing this yields for the existing simulator.** `efficiency_loss` is currently
reported as a welfare statistic — "how much is being lost to the security dilemma". Under
this model it is also a **revenue pool**: the aggregate `Λ` across all dyads is the total
annual amount of conflict-rent available to be captured, and the model's own reported
values put a number on what spoiling that world is worth. §8's `--spoiler` mode reports both
readings side by side, and the difference between them is the deadweight.

---

## 4. Predictions, and how they could be refuted

| # | Prediction | What would refute it |
| --- | --- | --- |
| P1 | Spoiling concentrates on dyads with a **narrow** peace margin `σ`, not on the deepest hatreds and not on settled disputes | Sponsorship distributed uniformly across dyads, or concentrated where the margin is widest |
| P2 | The instrument is **narrative** where verification is weak and **matériel** where it is strong (Corollary 1.1) | Armed sponsorship that does not vary with the strength of the target audience's independent media, courts or inspection capacity |
| P3 | Spoilers are recruited from blocs with **stalled growth**, because the rate of return is common and the outside option differs (Theorem 5(v)) | Sponsorship concentrated among the fastest-growing powers |
| P4 | The divided party is the **largest** member of the coalition being broken (Theorem 2(iii)) | Division aimed at small members, or coalitions broken without targeting the largest |
| P5 | Punishment, where it occurs, correlates with the sponsor's **entanglement** with the victims rather than with the severity of the crime, and punished sponsors sit in the band `(ē, e†)` of Theorem 6(iv) rather than being drawn at random from the population of sponsors | Severe, well-attributed operations going unpunished against highly entangled sponsors; or punished and unpunished sponsors indistinguishable in exposure |
| P6 | Proxy support is **increasing in the sponsor's discount rate** (Theorem 4(ii)), and so is the frequency of blowback | Patient sponsors supporting proxies harder than impatient ones |
| P7 | Sponsors facing weak victims buy **no deniability** (Theorem 3(iv)) | Heavy covert apparatus deployed against victims with no capacity to retaliate |

P1 and P5 are the load-bearing ones. P1 says the strategy is about *cheapness* rather than
about *hatred*, which is a claim that can be got wrong. P5 says impunity is a selection
effect rather than an absence of law, which is a claim that can be got wrong too.

A prediction the model does **not** make, stated because the prose invites it: that third
parties trigger wars. Theorem 1 says a spoiler need not trigger anything — it needs to make
the dyad's own dilemma cheap enough to fire. The difference matters, because it means the
observable evidence of spoiling is often *absence*: the propaganda, the money and the arms
are visible, and the "trigger" is a border incident that the two victims genuinely blame each
other for.

---

## 5. The historical record

*(This section is being completed from sourced material. It will report, for each case, what
the third party is documented to have done as distinct from what is alleged, what benefit is
on the record as distinct from inferred, whether any punishment occurred, whether the
instrument later turned on its sponsor, and exactly which parts are contested. The
Yugoslavia material in particular will separate the documented administrative history — the
creation of the "Muslim" nationality category, the constitutional changes, the external
actors and their dates — from the interpretive claim that the war was engineered from
outside, and will cite the strongest versions of both readings.)*

---

## 6. Relation to existing work

- **The spoiler literature.** Stedman's "spoiler problem" (1997) identifies actors who
  wreck peace processes. This document is about a spoiler of a different kind — one who
  wrecks a *peace* rather than a settlement, and whose objective is the wreckage. The
  vocabulary is borrowed; the object is not the same.
- **Divide and rule.** The historiography is large and contested, and the formal content
  has been thin. Theorem 2 supplies the missing mechanism: the strategy works on
  **correlation**, and its payoff is the fall in the probability of collective action, not
  any change in aggregate weight.
- **The security dilemma.** Jervis (1978), and `MULTIPOLAR_GAME.md` §4. The contribution
  here is to treat the dilemma's inefficiency as a *rent* that a third party can install and
  harvest, rather than as a tragedy nobody chose.
- **Engineered obsolescence.** `THEORY_OF_SPARING.md` §7d already treats a
  Pareto-inferior Nash equilibrium as something a firm can produce deliberately. Theorem 6
  and §3.7 are the same structure at the level of blocs, which is the reason the two
  documents should be read together.
- **Olson (1965)** on the exploitation of the large by the small, and the public-good
  structure of collective action, supply Theorem 3(i) directly.
- **Banzhaf (1965) and Penrose (1946)** supply the quota machinery of Theorem 2, reused from
  `INFORMATION_WARFARE.md` Theorem 0.
- **Conflict economics.** The measured cost of war, the external-support-and-duration
  literature, and the arms-transfer evidence are the empirical counterpart of `Λ` and are
  reported with sources in §7.

**The novelty claim, stated modestly:** defining conflict as a failure of equilibrium rather
than as crossed interests or as violence (§1), and then treating the resulting inefficiency
as a **product with a price** — so that (a) leverage is harvest over margin and grievance is
therefore cheaper than subsidy (Theorem 1), (b) division works by destroying correlation
(Theorem 2), (c) impunity is an equilibrium consequence of successful spoiling rather than
an institutional gap (Theorem 3), (d) blowback is the cost of an impatient principal rather
than a failure of foresight (Theorem 4), (e) destruction is a perfect substitute for growth,
so the strategy needs no capability at all (Theorem 5), and (f) profit and impunity are
monotone in the same variable, so impunity attaches to *successful* spoiling and the caught
are selected on incompetence (Theorem 6).

It should also be said what this document is not. It is not an account of any particular
war, and §5 does not treat the interpretive claim that a given conflict was engineered from
outside as established. Theorem 1 is deliberately weaker than that claim: a spoiler need not
trigger anything, only make an existing dilemma cheap enough to fire — which means the
evidence of spoiling is often the money and the propaganda rather than the trigger.

---

## 7. What is not established, and what would have to be measured

- **The theorems are about the model.** Theorems 1–6 are exact given §2.7's assumptions.
  They say nothing directly about any war.
- **`g`, the capture rate, has no measured counterpart for any conflict.** Nobody has
  estimated what fraction of a war's efficiency loss accrues to a third party, and the
  national-accounts data do not identify it. This is the model's most important unknown and
  it is the one that decides whether `g = 1` (a transfer) or `g ≈ 0` (destruction).
- **Exposure `e_C` is measurable in principle** — trade, finance, energy and diaspora links
  are observable — and the model's P5 turns on it. Nobody has assembled the panel.
- **The abuse hazard `h(s)` is unmeasured.** The probability that a proxy turns on its
  sponsor, as a function of support, has no estimate; the historical record is a handful of
  cases with no denominator, which is exactly the problem `INFORMATION_WARFARE.md` §6.1
  identifies for the detection hazard. The same structural obstacle applies: the number of
  proxies that *did not* turn is not recorded.
- **The coordination cost `κ` is unmeasured**, though alliance-size and transaction-cost
  literatures bear on it. P4's strength depends on it.
- **And the counterfactual is unobservable by construction.** "This war would not have
  happened without the third party" is not a measurable quantity; it is the model's
  perturbative claim and nothing more.

The honest summary: **the mechanism is fully specified and its comparative statics are
exact; the two parameters that decide whether the strategy is a transfer or a destruction —
`g` and `h` — have no measured values at all.**

---

## 8. Reproduction

```
cargo test -p multipolar_sim spoiler::          # the theorems, as invariants
cargo run -p multipolar_sim -- --spoiler        # the strategy, run
python tools/plot_spoiler.py                    # the figures
```

`multipolar_sim/src/spoiler.rs` implements §2 and its test module asserts the theorems as
invariants: the leverage ratio's monotonicity and its divergence at the margin, the
instrument-choice equivalence, the cap on the joint contribution under division, the
attribution threshold and the deniability corner, the discount-rate comparative static, the
common-cause containment of Theorem 6, and the transfer/deadweight decomposition.

**Backwards compatibility is a hard requirement and is tested.** The spoiler layer is off by
default and changes nothing unless `--spoiler` is passed; the mode is additive and the
existing simulation, its figures and its exports are untouched.

---

## References

*(Theory references are listed here; the historical and empirical sources are cited inline
in §5 and §7, and the sourced case table is in the companion file.)*

- Banzhaf, J. F. (1965). Weighted voting doesn't work: a mathematical analysis. *Rutgers Law Review* 19, 317–343.
- Jervis, R. (1978). Cooperation under the security dilemma. *World Politics* 30(2), 167–214.
- Olson, M. (1965). *The Logic of Collective Action*. Harvard University Press.
- Penrose, L. S. (1946). The elementary statistics of majority voting. *Journal of the Royal Statistical Society* 109(1), 53–57.
- Stedman, S. J. (1997). Spoiler problems in peace processes. *International Security* 22(2), 5–53.


