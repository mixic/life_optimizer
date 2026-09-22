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

This section reports, for each case, what the third party is **documented** to have done as
distinct from what is alleged, what benefit is **on the record** as distinct from inferred,
whether any punishment occurred, whether the instrument later turned on its sponsor, and
which parts are contested.

**How the record is read.** Three notation rules, applied throughout:

* A citation is to a primary document wherever one exists — a treaty or resolution text, a
  judgment, a parliamentary record, a declassified file, an official inquiry. Secondary and
  participant accounts are labelled as such.
* **"No sourced figure located" means exactly that.** No figure appears below unless a named
  source states it. Where a popular claim is contradicted by the primary text, the primary
  text is quoted and the claim is named as contradicted.
* Where two official bodies differ on a date, a number, or a finding, **both are reported**.
  Reconciling them is not this document's business and would hide the disagreement.

**Two records are kept apart.** The *administrative and legal record* — what a court found,
what a resolution says, what a treaty requires, what a statute number is — is a different
kind of thing from the *interpretive claim* that a conflict was engineered from outside. The
first can be checked. The second usually cannot, and this section says so rather than
dressing it as the first.

### 5.1 Four corrections to this document's own working assumptions

Recorded because the method is part of the result, and because each was an assumption that
would have produced a confident false sentence.

**(i) The framework was expected to find a court holding a sponsor liable.**
It does not, in the case where it most looked as though it might. A **peace margin and
harvest are declared; the legal record is not**, and the record's answer is the opposite of
the expected one. France's responsibility in Rwanda is a finding of the **Duclert report**
(*La France, le Rwanda et le génocide des Tutsi (1990-1994)*, remis au Président de la
République le 26 mars 2021, pp. 971–972): *"La recherche établit donc un ensemble de
responsabilités, lourdes et accablantes… Elles sont de nature politique."* It is expressly
**political, not legal**, and the same report uses *complice* only to reject it. The courts
never reached the merits: **Tribunal administratif de Paris, jugement n° 2309845/4-1 du
14 novembre 2024** declined jurisdiction — the acts *"ne sont pas détachables de la conduite
des relations internationales de la France"* — the appeal was rejected (**CAA Paris, arrêt
n° 25PA00083, 28 mars 2025**) and the **Conseil d'État, n° 504595, 18 mars 2026** refused
leave. **Duclert and the courts are not in conflict about a finding of fault; the courts
never reached the merits.** These are two different kinds of "no".

**(ii) A parliamentary claim that turned out not to be in the statute.** The **Indian
Councils Act 1909 (9 Edw. 7 c. 4)** does **not** contain the phrase "separate electorate",
nor "electorate", "communal", "Muhammadan", "Muslim", "Hindu", "Sikh", nor the word
"religion" at all. Section 6 is a bare enabling power to **make regulations**. The communal
electorate was created by regulation under that section, not by the Act. The phrase itself
appears in the parliamentary record not in the statute but in **Morley's answer to Curzon,
HL Deb 4 May 1909, vol. 1, c.756**, reading the Viceroy's telegram: *"Our intention was that
Mahomedans should have, by means of separate electorates, a number of seats closely
approaching that to which their numerical proportion in the population would entitle them."*
**The Gazette notification containing the 1909 regulations could not be located, and no
citation for it is offered.**

**(iii) Three dates and one report title that do not exist.** The **Smithwick Tribunal**
report was published **3 December 2013**; the **de Silva review** (HC 802) on **12 December
2012**; the **Ballymurphy** deaths were in **August 1971** — and **no official report named
"Cassel" exists**. The phrase "connectivity, coincidence and correlation" could not be
sourced anywhere. All four were working assumptions; all four were wrong.

**(iv) The dossier's own arithmetic error, kept rather than quietly fixed.** The Angola
draft attributed the vote "13 to none, with 2 abstentions" to **S/RES/545 (1983)**. That vote
belongs to **S/RES/546 (1984)**, adopted at the 2511th meeting from S/16247/Rev.1.
**S/RES/545 (1983), 20 December 1983, 2508th meeting, from S/16226, was adopted 14–0–1 with
the United States abstaining.** The substantive point survives and is sharper with the
correction: a US abstention is weaker censure than a unanimous vote, which is the pattern.

