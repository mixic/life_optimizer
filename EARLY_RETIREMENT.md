# Early Retirement as an Optimisation Problem

*Code in `src/early_retirement.rs` and `src/simplex.rs`. Companion to
`PENSION_OPTIMIZATION.md`, which projects a plan, and to `monte_carlo.rs`, which
simulates one.*

---

## 1. What changes from projecting to optimising

`PENSION_OPTIMIZATION.md` answers: *given* a retirement age and a work percentage, what
pension results, and is it adequate? That is a projection with a check at the end, and it
leaves the two decisions that matter most — when to stop, and how to take the money out —
to the reader.

This document answers the decisions:

1. **When** to retire, priced in both pillars and against the consumption the person has
   declared.
2. **How** to take the capital out, because in a progressive system the *timing* of a
   withdrawal moves the tax on it by more than a year of contributions does.
3. **How much is enough** — the smallest accumulation that funds the declared consumption,
   rather than the largest that feels safe.

Point 3 is the one the request names, and it needs saying plainly: **over-saving is a
cost, not a safety margin.** A model that only asked "will it last?" would report
"yes, comfortably" for every plan that saved too much, and would therefore never find the
optimum. So the objective charges for the money that was never consumed.

---

## 2. The model

### 2.1 Inputs

| Group | What is specified |
| --- | --- |
| Income | Full-time salary, and the employment fraction actually worked |
| Family | Married or single, children, and per the tax schedule |
| Consumption | A **profile**, not a number: the mandatory floor and the lifestyle target come from the existing `ConsumptionTiers` in `consumption.rs`, so there is exactly one definition of "needed" in the repository |
| Accumulation | Existing BVG and Pillar 3a capital, taxable savings, AHV contribution years already accrued |
| Clock | Current age, life expectancy, and the retirement age under consideration |
| Law | The statutory parameters of §4, each with a provenance |

### 2.2 Entitlements

**Pillar 1 (AHV).** The pension scales with the contribution record up to the full number
of years and then with the early-draw reduction:

```
record   = min(1, (accrued + work_fraction · years_to_retirement) / 44)
early    = max(0, reference_age − retirement_age) · 6.8%      per AHVV Art. 56bis
AHV      = max_annual(family) · record · (1 − early)
```

Both terms matter and leaving either out understates the price of retiring early. A
60-year-old has four fewer contributing years *and* a 13.6% actuarial cut, and AHVG
Art. 40a Abs. 3 cuts that reduction by 40% for low incomes.

**Pillar 2 (BVG).** Contributions accrue at the statutory age-banded rates
(7/10/15/18% by age band) on insured salary, credited at the **minimum** interest rate:

```
insured    = clamp(salary · work_fraction − 26 460, 0, 64 260)
capital    = capital · (1 + 1.25%) + insured · rate(age) · work_fraction
annuity    = capital · (6.8% − years_early · reduction)
```

Crediting at the statutory *minimum* is deliberate. A projection that assumed a fund's
above-minimum crediting would be presenting an investment view as an entitlement.

**Pillar 3a.** Accumulates at the ceiling (CHF 7,258 for 2026) at the same minimum rate.

**The bridge.** The years between retirement and the AHV reference age have no AHV in them.
That gap is why early retirement is expensive, and it is the thing a single "annual
retirement income" figure hides.

### 2.3 The withdrawal problem, which is where the money is

Given a capital `W` to draw over `H` years, with committed base income `b_t` in year `t`
(AHV plus any annuity bought), choose withdrawals to minimise total tax.

The tax schedule is **banded**: an allowance, then bands of constant marginal rate. That
makes the problem a **linear program**. One variable per (year, band) pair:

```
minimise    Σ_t Σ_b  m_b · y[t][b]                    total tax
subject to  Σ_b y[t][b] ≤ available(t, b)  for all t   residual room in that band
            Σ_t Σ_b y[t][b] = W                        the whole capital is placed
            y ≥ 0
```

where `m_b` is band `b`'s marginal rate and `available(t,b)` is how much of band `b` year
`t` still has free once its base income `b_t` has filled the bands beneath it.

**Why residual capacity rather than a "fund at least the base" constraint.** That
constraint has a negative right-hand side, and `src/simplex.rs` refuses those — correctly,
because negating such a row to make it acceptable silently flips the meaning of its dual.
Measuring each band's *remaining* room encodes the same requirement with non-negative
numbers.

