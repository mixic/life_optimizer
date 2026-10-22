# The By-Product Theory of AI

### One mechanism behind four models, and what it predicts about where this ends

---

## 0. What this document is

Four separate models in this repository have each found a different way for AI to hurt the people
using it:

| Model | Its finding |
|---|---|
| `documents/AI_AS_CATALYST.md`, `src/ai_scenario.rs` | the quality of delivered work can consume the entire productivity gain, and the retention coefficient ρ is unmeasured |
| `knowledge_work` | a workflow normalised to high utilisation converts AI's extra capacity into extra arrivals, not into slack, so cycle time rises and rework accumulates |
| `documents/THE_PSYCHOLOGICAL_LAYER.md`, `human_capacity` | assistance is *solved for* rather than chosen, and it erodes the capacity that is supposed to judge it |
| `comprehension_cost`, `AI_AUTHORING_AND_COMPREHENSION.md` | the authoring time AI saves is smaller than the comprehension and maintenance cost it creates, and the conclusion rests on unmeasured parameters |

Four findings, four literatures, four modelling styles. This document argues they are **one
mechanism observed from four angles**, states that mechanism, and draws the consequence.

The claim in one sentence:

> **AI substitutes for production, and production was producing two things: the artefact, and the
> producer's capacity to judge the artefact. The artefact is sold, measured and priced; the
> capacity is not. Every one of the four models is a measurement of a different by-product running
> down.**

The epistemic status is stated up front because it governs how to read the rest. This is a
synthesis with a falsifiable core, not a projection. §8 gives the observations that would refute
it. Nothing here is a forecast of the future, and §7 is written as conditions rather than dates.

---

## Part 1 — The substitution

### 1.1 What is actually being replaced

The productivity framing says AI replaces *work*. It does not. It replaces **production**: the
transformation of intent into an artefact. What it leaves the human is a different job — inspecting
an artefact that arrived already formed.

That distinction sounds semantic and is not, because production is an unusual kind of work: it is
the only activity that reliably produces, as a side effect, a mental model of its own output. You
finish writing the module and you also, involuntarily, know how it works. The knowledge was never
the goal; it was the exhaust.

Economists and managers have always priced the artefact and treated the exhaust as free. For most
of industrial history that was close enough, because the exhaust accumulated automatically
wherever the artefact was produced. Remove the production and the exhaust stops, and nothing in
the accounting notices, because the exhaust was never on the accounting.

### 1.2 The four by-products

Each of the repository's four models is the same story with the by-product named differently.

| Model | AI supplies | By-product lost | Currency the loss is paid in | Where it is modelled |
|---|---|---|---|---|
| Catalyst | code at volume | fitness-for-purpose judgement — the ρ that decides whether generated work is usable | defect escape; rework | `src/ai_scenario.rs` |
| Knowledge work | throughput | slack — the buffer that absorbs variability | queueing delay; rising cycle time | `knowledge_work` |
| Psychological layer | decisions, practice | the practised capacity that evaluates assistance | deskilling; the judge degrades | `human_capacity` |
| Comprehension | authored code | the encoded mental model | review hours; time-to-diagnose | `comprehension_cost` |

These are not four problems. They are one problem measured in four units: **judgement, slack,
competence and comprehension are all the residue of doing the work yourself, and AI is the first
technology whose entire value proposition is to remove the doing while keeping the output.**

### 1.3 Why this is not the ordinary deskilling story

The ordinary story is that machines deskill workers, workers resist, and the labour market adjusts
over a generation. That story has a mechanism — the worker knows what is being lost, and bargains
over it.

This one does not. The loss is **invisible to the person losing it**, because the artefact arrives
finished and its arrival is fluent. The four models each document a different way the invisibility
works:

- The comprehension model measures a comprehension level that the engineer believes is adequate,
  because reading AI code *feels* like understanding it (§4.3).
- The psychological model shows assistance being *solved for* rather than chosen: once the machine
  can do the task, the schedule that does not use it becomes uncompetitive, and the choice
  disappears before anyone decides anything.
