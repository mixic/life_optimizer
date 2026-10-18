# The Psychological Layer: What Assistance Does to the Capacity That Judges It

*Code in the `human_capacity` crate (`src/lib.rs`, 15 tests in `tests/psyche.rs`). CLI mode
`--psyche` in `multipolar_sim`. Figures in
`figures/psychology-*.png`, drawn by `tools/plot_psyche.py` from the simulator's own CSV export.
A paper, not a result: every parameter in the model is declared, none is calibrated, and §11 says
what the model cannot establish.*

---

## 1. The claim, and why it is not the laziness story

The standard worry about AI at work is that it makes people lazy. That worry is comfortable
because it is answerable: lazy people can be told to try harder, and the tool can stay the same.
This document is about a different and less comfortable claim.

**The risk is not that assistance makes people lazy. It is that assistance removes the practice
that builds the capacity to judge, while the machine sets the tempo that decides how much
assistance is needed — and then nobody is choosing.**

Three things follow, and they are the load-bearing claims of everything below.

1. **The relationship inverts.** What is normally described as human-to-machine ("I use a tool")
   becomes machine-to-human ("the tool sets the rhythm I must keep"). The person adapts to a
   reference they did not choose, and the adaptation is what removes the practice.
2. **The capacities at risk are not the ones instruments measure.** Artistic, emotional, social
   and psychological capacities are built by doing the thing, and decay when it is done for you.
   They are also, unlike throughput, not in any performance review.
3. **The objective therefore cannot see the loss.** Output rises. Quality of what the person
   *catches* falls. Effective output — the product — still rises. So the myopic objective is not
   self-defeating; it is **incomplete**, which is worse, because a self-defeating objective would
   correct itself.

The last point is the one this document is built around, and it is the one that makes the
argument hard to dismiss: a reader who asks "but does the work get worse?" will find the model
answering "no, on the whole it gets faster and about as good in the aggregate, and that is exactly
the problem."

## 2. What is not claimed

- **Not that AI makes people stupid.** The model has no intelligence term. It has *capacity to
  judge what was produced*, and it treats that as a stock built by practice and spent by disuse.
- **Not that assistance is bad.** In most of the parameter space the model prefers more
  assistance, and §8 computes exactly where that stops being true. The model can argue for
  assistance as easily as against it. What it forbids is assuming that more is always better.
- **Not that machines are conscious, or that they are not.** Nothing here depends on machine
  mentality.
- **Not that any of these numbers is an estimate of anything.** Every parameter is declared; see
  §4.5 and §11.
- **Not a forecast.** The horizon is thirty years because capacity changes slowly, not because
  thirty years of this model describes the next thirty years of the world.

## 3. The inversion: assistance is solved for, not chosen

The mechanism is a single sentence in the model and it is worth stating before any algebra.

A person's period has to clear a bar. The bar is set by what counts as fast enough, which is set
by the machine's tempo. Output within a period is throughput — raised by assistance — times
quality — which depends on the capacity the person has. So for a person of given capacity facing
a given bar there is a **minimum assistance** that clears it:

```
required_assistance(bar, capacity) = clamp[ (bar / quality(capacity) − 1) / gain, 0, 1 ]
```

Nothing in that expression is a preference. The person who would rather keep the practice cannot
keep it, because the alternative is failing to clear a bar they did not set. This is the inversion
made numeric: assistance appears as the *solution* to a constraint, not as a decision variable.

The consequence is that the model's central object is not a utility function. It is a threshold,
and the threshold is about the tempo rather than about anyone's taste.

### 3.1 Why the runway is the only policy lever in the model

The bar does not rise immediately. An employer who buys a team time before expectations follow the
machine's tempo is buying the one thing that changes the answer: a period during which the bar is
still low, assistance can stay low, and practice can happen. That delay is the model's only
exogenous policy variable, and it is the parameter the phrase *"having the runway to learn is an
edge"* refers to. Whether teams get it before expectations land is, as §6.4 shows, a question the
model can pose more precisely than it can answer.

## 4. The model

### 4.1 The state

For each person and year, `State` carries: assistance actually used; capacity; self-model;
autonomy; quality of what the person catches; throughput; effective output; residual defect rate;
emotional coherence. Capacity and self-model are stocks with build and decay; the rest are
functions of them and of the bar.

### 4.2 The autonomy equation

```
autonomy = self_model × capacity × (1 − assistance)
```

