# Optimizing for What? Just-in-Time Production Logic, AI Adoption, and the Sustainability of Knowledge Work

*Code in `knowledge_work/` (30 tests) and `human_capacity/` (15 tests), which it composes with.
CLI `cargo run -p knowledge_work`. Figures in `figures/workflow-*.png`, drawn by
`tools/plot_workflow.py` from the model's own CSV export. Every parameter is declared; §9 states
the scope conditions and §10 states what the framework cannot establish.*

---

## 1. What this is, and what it deliberately is not

The question this work asks is not "does AI make knowledge work faster". It does, and the
interesting part begins after that. The question is:

> **Under what conditions does the intensification of knowledge work increase sustainable
> output, and under what conditions does it convert a productivity gain into fragility?**

A note on the ambition, because it was set high and then scoped deliberately. This is **not** a
theory of everything for the social world. A framework that claimed to cover the economic, the
political, the organisational and the psychological at once would not be a stronger claim than a
narrow one; it would be an unfalsifiable one, and this repository's standard — every claim sourced
or declared, and every model stating what it cannot establish — would reject it on sight.

What is here instead is a framework with **named scope conditions** (§9). It says: if these seven
things hold, then these conclusions follow. That is a weaker-sounding and much stronger claim,
because a reader can attack a scope condition and know what they have broken. The unification is
real without being total: the queue, the person, and the normative question are one model, and the
places where they are *not* one model are listed rather than papered over.

## 2. The transplantation, stated as a claim

Lean production and just-in-time logistics share a normative core: **buffer is waste, utilisation
is virtue, and variance should not be absorbed but eliminated or pushed to where it is cheapest.**

The claim of this document is narrow and testable: **AI adoption did not change that norm.** When
the machine multiplied the rate at which a team can *generate* work, organisations on the lean norm
did not bank the slack. They raised arrivals until utilisation was back where lean had left it. The
technology changed and the target did not.

That is a behavioural assumption, not a measurement. It is scope condition **S1**, it is the
framework's central causal claim, and the model's headline results are conditional on it. §6.5
prices the alternative rather than asserting that no alternative exists.

## 3. Is knowledge work actually a JIT system, or is this a metaphor?

A committee will ask this, and it deserves a real answer rather than a rhetorical one. The answer
is: **structural in five places, suggestive in six, and the framework claims only the five.**

### 3.1 Where the analogy is structural

The test for "structural" is that the same formal object applies without translation, and that the
conclusion would follow even if the vocabulary were never borrowed.

| Feature | In a JIT line | In knowledge work | Same formal object? |
|---|---|---|---|
| Buffer | Inventory between stations | Work in process in a review queue | **Yes.** Little's law relates both to throughput and cycle time |
| Utilisation | Run the line hot | Keep every reviewer booked | **Yes.** The same ratio, with the same convex cost |
| Variance pushed downstream | Defective parts reach assembly | Escaped defects reach production or customers | **Yes.** Same arithmetic, higher cost downstream |
| Effective process time | Includes rework and repair | Includes rework of caught and escaped defects | **Yes.** §4's load identity is exactly this |
| Bottleneck | The constraint sets the throughput | Review is the constraint | **Yes.** Adding capacity anywhere else does nothing |

These five are why the queue is not a metaphor. A review stage has servers, an arrival process, a
service-time distribution and a queue. The VUT approximation uses only the mean and the squared
coefficient of variation of two of those, which is a far weaker demand than specifying
distributions nobody has measured, and it is the same approximation whether the thing being served
is a gearbox or a pull request.

### 3.2 Where it breaks

| Feature | Why knowledge work breaks it |
|---|---|
| **Demand is exogenous** | In a factory, orders arrive from outside. In knowledge work the "order" is often discovered *by doing the work*, so the arrival process is partly a choice |
| **Specifications are the product** | A JIT line builds to a drawing. In software the drawing is frequently the deliverable, which is why rework costs more and why "get it right first time" cannot be enforced the same way |
| **Some buffers are learning** | Lean classifies slack as waste. The one buffer this framework cares most about — time to build the capacity to judge — is exactly what lean would remove first. This is not a nuance; it is the thesis |
| **The worker is the machine, and the machine degrades** | A lathe does not get worse at its job because it is used. A reviewer who reviews nothing but AI output does, and that feedback is what makes the model a system rather than a pipeline |
| **WIP is not physical** | You cannot count a half-finished design the way you count half-finished gearboxes, and the same artefact can be in several "queues" at once |
| **The customer is often internal** | The defect that escapes lands on a colleague, which is why no team-level metric counts it, and why the cost multiplier is systematically underestimated |