- The knowledge-work model shows firms restoring their old utilisation target after a capability
  jump, which is a rational local decision that destroys the slack the jump created.
- The catalyst model simply cannot measure ρ, so it reports a range and says the range decides the
  answer.

---

## Part 2 — Why the by-product goes unpriced

### 2.1 Flow and stock

Every economic instrument we have for measuring AI is a **flow**: tokens per second, pull requests
per week, story points, hours saved, lines merged. Every by-product at risk is a **stock**: a level
that accumulates slowly, decays slowly, and has a threshold below which it fails abruptly. The
difference is not rhetorical — flows and stocks have different mathematics, and a flow metric
cannot in principle detect a stock problem.

This is the structure of every natural-resource failure. A fishery's instruments measure the catch
per season, which can rise while the fish stock collapses, because the catch is effort times a
density that is falling. Nobody in the industry is lying; the metric simply does not contain the
state variable. Later, the collapse arrives as a surprise that was fully determined years earlier.

AI output is a harvest, and comprehension, skill and slack are the stock being drawn down. That
single substitution explains why the four models keep independently rediscovering the same shape,
and why each of them has to *build* a new stock variable to see it: comprehension level, capacity,
buffer, ρ.

### 2.2 The asymmetry in the accounting

| | Product | By-product |
|---|---|---|
| Unit | per artefact | per person, per team, per skill |
| Timing | immediate | deferred, often past any review cycle |
| Measurability | direct, countable | indirect, inferred from failures |
| Attribution | the tool, the team, the quarter | "maintenance", "tech debt", "burnout", "the market" |
| Visibility when it fails | a defect, dated | a capability, undated |

The gain is measurable, attributable and near; the cost is unmeasurable, diffuse and far. Any
organisation will therefore over-adopt relative to a correct accounting — not through folly, but
because its instruments cannot see the other side. **This is a measurement failure, not an
incentive failure, and the difference matters for what would fix it.** You cannot solve a stock
problem with better intentions; you need a stock variable in the ledger.

---

## Part 3 — The cognitive core: encoding, not authorship

The intuition behind this section is the one that motivated the whole line of work: material you
produce yourself goes into memory differently from material you read, because producing it connects
things that reading does not. That intuition is correct, and the literature is more specific — and
more useful — than the intuition.

### 3.1 What is established

The **generation effect**: information generated by the learner is remembered better than the same
information read. It is a robust, decades-old finding, demonstrated across free recall, cued recall
and recognition, and across materials from word fragments to arithmetic to keywords in text.

The proposed mechanisms are all about *processing*, not authorship. Generation forces a search
through semantic memory; the search activates related features; those features become retrieval
cues. The effect is not a reward for effort as such — it is that generating performs the encoding
work that reading can skip.

### 3.2 The finding that changes the theory

This is the part that makes the theory a theory rather than an intuition, and it cuts against the
simplest version of the claim.

**The generation advantage disappears when readers are given instructions that make them process
the material the way generators do.** Two manipulations are documented: instructing readers to
process in a generation-like way eliminates the gap; and readers using a more effective strategy
than plain reading — imagery — perform as well as generators. The advantage also weakens or
vanishes for *unfamiliar* material, and there is a documented trade-off in which generation
improves item-specific encoding at a cost to associative and contextual encoding, because encoding
draws on limited-capacity resources.

So the operative variable is **not** "did a human write it". It is **what processing the reader
performs**. Authorship is a very reliable *proxy* for deep processing, which is why the effect
looks like an authorship effect in ordinary life — but the proxy is breakable in both directions:

- A human can write code carelessly and understand nothing.
- A reviewer can read AI-written code while performing the same reconstructive processing that
  authorship would have forced, and get most of the benefit.

That second bullet is the non-obvious and actionable part. AI-authored code is not condemned to be
incomprehensible; it is incomprehensible **when review is passive**. Which is exactly what review
becomes by default, because passive reading is the cheap thing and there is a queue.