Each factor is necessary, and the multiplication is the entire claim. Any one at zero gives zero
autonomy whatever the others are:

- capacity without self-knowledge is competence that cannot say what it wants;
- self-knowledge without capacity is an opinion with nothing behind it;
- both, without doing the work, is a person who has outsourced the judgement while retaining the
  vocabulary for it.

That third case is the one the argument is about, and it is why the term is `(1 − assistance)`
rather than a quality deduction: at full delegation the person is *not judging less well*, they
are not judging.

### 4.3 Two channels, not one trade-off

The first version of this model had a single practice term in which assistance was multiplied by a
substitution coefficient and *boosted* by a scaffolding coefficient, and it could not answer its
own central question — "does assistance help?" — without first assuming an answer, because the two
effects fought inside one expression. The version here separates the activities:

```
producing:  practice_production(a) = max(0, 1 − σ·a)      σ = substitution
learning:   practice_learning(a)   = 1 + λ·a               λ = scaffolding
capacity change = κ[(1−t)·practice_production + t·practice_learning]·(1−h) − δ·h
```

where `t` is the share of the period spent learning and `h` is capacity. Assistance **removes reps
from producing** and **adds efficiency to learning**, and the training share decides which of those
reaches the person. This is the model's real content, and it makes the answer conditional in the
right way: the same tool deskills a person who spends the period producing under it, and upskills a
person who spends it learning.

The two channels cancel at the break-even training share `t* = σ/(σ + λ)`, and §5.1's test asserts
that assisted practice equals unassisted practice exactly there — not that build equals decay,
which is a different question with a different function.

### 4.4 The bar, and what bounds it

```
bar(year) = 1                                        during the runway
bar(year) = min( (1 + tempo)^(year − runway), ceiling )   after it
```

The ceiling is a declared assumption (A10) and it was added after the first version failed in a
way worth recording: with the bar compounding without limit, a 10% tempo over thirty years demanded
fourteen times baseline, every person in the cohort delegated everything, autonomy was identically
zero, and the cohort collapsed to a single point. That is not a finding about deskilling; it is a
model that has stopped describing a population. Expectations are bounded by what the work is worth
to whoever commissions it, not by how fast the machine can go.

A second collapse followed and needed a second fix. With one ceiling for everyone the recursion has
a single stable fixed point, so every person converges to the same capacity however differently
they started — `p10 = median = mean`, and every dispersion statistic in the summary measured
nothing. Roles differ in the bar they settle at, so the ceiling is drawn **per person** (A11). That
is the only heterogeneity that survives to the horizon, and it is why §6.2's second panel is a
scatter and not a histogram.

### 4.5 The parameters, all declared

Fifteen parameters, every one `Provenance::Declared`, and `provenance_table()` says so beside each.
The reason is not modesty: no published measurement exists for a substitution coefficient, an
emotional coherence stock, or the rate at which judgement tracks a machine's register. These are not
quantities that were estimated and rounded. They are quantities for which no estimate exists.

The consequence is a discipline that runs through the document: **the model's contribution is its
structure and its thresholds, never its levels.** A reader who wants to refute it should refute the
structure — A1/A2 (the two channels), A3 (the build and decay shape), A5 (that the tempo sets the
bar) and A10/A11 (that the bar saturates and differs by role). The arithmetic will follow.

## 5. Three structural results

These do not depend on the Monte Carlo and are checked by tests rather than by inspection.

### 5.1 The deskilling threshold

Setting build below decay at full assistance and inverting for the substitution coefficient gives

```
σ* = 1 − [ δ·h / (κ(1 − h)) − t(1 + λ) ] / (1 − t)
```

which at the declared defaults and `h = 0.7`, `t = 0.05` is **0.705**. Full assistance erodes
capacity once the substitution coefficient exceeds that, and the declared default is 0.80, so at the
default training share the model sits *above* its own threshold.

Two things about this deserve emphasis.

**The inversion is where a sign error hides.** It is easy to divide the `κ` term by `(1 − t)` and
not the whole bracket. The wrong inversion still returns a plausible number in `[0, 1]` and the
dynamics quietly disagree with it. This repository's version of that bug is recorded in §14: the
threshold was reported at 0.45 while the dynamics deskilled at 0.70. The fix is a test that probes
the dynamics on both sides of the closed form, and it is the only kind of test that catches it.