### 5.2 The cases

Read the table as the record's answer to four questions, and note where the answer is
"nothing located" — which is a result and not a blank.

| Case | Documented conduct *(primary source)* | Documented benefit | Punishment | Blowback |
| --- | --- | --- | --- | --- |
| **Yugoslavia / Bosnia, Kosovo** (1991–99) | FRY "made its considerable military and financial support available to the Republika Srpska" (*Bosnia v. Serbia*, ICJ, 26 Feb 2007, press release 2007/8). Arms embargo S/RES/713 (1991); ICTY indictments IT-02-54, IT-04-81 | **No sourced figure** for value or volume | ICJ: **declaration only** — "(9) … the findings constitute appropriate satisfaction, and … the case is not one in which an order for payment of compensation would be appropriate". Milosevic died before judgment (11 Mar 2006); Perišić, the most senior FRY officer, **acquitted on appeal** (28 Feb 2013). UN sanctions lifted by S/RES/1021 (1995), 1074 (1996) | **None confirmed.** Foreign-mujahideen and "green light" claims not verified |
| **Iran–Iraq** (1980–88), third-party supply to both sides | The **Riegle** material's actual subject: "U.S. **Chemical and Biological Warfare-Related Dual Use Exports** to Iraq and their Possible Impact on the Health Consequences of the Gulf War" (hearing S. Hrg. 103-900 and committee print, 25 May 1994) — **dual-use and biological exports**, listed with ATCC numbers | No sourced figure for supplier revenue. Riegle's own quantified datum: specimens averaging "less than $60" | **No UN sanctions on any third-party supplier located.** Council action addresses the parties: S/RES/540 (1983), 582 (1986), 598 (1987), 612 and 620 (1988) | The Riegle hearing's troop-exposure theory, disputed under oath **in the same hearing**: DoD's Dr Theodore Prociv — *"I do not believe that any chemical agents entered the theater of operations and exposed any of our soldiers."* Genuine intra-record contradiction |
| **Afghanistan** (1979–92) | CRS R41070 (Rollins, 25 Jan 2011): the US "did covertly finance (about $3 billion during 1981-1991) and arm (via Pakistan) the Afghan mujahedin factions". Presidential Finding **3 July 1979** (FRUS 1977–80, vol. XII, Doc 214); DCI Turner, 21 Feb 1980: total programme **$30 million**; Turner, 28 Dec 1979 (Doc 107): "will ultimately cost only $10 million" | **No sourced figure** for any sponsor's gain. The "Saudi dollar-for-dollar match" has **no document**; best source is participant recollection (Bearden, PBS Frontline) | **None located** for the US, Saudi Arabia or Pakistan | **Documented, but not in the popular direction.** 9/11 Commission Report, official edition, p. 56, verbatim: *"But Bin Ladin and his comrades had their own sources of support and training, and they received little or no assistance from the United States."* The same page affirms that the US and Saudi Arabia "supplied billions of dollars worth of secret assistance to rebel groups". DCI Tenet to the Joint Inquiry, 17 Oct 2002: *"we have no record of any direct US Government contact with Bin Ladin at that time."* Brzezinski's sourced words (*Le Nouvel Observateur*, 15–21 Jan 1998, p. 76) are *"We now have an opportunity of giving to the USSR its Vietnam war"* — **not** the widely circulated variant |
| **Nicaragua** (1980s) | ICJ, *Military and Paramilitary Activities in and against Nicaragua*, 27 June 1986, **operative part paragraph 292, subparagraphs (1)–(16)**. 292(3), 12–3: the US, "by training, arming, equipping, financing and supplying the *contra* forces … has acted … in breach of its obligation under customary international law not to intervene". The Court was **"not able to satisfy itself"** that the US "created" the contra force, but found it "largely financed, trained, equipped, armed and organized the FDN" | **Nothing on the record as a gain.** The record holds a finding of **intent to coerce** and the US's pleaded justification of collective self-defence, **rejected 12–3** in 292(2) | **A binding merits judgment and no enforcement.** **Five US vetoes** on the official UN veto list: S/16463 (4 Apr 1984, S/PV.2529), S/17172 (10 May 1985, S/PV.2580), S/18250 (31 Jul 1986, S/PV.2704), S/18428 (28 Oct 1986, S/PV.2718), S/21084 (17 Jan 1990, S/PV.2905). Reparations **never assessed**: 292(13)–(15) imposed the obligation and reserved form and amount; Nicaragua's **requested** interim award of $370.2m was never made. Case removed from the Court's list 26 Sept 1991 after Nicaragua renounced further action (ICJ press release 91/28) | Five vetoes and the discontinuance are the documented diplomatic cost. Iran–Contra is **attributed, not verified here** |
| **Angola** (1975–2002) | South African **TRC, Final Report, Vol. 2, Ch. 2**: Operation Savannah (second half of 1975) was an "**undeclared act of war**" that "did not receive the approval of the South African cabinet" and was illegal under the 1957 Defence Act, which Parliament amended in January 1976 **retroactively to August 1975**; the TRC "**was not able to access any files on Operation Savannah in the SADF archives**". Operation Protea (Aug 1981) occupied **50,000 km²** of Cunene, parts held "until 1989"; Askari (Dec 1983): "**324 Angolan and Cuban troops and twenty-one South Africans were killed**" | **No official document states a benefit.** The Council's documented response to the sponsor was to **resource the target**: S/RES/574 (1985) op. 6 and S/RES/577 (1985) request assistance to strengthen Angola's defence | Compensation demanded in S/RES/387 (1976), 475 (1980), 577 (1985) and **never paid**. The decisive datum is the veto: the only draft proposing **Chapter VII mandatory sanctions** — **S/14664/Rev.2, 31 Aug 1981 (S/PV.2300)** — **vetoed by the United States** (13–1–1); **S/18163, 18 Jun 1986 (S/PV.2693)** vetoed by the **US and the UK**. Where the Council acted with teeth — S/RES/864 (1993), 1127 (1997), 1173 (1998), 1295 (2000) — it acted **from 1993, after the Cold War, against UNITA, a non-state actor** | TRC Vol. 2, Ch. 2, para. 68, verbatim: "The town was shelled by SADF 155mm artillery for several weeks, and largely destroyed. **The SADF failed, however, to capture the town** and the stalemate led eventually to negotiations…" Causal weighting is disputed and is **not** asserted here |
| **Mozambique** (1977–92) | **TRC Vol. 2, Ch. 2, para. 192**: "**South Africa took over responsibility for RENAMO in March 1980**"; para. 195, from SADF file HSOPS/309/4: the 25 May 1983 drop of sixty palettes including **450 AK-47s, 894,888 rounds** and 800 hand grenades; para. 196: "**support for RENAMO never ceased; it simply changed its form**… **a two-year stockpile of weaponry was delivered to RENAMO in the two months preceding the signing of the Accord**"; para. 187: "all the files on surrogate operations were **destroyed by DST when it was closed in the early 1990s**". Nkomati Accord, **UNTS vol. 1352, No. I-22802**, 16 March 1984, Arts. 2(3), 3(1)–(2), 5, and Art. 9's Joint Security Commission | The treaty itself documents the reciprocal benefit in Arts. 3 and 5: Mozambican commitments to eliminate ANC presence, bases, transit and broadcasting. **No sourced figure** for any economic or transit gain | **The only official body to find a breach is the sponsor's own.** TRC para. 196: the Gorongosa diaries "provided **firm evidence of continuing SADF involvement with RENAMO in violation of the Accord**", whose authenticity **Pik Botha confirmed on oath**. **No Joint Security Commission finding located; no UN finding of breach located; no sanctions on South Africa on account of Mozambique located; no legal action by Mozambique located** | **Political, not military, and documented:** the diaries became "a major embarrassment to the government", and para. 197: "both the cabinet and the SSC, including even State President PW Botha, were **kept in ignorance**" — the instrument escaped civilian control inside the sponsor state |
| **Syria** (2011– ), chemical-weapons attribution | OPCW **Investigation and Identification Team**, established by C- SS-4/DEC.3 (27 Jun 2018): S/1867/2020 (Ltamenah), S/1943/2021 (Saraqib), S/2125/2023 (Douma): "**reasonable grounds to believe that the Syrian Arab Air Forces were the perpetrators**". **Correction to a common assumption: Khan Shaykhun (4 Apr 2017) was attributed by the OPCW-UN Joint Investigative Mechanism, S/2017/904 (26 Oct 2017), not by the IIT** | **No sourced figure** for any sponsor's gain | **No sponsor of the Assad government punished by any body** in anything verified. Russia's vetoes are the mechanism; the Council's P5 structure makes a Security Council consequence legally unavailable | **Entirely unverified** in this record. Every candidate item is listed as unverified rather than asserted |
| **Yemen** (2014– ) | **R (Campaign Against Arms Trade) v Secretary of State for International Trade [2019] EWCA Civ 1020**, 20 June 2019, ¶7: a coalition of nine states led by Saudi Arabia "responded to a request for assistance by President Hadi and commenced military operations". **Panel of Experts on Yemen, S/2018/594** (letter dated 26 Jan 2018), ¶90(i): Iran "**in non-compliance with paragraph 14 of resolution 2216 (2015)**" for failing to take measures to prevent supply of missile components — **in the same document**, ¶90(h): "the Panel has **no evidence as to the identity of the supplier, or any intermediary third party**" | **The only case with even a qualitative on-the-record finding**, and it is not a figure: **[2023] EWHC 1343 (Admin)** ¶51(ii) records UK evidence of "logistical and technical support and training provided to Saudi Arabia, which the details in the closed evidence suggest is considerable"; ¶13: "By June 2021 the Coalition had launched tens of thousands of air delivered weapons". **The quantification is qualitative and the support detail derives from closed evidence** | **No sanctions measure against Saudi Arabia or the UAE for coalition conduct located.** The arms-control architecture runs against the **Houthis** and, via the ¶14 findings, **Iran**. The one verified consequence for a coalition sponsor is **domestic UK judicial review**, producing a procedural remedy — the Secretary of State's New Decision announced by written statement of **7 July 2020** — and not a sanction | **Unverified:** Abqaiq–Khurais (14 Sept 2019), Abu Dhabi (17 Jan 2022), *Galaxy Leader* (19 Nov 2023), the US–UK strikes (11 Jan 2024). **On the record** from [2019] EWCA Civ 1020 ¶9: Saudi-*reported* figures of "745 Saudi soldiers and border guards killed along the Southern front, and over 10,000 injured since March 2015" — **reported figures recited by the court, not independent findings** |
| **Nagorno-Karabakh** (1988–94, 2020) | Four Council resolutions, **all 15–0–0**: S/RES/822 (30 Apr 1993), 853 (29 Jul 1993), 874 (14 Oct 1993), 884 (12 Nov 1993) — cessation of hostilities and withdrawal from Kelbadjar, Agdam, Zangelan and Goradiz. The **9 Nov 2020 trilateral statement** provides for a Russian peacekeeping contingent of **1,960 servicemen** and Russian FSS Border Guard control of transport links | **The one clean on-the-record benefit to a third party is written into a signed text:** Russia receives the peacekeeping mandate and control of the Lachin corridor, agreed by both belligerents | **None located for any external sponsor.** **None of the four 1993 resolutions imposed sanctions or any Chapter VII measure, and no UN sanctions regime on Nagorno-Karabakh at any time was located.** The ICJ orders of 7 Dec 2021 (cases 180 and 181) bind **only Armenia and Azerbaijan** | **None confirmed.** The strongest candidate — Russian peacekeepers failing to prevent the September 2023 offensive — is unverified and **not** asserted |
| **Rwanda** (1990–94) | The 1975 military technical assistance agreement, confirmed verbatim by a French court: *"le traité d'assistance militaire conclu le **18 juillet 1975** entre la France et le Rwanda"* (TA Paris, n° 2309845/4-1). The **Quilès report**, Assemblée nationale **n° 1271**, 15 Dec 1998, contains "La livraison d'armes au Rwanda par la France de 1990 à 1994" (p. 179) and the operations Noroît, Volcan, Chimère, Amaryllis, Turquoise. **UN Commission of Experts, S/1994/1405**, §183: "**overwhelming evidence**… acts of genocide against the Tutsi group were perpetrated by Hutu elements in a concerted, planned, systematic and methodical way" | Duclert frames the policy as a **failure**: *"La crise rwandaise s'achève en désastre pour le Rwanda, en défaite pour la France."* **No sourced figure** for any material gain | **None, and this is the sharpest instance of the pattern.** Duclert: responsibility found, **expressly political, not legal**. The courts: **non-justiciable** (*actes de gouvernement*), through three stages to the Conseil d'État, 18 mars 2026. Operation Turquoise criminal investigation: **non-lieu général, October 2023** (AFP, 14 Nov 2024) | **Documented and long:** 27 years of Franco-Rwandan rupture after 1994, ended by the Kigali speech of 27 May 2021; Rwanda not invited to the Biarritz summit of 8 Nov 1994 |
| **British India** (1906–47) | **Indian Councils Act 1909** — read in full: **communally neutral**, no reference to any religious community. **HC Deb 1 Apr 1909, vol. 3, cc496–601**, Under-Secretary **Buchanan**: the Government's own preferred alternative — "some general system of minority representation" — "**was viewed with the utmost suspicion and distrust by those chiefly concerned… and it found no friends in other quarters in India. The result of that was that it was dropped the moment it got to India**"; and "**a firm, stable, and permanent official majority in the Governor-General's Council. That we have secured.**" **Communal Award**, text dated London **4 August 1932**, ¶6: "Election to the seats allotted to Muhammadan, European and Sikh constituencies will be **by voters voting in separate communal electorates**"; ¶4 gave each community an effective veto over revision | **Nothing on the record states that the Raj benefited from communal division.** The stated objectives are the "**solemn promises**" to the Mahomedan community and the preservation of official control | **None.** No adjudication, no sanction, no compensation mechanism. Separate electorates were abolished by the successor state's Constitution of 1950 — **a change of policy by the successor, not a punishment of the sponsor** | **On the record as a contemporary warning, not a finding.** **Sir Henry Cotton, HC Deb 26 Apr 1909**: "making statutory for the first time in the history of this Empire this discrimination between different religious bodies… would be to introduce into India **that hostility which exists between Catholic and Protestant in Ireland**". Whether partition is "blowback" is a historiographical claim, and **no official body was found making that causal finding** |
| **Northern Ireland** (1970s–90s) | **Stevens Enquiries, *Overview and Recommendations*, April 2003**, ¶1.3: "**collusion, the wilful failure to keep records, the absence of accountability… and the extreme of agents being involved in murder**"; ¶4.7: "**I conclude there was collusion in both murders**" (Finucane, Lambert); ¶3.1: "**I was being obstructed**"; ¶3.4: "the night before the new operation **my Incident room was destroyed by fire**… I believe it was a deliberate act of arson." **Smithwick Tribunal**, report published 3 Dec 2013: satisfied "there was collusion in the murders" of Breen and Buchanan, while recording "**no record of a phone call, no traceable payment, no smoking gun**" and being "**unable to identify the IRA mole in Dundalk Station**". **de Silva review (HC 802, 12 Dec 2012)**, accepted "unequivocally" by the Government on 16 Jan 2015: "a series of positive actions by employees of the State actively furthered and facilitated [Finucane's] murder" | Stevens ¶1.7: in 1987–89 the RUC dealt with over **3,000** terrorist-related incidents, of which **261** were deaths related to the security situation; ¶4.2: **94 convictions from 144 arrests**. **No official source quantifies any individual agent's intelligence value** | **No criminal liability has ever attached to any collusion finding.** Stevens: 94 convictions, **none for collusion**; the Stobie prosecution **collapsed in November 2001** and he was shot dead two weeks later; **no one has ever been charged with Finucane's murder**. Operation Kenova: all **28 files declined** (PPS, 6 and 29 Feb 2024) for "insufficient evidence to provide a reasonable prospect of conviction", because intelligence records were inadmissible and "**original source materials were no longer available**". Ballymurphy: "**No one has ever been charged or convicted**" | **Officially found.** Saville (HC Deb 15 Jun 2010) as summarised by the Prime Minister: Bloody Sunday "**strengthened the Provisional IRA, increased nationalist resentment and hostility towards the Army and exacerbated the violent conflict of the years that followed**". And the state's own agents were central to the murders: Stobie supplied the Lambert weapon; Nelson "contributed materially" to the attack on Finucane |