**Why no bottom-up ordering constraints.** They look necessary and are not. The bands'
rates increase, so any solution that used an expensive band while a cheaper one still had
room is strictly improvable and cannot be optimal. Adding them anyway was a real bug: once
the base income fills the cheap bands their residual capacity is zero, and an ordering
constraint then forces every higher band to zero as well — making the problem infeasible
for *any* capital at all.

### 2.4 The dual is the answer

The LP's single coupling constraint is "place the whole capital". Its **dual is the
marginal tax rate of the next franc withdrawn**, and the KKT conditions say:

> At the optimum, no year offers a cheaper next franc than the multiplier, and at least one
> year offers exactly that rate.

That is the precise form of the rule of thumb the whole exercise is about: *spread the
withdrawal until the next franc costs the same wherever it goes, and no further.* It also
gives the closed form: a capital is spread over `ceil(W / cheap_room)` years.

**The obvious statement of this is wrong, and the test suite proves it.** Asserting that
every year receiving a withdrawal sits at the same marginal rate fails, because a year can
stop exactly on a band boundary and report the band above it. The correct assertion is
complementary slackness, and that is what the test checks — after the naive version failed.

---

## 3. Three independent routes, and why three

| Route | What it is |
| --- | --- |
| **Simplex** | The tableau solver in `src/simplex.rs`, two-phase, **Bland's rule** |
| **Greedy** | Fill the cheapest band in whichever year has room, band by band |
| **KKT** | Complementary slackness, checked against the LP's dual |

They are separate implementations on purpose. Two implementations that share code cannot
check each other, and a solver that agrees only with itself proves nothing. The tests
assert that simplex and the greedy fill produce the **same tax** on the same instances.

Bland's rule is chosen over Dantzig's deliberately: slower, and it cannot cycle. A solver
that returns a wrong optimum because it looped is worse than one that takes more
iterations. The test for it is **Beale's classic cycling example**, with a known optimum.

---

## 4. The Swiss parameters, and which of them are actually law

Every parameter carries a provenance, and `provenance_table()` prints them together.

| Parameter | Value | Status |
| --- | ---: | --- |
| AHV maximum, single | CHF 32,760/yr | **Sourced** — 13 × CHF 2,520 |
| AHV maximum, couple | CHF 49,140/yr | **Sourced** — 13 × CHF 3,780 |
| AHV reference age | 65 | **Sourced** — AHVG Art. 21 |
| AHV earliest draw | 63 | **Sourced** — AHVG Art. 40 |
| AHV early reduction | 6.8%/yr, max 13.6% | **Sourced** — AHVV Art. 56bis |
| AHV full contribution years | 44 | **Sourced** — AHVG Art. 29 |
| **Umwandlungssatz** | **6.8%** | **Sourced** — BVG Art. 14 Abs. 2 |
| BVG reference age / earliest draw | 65 / 63 | **Sourced** — BVG Art. 13 |
| Coordination deduction / entry threshold | 26,460 / 22,680 | **Sourced** — BVG Art. 8, BVV 2 Art. 5 |
| BVG minimum interest | 1.25% | **Sourced** — BVV 2 Art. 12 lit. k |
| BVG savings credits | 7/10/15/18% | **Sourced** — BVG Art. 16 |
| Pillar 3a ceiling | CHF 7,258 | **Sourced** — BVV 3 Art. 7 |
| Non-employed AHV contribution | CHF 530–26,500 | **Sourced** — AHVG Art. 10 |
| **BVG early-withdrawal reduction** | *declared* | **VARIES — no statutory schedule** |
| **Capital-withdrawal tax bands** | *declared* | **VARIES — cantonal** |

### 4.1 Three figures the research corrected

**(i) The AHV annual maxima are 13 monthly payments, not 12.** AHVG Art. 34ter introduced a
thirteenth old-age pension from 2026. Using the pre-2026 annual figures would understate a
funded plan by a full month of pension per year.

**(ii) The statutory BVG early draw is completed age 63, not 58.** BVG Art. 13 Abs. 2, as
amended by AHV 21. Funds *may* permit retirement from 58 under BVV 2 Art. 1i, but that is a
plan feature and not a right. The first version of this model defaulted to 58 and so made
early retirement look more available than the statute does.

**(iii) The conversion rate stands at 6.8% because the reform was rejected.** The attempt to
lower it to 6.0% failed in the referendum of **22 September 2024** by 1,655,513 votes to
810,569 — **32.9% in favour, 67.1% against**. A model that assumed the lower rate would have
been modelling a law that does not exist.

