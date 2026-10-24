# Does the Human Know How Much They Do Not Understand?

### Pre-registration for `perception_gap`: estimating the AI–human estimation discrepancy as a latent state

---

## 0. What this document is

A pre-registration, in the format of `AI_AUTHORING_AND_COMPREHENSION.md` Part 1 and `DELIVERY_FLOW.md`,
for a crate that does not exist. Nothing has been run.

The proposal under test is that a Kalman filter can estimate the **discrepancy** between machine and
human estimation of comprehension, productivity and quality. This document takes that seriously and
asks the question that decides whether it is research or decoration: **is the discrepancy a quantity
the filter can identify, or is it a subtraction wearing a filter?**

The answer is not obvious, and §3 is the part of this document that matters. The short version: a
Kalman filter returns an estimate and a covariance whether or not the state is observable, and the
covariance can be reassuringly small while the estimate is arbitrary. The central contribution
proposed here is therefore not the filter — it is the **identifiability analysis and the probe design
that restores it**.

---

## 1. The question, and what is already published

### 1.1 The question

Not *"is AI faster?"* — that is a measurement question the METR trial and the BairesDev survey
disagree about, and both are cited in `AI_AUTHORING_AND_COMPREHENSION.md` §1.2. The question here is
one level up:

> **Does the human's estimate of their own comprehension and productivity track the truth, and how
> does the gap evolve as assistance deepens?**

That is the question `documents/THE_BY_PRODUCT_THEORY.md` §3.3 poses with the fluency literature and
cannot answer with a model: reading AI output is fluent, fluency drives confidence, confidence is
read as comprehension. If that story is right, the gap is not a constant to be measured once — it is
a **state that drifts**, and drift is what a filter is for.

### 1.2 What exists, verified

