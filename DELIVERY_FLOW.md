# Where Does the Bottleneck Go?

### Pre-registration for `delivery_flow`: a two-stage queueing model coupled to comprehension

---

## 0. What this document is, and what it is not

A pre-registration, in the format of `AI_AUTHORING_AND_COMPREHENSION.md` Part 1, for a crate that
does not exist yet. Nothing here has been run. That ordering is the point: the decision rule below
was fixed before there was a model to run it on, so the verdict cannot be chosen afterwards.

The claim under test is a claim a blog post makes and does not derive:

> **"It can get worse than Amdahl predicts."**

Amdahl's law assumes the non-accelerated parts stay constant. If faster code generation also makes
downstream stages slower, Amdahl cannot express the result, because Amdahl has no variable for the
slowness of a stage. A queue can express it. This document specifies the smallest model in which
the claim is testable, and the observations that would refute it.

**It is not a model of a software organisation.** Two stages, one coupling, one horizon. §9 says
what is left out and why adding it now would make the model unfalsifiable.

---

## 1. The question, sharpened

### 1.1 The arithmetic that is already known

For a pipeline in which coding is a fraction `p` of idea-to-production time and AI multiplies coding
speed by `s`, whole-cycle speedup is

```
U(p, s) = 1 / ((1 - p) + p/s)
```

At `p = 0.3, s = 10` this is **1.37×**; at `p = 0.5, s = 10`, **1.82×**. The bound is `1/(1-p)` as
`s → ∞`, so at `p = 0.3` no code-stage speedup whatsoever exceeds **1.43×**. This is Amdahl and it
needs no model.

### 1.2 The arithmetic that is not known, and which motivates everything below

Invert the formula. Given a *measured* whole-cycle speedup `U` and a coding fraction `p`, the
code-stage speedup is

```
s = p / (1/U - 1 + p)
```

Now substitute the one controlled measurement available. METR's randomised trial found experienced
developers **19% slower** with AI, so `U = 1/1.19 = 0.840`. Then:

| Coding fraction `p` | Implied code-stage speedup `s` |
|---:|---:|
| 0.30 | **0.61** |
| 0.50 | **0.72** |
| 0.80 | **0.81** |

*(Each was checked by substituting back: `1/((1-p) + p/s) = 0.840` to four places. The earlier draft
of this table carried a mis-inverted value; the arithmetic above is the corrected one.)*

Since `s = p/(0.19 + p)`, the denominator always exceeds the numerator, so **for every `p < 1` the
implied `s` is strictly below 1.** Amdahl with a code-stage speedup of any size
cannot produce a 19% slowdown. So the measured result requires at least one of: the code stage was
actually *slower* with AI; the denominator grew for a reason Amdahl does not model; or the whole-task
measurement is not the quantity the formula describes.

That is the gap this crate is for. It is not "Amdahl is wrong". It is that **Amdahl is
under-specified in exactly the direction the post wants to claim**, and the missing specification is
a stage whose service rate depends on something AI changes.

### 1.3 The specific mechanism, in one sentence

Review of a change is slower the less the reviewer understands it; understanding is what authoring
the code would have produced; AI removes the authoring. So AI lowers the service rate of the stage
downstream of the stage it accelerates — and under a fixed review capacity, a slower service rate
does not produce a smaller queue, it produces an unstable one.

---

## 2. Provenance of every number

The repository's four marks, applied. No value enters the model without one.

### Sourced

| Quantity | Value | Source | How it is used |
|---|---|---|---|
| Whole-cycle speedup for experienced developers on their own repositories | **0.840** (19% slower), CI +2% to +39% | METR RCT, 2025, verified in `AI_AUTHORING_AND_COMPREHENSION.md` §1.2 | the inference in §1.2, and the *validation target* — the model should be able to reproduce a slowdown |
| Follow-up estimate, same cohort | **1.18** speedup, CI −38% to +9% | METR, 2026 | a second validation target; the bracket is wide and the model must not be validated to a point |
| Direction of review and debugging burden | 67% report more review time; 52% more debugging; 59% greater mental effort | BairesDev Q3 2026, verified in §1.2 | sign of the review-stage and defect parameters, not their magnitude |
| Downstream stage does not scale with upstream | throughput rises, stability falls | DORA 2025 — **cited, not fetched**; see `AI_AUTHORING_AND_COMPREHENSION.md` §1.2 | a declared prior on the deploy-stage behaviour; not a parameter value |

### Declared

Everything that sets the shape of the model. Plausible in sign, unmeasured, swept where the
conclusion could turn on it.