### 4.2 The two parameters that are *not* law, and why that is the finding

**The early-withdrawal reduction of the conversion rate has no statutory schedule.** It is a
reglementary benefit each fund writes into its own regulations. Two published funds differ by
roughly a **factor of two** per year of early draw — about 4.3% a year relative in the
mandatory part of one, about 2.3% in another. There is no number to source here. A single
figure would be an invention with a citation-shaped hole where the caveat should be, which is
why the field is injectable and the table says **varies**.

**The capital-withdrawal tax depends on the canton of domicile.** The federal share is one
fifth of the ordinary tariff (DBG Art. 38), but the cantonal tariff runs from a flat **2%**
(Zurich, Thurgau) to a progressive scale reaching about **6%** (Geneva). The default bands
are a declared stand-in in that shape.

**One rule that makes the optimisation possible, and one that constrains it.** A capital
benefit is taxed **separately** from ordinary income, as a full annual tax in the year of
receipt (DBG Art. 38, StHG Art. 11 Abs. 3) — so splitting a withdrawal across years lowers
the rate. But **all capital benefits paid in the same year are aggregated**, including
second and third pillar together and across spouses; and second-pillar capital may be drawn
in **at most three steps** (BVG Art. 13a Abs. 2). Both constraints belong in a plan the model
recommends, and neither is optional.

### 4.3 The 6.8% is a floor almost nobody receives, and that matters for the plan

The statutory minimum conversion rate is **6.8%**, but the federal supervisory commission's
own aggregate — 1,257 funds, CHF 1,352 bn — puts the **average rate actually applied at
5.22%**, against 5.17% planned five years ahead. Rates *below* the legal minimum on the whole
retirement assets are lawful, because the guaranteed minimum pension is protected by the
shadow calculation: the fund must compare the resulting annuity against the statutory minimum
and pay the higher.

This is the single largest gap between a legally correct projection and a useful one. A plan
priced at 6.8% would overstate the annuity from a given capital by roughly a quarter, and the
overstatement would land precisely on the decision this model exists to inform — whether the
pot is big enough. The parameter is therefore injectable, the default is the statutory floor
because that is what the law guarantees, and **any plan built on the default should say so**.
Reading the conversion rate from the fund's own statement is worth more here than any other
single input.

**Two timing rules that bound an early-retirement plan.** Third-pillar capital may ordinarily
be drawn **five years before the reference age** — age 60 — and the person does **not** have
to stop working (BVV 3 Art. 3 Abs. 1). Second-pillar capital may be drawn in at most three
steps (BVG Art. 13a Abs. 2), and the statutory early-draw age is 63 (BVG Art. 13 Abs. 2),
with funds permitted to go to 58 (BVV 2 Art. 1i). Between 60 and 63, then, the only capital
legally available is the third pillar and taxable savings — which is exactly the window the
bridge calculation is about.

---

## 5. Risk, made explicit rather than summarised

A 10,000-path Monte Carlo over real returns and longevity, seeded so that two strategies are
compared against **the same paths**. The reported measures, per strategy:

| Measure | What it answers |
| --- | --- |
| `probability_of_shortfall` | How often the money ran out before the end of life |
| `expected_shortfall` | How bad it was when it did — the mean gap, not just the frequency |
| `mean_terminal_wealth` | **The over-saving measure.** A large number is a cost |
| `p10 / median / p90 terminal wealth` | The spread, so a single mean cannot hide a fat tail |
| `mean_lifetime_tax` | What the timing of the withdrawals cost |

**One return draw per path, not one per year.** Averaging annual draws would shrink the
variance toward zero as the horizon grew, making a long retirement look *safer* than a short
one. The horizon has to enter as longevity risk, which is what it actually is.

**The trade-off the model exists to expose.** Annuitising more of the capital removes
sequence and longevity risk from that part of the pot, and it also leaves less unconsumed
wealth. Both halves are asserted in the tests: the shortfall probability must not rise, and
the terminal wealth must not rise either. A model in which annuitising reduced risk *and*
left more money behind would not have a trade-off in it at all.

---

## 6. The normative question, which this model prices and does not answer

> *A highly educated worker was educated at public expense and earns more, so they should
> also contribute more.*

The model's answer is that this is **two claims, and only the first is a calculation**.

**The calculable part.** Under the ordinary progressive schedule, a high earner's lifetime
income tax already exceeds what the state spent on their education by a large multiple. The
test fixture computes it and the code reports `tax_multiple_of_state_cost`. So the argument
cannot be "the educated pay nothing back" — they pay back several times over. The open
question is whether that multiple is the *right* multiple.

