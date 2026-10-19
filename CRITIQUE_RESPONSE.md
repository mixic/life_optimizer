# Responding to the Review

*A claim-by-claim audit of the external review of this repository, performed before acting on it.
Every verdict below names the file and line that settles it, including the two places where the
review is wrong and the two places where I was.*

---

## 1. Why the review is audited before it is answered

The review makes fifteen recommendations across correctness, usability and credibility, and most of
them are right. Some are not, and one of them — "split the repository" — would destroy the thing the
repository is for. A remediation plan that accepted all fifteen would spend its effort where the
review is weakest, because a review reads like a list of equals when it is not.

So stage one is this document: what is true, what is false, what is arguable, and therefore what to
do first. The plan in §7 follows from the verdicts rather than from the review's own ordering.

**Two corrections to my own first reading**, recorded because I made them in the goal statement and
they were wrong:

- I wrote that the review's single-canton claim was refuted because "cantons.rs covers all 26 with
  enforced provenance". That is half wrong, and it took a second pass to find out which half.
  `cantons.rs` has the full 26-canton `enum` with a no-fallback provenance rule, but at the time only
  Bern carried a verified table -- so the review's *finding* looked right and its *characterisation*
  wrong. Then the second review repeated the claim and sent me to `estv_scales_data.rs`, which is
  generated from ESTV Tarife exports for the whole country: **23 of the 26 cantons are priced**, from
  an imported scale multiplied by that canton's Steuerfuss, and `TaxSchedule::from_canton_scale` was
  already wired to all of them. So the finding was wrong too, with a residue of three: Fribourg and
  Valais have blank source cells, and Geneva's reads `"147.5%9)"`, which the importer refuses to
  interpret. See §5b.
- I wrote that the review's "no CI" claim was refuted by `.github/workflows/ci.yml`. The CI does
  exist and runs clippy, the full test suite on Ubuntu and Windows, and three rustdoc builds with
  `-D warnings`. But what the review actually observed — **no CI badge and no validation section** —
  is true, and the inference "therefore untrustworthy" is only mildly unfair. The badge is missing;
  the CI is not.

Both corrections are in the table below rather than in a footnote, because the review deserves the
same treatment this repository gives everything else: the claim, the evidence, and the verdict.

## 2. The verdict table