| Parameter | Symbol | Base value | Range swept | Why this value |
|---|---|---:|---|---|
| Changes arriving per month | `λ` | set to make baseline review utilisation **0.80** | 0.6–0.95 utilisation | a busy but stable pipeline; utilisation is the honest scale-free control |
| Code-stage capacity, changes/month | `μ_code` | 20 | — | sets the unit; the model is scale-free in it |
| Code-stage speedup | `s` | 1 (control) | **2, 5, 10** | Amdahl-relevant range; 10 is the post's own figure |
| AI share of incoming changes | `a` | swept | 0, 0.25, 0.5, 0.75, 1.0 | the adoption axis |
| Reviewer capacity, hours/month | `H` | 120 (one reviewer at 0.7 FTE) | 60–200 | one reviewer, because the first model should not model teams |
| Review hours per change at full comprehension | `k` | 2.0 | — | shared with `comprehension_cost`'s `review_hours_per_unit` |
| Review cost growth as comprehension falls | `θ` | 1.0 | 0.5–2.0 | shared with `review_slope` |
| Queue length at which reviews are time-boxed | `Q*` | 8 | 4–20 | the pressure threshold; the mechanism that makes the loop positive |
| Duration of a time-boxed review, hours | `T_max` | 0.75 | — | a quarter of an unpressured review; short enough to be plausible as "skim and approve" |
| Initial comprehension, human-authored | `c_h` | 0.85 | — | shared with `comprehension_cost` |
| Initial comprehension, AI-authored, passive review | `c_ai` | 0.25 | — | shared; and see §3.4 — it is a review-mode value |
| Comprehension gain per non-time-boxed review | `ρ` | 0.25 | — | shared |
| Retention half-life, human / AI | — | 24 / 9 months | — | shared |
| Horizon | — | 36 months | — | long enough for a queue to diverge if it will |

### Unverified

| Quantity | Why unverified |
|---|---|
| **The coding fraction `p`** | No published measurement pins it. It is the one parameter in the post with no anchor, and §8 specifies how to measure it. Note that in this model `p` is an **output**, not an input — see §3.5 |
| Whether the review-stage service function is linear in `(1-c)` | The form is a convention carried from `comprehension_cost`; nothing establishes it |

### Conjecture

That the coupling in §1.3 is what explains the METR inversion rather than a code stage that was
simply slower. §5 states what would decide between those two.

---

## 3. The model

Two stages in series, discrete months, one class of customer (a change). `t = 1..36`.

### 3.1 State

| Variable | Type | Meaning |
|---|---|---|
| `Q_code` | integer ≥ 0 | changes accepted but not yet coded |
| `Q_review` | integer ≥ 0 | coded changes awaiting review |
| `c̄` | [0,1] | mean comprehension of the modules touched by changes currently in review |
| `mode[m]` | {human, AI} per module | authoring mode, fixed at creation |
| `age[m]` | months | time since the module was last reviewed |

Two stages rather than five is deliberate: the pipeline is `code → review`, and every other stage is
folded into the arrival process. §9 defends this.

### 3.2 Transition, one month

```
1. ARRIVE     n_arr ~ Poisson(λ)
              each change is AI-authored with probability a
              Q_code += n_arr

2. CODE       μ_code(a) = μ_code0 · (1 + a·(s − 1))
              n_coded = min(Q_code, μ_code(a))
              Q_code -= n_coded ; Q_review += n_coded

3. COMPREHEND on entry to review, each change's module takes
              c = c_h (human) or c_ai (AI)
              then decays by exp(−ln2 / half_life(mode)) per month since last touch

4. REVIEW     pressure  = Q_review > Q*
              if not under pressure:
                  T_review = k · (1 + θ(1 − c̄))
              if under pressure:
                  T_review = min(T_review, T_max)          # time-boxed
              n_reviewed = min(Q_review, floor(H / T_review))
              Q_review -= n_reviewed

5. LEARN      a change reviewed without time-boxing raises comprehension:
                  c ← c + ρ(1 − c)     for its module
              a time-boxed review raises nothing. This is the loop:
              pressure → no learning → low c → slower review → more pressure
```

Step 5 is the whole model. Without it, faster coding produces a longer queue that drains at the old
rate and reaches a new steady state — which is Amdahl. With it, the queue's own length degrades the
service rate that would drain it.

### 3.3 What is deliberately not in it

- No defect escape and no rework. `comprehension_cost` has both; adding them here would give two
  mechanisms that both produce "worse", and the two-stage model's job is to test whether *one*
  mechanism suffices.
- No learning curve on the tool, no ramp-up, no seniority.
- No hospital-time or coordination cost.

### 3.4 One parameter is a review-mode value and must be treated as one