**The part that is not a calculation.** `ContributionPrinciple` has four variants, each a
defensible distributive principle, and the model prices all four and **ranks none**:

| Principle | What it says | What the ledger shows |
| --- | --- | --- |
| `AbilityToPay` | Contribute in proportion to income — what the existing tax does | Surcharge zero; the multiple is already several times over |
| `EducationCostRecovery` | Recover the state's outlay, as a surcharge | A definite annual figure, and regressive across careers of different length |
| `BenefitReceived` | Contribute in proportion to the earnings premium | Proportional to income, so formally similar to the first |
| `Flat` | Everyone contributes the same | Its share of income **falls** as income rises |

That last row is the mechanical fact that makes the flat principle contestable, and the
model reports it rather than burying it: a flat charge is a smaller share of a high income,
which is what "regressive in effect" means. The test suite asserts it.

**A model that baked in one of these would be presenting an argument as a calculation.** So
the principle is a declared input, the report prints all four ledgers side by side, and the
choice is left where it belongs.

---

## 7. What is not established

1. **No return forecast.** The Monte Carlo uses a declared lognormal in real terms with a
   volatility the caller supplies; it is not fitted and not a market view. `economic_regimes.rs`
   holds the regime-switching model and is deliberately *not* used here, because mixing a
   richer return model into this question would make the strategy comparison depend on a
   market view rather than on the withdrawal decision.
2. **Longevity is a simple spread** around life expectancy, clipped to the horizon. Declared.
3. **The fund's own regulations are not modelled.** The conversion rate by retirement age,
   any early-withdrawal reduction, and the crediting rate all come from the fund's
   `Vorsorgereglement`. The model takes them as parameters; it cannot derive them, and §4.2
   says so.
4. **Cantonal tax detail is a declared shape**, not a canton's published tariff. The mode is
   built to take the real bands.
5. **The bridge-year AHV contributions are parameterised but not yet charged in the
   projection**, and the 40% reduction of the early-draw penalty for low incomes is stated
   rather than applied. Both are recorded here rather than left to be discovered.
6. **`terminal_wealth_weight` is a preference and has no right value.** It is what separates
   "fund the plan" from "fund the plan and no more". One means indifference between consuming
   and leaving; below one penalises over-saving. It is exposed because hiding it would make
   the value judgement silently.
7. **No claim that early retirement is optimal for anyone in particular.** The model finds the
   optimum *given* the declared consumption profile and preferences. Change the profile and
   the answer changes, which is the point rather than a defect.

---

## 8. Reproduction

```sh
cargo test --test early_retirement     # the age matrix and the contract, 24 integration tests
cargo test --lib simplex               # the solver, including Beale's cycling example
cargo test --lib early_retirement      # the internals: the allocation, the ledger, the shadow tests

# The plan, the allocation, the risk sweep and the four ledgers. Pass your own fund's
# figures: --conversion-rate is the one that matters most.
cargo run -- early-retirement --salary 120000 --age 45 --married \
  --conversion-rate 0.052 --early-reduction 0.002 --annuity-share 1.0 \
  --retirement-age 63 --paths 10000 --export out/early-retirement

python tools/plot_early_retirement.py out/early-retirement \
  --out figures/early-retirement-1-risk.png
```

The mode prints, in order: the parameters and their provenance; the entitlements with the
bridge years called out; the allocation the LP chooses, checked against the water-filling
solution; the risk sweep across retirement ages; the smallest extra capital that funds the
consumption to a declared confidence; and the four education ledgers together. The figure
refuses to draw unless its three consistency checks pass, one of which is the claim that
retiring later cannot raise the shortfall probability.

## 9. What the figures show, and one inconsistency found while drawing them

`figures/early-retirement-1-risk.png` plots the shortfall probability, the median
unconsumed wealth at death, and the total capital, all against the retirement age.

The first version of that figure showed unconsumed wealth in the **millions** while the
terminal table showed tens of thousands. The cause was a real defect rather than a plotting
error: the table printed the **median** and the CSV exported the **mean**, under one name.
The mean is dominated by the lucky paths, where a fixed real withdrawal compounds for
decades and leaves a fortune, so it is a statement about the right tail of the return draw
rather than about a retirement. The export was changed to the median, and the check that
would have caught it is now in the plotting script's own consistency tests.