| # | Claim | Verdict | Evidence |
|---|---|---|---|
| C1 | The repository has overreached its scope; dozens of top-level documents for a calculator | **Verified, now restructured** | 38 top-level markdown files, 23 of them essays. They now live in `documents/` with an index, and the root carries 17 tool-facing files. The scope was never the problem; the front door was. |
| C2 | The 0–10 quality-of-life score is unexplained and treated as solved | **Partly — and my first verdict here was too harsh** | `README.md:97` advertises it, `README.md:327` has a "How the math works" section, `MATHEMATICS.md` documents it, and `display.rs:191-200` **does print all five components** beside the total. What is missing is the **weights** and any sensitivity analysis, so the decomposition is visible but unarguable. The remedy stands, with a smaller target than the review assumes. |
| C3 | 75% of net income is a rule of thumb dressed as a parameter, inconsistent with the tool's own critique | **Verified as a rule of thumb; narrower in scope than stated** | It is real and hardcoded at `main.rs` in the standalone pension command. The **main optimization path already uses a modelled household budget**, not the rule of thumb. So the inconsistency is real but confined to one command, and the fix is to report a range and a break-even in both. |
| C4 | The team externality is not modelled and is underplayed | **Verified as a gap; partly on "underplayed"** | No code: every `team`/`colleague` match in `src/*.rs` is a licence header. But `TEAM_DEPENDENCIES.md` exists as a design document and `README.md:457` states the limitation. What is missing is any warning at the point of *result*. |
| C5 | Maintenance: large volume, single author, aging tax data | **Verified** | `git log` has one author; last commit 2026-10-18; Bern 2024 tables; no `VALIDATION.md` |
| C6 | The GPL relicense is a notable, under-explained signal | **Verified, now explained** | The README stated the benefit and never what it prevents. It now names the case: a closed-source service wrapping the code and selling the answers back to the people whose assumptions it hides. |
| 1.1 | Model the team externality or warn on every reduction | **Verified** | See C4 |
| 1.2 | Make retirement needs a first-class input, report a range and break-even | **Verified, now fixed** | Needs were derived, never asked for. `--retirement-needs`, `--retirement-needs-percent` and an interactive prompt now set it; a 60/75/90 range and the break-even are printed. |
| 1.3 | Surface the utility aggregation, print components and weights, add sensitivity | **Partly — components already printed; weights and sensitivity are now added** | `display.rs:191-200` printed the five components and the total. The weights were never shown and nothing was perturbed; both are now done, by `src/robustness.rs`. |
| 1.4 | Single-canton calibration is a structural limit | **Refuted by the code, with a residue of three** | **23 of 26 cantons are priced**: Bern from its own official table, the other 22 from an ESTV Tarife import multiplied by the canton's Steuerfuss. Fribourg and Valais have blank source cells; Geneva's reads `"147.5%9)"` and the importer refuses to guess. See §5b. |
| 1.5 | Pension parameters asserted without figures in the README | **Verified** | `README.md:96` mentions "the higher conversion rate you earn by working longer" and never gives the number |
| 2.1 | The scope has swallowed the product | **Verified, now restructured** | Same evidence as C1; the root README is tool-first and `documents/README.md` indexes the arguments. |
| 2.2 | No validation or test evidence in the README | **Verified for the badge, now added** | CI exists and passes; there was no badge and no validation document. `VALIDATION.md` now records the fifteen ESTV-sourced reference points, the measured Bern discrepancies, the convergence check, and what could not be checked. The README now carries the CI badge and points at `VALIDATION.md`. |
| 2.3 | The output is dense but not decision-oriented | **Verified, now fixed** | It printed an optimum and a "closest option" fallback with no margins. `display::print_decision_margin` now prints the top three with margins and calls a margin inside a declared 2% band a tie rather than a win. |
| 2.4 | The AI-productivity CLI surface is overbuilt for its epistemic status | **Verified, now moved** | The eight flags are no longer on `optimize`: they live on a `stress-ai` verb, which accepts every optimize flag plus the employer coefficients. `--ai-scenario sceptical\|neutral\|optimistic` stays on `optimize` for the common path and prints what it set. Anchored to the published range in `AI_AS_CATALYST.md` §4. |
| 2.5 | The interactive mode is mentioned but undocumented | **Verified, now documented** | Three mentions and no transcript. `README.md` now carries a real session, including the retirement-needs question and an explicit note of what the interactive flow does *not* ask. |
| 3.1 | The repository reads as a personal manifesto | **Verified, now restructured** | Same evidence as C1. The essays remain, in their own directory, with an index that says what each one argues -- and why they are not a separate repository is argued in §4. |
| 3.2 | The AI contrast in the design philosophy is a strawman | **Verified, now rewritten** | The section opened by contrasting with "AI, gradient-based optimization, black-box simulators" and then conceded those methods "can be extremely useful", which invites the reader to wonder what is being argued against. It is now stated positively: what the model exposes, and what it refuses to claim. |
| 3.3 | The GPL rationale is under-explained | **Verified, now explained** | See C6. |
| 3.4 | Single-author velocity and undefined bus factor | **Verified, now stated** | `git log` shows one author. The README now carries a maintenance statement naming the bus factor of one, what is maintained carefully, and what ages. |

**Score: 10 verified, 7 partly true, 0 wholly false.** Two rows moved from verified to partly true
after checking `display.rs` and `cantons.rs` more closely than my first pass did; the review is a
good review. The two places it is wrong are both about *what the architecture already does*, and
both matter because they change which work is worth doing.