`c_ai = 0.25` describes **passive** review. The generation-effect literature says a reviewer who
reconstructs recovers most of the gap, so `c_ai` is a property of review mode, not of machine
authorship (`AI_AUTHORING_AND_COMPREHENSION.md` §2.8, `documents/THE_BY_PRODUCT_THEORY.md` §3.2).

This matters more here than there, because in this model the time-boxing in step 4 *is* the
mechanism that makes review passive. So `c_ai` is not an independent input: pressure forces passive
review, and passive review is why comprehension stays at `c_ai`. The model would be circular if
`c_ai` were a free parameter, so it is treated as the *floor* of a range and the sweep is over how
much a non-pressured review can recover.

### 3.5 The inversion that makes this different from the post

The post treats `p` as an input. A queueing model makes it an **output**:

```
p(t) = code-stage hours in month t / total pipeline calendar time in month t
```

As the review queue grows, waiting time grows, so `p` **falls** — and Amdahl evaluated at the
observed `p` therefore predicts *less* speedup than Amdahl evaluated at the baseline `p`. The post's
own formula, applied with an endogenous `p`, partly anticipates the post's own critique. Deriving
that is one of the four outputs in §6.

---

## 4. The two competing hypotheses

Pre-registered. Both are stated so that a run can favour either, and the rule for deciding was fixed
before the model existed.

### H_A — the Amdahl null

> Throughput speedup follows `U(p_0, s)` where `p_0` is the baseline coding share, and the review
> queue reaches a bounded steady state at every adoption share. The pipeline is a fixed-fraction
> system with a fast stage; nothing degrades.

Note what H_A is *not*: it is not "AI does not help". It is that the only thing that changes is the
service rate of one stage.

### H_Q — the queueing alternative

> Past a critical adoption share `a*`, the review queue has no bounded steady state within the
> horizon, because time-boxing suppresses the learning that would raise the review service rate.
> Throughput saturates below `U(p_0, s)`, and realised `p` falls as the queue grows.

### 4.1 Decision rule, fixed in advance

The base cell: `s = 10`, `a = 0.5`, `λ` set for 0.80 baseline review utilisation, 36 months.
The adoption sweep: `a ∈ {0, 0.25, 0.5, 0.75, 1.0}`.

| Verdict | Condition |
|---|---|
| **H_A wins** | In the coupled model at `s = 10`, the queue converges at every `a ≤ 1.0`, **and** realised throughput at `a = 0.5` is within **±15%** of `U(p_0, 10)` |
| **H_Q wins** | In the coupled model at `s = 10`, there exists `a* ≤ 0.75` at which the end-of-horizon queue exceeds **10× baseline** and is still rising month-on-month, **and** the decoupled control at the same cell converges with throughput within ±15% of Amdahl. Report `a*` |
| **Indecisive** | The effect appears only at `s > 10`, or only at `a* > 0.75`, or only with `c_ai < 0.15` — i.e. outside the ranges in §2. Report which constraint was binding |

**Win for the Amdahl null** therefore means: the model, with the comprehension coupling fully
switched on, still behaves like Amdahl. That is a substantive result and the post's claim would be
unsupported.

**Win for the queueing alternative** means: the coupling alone, with no defect mechanism and no
rework, produces a queue that does not converge at an adoption share a team might plausibly reach.

### 4.2 Falsification of the whole framing

If the coupled and decoupled models agree within 15% on throughput and queue length at **every**
cell in the sweep, the comprehension coupling contributes nothing measurable and the "worse than
Amdahl" claim is unsupported by this mechanism. The pre-registration commits to reporting that
outcome in those words rather than searching for a mechanism that works.

---

## 5. The control, and why it is the contribution

The strongest methodological move in `comprehension_cost` is that the model can report the opposite
of its own hypothesis. This crate needs the same one, and here it does more work than a sanity check.

**The control is the decoupled model: review service time held constant**, `T_review = k` regardless
of comprehension. In that model the pipeline has a fast stage and an unchanged stage, which is
Amdahl's assumption made literal.

| Model | Must reproduce | If it does not |
|---|---|---|
| Decoupled | `U(p_0, s)` within ±15% | the implementation or the flow accounting is wrong, not the world |
| Coupled | slower than decoupled, and slower as `a` rises | the coupling is not connected |

**The difference between the two runs is the entire contribution of the comprehension mechanism**,
and it is the only sense in which the model says anything Amdahl does not. Reporting the coupled run
alone would be reporting a number without its counterfactual.

---

## 6. What the simulation outputs

Four quantities, each answering a question the post raises and cannot answer.