| Work | What it does | Why it is not this |
|---|---|---|
| **Akumba (2024)**, *Improving Software Effort Estimation Accuracy with a Kalman Filter-Driven Ensemble Model*, IJCA ([link](https://www.ijcaonline.org/archives/volume186/number58/improving-software-effort-estimation-accuracy-with-a-kalman-filter-driven-ensemble-model/)) | a Kalman filter inside an ensemble, smoothing one noisy estimate stream against benchmark datasets | one stream, no second estimator, and the state is the effort itself rather than a discrepancy between two parties |
| **FSE 2025**, *Impact of Request Formats on Effort Estimation: Are LLMs Different Than Humans?* ([link](https://dl.acm.org/doi/10.1145/3715771)) | replicates human effort-estimation experiments with LLMs, studies anchoring bias; finds **human–LLM collaboration more accurate than either alone** | the empirical comparison this proposal needs as a *prior*, but it is a static comparison — it estimates no state and models no drift |
| **IEEE**, *From COCOMO to GPT: A Comprehensive Evaluation of LLM-Based Software Effort Estimation* ([link](https://ieeexplore.ieee.org/document/11421842)) | benchmarks LLM effort estimation against task-specific models | static accuracy comparison |
| **TechRxiv**, *Cognitive Load-Aware Edge AI* ([link](https://www.techrxiv.org/doi/full/10.36227/techrxiv.177015854.42176901/v1)) | Kalman filtering of behavioural and physiological signals to track operator cognitive load | filters a measurable state, not a discrepancy between a belief and a truth |

The FSE 2025 result deserves emphasis because it is the closest thing to a prior for this work and it
cuts a specific way: if human–LLM collaboration beats both, then the useful quantity is not "which
estimator is right" but "how the human's estimate relates to the machine's", which is exactly a
two-stream problem.

### 1.3 What I could not verify, and the status of the novelty claim

Two candidate prior-art items from the proposal's own literature review — a 2010 Kalman-filter paper
on tracking remaining function points, and a proposed Kalman formulation for detecting LLM
"contextual drift" — **I could not confirm**. They are marked, not assumed.

**The novelty claim is `Unverified`, and by the repository's own taxonomy that is the honest mark.**
A claim that no paper exists cannot be checked by not finding one; my search was four queries deep.
So the contribution is designed so that it does not depend on novelty:

> Even if this exact model exists somewhere, §3's identifiability result stands on its own: **the
> AI–human perception gap is generically not identifiable from behavioural telemetry alone**, and §8
> gives the smallest observation design that makes it identifiable.

A paper whose contribution is an identifiability result and a measurement design does not need to be
first. A paper whose contribution is "we applied a Kalman filter" does.

---

## 2. The hazard: when a Kalman filter is decoration

State it first, because it is the failure this design is built to prevent.

If the discrepancy is defined as

```
g_t = (human estimate) - (AI estimate)
```

then estimating it requires no filter. It requires subtraction. A Kalman filter added on top would be
a formal apparatus around a tautology, and it would produce a covariance matrix that implies
precision the quantity does not have.

**The filter earns its place only if all three hold:**

1. the state is **not directly observable** — there is no number you can measure that *is* the gap;
2. it **evolves** — a static gap is estimated once and needs no dynamics;
3. at least one observation is a **noisy proxy** that also responds to something else, so the
   inference is genuinely inferential.

§4 satisfies all three. Then §6 requires the filter to **beat subtraction** on synthetic data with
known ground truth. If it does not, the honest conclusion is that this is a measurement-design paper
and not a filtering paper, and the pre-registration commits to writing that sentence.

---

## 3. The central obstacle: identifiability

This is the section the rest of the document exists to support.

### 3.1 Why a filter can be confidently wrong

A Kalman filter maintains `(x̂, P)` where `P` is the posterior covariance. A small `P` means the
filter is *confident*. It does not mean the filter is *right* in the sense of recovering the true
state. If the state is unobservable given the observation model, the estimate is determined along a
null space by the prior, and `P` can be small in every direction because the prior was tight.

That distinction — confident versus identified — is the one this repository keeps finding in other
guises: the fluency illusion in the theory document, the single-cell verdict in the comprehension
paper. Here it appears as a filter that reports a gap with a tight band.

### 3.2 The formal test

For a linear state-space model `x_{t+1} = F x_t + w`, `z_t = H x_t + v`, the state is observable iff
the observability matrix

```
O = [ H ; H F ; H F² ; ... ; H F^{n-1} ]
```

has full column rank. Rank deficiency is not a small-sample problem that better data fixes; it is a
statement that some direction in state space never affects any measurement, and no estimator recovers
it.

`perception_gap` therefore computes `rank(O)` and its null space **as a first-class output**, before
any filtering is done, for every declared observation model. A design whose null space is
non-trivial is reported as unidentifiable rather than filtered.

### 3.3 The confounding that makes this real

The gap is generically unidentifiable from behaviour, and the reason is simple once stated:

> **Competence and confidence push the same observables in the same direction.**

A confident-but-incompetent engineer reviews quickly. So does a competent one. Review time therefore
loads on both the true state and the believed state *with the same sign*, and the difference between
them — which is the quantity of interest — is invisible in that channel.

Formally, if `H` has two columns whose rows are proportional, `rank(O)` collapses. The pre-registration
sweeps the **cross-loading coefficient** (how strongly confidence moves review time independently of
competence) and reports the value at which identifiability fails. That coefficient is the paper's
most important unknown, and it is unmeasured.

### 3.4 The consequence for the research design

The filter cannot be rescued by more data or a better implementation. It can only be rescued by an
**observation the confound does not touch**, which is a design question about what to measure, not a
statistical question about how to estimate. §8 specifies it.

---

## 4. The model

### 4.1 State

Two latent scalars, not one, because the gap is a difference and differences of unobservables are the
thing being estimated:

```
x_t = [ τ_t , β_t ]ᵀ
```

- `τ_t` — **true** competence on the task class: how much of the work the person can actually
  diagnose, extend and review correctly.
- `β_t` — **believed** competence: the person's own estimate of the same quantity.

The quantity of interest is the gap `g_t = β_t − τ_t`. It is never observed.

### 4.2 Transition — assistance pushes the two apart

With `a_t` the AI adoption share in month `t`, treated as a **known input**, so that the state
transition stays linear and a standard Kalman filter is exact:

```
τ_{t+1} = τ_t + κ_a · a_t · (τ_floor − τ_t)   + w_τ
β_{t+1} = β_t + κ_b · a_t · (β_ceiling − β_t) + w_β
```

The two coefficients carry the mechanism from `documents/THE_BY_PRODUCT_THEORY.md` and have opposite
signs in effect: assistance **lowers** true competence toward a floor and **raises** believed
competence toward a ceiling. The gap therefore widens from both directions, which is the theory's
claim stated as a state equation.

Both equations are affine in the state given `a_t`. **This is why a standard Kalman filter is exact
here**: the nonlinearity lives in the input, not in the transition. The usual objection — that
comprehension dynamics are nonlinear and need an EKF, UKF or particle filter — does not bind at this
level of specification. If a later version puts `a_t`'s effect inside the covariance or makes the
transition depend on `τ_t` multiplicatively, that objection returns and the pre-registration requires
the filter choice to be justified then rather than presumed now.

### 4.3 Observation — five channels, two of them confounded

| Channel | Symbol | Loads on | Confounded? |
|---|---|---|---|
| Review time per change | `z₁` | `τ` (slower when less competent) **and** `β` (confident people review less) | **yes — this is the sweep** |
| Defect escape rate | `z₂` | `τ` only | no |
| Measured time saved on a task | `z₃` | `τ` only | no |
| Self-reported time saved | `z₄` | `β` only | no |
| Self-reported confidence | `z₅` | `β` only | no |

```
z_t = H x_t + v_t ,   v_t ~ N(0, R)
```

With `H` as declared, the state is observable: `z₂` and `z₃` see `τ` alone, `z₄` and `z₅` see `β`
alone. **That is the identified design.** The confounded design is the same model with the review-time
cross-loading set to its true, unknown, probably non-zero value — and §6 requires both to be reported.

### 4.4 What the filter produces

Not a discrepancy number. A trajectory `ĝ_t = β̂_t − τ̂_t` with a band, and — the part that is new —
**which observation channel carries the information**. That is answerable from the filter: the
information contribution of channel `j` is the reduction in posterior variance attributable to it,
`tr(P_{t|t-1}) − tr(P_{t|t})` with channel `j` alone, averaged over the run.

No existing work reports which proxies are informative about the AI–human gap, because none models the
gap as a latent state. That is the output this design is built to produce even if the filter itself
turns out to add nothing over subtraction.

---

## 5. Parameters and provenance

### Sourced

| Quantity | Value | Source | Use |
|---|---|---|---|
| Human–LLM collaborative estimates beat either alone | direction, not magnitude | FSE 2025 (verified, §1.2) | justifies modelling two estimator streams rather than one |
| Perceived versus measured AI speedup diverge | 20% believed vs 19% measured slower | METR, verified in `AI_AUTHORING_AND_COMPREHENSION.md` §1.2 | the initial condition `g_0 > 0`; the model must be able to reproduce this sign |
| Review burden rises with AI share | direction only | BairesDev Q3 2026, verified | sign of the `z₁` loading on `β` |

### Declared — every number below is invented

| Parameter | Symbol | Base | Swept | Why |
|---|---|---:|---|---|
| Assistance effect on true competence | `κ_a` | 0.05 | 0.02–0.15 | per month at full adoption; sets how fast `τ` falls |
| Assistance effect on believed competence | `κ_b` | 0.08 | 0.02–0.20 | deliberately larger than `κ_a`, which is the theory's claim |
| Competence floor | `τ_floor` | 0.35 | 0.2–0.5 | assistance cannot remove all diagnostic ability |
| Belief ceiling | `β_ceiling` | 0.95 | 0.8–1.0 | near-total confidence |
| Process noise, true | `σ_τ` | 0.03 | — | monthly drift beyond the systematic term |
| Process noise, belief | `σ_β` | 0.02 | — | beliefs are stickier than competence |
| **Review-time cross-loading** | `γ` | **0.0** | **0.0–0.5** | **the identifiability parameter**; `γ > 0` is the confound |
| Measurement noise, per channel | `σ_j` | 0.10 | 0.05–0.2 | proxy reliability |
| Horizon | — | 36 months | — | matches the other two models |

### Unverified

| Quantity | Why |
|---|---|
| The novelty of the topic | §1.3 |
| The true value of `γ` | no measurement exists of how strongly confidence moves review time independently of competence — **this is the paper's most important missing number** |
| Whether `τ` is one-dimensional | competence is task-class specific; treating it as scalar is a declared simplification (§9) |

---

## 6. Hypotheses and the pre-registered decision rule

### 6.1 The three competing claims

| Claim | Statement |
|---|---|
| **H₁ — the gap is identifiable and the filter helps** | Under the identified observation design, the filter recovers `g_t` from proxies, beats subtraction and beats a static estimator, on both RMSE and interval calibration |
| **H₂ — the gap is identifiable but the filter adds nothing** | Subtraction from the direct channels is as good; the filter's dynamics contribute no measurable information. Then this is a measurement-design paper, and says so |
| **H₃ — the gap is not identifiable** | At plausible `γ`, the observability matrix is rank-deficient and the estimate is prior-determined. Then no amount of modelling recovers the gap from telemetry, and the result is a specification of what must be measured instead |

All three are outcomes the design can reach. **H₂ and H₃ are not failures of the project**; they are
the findings that would stop a weaker version of this paper from being written.

### 6.2 The rule, fixed in advance

Ground truth is synthetic and known, which is the only way to check a state estimator: simulate
`x_t` from §4.2, generate `z_t` from §4.3, filter, and compare `ĝ_t` with the true `g_t`.

| Verdict | Condition |
|---|---|
| **H₁ wins** | On the identified design, `RMSE(ĝ) / RMSE(subtraction) ≤ 0.80` **and** `RMSE(ĝ) / RMSE(static) ≤ 0.90`, **and** empirical coverage of the 68% band lies in **[0.60, 0.76]** |
| **H₂ wins** | The filter meets the coverage criterion but fails to beat subtraction by the 20% margin |
| **H₃ wins** | `rank(O) < 2` for `γ` at or below the value declared plausible, **or** the filter's interval coverage collapses below 0.40 while its reported band stays narrow — the confident-and-wrong signature of §3.1 |
| **Indecisive** | H₁'s margin is met only at `σ_j ≤ 0.05` for all channels, i.e. only with implausibly clean proxies. Report the binding constraint |

The coverage criterion is the one that matters and the one easiest to skip. A filter whose band
contains the truth 40% of the time while claiming 68% is not a working estimator, however small its
RMSE.

### 6.3 Falsification of the enterprise

If, across the declared parameter ranges, no observation design achieves `rank(O) = 2` except by
measuring `τ` and `β` directly, then the gap is not estimable from proxies at all and the topic
reduces to a measurement instrument. The pre-registration commits to reporting that in those words.

---

## 7. The baselines, and why the filter must beat them

Three estimators on the same data. This is what stops the paper being decoration.

1. **Subtraction.** Take the direct channels — `z₃` for `τ`, `z₅` for `β` — and form `ĝ = ẑ₅ − ẑ₃`.
   No dynamics, no filter, one line of code.
2. **Static Bayes.** Invert `H` per time step with no state transition. Uses every channel, assumes
   the state does not evolve.
3. **The Kalman filter.** Dynamics plus all channels.

**H₁ requires the filter to beat (1) by 20% on RMSE and (2) by 10%.** If it does not, the dynamic
state is not earning its keep and the honest report is H₂.

The **control** is the identified-versus-confounded pair: the same filter, the same data-generating
process, `γ = 0` versus `γ` at its declared plausible maximum. H₃ is read off the difference. Without
that pair, a poor estimate cannot be attributed to the confound rather than to a bug.

---

## 8. What restores identifiability: an observation, not a statistic

If §3 is right, no filtering recovers the gap from passive telemetry, because competence and
confidence are confounded in every behavioural channel. The fix is an observation where the confound
cannot enter.

Four candidates, in increasing cost:

| Probe | What it does | Why it breaks the confound |
|---|---|---|
| **Prediction before performance** | ask the engineer to predict their score on a task, then measure it | `β` is measured against a realised outcome, so it is anchored to `τ` at a single point rather than inferred from behaviour |
| **Explanation demand** | ask them to explain a module's failure modes before being told the answer | the illusion-of-explanatory-depth literature says the overrating collapses on attempt; this *moves* `β` toward `τ` and observes both |
| **Informativeness manipulation** | give a hint that raises confidence without raising ability | moves `β` alone, which is a natural experiment for `γ` |
| **Randomised review regime** | generative versus passive review, randomised within team | moves `τ` alone in the short run, since beliefs about productivity do not update in a month |

The fourth is the one this repository can specify most concretely, because `comprehension_cost` already
parameterises generative review (§5.5 of `AI_AUTHORING_AND_COMPREHENSION.md`) and measured that it
recovers about 70% of the comprehension gap. That makes it a **usable instrument**: randomise review
mode, hold belief fixed, and `τ` moves. Any study of the perception gap should randomise review mode,
because it is the cheapest available lever that separates the two states.

**The measurement this design specifies, in one sentence:** a within-person design with a
pre-declared explanation probe and a randomised review regime, instrumented on the five channels of
§4.3, reporting `ĝ` with its band and the information contributed by each channel.

---

## 9. Scope, and what is not modelled

- **One agent, one task class.** Competence is task-specific; a scalar `τ` is a declared
  simplification. A multi-task model is a different paper.
- **No team effects.** `TEAM_DEPENDENCIES.md` argues that the externality is real; here it is excluded
  so that the identifiability result is about the cognitive quantity and nothing else.
- **No nonlinear dynamics.** §4.2 keeps the filter exact by putting `a_t` in the input. If a later
  version moves it into the transition, the EKF/UKF/particle choice must be justified then.
- **No real data.** This is a simulation whose purpose is to establish whether the quantity is
  estimable at all, and to specify the study that would measure `γ`.

---

## 10. What would make this worthless

1. **Defining the state as the difference.** Then the filter is a subtraction with a covariance matrix
   attached. §2 and §7 exist to prevent this and to detect it if it happens.
2. **Reporting the estimate without the observability rank.** A tight band on an unidentifiable state
   is the most misleading object this project could publish, and it would look exactly like a success.
3. **Choosing `γ` after seeing whether the filter works.** `γ` is the identifiability parameter;
   it is swept, and the sweep is reported in full, including the value at which the estimator dies.
4. **Claiming novelty.** Marked `Unverified` in §1.3, and the contribution is designed not to need it.
5. **Mistaking a synthetic success for a finding about engineers.** The filter will be validated
   against data this model generates. That establishes whether the estimator works on its own
   assumptions, which is all a simulation can establish.