### 3.3 Why the loss is invisible: fluency mistaken for knowing

Two further findings close the loop, and they are the bridge to the empirical puzzles in the other
documents.

**The illusion of explanatory depth.** People rate their understanding of how things work far
higher than their ability to explain them, and the overrating collapses when they attempt an
explanation. Being asked to explain is what reveals the gap.

**Fluency misattribution in judgements of learning.** Confidence in one's own learning is driven
substantially by how *fluent* the material feels, which is highest when processing is easy — that
is, precisely when encoding is shallow. Easier text produces higher confidence and no better
retention.

Put together: reading AI output is fluent, so it produces high confidence; confidence is read as
comprehension; comprehension is therefore never checked. The check that would reveal the gap is
generative — explain, reconstruct, predict — and that is the expensive action nobody schedules.

**This is the same error METR measured.** Experienced developers using AI expected a 24% speed-up,
were measured 19% slower, and afterwards still believed they had been sped up by about 20%. The
standard reading is "people are bad at estimating productivity". The better reading, on this
theory, is that they were reporting fluency: the work felt easier, and felt-easier was collapsed
into is-faster. The BairesDev self-reports — 13 hours saved a week, 59% reporting more mental
effort to catch subtle bugs — are the same instrument pointed the other way, which is why the two
sources appear to contradict each other and do not. They are measuring different things: one the
clock, the other the feeling of the clock.

---

## Part 4 — The formal core

The theory needs one identity and one condition, and nothing else in this document depends on
quantities I cannot source.

### 4.1 The identity

Over a period, measured output rises by the flow gain `G` — the productivity effect, which is the
thing every study tries to estimate. Three stocks are drawn down by that gain:

- `C` — comprehension capital: the encoded mental models available to diagnose and extend the
  system;
- `S` — skill capital: the practised capacity to evaluate work of this kind at all;
- `B` — slack: the buffer that absorbs variability in a workflow.

Write the drawdowns as functions of the adoption level `a`, and the regeneration as functions of
the deliberate effort `e` spent restoring them — review that reconstructs, practice that is not
delegated, capacity held in reserve.

```
dC/dt = r_C(e) - d_C(a)      dS/dt = r_S(e) - d_S(a)      dB/dt = r_B(e) - d_B(a)
```

The net welfare effect is `G - λ_C·d_C - λ_S·d_S - λ_B·d_B`, where the λ convert a stock drawdown
into current cost. **The λ are the whole argument.** They are near zero while the stock is far
above its threshold, and they are effectively infinite at the threshold — a system whose
comprehension capital hits zero does not degrade gracefully, it stops being maintainable, and the
cost of recovery is not linear in the deficit.

### 4.2 The condition

> The arrangement is sustainable if and only if, for each stock, the regeneration rate induced by
> deliberate effort is at least the drawdown rate induced by adoption.

Three things follow, and they are the theory's testable content.

1. **Sustainability is per-stock, not aggregate.** Slack can be regenerated cheaply by lowering
   the utilisation target; skill cannot be regenerated cheaply at all. A firm can be sustainable on
   `B` and failing on `S` at the same time, and its dashboards will show the first.
2. **`e` is a cost the flow accounting does not contain.** Every hour spent making review
   generative is an hour not counted as output. So rational local management sets `e` to zero, and
   the condition is violated by default in every organisation that does not measure the stocks.
3. **The failure is discontinuous in `λ`, not in `d`.** Nothing dramatic happens as the stocks
   fall, until they cross the threshold, and then a great deal happens at once. That is why the
   problem will not be caught by trend-watching, and why it will be described afterwards as sudden.

---

## Part 5 — Where this lands

Not a forecast. Three regimes, each with the condition that selects it, the observable that
identifies it, and what would falsify the claim that it is the one we are in.

### 5.1 Regime A — extractive

**Condition:** flow gains are banked as output rather than held as slack, and review stays passive
because `e = 0` is locally optimal.