1. **The effective speedup curve** — throughput against `s ∈ {2, 5, 10}` at each `a`, with the
   Amdahl prediction drawn for the *measured* `p`. Shows where the arithmetic saturates and whether
   the model follows it.
2. **The queue trajectory** — `Q_review` by month after adoption, for each `a`. This is the
   "worse than Amdahl" result in its native form: a queue, not a speedup.
3. **The critical adoption share `a*`** — the share at which the queue stops being transient. This
   is the number an engineering leader could act on, and it is the model's most useful output.
4. **The variance decomposition** — Sobol first and total order over `λ`, `s`, `θ`, `Q*`, `T_max`,
   `c_ai`, `ρ`, and the half-lives, with bootstrap intervals, following
   `comprehension_cost/src/global_sensitivity.rs`. The output is which assumption the verdict rests
   on, not a point estimate.

And one derived quantity worth its own line: **realised `p`**, and whether it is stable. The model
predicts that `p` falls as the queue grows, which is a testable claim about a quantity nobody has
measured.

---

## 7. What would make this worthless

Stated in advance, following §1.6 of the comprehension paper.

1. **A five-stage network.** Every added stage adds parameters with no anchor, and a model that can
   produce any answer is not a test. Two stages, one coupling.
2. **Choosing `λ` after seeing the queue.** Utilisation is the control, fixed at 0.80 before the run.
   A model evaluated at 0.79 and at 0.99 are different claims about different pipelines.
3. **Reporting `a*` without its sensitivity.** Any critical share is conditional on `Q*`, `T_max` and
   `H`; reporting it as a fact about software teams would be the error the whole document is
   guarding against.
4. **Letting the coupling be a free parameter.** `c_ai` is the floor of a range defined by review
   mode, not a dial (§3.4). A dial would let the queueing result be tuned into existence.

---

## 8. The measurement that would settle `p`

The post's 30% is illustrative and the commenters correctly observe that `p`, not `s`, is the
interesting variable — as `s → ∞` the bound is `1/(1-p)`, so `p` alone sets the ceiling. This model
makes `p` endogenous, which sharpens the question: not "what is `p`" but **"does `p` fall when the
code stage accelerates, and by how much"**. That is a claim the model makes and measurement can
refute.

A protocol, in the order a team could actually run it:

1. **Declare the stage boundaries before instrumenting.** The ambiguity is in the edges — does
   debugging count as coding? Does a rework loop count twice? Pre-declaring the boundaries is what
   makes `p` comparable across teams rather than a number each team defines into existence.
2. **Timestamp entry and exit of each stage per change**, on the same clock, including waiting. `p`
   is then the ratio of code-stage elapsed time to total elapsed time, per change.
3. **Report the distribution, not the mean.** `p` will be bimodal across task types — a bug fix and a
   new subsystem are not the same pipeline. Report median and interquartile range per task class.
4. **Test stability, which is the part nobody does.** Compute `p` over rolling three-month windows.
   If `p` drifts as AI adoption rises, then the Amdahl framing is invalid *by its own assumption*,
   and the direction of the drift is the queueing model's prediction.
5. **Minimum viable study**: one team, six months, pre-declared boundaries, `p` reported per task
   class with its window-to-window variation. That is enough to distinguish a stable `p` from a
   falling one.

The measurement is hard for a reason worth naming: the denominator of `p` contains idle and waiting
time, which is exactly the term the queueing model says grows. A team that measures stage hours and
excludes waiting will find a stable `p` and will have measured the one thing that does not change.

---

## 9. Scope, and what would justify a third stage

Not modelled, and why:

- **Specify and agree.** The post's `p` includes them; folding them into the arrival process is what
  keeps the model two-stage. If the two-stage run shows the queueing effect, the first addition is a
  fixed-latency upstream stage to test whether it dilutes `a*`.
- **Deploy and defect escape.** `comprehension_cost` already models leakage and its defect
  multiplier. Adding them here would give a second mechanism that also produces "worse", and the
  two-stage model exists to test whether one mechanism suffices. If it does not, the honest
  conclusion is that the effect needs both, which is a finding.
- **Teams and specialisation.** One reviewer is a simplification that makes `H` interpretable. A
  second model with heterogeneous reviewers would be a different paper.

A third stage is justified only if the two-stage model wins H_Q on the pre-registered rule. If it
does not, adding stages to find a version that does is the failure mode §7.1 names.

---

## 10. What this pre-registration does not claim

That the queueing mechanism explains the METR inversion. §1.2 establishes only that Amdahl cannot
explain it with `s > 1`, and §5's control is what would show whether comprehension is the
explanation or whether the code stage was simply slower. Both remain open, and the crate is designed
so that either answer is reportable.