**The threshold falls with capacity, which predicts that experts lose fastest.** That is a
consequence of the declared `(1 − h)` build shape — at high capacity there is little room left to
build, so disuse wins — and it should be treated with suspicion rather than enthusiasm. Panel (a) of
`figures/psychology-2-thresholds.png` shows it. It is a prediction of this model, not an observation
about skilled people, and §11 lists it as a limitation.

### 5.2 The defect floor: perfection is not purchasable

If the same machine writes an artefact and its tests, a share `ω` of defects are *characteristic* of
that machine's distribution — the failure modes it does not represent and therefore does not think
to test for. Those defects are caught by neither reviewer at the machine's own detection rate. The
residual rate is

```
residual(h) = ω(1 − ρ_machine) + (1 − ω)(1 − ρ_human·quality(h))
```

and its floor is `ω(1 − ρ_machine)`, which no amount of assistance removes because the binding
constraint is the **correlation**, not the effort. At the defaults that floor is **0.280**: more
than a quarter of the characteristic defects survive however good the tooling is. Perfection is not
for sale, and no QA budget buys it.

This is also the model's sharpest reply to "we will test it with AI from the specifications". That
practice is good — it is the scaffolding channel, and it raises human detection. It cannot move the
floor, because the floor is set by what both processes are blind to *together*.

### 5.3 The emotional inversion

Coherence is built by interaction with other people and worn by entrainment to a machine register:

```
coherence(H) = αH / (αH + β(1 − H)),     H = share of interaction that is with a person
```

Monotone in `H`, with a critical mix. At the declared `α = 0.6`, `β = 0.35`, coherence falls below
0.6 once the human share drops beneath **0.467**; the declared default mix of 0.55 gives 0.677. The
inversion is not that the machine is cold — it is that the *equilibrium register* is the machine's
whenever the machine's share dominates, and the person adapts to it rather than the reverse.

This channel is the weakest in the model and is labelled as such: it is a static equilibrium with no
dynamics, and the two coefficients are declarations. It is included because omitting the emotional
channel from an argument about psychological capacities would be a strange omission, not because it
carries any weight.

## 6. The Monte Carlo

Command: `cargo run -p multipolar_sim --release -- --psyche`. A cohort of 2,000 is drawn once and
run twice on the same seed; the default run takes well under a second.

### 6.1 Design: the same people twice

The cohort varies in starting capacity (0.35–0.95), self-knowledge (0.05–0.75), the share of
interaction that is human, and the bar the person's role settles at. Two arms:

- **tracked** — assistance solved for each period so the person clears the bar;
- **held** — the same people with assistance frozen at its initial level.

The second arm is a counterfactual, not a choice anyone has. It exists because the model's claim is
about a *gap*: what the tempo costs is the difference between what these people would have had and
what they get, and the only way to see that difference is to run them twice.

### 6.2 Two collapses, and why they matter

The Monte Carlo found two defects that the single-person trajectory could not, and both are recorded
because they are the kind of error that produces a paper rather than a correction:

1. **Unbounded expectations** pinned every person at full delegation. Autonomy was identically zero
   and the cohort was one point. Fixed by A10.
2. **A common bar** gave every person the same fixed point even with A10, because the recursion is
   contractive and forgets its initial condition. `p10 = median = mean` at the horizon. Fixed by
   A11.

Both failures have the same shape: a statistic that looks like a result and is actually a constant.
The test `the_cohort_keeps_dispersion_at_the_horizon` now asserts that this cannot happen silently.

### 6.3 What the cohort says

At the declared defaults, over thirty years:

| | tracked | held | difference |
|---|---:|---:|---:|
| mean capacity | 0.776 | 0.834 | −0.058 |
| p10 capacity | 0.705 | 0.834 | −0.129 |
| median capacity | 0.787 | 0.834 | −0.047 |
| mean autonomy | 0.223 | 0.411 | −0.187 |
| mean effective output | 1.502 | 1.138 | +0.364 |
| mean residual defects | 50.1% | 46.9% | +3.2pp |
| share below the declared autonomy floor | 30.9% | 0.0% | +30.9pp |

Read that table with §1 claim 3 in mind. Tracking **raises effective output by a third** and **lowers
capacity by seven percent**, and the loss is concentrated: the bottom decile loses more than twice
what the mean loses. The held arm has no dispersion at all — with assistance fixed, nothing in the
recursion varies by person, so capacity converges to the same point for everyone. All the
heterogeneity in the tracked arm comes from the bar a role settles at, which is why panel (b) of
`figures/psychology-3-cohort.png` plots capacity against the bar and gets a curve.