## 3. The four claims that should drive the plan

The review's own top three are close to mine, with one substitution.

### 3.1 The team externality — the review is right, and it is worse than stated

`README.md:457` says reduced availability is modelled as having no effect on colleagues, and that
this makes "push work onto teammates" schedules look better here than they are. Every `team` or
`colleague` match in `src/*.rs` is a licence header: there is **no code at all**.

The review's framing is right and understates the consequence. This is not an omission on the
periphery of the decision; it is the decision. An 80% schedule that redistributes a fifth of the work
is a different act from one that genuinely produces a fifth less, and the tool prices both
identically. A tool that prefers the redistributing schedule because it looks cheaper is not
neutrally incomplete — it is **systematically biased toward the option that externalises cost onto
colleagues**, which is a moral failure and not only a modelling gap.

That reframes the fix: a crude model is good, an unmistakable warning is mandatory, and neither is
sufficient alone. A user who takes a reduction recommendation away without knowing they may be
pushing work onto others has been misled by the tool's silence, and the silence is the tool's
responsibility.

### 3.2 The needs default — the same error the tool was built to avoid

The tool's "Why this exists" (`README.md:15-28`) attacks calculators that stop at gross-minus-tax and
miss what determines the decision. Then it adopts the least examined retirement assumption available:
75% of net income, "a common rule of thumb, not a personalized budget" (`README.md:450-451`).

The critique of gross-minus-tax was that it hides the thing that decides the answer. So does this.
The fix the review proposes is the right one and goes further than it says: **a break-even needs
figure** — the level of need at which the plan stops being adequate — is more informative than any
coverage percentage, because it tells the user which side of a cliff they are standing on and how
far the edge is.

### 3.3 The aggregation is invisible at the point of decision

The review says the README treats the 0–10 score as solved. The evidence is more specific and less
damning: the mathematics *is* documented (`MATHEMATICS.md`, and a "How the math works" section at
`README.md:327`). What is missing is that **the output prints `TOTAL UTILITY: 31.75` and never the
components or the weights**. A tool whose stated selling point is that "the user must be able to
challenge each assumption" prints a number that cannot be challenged without opening a document.

The remedy is the review's and it is cheap: print the components, print the weights, and report how
the optimum moves when they are perturbed. If the answer is stable, that is a finding worth more
than the point estimate; if it flips, the user has learned that the answer is a preference, not a
measurement.

### 3.4 There is no validation against anything external

This is the item the review calls the single highest-leverage credibility improvement, and it is
right. `README.md:478` solicits tax tables and model calibration from contributors, while nothing in
the repository demonstrates that the tax engine reproduces an official calculator's answer.

Everything else in this list is about the tool's honesty. This one is about its **correctness**, and
it is the only one where the tool might be quietly wrong rather than openly incomplete. It goes
first in stage three for that reason.

## 4. The one recommendation I will not follow

The review's third priority is to split the repository, moving `multipolar_sim`, `human_capacity`,
`knowledge_work` and every non-tax document into a separate project.

**I am not doing that, and the disagreement is substantive rather than sentimental.**

The review's diagnosis is right: a reader arriving at a Swiss pension calculator and finding 38
documents about information warfare, Plato's tripartite order and NBA analogies will bounce. But the
prescription assumes the essays are an unrelated hobby that accumulated around a calculator. They are
not. They are the same argument at different scales: the pension tool prices what a working life is
worth, `EARLY_RETIREMENT.md` prices when to stop, `THE_RECIPROCITY_OF_THE_EDUCATED.md` asks what is
owed for the qualification that made the salary possible, and the psychological and workflow layers
ask what the work does to the person doing it. Splitting them leaves the calculator without the
argument that justifies its existence and the essays without the numbers that keep them honest.

What the review is actually right about is **presentation**, and presentation is fixable without
amputation:

- the root README becomes tool-first: what it does, how to run it, what it cannot do — the essays
  move to `documents/` with an index that says what each one argues and why it is there;