This is the default. It requires no decision, only the absence of one. Its signature is rising
throughput with rising rework: exactly the pattern `knowledge_work` derives from the lean
normalisation and `AI_AS_CATALYST` flags as the case where ρ consumes the gain.

Observables that would identify it: review hours per merged change rising faster than changes;
incident rate per unit of change rising; time-to-diagnose on unfamiliar modules rising; a growing
share of shipped changes for which no one can produce an explanation on demand.

Note that every one of those indicators is available *now* and none of them is in a productivity
dashboard. That is the measurement failure of §2.2 restated as an operational consequence.

### 5.2 Regime B — re-encoded

**Condition:** organisations deliberately convert review from reading into reconstruction, so the
encoding still happens — just later and by someone else. "Review the AI's code" becomes "rewrite it
with the AI's version in front of you", which is what the generation-effect literature says is the
difference between a 7% and a 40% memorial advantage.

This regime is reachable, and the theory says precisely what it costs: the comprehension cost is
paid as review time, up front, and it is the same order of magnitude as the authoring time saved —
which is what `comprehension_cost` found when it reported the authoring saving as tens of hours
against comprehension and maintenance costs in the hundreds. Regime B does not make the by-product
free. It buys it back at close to its original price, and it is worth it because the alternative is
§5.3.

### 5.3 Regime C — deskilled, and why it is an attractor

**Condition:** the encoding is not performed at all, and the organisation loses the ability to
verify what it is producing.

The reason this is the dangerous equilibrium, rather than a stage on the way to A, is that it is
*self-concealing*. Once the skill stock `S` is below the level needed to evaluate work of this
kind, the organisation cannot detect that its evaluation has stopped working. Its instruments all
read normal: changes merge, tests pass, the pipeline is green. The failure mode is not a bad
decision, it is the loss of the capacity to make one.

`THE_PSYCHOLOGICAL_LAYER.md` models this as the capacity that judges the assistance being eroded by
the assistance, which is a closed loop with no external reference. Regime C is that loop at
organisational scale.

### 5.4 The one prediction I will commit to

Across the three, the parameter that decides the destination is **whether verification is
generative**. Not model capability, not adoption rate, not regulation. Two organisations with the
same tools and the same codebase land in different regimes according to whether the person
approving the change could reconstruct it.

And the corollary, which is the uncomfortable one: because generative verification is expensive and
its absence is invisible, **the modal outcome is A sliding into C**, and it will be reported as a
quality crisis rather than as an accounting error.

---

## Part 6 — What this changes about the earlier documents

Intellectual honesty requires listing the corrections this synthesis forces on work already
committed.

1. **`comprehension_cost`'s `c_ai` is not a constant.** It is modelled as the comprehension an
   AI-authored module starts with, and the model treats 0.25 as a property of machine authorship.
   The generation-effect literature says it is a property of *review mode*: a reviewer who
   reconstructs can start far higher. The parameter is not wrong as a description of passive
   review; it is wrong as a description of AI code. **Correction recorded, not yet re-run** — the
   model's verdict is conditional on passive review, and §2.8 of
   `AI_AUTHORING_AND_COMPREHENSION.md` should be read with that in mind.
2. **The measured 19% slowdown is predicted, not surprising.** On this theory the divergence
   between measured time and perceived time is the expected consequence of fluency-driven
   confidence. That makes the METR result evidence *for* the theory rather than an anomaly within
   the productivity frame.
3. **`knowledge_work`'s behavioural assumption is the `B`-stock condition.** The crate is explicit
   that firms restore their old utilisation target after a capability jump and that this is a
   declared assumption rather than a measurement. The theory says why: slack is the by-product,
   utilisation is the flow, and the flow is what the dashboard sees.
4. **`AI_AS_CATALYST`'s unmeasured ρ is a by-product, and that is why it resists measurement.**
   Delivered quality is the judgement of the person who would have caught the defect; if that
   person's capacity is itself being degraded, ρ is not merely unknown, it is *unmeasurable from
   inside the system that is losing it*.

---

## Part 7 — Provenance

