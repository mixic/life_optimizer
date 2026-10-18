# Validation

*What has been checked against something outside this repository, what has not, and what the checks
found. The review's single highest-leverage credibility request was for this document; writing it
is also how the largest defect in the repository was found.*

---

## 1. What this document can and cannot be

It cannot be a comparison of this tool against the official ESTV calculator, because no offline
copy of that calculator exists here and this work has no way to query one. Claiming a comparison
that was not performed would be worse than having no document at all.

What it can be is a record of three things: the reference figures the repository *does* hold from
an external source, what the code produces against them, and the checks that establish shape rather
than level. Every number below was produced by running the code in this repository, and every
command to reproduce it is in §6.

## 2. The tax engine, against externally sourced figures

### 2.1 The source

`tests/published_tax_reference.rs` holds fifteen reference points: total tax (Bund + Kanton +
Gemeinde) against taxable income, for Zürich, Schwyz and Bern, at CHF 25,000, 50,000, 100,000,
250,000 and 500,000. They come from [finpension.ch, "Steuerprogression in der
Schweiz"](https://finpension.ch/en/blog/swiss-tax-progression/), published 16.06.2026, quoting the
**ESTV Steuerrechner for Steuerjahr 2025**, under the stated assumption "alleinstehend,
konfessionslos, keine Kinder".

These are the only externally sourced cantonal figures in the repository. Fifteen points for three
of twenty-six cantons is not a tax scale, and the file says so; it can validate shape and ordering,
not level.

### 2.2 Bern, the one canton with a complete table

| Taxable income | Model (cantonal + municipal + church) | Published (incl. federal) | Difference |
|---:|---:|---:|---:|
| CHF 25,000 | 5.41% | 16.01% | **−10.60 pp** |
| CHF 50,000 | 12.00% | 18.50% | **−6.50 pp** |
| CHF 100,000 | 17.24% | 22.98% | **−5.74 pp** |
| CHF 250,000 | 24.73% | 32.15% | **−7.42 pp** |
| CHF 500,000 | 24.73% | 37.09% | **−12.36 pp** |

**The gap is expected and is not evidence of an error**: the two columns are not the same quantity.
The published figure is total tax including the federal share; `tax_rate_on_taxable` is the
cantonal, municipal and church schedule alone. The table is also Steuerjahr 2024 against 2025
figures. The test therefore bands the difference at −5.0 to −13.5 pp rather than demanding equality,
and says in its comment that it is a regression guard rather than a calibration.

A rate-base bug once made these numbers disagree with `SWISS_TAX_DATA.md` §3: the schedule was
looked up on income that had already been reduced, deducting twice and understating every model
figure by 1.3–1.8 pp. That is the kind of defect the band is there to catch, and it did.

### 2.3 The open question this comparison surfaced

The Bern model's average rate reaches **24.73% at CHF 200,000 and stays there to CHF 1,000,000** —
the tax is exactly proportional above that point. When a test was written asserting that a
progressive schedule's average rate keeps rising, it failed.

That assertion was wrong, not the code. A **satzbestimmendes** cantonal tariff determines a rate
from the income and applies it to the whole of it, so once the top rate is reached the tax *is*
proportional. The test now pins the observed behaviour, and `tests/validation.rs` records why.

What remains unverified: whether 24.73% is the correct top rate for Bern. That needs the cantonal
tariff itself, which this work could not read offline. It is listed in §5, not resolved.

### 2.4 Shape checks

Three properties are asserted in `tests/validation.rs`, because they are the ones a mistyped table
row breaks:

- the average rate never falls as income rises (checked from CHF 10,000 to 1,000,000);
- below the top bracket the rate is genuinely progressive (5.41% at 25,000 against 17.24% at
  100,000);
- above it the rate is constant and the tax is the rate times the income.

These are observations pinned, not law verified. That distinction is in the file header.

## 3. The pension projection — and the defect validating it found

Asking the question *what does this projection assume?* was enough to expose an error that had been
in every adequacy verdict the tool ever printed.

**Savings contributions were levied on gross salary.** The BVG applies its age-banded rates to
`min(salary, upper limit) − coordination deduction`, and to nothing below the entry threshold (BVG
Art. 2 Abs. 1 and Art. 8 Abs. 1; BVV 2 Art. 5). The projection used gross salary.

| | Before | After |
|---|---:|---:|
| Contributions, CHF 120,000 at age 40 | CHF 12,000 | CHF 6,426 |
| Projected retirement income, same case | CHF 2,765/month | CHF 2,207/month |

**Contributions were 46% too high and projected income 20% too high**, across every case the tool
has ever reported. The upper-limit cap was also missing from `early_retirement.rs`, which
documented it in its parameter table and never applied it.

Two things about the survival of this defect matter more than the arithmetic. The same crate
already had it right — `early_retirement.rs` subtracts the coordination deduction with the sourced
figures and their citations — so this was two models disagreeing about one statute, and the one
driving the adequacy verdict was the wrong one. And **458 tests passed before the fix and after it**,
because none of them asked what salary a contribution is levied on.

`tests/bvg_contribution_base.rs` now pins the base, the entry threshold and the cap. The test that
would have caught it is the sharpest of the five: **two salaries far apart but both above the upper
limit must accumulate the same BVG capital**, because the scheme insures the same amount of both.
Under the old code those differed by nearly a factor of two.

What is *still* not validated: the projection has not been compared against a real BVG statement or
a fund's own projection, only against the statutory formula. That is §5's largest gap.

## 4. Monte Carlo convergence

The review asked for convergence checks, and there were none — only a reproducibility test that the
same seed gives the same answer.

`the_monte_carlo_median_spread_shrinks_as_paths_increase` measures the dispersion of the median
capital **across five seeds** at two path counts, because a single run at two counts can be closer
to or further from the truth by luck, whereas the standard error falling with the square root of the
count is a property that has to hold:

| Paths | Spread of the median across 5 seeds |
|---:|---:|
| 250 | CHF 24,769 |
| 2,000 | CHF 6,618 |

An eight-path increase in sample size cutting the spread by roughly three quarters is consistent
with `1/√n` within the noise of five seeds. The test asserts the direction rather than the constant,
since five seeds cannot estimate a standard error precisely enough to assert the constant.

## 5. What has not been validated

Stated plainly, because the value of §2–§4 depends on it.

1. **No comparison against the ESTV calculator.** Fifteen published points for three cantons is what
   exists here. A full matrix of income × status × canton against the official calculator would
   subsume §2 and supersede it; it needs the calculator, which needs a network session.
2. **Twenty-three of twenty-six cantons have no external reference point at all.** Their Steuerfuss
   values are sourced, but the base scales are not checked against anything outside the repository.
3. **The Bern table is Steuerjahr 2024 against 2025 figures.** A one-year drift is inside the band
   and cannot be separated from a table error by this method.
4. **The Bern top rate of 24.73% is unverified** (§2.3).
5. **The pension projection is checked against the statutory formula, not against a real pension
   fund statement.** Deductions for coordination, the insured-salary cap and the entry threshold are
   now correct; the contribution *history*, the interest credited, and the fund's own conversion rate
   are modelled, not observed.
6. **The market assumptions are not calibrated to any fund.** Three presets are run and their range
   is reported; none is fitted to data.
7. **The tax engine's *deductions* are not validated here.** `SWISS_TAX_DATA.md` records their
   provenance and the known defects; §2 above validates the rate schedule, not the deduction
   estimate that feeds it.

## 6. Reproducing this document

```bash
# the published reference points and the Bern comparison, with its measured discrepancies
cargo test --test published_tax_reference -- --nocapture

# the shape checks and the convergence check, with the measured spreads
cargo test --test validation -- --nocapture

# the contribution base, including the test that would have caught the defect
cargo test --test bvg_contribution_base

# the projection's own parameters, on screen
cargo run -- optimize --salary 120000 --age 40 --verbose
```

Every table in this document was produced by one of those commands. None of the numbers was
estimated.