- the calculator's own documentation stops being interleaved with manifestos;
- the crate boundary stays, because it is already the separation of concerns the review praises.

Same repository, different front door. That is the version of the review's point I can defend; the
split is the version I cannot, and saying so is more useful than complying.

## 5. Where the review sees something the repository had not said

An audit that only defends is not an audit.

1. **The bias direction.** The review notes the tool makes push-work-onto-teammates schedules "look
   better here than it is". It does not say the obvious next sentence, which is that an incomplete
   model with a known directional bias is worse than an incomplete model with no bias, because it
   recommends the wrong thing rather than merely failing to answer. This is the strongest point in
   the review and it was not labelled as such.
2. **The consistency argument.** C3's real force is not that 75% is a bad number. It is that the tool
   uses a rule of thumb *of exactly the kind it built itself to replace*. That is a self-refutation,
   and it is more damaging than an ordinary weak default.
3. **The ratio of surface to evidence.** 2.4 is another instance: eight CLI flags for coefficients the
   README calls illustrative. The general form — *a parameter surface larger than the evidence base
   behind it* — applies to more of this repository than the AI feature, including to the essays. It
   is the best single sentence in the review and it should probably be the repository's epigraph.
4. **"A near-tie is a different decision from a clear winner."** 2.3's best line. The tool reports an
   optimum; it never reports whether the optimum is *distinguishable* from the runner-up, which is
   the question a person actually faces.

## 5a. The defect the review's suspicion found

The review said the tax engine is useful "if the tables are accurate" and asked for a pension
projection checked against a real BVG case. Writing the `--verbose` block that would have made
those assumptions visible exposed the defect it was looking for, and it was not in the tax engine.

**Savings contributions were levied on gross salary instead of the coordinated salary.** The BVG
applies its age-banded rates to `min(salary, upper limit) − coordination deduction`, and to nothing
at all below the entry threshold (BVG Art. 2 Abs. 1 and Art. 8 Abs. 1; BVV 2 Art. 5). The
projection used gross salary, so at CHF 120,000 and age 40 it contributed 12,000 where the law
requires 6,426 — **46% too much** — and the projected retirement income was **20% too high** across
the board (CHF 2,765/month against the corrected 2,207 for the same case).

Two things about it are worse than the arithmetic.

The same crate already had it right: `early_retirement.rs` subtracts the coordination deduction,
with the sourced figures and their citations in a provenance table. `monte_carlo.rs` did not. So
this was not an unknown law to the project; it was two models disagreeing about the same statute,
and the one that drives the adequacy verdict was the one that was wrong.

And *nothing in the suite caught it*. Four hundred and fifty-eight tests passed before and after,
because none of them asserted what salary a contribution is levied on. The defect survived not
because it was subtle but because it was unasked, which is precisely the failure mode the review
identified from the outside: a tool making strong claims about auditability while leaving its most
consequential input unexamined.

`tests/bvg_contribution_base.rs` now pins the base, the entry threshold and the cap, and includes
the test that would have caught it: two salaries far apart but both above the upper limit must
accumulate the same BVG capital, because the scheme insures the same amount of both.

The cap was also missing from `early_retirement.rs`, which documented the upper limit in its
parameter table and never applied it. Both are fixed.

## 5b. The second review: what it got right, and the two claims it got wrong

A second review, after the restructure, is recorded here because it repeated one claim this
document had already corrected and added a new one that was also wrong about the code.

**Right, and fixed.** The README's example output was stale: it still showed the old single-point
`Coverage: 117% of needs` format while the README promised a 60/75/90 range, a margin, a robustness
block and a team warning. The features were all wired — a live run prints every one of them — so
this was a documentation failure rather than a code one, which is the better of the two. It is fixed
by regenerating the example from a real run, which also exposed that the old 80% recommendation no
longer reproduces and that the search grid started at 50%, making 50% the answer for anyone
preferring less work.