### 6.4 The runway changes when, not where

This is the model's most interesting result, and it was found by measuring the wrong thing first.

Six extra years of runway moves mean capacity at the horizon by **+0.002** — nothing. A reader
looking for "the runway is an edge" in that number would conclude it is not one. But the runway is
not buying a different endpoint; it is buying **years spent judging**:

| runway (years) | mean years above the autonomy floor | mean capacity at horizon |
|---:|---:|---:|
| 0 | 18.2 | 0.775 |
| 1 | 18.9 | 0.776 |
| 2 | 19.4 | 0.776 |
| 3 | 20.0 | 0.777 |
| 4 | 20.5 | 0.777 |
| 5 | 20.9 | 0.777 |
| 6 | 21.4 | 0.778 |

Six years of runway buys **3.2 years of autonomous practice** and almost no change in where the
person ends up. The mechanism is that the endpoint is set by the bar the role settles at, and a
contractive recursion forgets its initial condition within a decade. The runway therefore decides
*how long the person has before the bar arrives*, which is exactly the period in which the inner
work happens — and not whether they end up delegated.

Two consequences. The encouraging one: the runway is the only variable in the model an employer or
funder controls, and it has a large, monotone, defensible effect on the quantity that matters. The
dispiriting one: it does not change the destination, so a policy that buys runway is buying time
rather than outcome. A test asserts both halves, so that anyone changing the dynamics will be told
when the document's sentence about the runway has stopped describing the code.

### 6.5 The tempo costs years and buys no output

| tempo | mean years above the floor | mean capacity |
|---:|---:|---:|
| 0.00 | 28.0 | 0.834 |
| 0.05 | 21.2 | 0.778 |
| 0.10 | 19.4 | 0.776 |
| 0.15 | 18.9 | 0.776 |
| 0.30 | 18.2 | 0.775 |

A ten-fold faster tempo costs **8.6 years of autonomous practice** and about 0.06 of capacity, and
the throughput column is flat across the whole range because the bar reaches its ceiling either way.
The speed decides how quickly the person is cornered, not how much they end up producing.

### 6.6 The nearest documented instance, and why it is not a measurement

This model has one historical analogue worth citing, and it should be cited carefully because it is
the closest thing to evidence in this document.