Following the convention of this repository: every claim is either sourced, cited without having
been read, or declared as the theory's own construction.

**Sourced** — read, or read via a secondary summary that names the primary:

| Claim | Primary source | How verified |
|---|---|---|
| Generated information is remembered better than read information | Slamecka & Graf (1978), *The generation effect: delineation of a phenomenon*; the phenomenon is reviewed with primary citations in the [Wikipedia summary](https://en.wikipedia.org/wiki/Generation_effect) | secondary summary read in full; primary cited, not read |
| The advantage is eliminated when readers are instructed to process as generators do, and when readers use a more effective strategy | deWinstanley & Bjork (2004), *Processing strategies and the generation effect*, Memory & Cognition 32(6):945–955 | named with DOI in the summary above |
| The advantage is reduced or absent for unfamiliar material | reported in the same summary, attributing a study where it failed for novel material | secondary only; I did not read that study |
| Generation can cost associative encoding because encoding uses limited capacity | reported in the same summary | secondary only |
| **Generation effect does not transfer to unfamiliar material** | — | **this is the limitation most likely to matter for code, and I have not verified it for code** |

**Cited, not read** — identified over the network but the source itself was not retrieved:

- Rozenblit & Keil, *The misunderstood limits of folk science: an illusion of explanatory depth*
  (Cognitive Science, 2002). The phenomenon is well known; I did not read the paper.
- The fluency–miscalibration literature on judgements of learning, including the review whose title
  is *When confidence is not a signal of knowing*. Same status.

**Sourced from this repository's own measurements:** the −19% / +20% / +24% figures (METR, verified
in `AI_AUTHORING_AND_COMPREHENSION.md` §1.2), the 13 h and 59% figures (BairesDev, same place), and
the `comprehension_cost` decomposition and variance shares (Part 2 and Part 3 of that document).

**Declared** — this document's own construction, with nothing behind it but argument:

- the four-by-product mapping;
- the flow/stock framing and the fishery analogy;
- the formal identity and the sustainability condition;
- the three regimes, and the claim that Regime C is an attractor;
- the assertion in §5.4 that generative verification is the deciding parameter.

### What is not established

That the generation-effect findings transfer from word lists to software comprehension. Every
memory study cited here uses verbal materials under laboratory conditions; the leap to
understanding a module is the theory's largest unverified step, and it is the one that carries the
most weight. The honest position is that the mechanism is plausible, well-supported in its own
domain, and untested in this one — which is exactly the position `AI_AUTHORING_AND_COMPREHENSION.md`
reached from the opposite direction, when it found that the parameters its conclusion depends on
are the ones nobody has measured.

Two documents, arrived at independently, pointing at the same missing measurement: **how much
longer does it take to understand and repair work you did not generate?** That is the experiment
this theory says would settle it, and it does not exist yet.

---

## Part 8 — What would refute this

Stated as observations rather than arguments, so that the theory is falsifiable rather than merely
persuasive.

1. **If generative review protocols produce no measurable advantage** in defect escape or
   time-to-diagnose over passive review, at equal hours, the cognitive core (§3) fails.
2. **If organisations with high adoption and flat or falling review hours per change show no rise in
   incident rate per change over several years**, the stock-drawdown claim (§4) fails: the
   by-products were cheaper to regenerate than the theory says.
3. **If skill stocks recover as fast as they erode** — if a deskilled team returns to prior
   capability in weeks rather than years — then `λ_S` is small, the threshold argument collapses,
   and Regime C is not an attractor.
4. **If the flow and stock are actually the same variable** — if comprehension were fully captured
   by the same metrics that capture output — the accounting asymmetry of §2.2 would be an artefact
   of measurement practice rather than a structural feature, and the theory would reduce to
   "measure better".

Number 4 is the one a sympathetic critic should press hardest. The theory's defence is that
comprehension, skill and slack are *latent* variables: they never appear in a transaction, so no
improvement in transactional measurement will find them. But that is an argument, not evidence, and
it is where the theory is weakest.