**Wrong: "the tool remains Bern-accurate and Bern-adjacent-approximate."** Twenty-three of the
twenty-six cantons are priced, from ESTV Tarife exports multiplied by each canton's Steuerfuss, and
`TaxSchedule::from_canton_scale` was already wired to all of them. Four of the five cantons the
review named — Zürich, Vaud, Basel-Stadt and Zug — have worked all along; naming them as gaps
suggests the review tested the README rather than the binary.

What is true is smaller and more interesting. Three cantons cannot be priced: **Fribourg and
Valais** because the source cells are blank, and **Geneva** because the cell reads `"147.5%9)"` — a
value with a footnote marker fused to it, or a percentage of another base, and the importer
deliberately refuses to guess which. That refusal is the repository's own discipline applied to its
own data, and inventing `1.475` would be exactly the sin the rest of this work is about.

The README now says 23 of 26 rather than "Stadt Bern 2024", the assumptions block names the three
cantons and the reason, and `tests/canton_priceability.rs` pins the split so a fourth canton cannot
silently join them.

**Wrong: "either the warning isn't wired into the display, or the example predates the feature."**
It was the second, and the review's own hedge was the correct answer. `display::print_team_externality`
prints a bordered warning for any reduction whose team effect is unmodelled; the old example was
written before it existed.
## 6. What the review gets wrong, stated plainly

- **The canton architecture.** See §1. The review infers a Bern-only design from a Bern-only
  calibration. The design is general, the data is not, and the remedy it proposes is correct anyway —
  which is why this correction saves effort without changing the plan.
- **The CI.** It exists, it runs on two platforms, and it includes three doc builds with warnings
  denied. The badge is what is missing. A reviewer who checked `README.md` for a badge found none and
  concluded none existed; that is a presentation failure on the repository's side, but the
  engineering is not absent.
- **"Use it as a checklist of considerations, not as an oracle."** This is offered as a
  consolation and is, in fact, the design intent. The tool says so itself (`README.md:463`). The
  review is right that a user may not read that line; it is wrong to treat the intent as a retreat.

## 7. The plan, and why in this order

The goal has three stages. The order below is set by §3 rather than by the review's numbering.

**Stage one — this document.** Done when every claim has a verdict with evidence.

**Stage two — correctness and usability**, in this order:

1. team externality: a minimal declared model (`--team-size`, `--redistribution`, a load penalty) plus
   an unambiguous warning on every reduction result, because §3.1 is the only item where the tool
   currently recommends the wrong thing;
2. needs as a first-class input: interactive prompt, range over 60/75/90, and the break-even figure,
   because §3.2 is where the tool contradicts itself;
3. component utilities, weights, and a robustness block, because §3.3 is the cheapest fix with the
   largest effect on trust;
4. pension parameters behind `--verbose` with overrides;
5. a comparison line with margins and an explicit near-tie verdict;
6. the interactive transcript;
7. AI-productivity presets on a subcommand, keeping the coefficients documented.

**Stage three — credibility and structure:**

1. `VALIDATION.md` against external references, because §3.4 is the only correctness item;
2. the CI badge, `CHANGELOG.md`, and a maintainership statement;
3. the design-philosophy rewrite and the GPL explanation of what it prevents;
4. the restructure: tool-first README, `documents/` with an index, cross-references updated and
   verified.

## 8. What this will not fix

Worth saying before the work starts, so the result is not oversold the way the tool has been.

The review's bottom line — *"treat the output as a structured way to think about the tradeoffs, not as
an answer"* — will still be true when all fifteen items are done. A model with a team externality
term, a needs range, a robustness block and a validation document is a better instrument, and it is
still an instrument whose dominant inputs are assumptions the user cannot verify: their own
preferences, their employer's behaviour, and the next forty years of Swiss pension law.

The honest end state is not "now you can trust the number". It is "now the number comes with the
reasons it might be wrong, in the order they are likely to matter". That is what stage two and three
are for.
