# Verifying Two Passages

*A claim-by-claim check of an industry passage attributed to Milan Milanovic and of the author's own
paragraph, against what can be sourced and what the repository's models can settle. Every verdict
below names the kind of claim it is, because the failure mode in this genre is treating a
sociological intuition and a queueing identity as if they were the same kind of statement.*

---

## 1. The passages, as given

**Passage A**, attributed to Milan Milanovic:

> It's obvious that a lot of people is in the state of fear. Developers are working very hard to
> prove they are still worth with AI, fearing they will be replaced. On the other side owners are in
> fear that competition will outrun them fast.
>
> What is definite is that the landscape is changed. We don't need so much developers any more, as
> current ones can do much more than before. Yet, we always wanted more and more, so Jevons paradox
> hits now.
>
> What will happens no one knows for sure. We mostly improved only one part of SDLC, while others are
> still as they were. What we know from aviation industry is that FAA pushed towards automation in
> cockpits a lot, but after a few disasters, they found out that pilots are too much relied to it.
> So, they switched back to mid level of automation and pilot work together.

**Passage B**, the author's own:

> There's a psychological layer we're skipping and it points at a real meltdown, not just a gap. The
> younger generation is increasingly not developing the artistic, emotional, social, and
> psychological muscles. The relationship inverts instead of human-to-human, or even
> human-to-machine, it becomes machine-to-human: the machine shapes the person, setting the rhythm,
> the framing, the emotional register, and the individual adapts to it. Emotion gets disrupted by a
> different way of thinking. That's the risk worth naming: not that AI makes us lazy, but that it
> quietly reshapes how we feel, relate, and judge, and we optimize for output while hollowing out the
> human capacities that make the output worth anything.
>
> This is where the industry is going, and treating AI as a chatbot leaves leverage on the table. The
> real question isn't whether to lean in, it's how to do it without burning out the people carrying
> the load or degrading the human skills behind the work. Having the runway to learn is an edge,
> whether teams get that runway before expectations land is the open question.

## 2. What "verify" can mean, and which of the four applies

A passage like this mixes four kinds of statement, and the same word — *true* — does not apply to all
of them. Every verdict below is tagged with which one it is:

| Kind | What would settle it | Who can settle it |
|---|---|---|
| **Empirical** | A measurement, a survey, a dataset | Field work, not this repository |
| **Historical** | A primary document | A citation, which is checkable today |
| **Structural** | Arithmetic or a theorem | This repository, and it is checkable |
| **Normative** | An argument, not a fact | Nobody's data; only reasons |

The passages are mostly empirical and normative, which is why most of them cannot be settled here.
Two of them can, and one of those turns out to be the strongest claim in either passage.

## 3. Passage A, claim by claim

| # | Claim | Kind | Verdict |
|---|---|---|---|
| A1 | Developers are afraid of replacement and working to prove their worth | empirical | **Plausible, weakly sourced** |
| A2 | Owners fear being outrun by competitors | empirical | **Plausible, and it is the framework's own scope condition** |
| A3 | "We don't need so much developers any more" | empirical | **Not established; the phrasing conflates two different claims** |
| A4 | "Jevons paradox hits now" | structural | **Defensible as a rebound claim; the model gives it a number** |
| A5 | Only one part of the SDLC improved | structural | **Supported, and it is the framework's central asymmetry** |
| A6 | The aviation lesson | historical | **Substantially correct, and needs three corrections** |
| A7 | Nobody knows for sure what happens | — | True, and it is the honest note in the passage |

### A1 — the fear claim