#### Where a sponsor *was* made to pay

Four cases, and their shape is the finding.

| Case | What happened | What distinguishes it |
| --- | --- | --- |
| **Libya / Lockerbie** | S/RES/748 (31 Mar 1992, 10–0–5), S/RES/883 (11 Nov 1993, 11–0–4); trial at Camp Zeist, conviction of al-Megrahi **31 Jan 2001**; suspension S/PRST/1999/10; sanctions **lifted by S/RES/1506 (12 Sept 2003), 13–0–2, France and the United States abstaining**. Libya's letter S/2003/818 conveyed acceptance of responsibility "for the actions of Libyan officials" and payment of compensation | **The victims were the nationals of powerful states.** 270 dead of 21 nationalities; the pressure came from the victims' own governments. A negotiated state-level admission unlocked the settlement — **no unpunished sponsor in this record made an equivalent admission** |
| **Eritrea** | S/RES/1907 (23 Dec 2009) imposed the arms embargo and travel ban; **lifted by S/RES/2444 (14 Nov 2018), 15–0**. Para. 1 verbatim: "**recognises that during the course of its current and four previous mandates the SEMG has not found conclusive evidence that Eritrea supports Al-Shabaab**" | **A small, isolated state with no P5 protector.** And the punishment was reversible: the Council's own final finding is that conclusive evidence was never found across five consecutive mandates |
| **Charles Taylor** | S/RES/1638 (11 Nov 2005) op. 1, verbatim: UNMIL "to apprehend and detain former President Charles Taylor in the event of a return to Liberia and to transfer him… for prosecution"; S/RES/1688 (16 Jun 2006); **SCSL-03-01**: convicted, 50 years, upheld on appeal 26 Sept 2013 | **An individual, and a defeated one** — an exiled former president. A treaty-based hybrid court created by agreement under S/RES/1315 (2000) supplied a forum a state could not be reached by |
| **Frans van Anraat** | District Court of The Hague, **23 Dec 2005**, parketnummer 09/751003-04, ECLI:NL:RBSGR:2005:AV6353: **acquitted of complicity in genocide**, convicted of **complicity in war crimes**, 15 years; Court of Appeal, 9 May 2007: 17 years; Hoge Raad, 30 June 2009, ECLI:NL:HR:2009:BG4822: upheld | **A private supplier, reachable by a domestic criminal court.** The court held complicity is an autonomous offence and reasoned that a contrary view "would result in impunity for a considerable number of suspects" |