In September 2013 the FAA's Flight Deck Automation Working Group published *Operational Use of
Flight Path Management Systems*, an evidence-based review whose underlying themes include
"concerns about degradation of pilot knowledge and skills" ([report summary and full text](https://skybrary.aero/articles/operational-use-flight-path-management-systems)).
Its 28 findings produced 18 recommendations, two of which are the model's mechanism stated by a
regulator:

- **Recommendation 1, Manual Flight Operations** — pilots "must be provided with opportunities to
  refine this knowledge and practice the skills";
- **Recommendation 9, Operational Policy** — the policy should stress "that the responsibility for
  flight path management remains with the pilots at all times", and should "recognize the importance
  of automated systems as a tool (among other tools)".

Three things about it matter here, and two of them are corrections to how it is usually told.

First, **automation was not rolled back.** What was redesigned was training, procedures, policy and
design philosophy, with accountability re-anchored on the pilot. The lesson is not "use less
automation"; it is "the same automation with a protected practice is safer than the same automation
without one" — which is exactly what this model's training share is for.

Second, **it took seventeen years.** The 2013 review was the first comprehensive one since a 1996 FAA
report on flightcrew interfaces, and the working group noted further accidents after its own data
collection with vulnerabilities similar to those it had identified. Institutional learning on this
question is measured in decades, which is the opposite of reassurance for a technology moving in
years.

Third, **it is not a measurement of this model.** Aviation is a safety-critical, heavily regulated,
tightly instrumented domain with a professional licensing regime; knowledge work is none of those
things. What the report documents is that a regulator with good data and enforcement power needed
seventeen years to act on skill degradation. Reading it as evidence that the same will happen here is
a projection, and §11 lists the absence of a knowledge-work equivalent as the model's central gap.

## 7. Who is actually deskilled

The question the previous sections keep deferring is the one a reader will ask: *is anyone actually
worse off?* The answer is conditional, and the condition is the bar.

Same well-trained person, capacity 0.75 at the start, against the bar their role settles at:

| settled bar | assistance at horizon | capacity at horizon | change from 0.75 | effective output |
|---:|---:|---:|---:|---:|
| 1.10 | 0.200 | 0.834 | **+0.084** | 1.138 |
| 1.30 | 0.338 | 0.817 | **+0.067** | 1.300 |
| 1.50 | 0.522 | 0.789 | **+0.039** | 1.500 |
| 1.70 | 0.738 | 0.742 | **−0.008** | 1.700 |
| 1.90 | 1.000 | 0.650 | **−0.100** | 1.843 |
| 2.20 | 1.000 | 0.647 | **−0.103** | 1.838 |
| 2.50 | 1.000 | 0.647 | **−0.103** | 1.838 |

This table is the honest centre of the document, and it says something a reader expecting a decline
will not expect: **at the declared default bar, the person ends above where they started.** The
decline begins at a bar of about 1.7 and saturates at about 1.9, where full delegation pins and the
person loses a tenth of their capacity. Effective output, meanwhile, rises to a maximum around
1.9–2.2 and then falls very slightly.

So the model's claim is not that assistance lowers capacity. It is that **deskilling appears once the
bar is high enough to force near-total delegation — and the bar, not the tool, is what decides.**

That is simultaneously the argument's strongest form and its weakest point, because the bar is the
one quantity in the model that nobody has measured. A reader who doubts the argument should attack
the bar; this table is where to do it. Two declared numbers would move it: the ceiling (A10) and the
tempo (A5). Neither is an empirical estimate, and §11 says so.

## 8. Where full assistance stops being optimal

The argument's condition can be computed rather than asserted. For a representative person
(capacity 0.70, self-knowledge 0.40), searching assistance on a grid for the level that maximises
mean effective output over the horizon gives a phase boundary with two axes:

- **speed-up:** holding back becomes the better policy below a throughput gain of **0.70**;
- **substitution:** at the current speed-up of 1.5, holding back becomes optimal once the
  substitution coefficient exceeds **0.87** (the declared default is 0.80).

`figures/psychology-5-phase.png` draws the whole plane. The region where maximum assistance is
optimal is most of it, which is the model being honest about its own direction: the argument for
holding back is an argument about a corner, not about the general case.

Note that this boundary and the deskilling threshold are **different numbers** (0.87 and 0.705 at
their respective settings): deskilling is about *capacity*, optimality is about *effective output*,
and output is maximised by trading the two off. A model in which they coincided would have collapsed
a trade-off into a single quantity.

## 9. Faster, but of questionable quality — what the model does and does not say

The intuition that the job is done faster and the quality is questionable is *half* in the model,
and the honest report is which half.

**In:** quality falls with capacity, residual defects rise, and both are visible — the tracked arm
carries 3.2 percentage points more residual defects, and the bottom decile of capacity is 0.129 below
the counterfactual.

**In:** the defect floor. No amount of assistance, review, or testing removes the share both
processes are blind to.

**Not in:** effective output falling. It rises by a third. The model does not support the claim that
maximal assistance is self-defeating on its own terms, and a document that claimed so would be
misreading its own simulation.

What follows is a different argument, and the one this document actually makes: the objective that
rises is the objectively measurable one, and the thing that falls — the capacity to judge, and the
autonomy to disagree — is the thing that makes the output worth having. That is not a claim a
dashboard can refute, because it is not a claim about the dashboard.

## 10. A position the author holds, and why the argument does not depend on it

This section is quarantined deliberately. It records a position, it does not support the argument,
and the argument stands or falls without it.

### 10.1 The position, as stated by the author

The author's view is that a person is not only a body: that consciousness may be understood as a
field rather than as an emergent property of computation, that the body and brain act as
**biological constraint operators filtering a non-local field** rather than as its generator, and
that this framing is continuous with work on a "Wave Genome", junk DNA as a fractal antenna, and
biophoton signalling. The author's own note on this, given with the position, is that the mainstream
scientific community classifies the Wave Genome as pseudoscience.

That self-classification is the most important sentence in this section. It is included because
omitting it would misrepresent the author's epistemic situation, in which the position is held as a
conviction while its mainstream status is acknowledged.

### 10.2 Faggin, and a correction to the attribution

The position draws on Federico Faggin's public arguments: that consciousness is fundamental and
cannot emerge from unconscious matter; that a classical computer is "two steps removed" from what a
brain does; that people who treat AI as the expert "will become less and less intelligent. And they
can be manipulated."

**A correction, recorded because it matters.** A widely circulated paraphrase — that humans "did not
do their work investigating who they are, and without this knowledge they can be led by AI
machinery" — is **not a quotation and not textually his**. Research for this document found no
passage in which he says people will be "led" by AI. What he says are three separate things:

- cognitive atrophy through deference (L'Espresso, 21 May 2026, as reported);
- that those who cling to a materialist self-conception will end up "surpassed and enslaved by
  computers" (VareseNews, 16 October 2025, as reported);
- that freedom requires self-knowledge — "only an awakened ego can actually have really free will"
  (Know Thyself interview, 27 August 2024, surviving as auto-captions).

A plausible vector of the distortion is worth naming, because it is itself an instance of the
document's subject: an explicitly AI-generated news article ("produced by The Rio Times' automated
newsroom system", 2 February 2026) renders the same idea as technological control by elites. That is
machine-written journalism, not Faggin, and it shows exactly the drift this document is about — a
human position compressed by a machine into something more quotable and less careful.

Citation weight: his books are his own prose; the press quotes are journalists reporting live talks;
the interview transcript is auto-captioned and should not be quoted to the word. They are not of
equal evidential value and are not presented as if they were.

One link supplied with the source material, `lnkd.in/p/ef9c5WNz`, could not be resolved: it
redirects to the LinkedIn front page rather than to a post, and no content has been reconstructed
from it. Nothing here rests on that link.

### 10.3 Why none of it is load-bearing

The deskilling argument in §1–§9 is an argument about **practice, capacity and the bar**. It needs no
field, no biophoton, and no claim about the nature of consciousness. Its premises are that capacities
are built by doing and decay by not doing, that a machine can set the tempo of what counts as done,
and that the measurable objective cannot see the loss. A materialist who accepts those three
premises gets the same conclusion, and the model in `psyche.rs` contains no term that a materialist
would have to reject.

The field position, if it is right, would make the conclusion *more* urgent — a person who is a
constraint operator on a field is a person for whom the capacity to attend is the whole apparatus.
But the argument is constructed so that it does not borrow that urgency. Anyone who wants to reject
the metaphysics can reject all of §10.1 and §10.2 and still be arguing with §5, §6 and §7.

This separation is not a rhetorical courtesy. A document that needed the contested premise to reach
its conclusion would have made the conclusion as weak as the premise, and this one has not.

## 11. What is not established

Stated as a list because a list can be checked.

1. **That capacity to judge falls in the world.** No longitudinal measurement is used or produced
   here, and none is known to the author. The model says what would have to be true for the claim to
   hold, not that it is true.
2. **The size of anything.** Every parameter is declared. The levels in every table are illustrations
   of mechanism, not estimates of magnitude.
3. **Where reality sits on the deskilling table.** §7 shows the decline beginning around a bar of
   1.7. Whether real roles settle above or below that is an empirical question this model cannot
   answer, and it is the argument's weakest joint.
4. **Whether the build shape is right.** The `(1 − h)` term implies an expert loses capacity fastest,
   which is a consequence of a declared functional form and not a robust prediction. A model with a
   different learning shape would move §7's threshold.
5. **That assistance affects the self-model.** In this model it does not: self-knowledge is built by
   the training share alone, and the two arms coincide in that panel. This is a gap, not a finding.
   An argument for the field position would presumably want assistance to bear on self-knowledge
   too, and the model as built cannot test that.
6. **The autonomy floor.** 0.15 is declared. "Share below the floor" moves with it and nothing else
   does.
7. **The distribution of the bar.** The per-person spread of 25% around the model's ceiling is
   declared, and it is what produces all the dispersion in §6.3.
8. **Any labour-market equilibrium.** The bar is exogenous. Firms do not compete, wages do not
   adjust, and nobody notices that their competitors are delegating too. A model in which the bar
   rises *because* everyone delegated would be a different and more alarming model, and this is not
   it.
9. **That the emotional channel is real.** §5.3 is the weakest part of the model and is labelled so.

What would make the argument falsifiable, in the sense of putting it at risk:

- a longitudinal measure of judgement quality on tasks where assistance was available, which would
  give the substitution coefficient an estimate;
- an estimate of the bar — the multiple of pre-AI output that roles actually settle at — which is
  the single number §7 turns on;
- a test of whether time-to-expectation (the runway) protects the capacity, which is the model's
  only policy lever and its most testable prediction.

None of the three is a modelling problem. All three are measurement problems, and none is solved
here.

## 12. Relation to the rest of the repository

- **`AI_AS_CATALYST.md`** — the same question one level down. That document's `ρ` is the share of
  the speed-up consumed by defects; this document's capacity is what sets `ρ`. Its §8 on validating
  delivered quality is the engineering counterpart of §5.2 here: the same correlation that makes
  machine-written tests insufficient is what `ρ` cannot see.
- **`EARLY_RETIREMENT.md`** — the machinery of the myopic objective. §5 of that document optimises a
  plan over a horizon; §1 of this one explains why an agent that optimises the measurable thing will
  do the same to a person, correctly and to their cost. The `simplex.rs` dual is a good picture of
  what a shadow price looks like; the point here is that no price exists for the capacity.
- **`THE_RECIPROCITY_OF_THE_EDUCATED.md`** — the other side of the same moral ledger. That document
  asks what the educated owe a society that paid for their qualification; this one asks what is
  spent, silently, in the practice that qualification presupposed. Reciprocity presupposes a person
  capable of discharging it.
- **`STRATEGY_COMPARISON.md` and `THIRD_PARTY_SPOILER.md`** — the same structure of argument one level
  up: an actor optimising a measured quantity while the unmeasured one degrades. The deskilling
  threshold here is a cousin of the spoiler's crossing horizon.
- **`human_capacity`** — the model, as a library crate rather than a module, because the
  workflow model in `knowledge_work` composes with these dynamics instead of restating them;
  `multipolar_sim --psyche` is its command-line face; the fifteen tests live in
  `human_capacity/tests/psyche.rs`.

## 13. Reproduction

```bash
# the model, the cohort, the sweeps and the phase boundary
cargo run -p multipolar_sim --release -- --psyche

# the same, with the CSV the figures are drawn from
cargo run -p multipolar_sim --release -- --psyche --export out/psyche

# the figures, into the directory committed with this document
python tools/plot_psyche.py out/psyche figures

# the tests behind every number quoted above
cargo test -p human_capacity
```

Parameters can be moved from the command line: `--psyche-tempo`, `--psyche-runway`,
`--psyche-training`, `--psyche-autonomy-floor`, `--psyche-persons`, `--seed`. Every figure is drawn
from the CSV only; the plotting script never runs the simulation and never invents a number, and it
refuses to draw if the export is internally inconsistent — a guard that caught a real defect during
this work (§14, item 5).

The full workspace suite is 390 tests. The psyche module contributes 15, and each one is named for
the thing it would catch rather than for the function it calls.

## 14. Corrections recorded

Six defects were found while building this, and all six are listed because the repository's rule is
that a number that changed should say so.

1. **Substitution and scaffolding in one term.** The first model had a single practice expression with
   both coefficients, so "does assistance help?" had no answer independent of capacity. Replaced by
   two channels chosen by the training share (§4.3). The "neutral capacity" idea that went with the
   old form became the break-even training share `t* = σ/(σ + λ)`.
2. **A premise that was actually a conclusion.** The neutral-training assumption was fixed at a value
   that made assistance look neutral rather than honest. Replaced by the derived break-even share.
3. **Unbounded expectations.** §6.2, collapse one. Fixed by A10.
4. **A single common bar.** §6.2, collapse two. Fixed by A11.
5. **The threshold inverted wrongly.** Reported 0.45 where the dynamics deskilled at 0.70, because
   the `(1 − t)` was applied to one term instead of the bracket. Caught by probing the dynamics on
   both sides of the closed form (§5.1).
6. **A threshold that was not a coefficient.** `deskilling_threshold` returned values above 1, where
   no admissible substitution deskills. It now returns `None`, and the plotting script's consistency
   check is what surfaced it — a figure refusing to draw was the error message.

One methodological correction belongs with them: **the runway was first measured at the horizon**,
where it showed nothing, and the conclusion "the runway is not an edge" would have been an artifact
of measuring the wrong quantity. Measured as years spent above the autonomy floor it is worth 3.2
years for six years of runway (§6.4). The metric was wrong, not the model — but a document that had
reported the first measurement would have been wrong about its own subject.