There is reporting of rising burnout and of developers' distrust about what AI will do to their jobs
([reported study summary, retrieved headline only](https://tech.yahoo.com/ai/articles/study-shows-concerning-rise-developer-184500858.html);
the article body was truncated in retrieval, so no figure is quoted here). That supports the
*atmosphere*. It does not support "working very hard to prove they are still worth", which is an
inference about motive and has no measurement behind it in anything I could reach.

**Verdict:** keep as observation, not as finding. "A lot of people are afraid" is defensible as an
impression; "developers are working to prove their worth" needs a study.

### A2 — the owners' fear, which is load-bearing

This looks like the softest claim in the passage and it is actually the most important one, because
**the framework cannot explain its own result without it.** Scope condition S1 in
`OPTIMIZING_FOR_WHAT.md` §9 says utilisation is held at its lean target when capacity rises. A2 is
that condition stated as a motive: owners raise the bar rather than bank the slack because they fear
being outrun.

That makes A2 the sentence to defend. If owners *do* bank the slack — shorter weeks, same output —
then the fragility result does not follow, and the whole passage collapses into good news. Nobody in
this repository has measured which of the two happens. It is the single most valuable empirical
question either passage raises, and it is not the one either passage asks.

**Verdict:** keep, and promote it. It is a scope condition wearing a remark's clothing.

### A3 — "we don't need so much developers any more"

This compresses two different claims that can move in opposite directions:

- **quantity**: fewer developers are employed;
- **price**: the developers who remain are worth more each.

A headcount fall with a price rise is entirely consistent, and it is roughly what the sources I
reached describe from different angles — a book titled *Code is Cheaper but Expertise are More
Valuable* ([freeCodeCamp](https://www.freecodecamp.org/news/the-state-of-ai-and-software-engineering-book)),
and a report that AI is replacing engineers who "only know how to code"
([Business Insider](https://africa.businessinsider.com/careers/former-google-distinguished-engineer-says-ai-isnt-replacing-software-engineers-just/jtxxt9h)).
These are headlines, not measurements, and none of them settles the question.

What the repository can say is narrower and more useful: **it has no labour market at all.** The
framework models one workflow with an exogenous demand (§S5), so it cannot adjudicate whether the
economy needs fewer developers. Anyone quoting this document for that claim would be overreading it.

**Verdict:** mark as hypothesis. And split it: "fewer heads" and "less need" are not the same claim,
and the second one is the one that would need a demand model.

### A4 — Jevons, and the number the model gives it

The terminology is doing too much work. Jevons's paradox is the specific case where an efficiency
gain *increases* total consumption of the resource; the general phenomenon is a **rebound effect**,
of which Jevons is the extreme. Whether AI-driven code generation produces Jevons rather than ordinary
elastic demand depends on a mechanism nobody in this passage supplies.

What the framework can do is turn the slogan into a threshold, because its `generation_multiplier` is
exactly a rebound parameter: how much more work the machine makes it cheap to generate. Sweeping it
against the capacity to judge gives a ceiling:

| capacity to judge | largest generation rebound the workflow absorbs |
|---:|---|
| 0.908 (at adoption) | **2.00×** |
| 0.742 (after 29 years of tracking) | **1.75×** |

At the adoption capacity a rebound above about 2.0 leaves the workflow without a steady state; after
deskilling the same ceiling has fallen to about 1.75. So "Jevons hits now" acquires a meaning: **the
paradox does not hit as an event, it has a ceiling, and the ceiling falls as the capacity to judge
falls.** The declared world sits at a rebound of exactly 2.0 — on the ceiling at adoption, over it
within a few years.

That is a stronger claim than the one in the passage, and it is testable: it says what to measure
(actual generated volume per team) and what would refute it (teams doubling generated volume and
staying inside their commitments indefinitely).

**Verdict:** keep the idea, replace the word, and quote the ceiling.

### A5 — "only one part of SDLC improved"

This is the framework's central asymmetry, and it is the one claim in Passage A that the repository
*formalises*: generation is multiplied while review is unchanged, and the load identity then does the
rest. It is also theory of constraints, which needs no new evidence: improving a non-constraint
raises inventory, not throughput.

One correction. "Others are still as they were" is too strong — continuous integration, observability
and deployment tooling have all improved. The defensible version is narrower and better: **the stage
that improved is not the stage that constrains the system.** That is what the model assumes and what
the evidence on review burden supports.

**Verdict:** keep, with the constraint framing. It is a theorem, not an observation, and it should be
stated as one.

### A6 — the aviation record, checked

This is the strongest thing in either passage, and it is checkable. It is **substantially correct**,
with three corrections that make it more useful rather than less.

**Sourced:** the FAA's Flight Deck Automation Working Group published its final report,
*Operational Use of Flight Path Management Systems*, on 5 September 2013 — an evidence-based review
whose underlying themes include "concerns about degradation of pilot knowledge and skills", and whose
28 findings produced 18 recommendations ([SKYbrary summary and full report](https://skybrary.aero/articles/operational-use-flight-path-management-systems)).
Two of those recommendations are the ones the passage is reaching for:

- **Recommendation 1, Manual Flight Operations** — "Pilots must be provided with opportunities to
  refine this knowledge and practice the skills."
- **Recommendation 9, Operational Policy** — the policy should "highlight and stress that the
  responsibility for flight path management remains with the pilots at all times", and should
  "recognize the importance of automated systems as a tool (among other tools)".

**Correction 1 — not a rollback.** The FAA did not switch back to a lower level of automation.
Automation stayed; what was redesigned was *training, procedures, policy and design philosophy*, with
accountability explicitly re-anchored on the pilot. "Mid-level automation" is the right intuition and
the wrong description: the change was to protect the skill and reassert the responsibility, not to
remove the automation.

**Correction 2 — not the FAA alone, and not sudden.** It was a convened working group, and it was the
first comprehensive review since a 1996 FAA report on flightcrew interfaces — a seventeen-year gap.
The report also notes that further accidents occurred *after* its data collection, with "vulnerabilities
similar to those identified in this report". Institutional learning here took a generation, which is
the opposite of the reassurance the passage is looking for.

**Correction 3 — the mechanism matters more than the precedent.** The pilots were not deskilled
because automation was used. They were deskilled because the practice that maintained the skill was
removed from the job. That distinction is the whole of `THE_PSYCHOLOGICAL_LAYER.md`, and it predicts
what the aviation lesson does *not* imply: that less AI is safer. It implies that the same AI with a
protected practice is safer, which is precisely Rec 1.

**Verdict:** keep, and rewrite the three specifics. As written, a reviewer who knows the file will
correct it; corrected, it is the passage's strongest evidence.

## 4. Passage B, claim by claim

| # | Claim | Kind | Verdict |
|---|---|---|---|
| B1 | "a real meltdown, not just a gap" | structural | **Contradicted by the model's own finding** |
| B2 | The younger generation is not developing the capacities | empirical | **Not established; the weakest sentence in the passage** |
| B3 | The relationship inverts to machine-to-human | structural | **Supported as a mechanism** |
| B4 | "Emotion gets disrupted by a different way of thinking" | — | **Vague; cut or make specific** |
| B5 | We optimise for output while hollowing out the capacities that make it worth anything | structural + normative | **The strongest sentence, and the model backs its structure** |
| B6 | The runway is an edge | structural | **Supported, but it buys time rather than safety** |

### B1 — "meltdown" is the one word the model forbids

The framework's sharpest result is proposition **P7**: from adoption to the breach the workflow
admits the same volume every year — 24.00 items per day throughout — while the share of defects
escaping rises from 0.428 to 0.468. The failure is *silent in throughput*. That is not a meltdown.
A meltdown is visible, and visibility is what the model says does not happen.

So the word undercuts the argument it is making. "Meltdown" is what a reader will check for and
fail to find; "quiet erosion" is what the numbers show, and it is more alarming because nothing
alarms.

**Verdict:** replace. The passage's own later sentence — "quietly reshapes" — is the accurate one,
and the paragraph should not contradict itself three lines apart.

### B2 — the generational claim

This is the sentence a committee destroys in one question, and the repository has already said so in
`THE_PSYCHOLOGICAL_LAYER.md` §11 item 1: no longitudinal measurement of the capacity to judge is used
or produced, and none is known to the author. "The younger generation is increasingly not developing
the muscles" is a claim about a trend in a population, and there is no measure behind it anywhere in
this work.

There is also a competing explanation that must be excluded before the claim can stand: the capacities
may be developed *elsewhere than work*, in which case the phenomenon is a relocation rather than a
loss. Nothing here distinguishes those.

**Verdict:** mark as hypothesis, or cut. If it stays, it should read as a hypothesis the work is
built to test, not as a premise the work depends on — because the deskilling argument does not need
it. That is the same discipline §10 of the psychological document applies to the field position.

### B3 — the inversion

Supported as a mechanism rather than an observation, which is the right status for it.
`required_assistance` makes assistance a *solution to a constraint* rather than a choice, and the
tempo sets the constraint. The model's version is worth quoting because it is checkable: the person
does not choose more assistance, they are cornered into it, and the cornering is what removes the
practice.

### B4 — "emotion gets disrupted"

This is the only sentence in either passage with nothing behind it. The model's emotional channel is
declared, static, and explicitly labelled its weakest part; "a different way of thinking" does not
say what changes or in which direction.

**Verdict:** cut, or replace with the one thing the model does say: coherence falls below a half once
the human share of interaction drops beneath 0.467, and below that the machine's register is the
equilibrium rather than the exception. That is specific, falsifiable, and it is what the sentence is
trying to say.

### B5 — the strongest sentence

> not that AI makes us lazy, but that it quietly reshapes how we feel, relate, and judge, and we
> optimize for output while hollowing out the human capacities that make the output worth anything.

The model backs the *structure* of this exactly. Effective output rises — 1.502 against 1.138 for the
held counterfactual — while mean capacity falls to 0.776 against 0.834, and 30.9% of the tracked
cohort ends below the declared autonomy floor against none of the held one. The objective is not
self-defeating; it is **incomplete**, which is the harder claim and the one this sentence makes.

One honesty note: of the three verbs, the model addresses **judge** directly, **relate** through the
human share, and **feel** only through the weakest channel in the model. So the sentence is
one-third measured mechanism and two-thirds gesture — which is fine for a sentence and not fine for a
claim.

**Verdict:** keep verbatim. It is the thesis, compressed, and it is correct.

### B6 — the runway

Supported with a sharpening the model forces. At the declared parameters a six-year runway buys
**3.2 years spent judging** and **+0.002** of capacity at the horizon. And proposition P6 says the
learning buffer delays the breach from year 5 to year 8 rather than preventing it.

So the runway is an edge in **time**, not in destination. "Having the runway to learn is an edge" is
true and should stay; "whether teams get that runway" is the right open question; and the sentence
should carry the finding that makes it consequential: the runway buys years, not safety.

## 5. The attribution, which I could not verify

I did not find the text of Passage A in any source I could reach. A search for "Milan Milanovic"
returns **Branko Milanović**, the economist, discussing AI and inequality — a different person — so
the attribution risk here is concrete and specific rather than hypothetical.

What that means for use:

- the **reasoning** in Passage A can be evaluated, and is evaluated above;
- the **words** cannot be attributed to him on the evidence available here;
- if it is quoted publicly, it should be quoted from the post itself with a link, exactly as the
  Faggin correction in `THE_PSYCHOLOGICAL_LAYER.md` §10.2 insists: a paraphrase that reads well is
  not a quotation.

## 6. What the two passages get right that the repository had not said

Verification should also record where the passages outrun the documents.

1. **A2 is a better statement of the framework's key assumption than the framework's own.** "Owners
   fear being outrun" is why utilisation is held constant, and it is more legible than S1.
2. **A6 is the empirical precedent this work lacks.** The aviation record is a documented case of
   exactly the mechanism the psychological layer models: skill degradation identified by the
   regulator, with practice-protection recommended as the remedy. It should go into the document as
   the one historical instance, cited properly.
3. **A3 and B2 are two different failure modes, and only one is loud.** If headcount falls, the
   loss is visible and someone will act. The framework's case is that the *stable-headcount*
   version — same people, same volume, quietly less capacity — is the dangerous one, because no
   metric moves. Neither passage says this; it is the synthesis they add up to.

## 7. The defensible version

Same argument, with everything that would be destroyed in a review removed. This is offered as text
to use, not as a summary of the analysis:

> Two fears are now doing the work: developers afraid of being replaced, and owners afraid of being
> outrun. The second one is the interesting one, because it decides what happens to the slack.
>
> The landscape has changed in a specific way: we improved the stage that generates code and left the
> stage that reviews it where it was. Improving a non-constraint does not raise throughput; it raises
> what waits. That is why the load per work item now includes rework at a cost the old metrics never
> counted — roughly 29% of the review load at the parameters we use.
>
> Whether this becomes the rebound that Jevons described depends on a number nobody measures: how much
> more work the tooling makes cheap to generate. In our model the workflow can absorb about a
> doubling. After a decade of tracked assistance, the ceiling falls to about 1.75×. The paradox, on
> this reading, is not an event — it is a ceiling, and the ceiling moves down as the capacity to judge
> falls.
>
> Aviation learned this the expensive way. The FAA's 2013 working group found degradation of pilot
> knowledge and skills and recommended two things: keep the manual skill exercised, and keep the
> responsibility with the pilot. It did not remove the automation. That distinction matters, because
> the lesson is not that we should use less AI — it is that the same AI with a protected practice is
> safer than the same AI without one.
>
> The risk worth naming is not that AI makes us lazy. It is that it quietly reshapes how we judge, and
> we optimise for output while the capacities that make the output worth anything erode — invisibly,
> because throughput does not move. The failure we model is silent by construction: the same volume
> every year, with more of it escaping review.
>
> The one lever we have is time. A runway bought before expectations land is worth about three years
> of autonomous practice — not a different destination, but three more years of the work that builds
> the capacity. Whether teams get that runway before the bar arrives is the open question, and it is
> not a technical one.

## 8. What would settle the open claims

| Claim | What would settle it | Where it belongs |
|---|---|---|
| A1 | A survey with a motive measure, not an atmosphere measure | Field work |
| A2 | Do firms bank slack or raise the bar? Observed, not asserted | **The thesis's missing empirical chapter** |
| A3 | Separating headcount from price in developer labour data | Economics, not this repository |
| A4 | Measured generated volume per team, against its commitment | Instrumentation, which many teams already have |
| B1 | Nothing — the word should go | Edit |
| B2 | A longitudinal measure of judgement quality on assisted tasks | The thesis's first falsification test |
| B4 | Nothing yet; the emotional channel is the model's weakest part | Model work before claim work |
| B6 | A cohort with a protected runway, measured on autonomy years | The one intervention the framework recommends |

## 9. Verdict in one paragraph

Passage A is mostly right about the mechanism and too confident about the facts: its strongest claims
are the ones that look softest (A2, A5, A6) and its weakest is the one stated most flatly (A3). Its
Jevons line is a good idea wearing the wrong word, and the model gives it a number — a ceiling near
2.0 at adoption, 1.75 after deskilling. Passage B is the better passage, and one word in it is wrong:
"meltdown" is precisely what the model says does not happen. Cut B2 or mark it as hypothesis, cut B4
or make it specific, keep B3, B5 and B6 almost verbatim, and add to B6 the finding that makes it
matter — the runway buys time, not safety.