### 5.3 The pattern

Four discriminators separate the cases where the sponsor paid from those where it did not.
None of them is the gravity of the conduct.

**1. Whether the sponsor had pre-committed to a forum it could not veto.** In Nicaragua the
United States had deposited an Article 36(2) declaration and was party to the 1956 Treaty of
Friendship, Commerce and Navigation, so the Court reached the merits. In Rwanda France had
made no such commitment covering the conduct at issue and the claim died on **justiciability**,
not on the facts. In Angola and Mozambique no adjudicative forum was engaged at all. **A
sponsor pays only where it has already conceded the forum — and even then only if the
judgment is self-executing.**

**2. Whether the enforcement route ran through a body the sponsor or its ally could veto.**
Every interstate enforcement route in this record ran through the Security Council. Five US
vetoes on Nicaragua; the only Angola draft proposing Chapter VII mandatory sanctions vetoed
by the United States (S/14664/Rev.2) and a later complaint by the US and the UK (S/18163);
no sanctions on South Africa on account of Mozambique; and, in Northern Ireland, the sponsor
was the state whose own courts and police conducted the investigation.

**The consequence is structural: an adverse judgment plus a veto is worth less than no
judgment at all, because it documents the impunity.** Nicaragua is the limiting case.

**3. Whether the sponsor's own records survived and were disclosable.** The SADF's
Directorate of Special Tasks **destroyed all its surrogate-operation files** when it closed in
the early 1990s (TRC Vol. 2, Ch. 2, para. 187); the TRC likewise "was not able to access any
files on Operation Savannah"; Stevens' substantive report was withheld because "the
overwhelming bulk of the detail has been withheld because of potential future prosecutions",
and the prosecutions largely did not follow; Kenova failed precisely because "**original
source materials were no longer available**"; Smithwick found collusion and could identify no
one. Conversely, the one place where the sponsor's records **did** survive and were
**compelled** is South Africa after 1994 — a truth commission with subpoena power and an
amnesty incentive — and it is the only place in this record where an **official body of the
sponsor state found its own policy in breach**.