### 3.3 The verdict, and what follows from it

The analogy is **structural on the queueing layer and suggestive everywhere else**. So the
framework claims exactly the queueing layer plus the one mechanism that has no factory analogue:
the coupling between load and the capacity that catches defects.

That coupling is where the analogy earns its keep and where it stops being one. In a factory the
rework rate is a property of the process. Here it is a property of the *person*, and the person is
being changed by the same tool. Everything in §6 follows from that single difference.

## 4. The model

### 4.1 The load identity

One generated item consumes review time in three ways: it is reviewed, and then either its defect
is caught and reworked, or its defect escapes and is reworked later at greater cost.

```
L(e) = 1 + d·[(1 − e)·c_caught + e·c_escape]
```

with `d` the defect share, `e` the share of defects that escape review, and the two `c` terms the
review cost of fixing a caught or an escaped defect, in multiples of the base review time.

`L` is an **effective process time**, the same object as a lean system's total processing time
including rework. Utilisation is `λ·L/capacity`, and because `L > 1` a team that counts only
first-pass review systematically understates its own load. At the declared defaults
(`d = 0.20`, `c_caught = 0.50`, `c_escape = 3.00`) the load is **1.29**, so the understatement is
**29%**.

### 4.2 The utilisation term

Waiting time follows the VUT approximation, using only two coefficients of variation:

```
E[W_q] = V · U(ρ) · S / c,     V = (c_a² + c_s²)/2,     U(ρ) = ρ^(√(2(c+1)) − 1) / (1 − ρ)
```

`U` is the finding. It is convex and it diverges at one, so a system run at 0.95 has roughly twice
the waiting of one run at 0.90, and the last five points of utilisation cost more than everything
below 0.90 together. Lean's target of high utilisation is therefore not a neutral efficiency
choice: **it is a choice to sit near an asymptote.**

At one server the exponent is exactly one and the expression reduces to Kingman's `ρ/(1 − ρ)`,
which a test asserts.

**Measured accuracy**, reported rather than assumed: against the exact M/M/c result, the
approximation overstates waiting by up to **28.5%** at ρ = 0.50 with four servers, and understates
by at most **0.1%** at ρ = 0.98 with sixteen. In the light-traffic corner it is off by a factor of
67 at ρ = 0.30 with sixteen servers — where both answers are a ten-thousandth of one service time,
so neither is a queue. That corner is pinned by a test of its own. What matters for the argument is
the **direction** of the error: it errs toward a worse queue, so the envelopes derived from it are
not inflated.

### 4.3 Two thresholds

The framework keeps two failures separate because they are different findings:

- **stability** — effective utilisation reaching one, beyond which cycle time does not lengthen but
  grows without bound;
- **the commitment** — the cycle-time promise, which bites earlier and is a choice rather than a
  law.

At the declared defaults the commitment fails at review capacity **0.888** and stability at
**0.839**. The gap is the range in which the queue still converges while the promise is already
broken, and it is where most organisations actually live — because utilisation targets do not get
revised when lateness starts.

The `absorbable_share` is the operational form: the fraction of generated work that can be admitted
while still meeting the commitment, obtained by inverting the queue at the deadline. At 1.00 the
workflow can take everything it generates.

### 4.4 The parameters, and their provenance

Every one of the twelve is `Declared`, and `WorkflowParameters::provenance_table()` says so beside
each in the code and in the exported `provenance.txt`. The table is reproduced here because a
reader should not have to open the source to find out that nothing is calibrated.

| Parameter | Value | Provenance | Why the value is defensible in sign |
|---|---:|---|---|
| demand per day | 12 | declared | An arbitrary team's intake; sets the scale, not the shape |
| generation multiplier | 2.0 | declared | "The machine makes it cheap to make more work"; the first thing to sweep |
| reviewers | 4 | declared | Small enough to be a coherent team |
| review rate | 8/day | declared | About an hour per item, which is long for a review and short for a proper one |
| defect share | 0.20 | declared | One generated item in five carries a defect before review |
| caught rework cost | 0.50 | declared | Half a first-pass review to fix something caught in time |
| escape rework cost | 3.0 | declared | Six times the caught cost; this is the parameter the thesis turns on |
| arrival coefficient of variation | 1.0 | declared | Poisson-like intake; raised by bursty generation |
| service coefficient of variation | 1.5 | declared | A review is fast or it is a week (§S4 explains why it is not derived) |
| utilisation target | 0.90 | declared | The lean norm, chosen because it is the norm rather than because it is right |
| WIP cap | 40 items | declared | About two days of generated work at the declared rates |
| cycle-time deadline | 5 days | declared | A plausible commitment; §6.3 shows what other commitments cost |

