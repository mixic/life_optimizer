# Changelog

*What changed, when, and why. Written from the commit history rather than from memory, and grouped
by the thing a reader would care about rather than by the order the commits landed.*

The project does not issue numbered releases. Dates are the date of the change.

---

## Unreleased — 2026-10-18

A response to an external review of the repository. The review was audited first, in
`CRITIQUE_RESPONSE.md`, because it reads as a list of equals and is not one. Every item below is
answered in that document.

### Correctness

- **BVG contributions were levied on gross salary.** The statutory base is
  `min(salary, upper limit) − coordination deduction`, and nothing at all below the entry threshold
  (BVG Art. 2 Abs. 1, Art. 8 Abs. 1; BVV 2 Art. 5). Contributions were 46% too high and projected
  retirement income 20% too high, in every case the tool had ever reported. Found while writing the
  `--verbose` block that was meant to make those assumptions visible.
- **The BVG upper-limit cap was missing** from `early_retirement.rs`, which documented it and never
  applied it.
- **The team externality is now modelled.** `--team-size`, `--redistribution` and
  `--externality-weight` compute the hours a reduction moves onto colleagues; every reduction result
  carries a warning; a reduction with no team parameters is reported as *not modelled* rather than
  as having no effect. Before this, the tool preferred a schedule that pushes work onto other people
  over one that genuinely produces less.

### Decision support

- **Retirement needs are an input.** `--retirement-needs`, `--retirement-needs-percent` and an
  interactive prompt set the figure; the report shows a 60/75/90 range and the level at which the
  plan stops working.
- **The margin is reported, not just the leader.** The top three feasible options print with their
  margins, and a margin inside a declared 2% band is called a tie rather than a win.
- **A robustness block** re-runs the search with each declared preference halved and doubled. It
  reports an optimum sitting at the edge of the search range as uninformative rather than stable.
- **`--verbose`** prints the pension assumptions: conversion rate with its override flag, the
  contribution base with its statutory citation, the age-banded schedule, and the market
  assumptions.
- **`--ai-scenario sceptical|neutral|optimistic`** sets the five AI coefficients together, anchored
  to the published range in `AI_AS_CATALYST.md` §4. An unknown name is refused, not defaulted.

### Documentation and credibility

- `VALIDATION.md` — what has been checked against an outside source, what has not, and the measured
  discrepancies. Records that twenty-three of twenty-six cantons have no external reference point.
- `CRITIQUE_RESPONSE.md` — the review, claim by claim, with the file and line that settles each.
- `tests/validation.rs` — shape checks on the tax engine and a Monte Carlo convergence check, which
  did not exist.
- `tests/bvg_contribution_base.rs` — the contribution base, entry threshold and cap, including the
  test that would have caught the defect above.
- The interactive mode is documented with a transcript of a real session.
- CI status badge; the clone URL in the install instructions corrected from a placeholder.
- Design philosophy rewritten positively; the GPL rationale now says what the licence is meant to
  prevent.

---

## 2026-10-05 — the psychological and workflow layers

- `THE_PSYCHOLOGICAL_LAYER.md` and the `human_capacity` crate: assistance solved for rather than
  chosen, a deskilling threshold, a defect floor, and a paired Monte Carlo over 2,000 people.
- `OPTIMIZING_FOR_WHAT.md` and the `knowledge_work` crate: the JIT transplant, the load identity,
  two thresholds (commitment at 0.888, stability at 0.839), a nine-proposition certificate, and a
  direction framework that reports its own indeterminacy.
- `VERIFYING_THE_CLAIMS.md`: two passages checked claim by claim against the sources and the models.
- Two defects found by running rather than by reading: a utilisation computed eight times too small,
  and a cohort that collapsed to a single point.

## 2026-10-04 — early retirement

- `EARLY_RETIREMENT.md`, `src/early_retirement.rs`, `src/simplex.rs`, and `tests/early_retirement.rs`:
  the withdrawal allocation as a linear program, its dual, a 10,000-path risk sweep, and the
  education-contribution question priced but deliberately not ranked.
- `THE_RECIPROCITY_OF_THE_EDUCATED.md`: what a publicly funded qualification obliges its holder to
  return.
- Four defects found by the run: a withdrawal applied after the pot was empty, a retirement age
  subtracted from a horizon already in retirement years, an allocation that ignored *when* the money
  was needed, and an AHV pension penalised twice.

## 2026-09-22 — strategy, spoilers and information

- `STRATEGY_COMPARISON.md`, `THIRD_PARTY_SPOILER.md` and `INFORMATION_WARFARE.md`, with their
  modules and figures: two strategies priced on one budget and one clock, conflict as a failed
  equilibrium, and legitimation as a production function whose capital is credibility.

## Earlier

- The optimiser itself: the tax engine, the cantonal schedules and Steuerfuss data, the utility
  function, the Monte Carlo pension projection, and the multi-crate split that keeps illustrative
  simulator figures out of the reported tax numbers.
- `AI_AS_CATALYST.md`: the published evidence on AI-assisted software work, and what it implies for
  validating delivered quality.

---

## How to read this file

An entry that says a defect was found also says what found it, because the second half is the useful
part. The pattern in this repository is that running the tool finds more than reading it does: the
BVG error above survived 458 passing tests and was found by writing the document that asked what the
assumptions were.