**4. Whose nationals were killed, and whether the sponsor was already weak.** The punished
cases are the ones where a powerful state's own citizens were the victims (Lockerbie,
UTA 772), or where the sponsor was small, isolated or defeated (Eritrea, Taylor), or where
the actor was a private individual a domestic court could reach (van Anraat). **No P5 state,
and no client of a P5 state, was sanctioned for sponsorship in any case above.**

### 5.4 Two results that are results, not gaps

**(i) No official body has quantified any sponsor's material gain, in any case.** Not in
Yugoslavia, Iran–Iraq, Afghanistan, Syria, Karabakh or — beyond a qualitative finding — Yemen.
Every "profit" claim in this literature is, on this record, inference. This bears directly on
the model: the harvest `Λ` is a **declared** quantity in §2, and the record is the reason it
has to be. It is not that the figure was hard to find; **it is that the cases in which the
question was asked were asked by bodies with no reason to ask it.**

**(ii) Where the Security Council acted with teeth, it acted against the weak actor.**
Throughout the Angola file the Council's response to *state* sponsorship was to **condemn and
to resource the victim** — S/RES/574 and 577 request assistance to strengthen Angola's
defence — while the **only binding Chapter VII sanctions anywhere in that file attached, from
1993 and after the Cold War, to UNITA, a non-state actor.** The same asymmetry appears in
Yemen: the arms-control architecture runs against the Houthis and, via the ¶14 findings,
Iran, and **no sanctions measure against Saudi Arabia or the UAE for coalition conduct was
located.** This is Theorem 6's comparative static — `K_max = k·S·e`, punishment capacity
proportional to the punisher's surplus and the target's exposure — showing up as a
distribution of *targets* rather than of crimes.