The two that carry the argument are `escape rework cost` and `generation multiplier`. The first
determines whether deskilling destabilises the queue at all — at a multiplier of one, escapes cost
the same as caught defects and the coupling has no direction. The second determines whether there is
any margin for the coupling to eat: at a generation multiplier of 1.5 rather than 2.0 the defaults
would run at about 73% of capacity, and nothing in §6.4 would happen within the horizon.

Both are swept rather than fixed in the figures, and neither is a measurement. Anyone who wants to
attack the framework's conclusions should move those two numbers first, because they are where the
conclusions come from.

Both are swept rather than fixed in the figures, and neither is a measurement. Anyone who wants to
attack the framework's conclusions should move those two numbers first, because they are where the
conclusions come from.

## 5. The coupling

`e` is not a constant. It is the residual defect rate of the person doing the reviewing, and
[`human_capacity`](THE_PSYCHOLOGICAL_LAYER.md) models how that falls when assistance is *solved
for* rather than chosen: a period has to clear a bar, the bar is set by the machine's tempo, and a
person who would rather keep the practice cannot do so without falling behind a standard they did
not set.

The loop then closes:

> less capacity catches fewer defects → more defects escape → escapes cost more to fix → the review
> stage carries more load at the same nominal utilisation → the workflow leaves its envelope.

This is the one mechanism with no factory analogue, and it is why `psyche` had to become a library
rather than a module. The workflow model calls `human_capacity::residual_defect_rate` directly. A
second, copied defect function would have let the two models disagree about the same person, and
the disagreement would have been invisible: two plausible curves, one of them wrong.

The model is also explicit about what it does **not** contain: the workflow does not feed back into
capacity. The person's trajectory is exogenous here. A model in which an unstable queue made
capacity fall faster would be more alarming and is not this one (scope condition **S3**).

## 6. Results

### 6.1 The defaults sit on the edge, not in comfort

At full capacity the declared workflow runs at **96.7%** of its review capacity. That is inside the
envelope with about three points of margin — which is precisely what a lean target of high
utilisation leaves, and why a small loss of capacity to judge is enough to matter.

This is worth stating as a result rather than a setting: a workflow designed to lean discipline has
no room for the coupling to occur. The margin is not a parameter; it is a consequence of the target.

### 6.2 The envelope

| capacity to judge | escape | load | ρ | absorbable | steady | on time |
|---:|---:|---:|---:|---:|:--:|:--:|
| 0.50 | 0.654 | 1.43 | 1.070 | 0.925 | **no** | no |
| 0.70 | 0.543 | 1.37 | 1.029 | 0.962 | **no** | no |
| 0.80 | 0.488 | 1.34 | 1.008 | 0.982 | **no** | no |
| 0.85 | 0.460 | 1.33 | 0.998 | 0.992 | yes | no |
| 0.90 | 0.433 | 1.32 | 0.987 | 1.000 | yes | yes |
| 1.00 | 0.378 | 1.29 | 0.967 | 1.000 | yes | yes |

Read the last four rows together: between capacity 0.90 and 0.85 the workflow keeps a steady state
but stops being able to take everything it generates. The failure is not a queue blowing up. It is
an organisation quietly admitting less than it produces while every utilisation chart reads
normally.

### 6.3 What the commitment costs

| commitment (days) | capacity required |
|---:|---:|
| 10 | 0.863 |
| 7 | 0.874 |
| 5 | 0.888 |
| 3 | 0.921 |
| 2 | 0.962 |
| 1 | **unmeetable** |
| 0.5 | **unmeetable** |

At one day the workflow cannot meet its commitment **at any capacity**, and the function returns
`None` rather than a capacity outside its own range. That is a design failure rather than a tight
target, and reporting it as a threshold would dress it up as a trade-off. The plotting script
refuses to draw a figure in which the commitment outlives stability, which is the arithmetic
version of the same check.

### 6.4 The year it leaves the envelope

Same workflow in every year. The only thing that changes is the person's capacity to judge, which
falls from 0.908 to 0.742 because assistance is solved for rather than chosen.

| year | capacity | escape | load | ρ | verdict |
|---:|---:|---:|---:|---:|---|
| 0 | 0.908 | 0.428 | 1.31 | 0.986 | inside |
| 3 | 0.853 | 0.459 | 1.33 | 0.997 | over commitment |
| 6 | 0.826 | 0.474 | 1.34 | 1.003 | **no steady state** |
| 15 | 0.758 | 0.511 | 1.36 | 1.017 | **no steady state** |
| 29 | 0.742 | 0.520 | 1.36 | 1.020 | **no steady state** |

**Year 1 breaches the commitment. Year 5 has no steady state.** Nothing about the workflow changed:
no new tool, no reorganisation, no increase in demand. The workflow's stability is a property of
the person reviewing it, and the person was the thing the workflow was slowly changing.

This is the result the framework exists to produce, and it is a claim no single-discipline model
would make. Queueing theory has nothing to say about why the service rate deteriorates. The
psychological model has nothing to say about what the deterioration does to a system. Together they
produce a date.

### 6.5 What the slack is spent on

Three configurations of the same workflow, each evaluated at its own steady state:

| configuration | capacity at horizon | throughput/day | cycle time | escapes/day | autonomy years | steady |
|---|---:|---:|---:|---:|---:|:--:|
| lean (push everything) | 0.742 | 24.00 | — | 2.50 | 8 | **no** |
| buffered (WIP cap) | 0.742 | 21.70 | 1.84 d | 2.26 | 8 | yes |
| learning buffer | 0.831 | 17.44 | 2.29 d | 1.64 | **30** | yes |

The third row is the framework's practical content. The same buffer can be spent on throughput or
on the capacity to judge, and **only the second buys any autonomy**: 8 years against 30. The
buffered configuration is stable and faster and buys *nothing* for the person, because the person's
training time is unchanged.

Note also that the learning buffer deskills *less* (0.831 against 0.742) as well as carrying fewer
escapes — because a quarter of the period spent learning is a quarter not spent producing under
assistance. The autonomy gain is not a transfer from output to virtue; it partly pays for itself.

## 7. The direction question

### 7.1 Why not a rank order

"Productivity toward what?" cannot be answered by arithmetic, because the answer is a weighting and
the weighting is a moral position. A framework that reported a winner would be smuggling a
normative commitment in as a calculation.

So the framework reports the **surface instead of a point**. It scores each configuration under
every weighting in a grid over the five-criterion simplex, and reports for each configuration the
share of weightings under which it comes first. That is the honest form of a multi-criteria claim:
*if you weight autonomy above throughput, this wins; if the reverse, that one does; here is where
the disagreement lives.*

### 7.2 The surface

Over 495 weightings of throughput, cycle time, escaped defects, autonomy years and backlog:

| configuration | share of weightings won |
|---|---:|
| learning buffer | **51%** |
| lean | 31% |
| buffered | 18% |

**No rank order exists.** All three configurations win under some defensible weighting, and the
report says so instead of promoting the declared weighting's winner to the answer. The declared
weighting — throughput 0.30, escaped defects 0.25, autonomy 0.20, cycle time 0.15, backlog 0.10 —
is printed as *one point on that surface*, named as the ordinary managerial view, with its
provenance marked `declared`.

The five criteria and why each is in the framework at all:

| Criterion | Declared weight | Why it is included |
|---|---:|---|
| throughput | 0.30 | The quantity every productivity metric already reports — which is precisely why the framework has to show what ignoring the others would choose |
| cycle time | 0.15 | The commitment made to whoever receives the work |
| escaped defects | 0.25 | The cost that lands on someone outside the team, which is why no team-level metric counts it |
| autonomy years | 0.20 | Years spent above the autonomy floor; weighting it at all is the normative claim the framework exists to examine |
| backlog | 0.10 | Work generated and never admitted, which is what a utilisation target buys itself |

### 7.3 What the framework cannot decide, and why that is not a failure

It cannot tell you whether an organisation should hold 0.90 or 0.80 utilisation, whether a
two-day commitment is worth its capacity, or whether the learning buffer's 18% throughput cost is
worth 22 extra years of autonomy.