### 5.5 What the historical record does not establish

1. **It does not establish that third parties trigger wars.** Theorem 1 says a spoiler need
   not trigger anything; it needs to make the dyad's own dilemma cheap enough to fire. The
   record is consistent with that and does not test it: the observable evidence of spoiling
   is often *absence*.
2. **It does not establish intent to divide.** In the case where the claim is oldest and
   loudest — British India — the statute is communally neutral, the initiative is documented
   as coming from a section of Muslim elites under a Viceregal pledge, and **no documentary
   statement of divisive intent was located in the Act, the debates, or the Award**. The
   divide-and-rule reading rests on inference from structure: the preserved official
   majority, the proliferation of communal categories, and the ¶4 veto. **That is a real gap
   in the record, not a refutation**, and it is stated as a gap.
3. **It does not sort the cases by the prediction that they should be sorted by.** P1 says
   spoiling concentrates on dyads with a **narrow** peace margin. The record cannot test this,
   because the margins are the declared quantities of §2 and no source measures them. What
   the record *can* say is that the observed sponsors are not drawn from the deepest hatreds —
   and that is a weak observation, not a test.
4. **It does not show blowback.** P6 and Theorem 4 predict blowback increasing in the
   sponsor's support and impatience. The record contains **one officially found instance** —
   Saville's finding that Bloody Sunday strengthened the Provisional IRA — and a documented
   failure of control inside the sponsor state (Mozambique: the cabinet and SSC "kept in
   ignorance"), plus the documented collapse of the Stobie prosecution. It does not contain a
   measured case.
5. **It does not support the popular Afghanistan causal chain.** The 9/11 Commission's own
   text on p. 56 denies assistance **to Bin Ladin and the Arab volunteers** while affirming
   billions to the Afghan rebels. Anyone asserting the mujahideen-to-al-Qaeda chain must
   engage that sentence, and this document does not assert the chain.


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

**Two of these are strengthened by §5 rather than merely repeated there.**

On `g`: the record is not that the capture rate is hard to find, but that **no official body
has quantified any sponsor's material gain in any case examined** — not in Yugoslavia,
Iran–Iraq, Afghanistan, Syria or Karabakh, and in the one case with any on-the-record finding
at all (Yemen) the quantification is qualitative and rests on closed evidence. The bodies
that investigated these conflicts were not constituted to ask what the sponsor gained, so the
absence is a fact about the inquiries as much as about the world. **`g` is not so much
unmeasured as unasked.**

On P5: §5.3 finds the record's enforcement pattern is **not** sorted by the severity of the
conduct but by whether the sponsor had conceded a forum, whether its records survived, and
whose nationals were killed. §5.4 adds the sharper form — where the Security Council acted
with teeth it acted against the **weak** actor, the only binding Chapter VII sanctions in the
Angola file attaching to UNITA rather than to any state. This is Theorem 6's comparative
static showing up as a distribution of *targets*, and it is the strongest empirical support
anything in this document has. It is still not a test of P5, because the counterfactual —
what the same conduct would have met with under a different exposure — is the unobservable
quantity above.

The honest summary: **the mechanism is fully specified and its comparative statics are
exact; the two parameters that decide whether the strategy is a transfer or a destruction —
`g` and `h` — have no measured values at all.**

---

## 8. Reproduction

```
cargo test -p multipolar_sim spoiler::          # the theorems, as invariants
python tools/plot_spoiler.py                    # the six panels, checked against the model
```

**There is no `--spoiler` CLI mode, and an earlier version of this section advertised one.**
The module is a library plus its test module, deliberately: the CLI already has four analysis
modes, and a fifth that nobody would pass is not a stronger claim than a test that runs. What
the framework offers instead is `--strategies`, which prices the spoiler's instrument against
the cooperative one on the same budget and clock and differences the two over a paired Monte
Carlo — see `STRATEGY_COMPARISON.md`.

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
in §5, where every figure asserted is attributed and every gap is named as a gap.)*

- Banzhaf, J. F. (1965). Weighted voting doesn't work: a mathematical analysis. *Rutgers Law Review* 19, 317–343.
- Jervis, R. (1978). Cooperation under the security dilemma. *World Politics* 30(2), 167–214.
- Olson, M. (1965). *The Logic of Collective Action*. Harvard University Press.
- Penrose, L. S. (1946). The elementary statistics of majority voting. *Journal of the Royal Statistical Society* 109(1), 53–57.
- Stedman, S. J. (1997). Spoiler problems in peace processes. *International Security* 22(2), 5–53.