Those are commitments, not measurements. What the framework can do — and does — is make the
commitment **explicit and consequential**: it shows what each choice costs in the units of the
others, and it shows how much of the choice is robust (the acceptance shares) and how much is
knife-edge (the declared weighting's ranking, which reverses under modest reweighting).

## 8. Governance: the envelope as a policy object

Three mechanisms follow from the model, and they are stated as obligations rather than
optimisations because the model has no objective function it is entitled to maximise.

1. **A buffer target rather than a utilisation target.** The envelope says which utilisations meet
   the commitment at a given capacity. A utilisation target is a single number chosen without
   reference to the person's capacity; a buffer target is a *rule* that survives capacity falling.
   The cost of the tighter commitment is §6.3's curve, and it is smaller than intuition suggests
   until it becomes infinite.
2. **Slack that is spent rather than banked.** §6.5 is the operational point: a buffer that is not
   designated for learning is a buffer that will be spent on admissions. The distinction is not
   cosmetic — it is the difference between 8 autonomy years and 30.
3. **Duty of care toward the capability, not the output.** The model's failure mode at 0.85
   capacity is not a visible one. Every utilisation chart reads normally; the intake quietly falls.
   A governance mechanism that watches throughput cannot see this, which means the obligation has
   to be attached to the capability that is eroding — the declared autonomy floor, the training
   share, the escape rate — and not to the output.

The third is the one no dashboard will enforce, and the model's contribution to it is to show that
**the failure is silent by construction**: the quantities that deteriorate are precisely the ones
the organisation chose not to measure.

## 9. Scope conditions

The framework's conclusions hold only if these seven conditions hold. Each is a place a reader can
push, and the list is the honest form of the framework's claim.

| | Condition | Status |
|---|---|---|
| **S1** | Utilisation is held at its lean target when capacity rises | **Declared behavioural assumption, and the central causal claim.** If firms bank slack as shorter weeks, the fragility result does not follow |
| **S2** | Review throughput is independent of review capacity within the horizon; capacity affects detection, not speed | Declared. A model in which reviewers also slowed would be worse, not better |
| **S3** | The workflow does not feed back into capacity | Declared. The feedback would be destabilising, so its absence is conservative |
| **S4** | Effective service-time variability is declared, not derived from the rework mix | Declared. The compound-CV arithmetic gives a non-monotone term that is an artefact of the algebra |
| **S5** | Demand is exogenous; the bar is not set by a market in which everyone delegates | Declared. A model with a competitive bar would be more alarming and is not this one |
| **S6** | One station, identical servers, FCFS | Standard queueing idealisation; heterogeneity would sharpen the variance term |
| **S7** | The direction framework's five criteria and min-max normalisation within the option set | Declared, and it means the framework cannot say an option is unacceptable in absolute terms — only that it is worse than another |

S1 deserves emphasis. It is not an assumption of convenience: it **is** the transplantation
hypothesis, and the model is a way of working out what follows from it. Anyone who rejects S1
rejects the thesis, and should say so, rather than arguing about the parameters.

## 10. What is not established

1. **That any real workflow runs at these numbers.** Every parameter is declared, and none was
   measured in a workplace.
2. **The service-time distributions.** The approximation uses two coefficients of variation and
   overstates waiting by up to 28.5% at moderate utilisation (§4.2).
3. **That the workflow feeds back into capacity.** It does not (S3).
4. **Which configuration is better.** That is a weighting (§7).
5. **That the utilisation target is what organisations actually hold.** S1 is a claim about
   behaviour with no measurement behind it, and it is the framework's weakest joint.
6. **Anything about wages, employment, or who bears the cost.** The model has no labour market.
7. **Anything about the climate or agriculture framing** that motivates the critique; that is
   motivation and nothing more.
8. **That the queue is the whole of knowledge work.** §3.2 lists six places where it is not.

## 11. The thesis this would become

Stated as a proposal, because the work above is a chapter and not a thesis.

**Discipline spine:** operations research / management science. The formal core is the envelope.
Organisational behaviour supplies the mechanisms that S1 and S3 currently declare; the normative
material is the framing and the closing chapter.

**Chapter structure:**

1. **The transplantation claim** — JIT/lean logic in knowledge work, with §3's structural-versus-
   suggestive test as the chapter's result. *Claim: five features transfer, six do not.*
2. **The queueing model** — the load identity, the VUT approximation and its measured error, and
   the two thresholds. *Claim: utilisation is not the only thing that can break, and the commitment
   fails before stability does.*
3. **The rework loop** — escapes, effective process time, and the collapse into fragility.
   *Claim: the failure is silent in throughput and visible only in the load.*
4. **The coupling** — capacity as the missing variable, using `human_capacity`. *Claim: a workflow
   can be stable at adoption and outside its envelope within five years.*
5. **The organisational mechanisms** — empirical work on S1 and S3: do firms bank slack, and does
   review load affect capacity? *This is the chapter that needs field data.*
6. **The direction question** — the acceptance surface and its limits. *Claim: no rank order
   exists, and that is a finding rather than a failure.*
7. **Governance** — buffers, duty of care, and what an enforceable obligation would look like.

**Expected contributions:** (i) the effective-utilisation account of AI-assisted review, with the
load identity as a usable estimator; (ii) the two-threshold decomposition of workflow failure;
(iii) the coupling of deskilling to queueing stability, which produces a date rather than an
admonition; (iv) a multi-criteria framework that reports its own indeterminacy.

**What would falsify it** — stated as tests, because a thesis without them is an essay:

- a workflow running at ρ ≥ 0.95 for years with stable cycle times would refute the utilisation
  claim outright;
- a measured `c_escape ≤ c_caught` would remove the channel through which deskilling destabilises;
- evidence that review capacity does not fall under tracked assistance would break the coupling;
- evidence that firms **bank** slack as capacity rises would refute S1, and with it the thesis;
- a set of declared criteria on which no defensible weighting ever disagrees would make §7 moot.

**No timeline is offered.** A timeline is a commitment about the author's circumstances, and this
document has no more business inventing one than it has inventing a coefficient.

## 12. Relation to the rest of the repository

- **`THE_PSYCHOLOGICAL_LAYER.md`** — the capacity side of this model, in full. Its `(1 − a)`
  autonomy term and its deskilling threshold are what make §6.4's date a computable quantity.
- **`AI_AS_CATALYST.md`** — the same question one level down: what fraction of an AI speed-up
  survives verification and rework. Its `ρ` is this framework's `ρ` seen from inside a single
  engineer's week.
- **`EARLY_RETIREMENT.md`** — the optimisation machinery. Its linear program is what a myopic
  objective looks like when it is formalised; §4.3 here is why the objective that rises is not the
  one that matters.
- **`STRATEGY_COMPARISON.md` and `THIRD_PARTY_SPOILER.md`** — the same structure one level up: an
  actor optimising a measured quantity while an unmeasured one degrades.
- **`multipolar_sim`** — the world layer. The bar in this model is exogenous (S5) and in a fuller
  account it would be set by a market in which everyone is delegating, which is where that
  simulator's blocs would enter.

## 13. Reproduction

```bash
# the envelope, the thresholds, the trajectory and the direction analysis
cargo run -p knowledge_work --release

# with the CSV the figures are drawn from
cargo run -p knowledge_work --release -- --export out/workflow

# the figures, into the directory committed with this document
python tools/plot_workflow.py out/workflow figures

# the tests behind every number quoted above
cargo test -p knowledge_work
```

The full workspace suite is 420 tests. `knowledge_work` contributes 30 — eight on the queueing
building blocks, ten on the workflow, five on the envelope, seven on the direction framework — and
each is named for the failure it would catch.

## 14. Corrections recorded

Four defects found while building this, all worth recording because three were invisible to the
test that should have caught them.

1. **Utilisation eight times too small.** `effective_utilisation` multiplied by the base service
   time *and* divided by capacity, which already contains the review rate. Every closed-form test
   still agreed with itself; what found it was the discrete-event simulation realising a different
   utilisation from the one the model claimed. The utilisation-target policy had the same double
   count.
2. **A monotonicity test that failed its first correct value.** The comparison was initialised at
   zero, so the very first — correct — observation tripped it.
3. **A ranking test that could not detect an inverted direction.** The first single-criterion test
   used a dominated pair, where every weighting returns the same winner. It now uses a fixture that
   disagrees on every criterion.
4. **An inversion that conflated two capacities.** `capacity_needed_for` mixed the person's
   capacity to judge with the firm's reviewer count and could never return a sane answer. It is
   deleted rather than patched, because `absorbable_share` already answers the operational question.

Two methodological corrections belong with them. First, the **commitment threshold was assumed to
be the weaker constraint** and turned out, correctly, to be the earlier one — a promise fails before
the arithmetic does, which is the whole reason to keep them apart. Second, the **one-day commitment
was expected to be reported as expensive** and is reported as impossible; the model refuses to
return a threshold that does not exist, and the refusal is a result rather than an error path.
