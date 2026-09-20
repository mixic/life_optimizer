# Example Scenarios

## Scenario 1: Young Software Engineer in Zürich

```bash
./life-optimizer optimize \
  --salary 95000 \
  --age 28 \
  --married false \
  --children 0 \
  --profile career
```

**Expected Result:** 100% work optimal
- Young, no family obligations
- Build career and savings foundation
- High stress tolerance

---

## Scenario 2: Family with Young Children

```bash
./life-optimizer optimize \
  --salary 110000 \
  --age 38 \
  --married true \
  --children 2 \
  --youngest-child-age 4 \
  --profile family
```

**Expected Result:** 80% work optimal
- Time with young children is invaluable
- 4-day work week provides balance
- Still meets financial requirements

---

## Scenario 3: Dual Income Household

**Partner A:**
```bash
./life-optimizer optimize \
  --salary 90000 \
  --age 35 \
  --married true \
  --children 2 \
  --profile balanced
```

**Partner B:**
```bash
./life-optimizer optimize \
  --salary 80000 \
  --age 35 \
  --married true \
  --children 2 \
  --profile balanced
```

**Strategy:** Both at 80% = 136k combined net
- Both get 3-day weekends
- Childcare coverage easier
- Career options remain for both

---

## Scenario 4: Mid-Career Professional

```bash
./life-optimizer optimize \
  --salary 130000 \
  --age 48 \
  --married true \
  --children 2 \
  --youngest-child-age 15 \
  --profile balanced
```

**Expected Result:** 90% work optimal
- Teenagers need less active parenting
- Peak earning years for pension boost
- Still maintain work-life balance

---

## Scenario 5: Pre-Retirement

```bash
./life-optimizer optimize \
  --salary 120000 \
  --age 60 \
  --married true \
  --children 0 \
  --profile balanced
```

**Expected Result:** 60-70% work optimal
- Health becomes priority
- Pension already secured
- Enjoy life before full retirement
- Prepare for transition

---

## Scenario 6: Single Parent

```bash
./life-optimizer optimize \
  --salary 85000 \
  --age 42 \
  --married false \
  --children 1 \
  --youngest-child-age 8 \
  --profile family
```

**Challenge:** Must balance high childcare needs with income requirements
**Strategy Considerations:**
- May need 100% work due to single income
- Extended family support crucial
- Government childcare subsidies important

---

## Scenario 7: High Earner

```bash
./life-optimizer optimize \
  --salary 180000 \
  --age 45 \
  --married true \
  --children 2 \
  --profile balanced
```

**Expected Result:** 70-80% work optimal
- Requirements easily met at reduced hours
- High marginal tax rate (30%+)
- Diminishing returns to additional income
- Time becomes more valuable than money

---

## Lifetime Planning Examples

### Example A: Career Trajectory

```bash
./life-optimizer lifetime \
  --salary 80000 \
  --age 25 \
  --married false \
  --children 0 \
  --retirement-age 65
```

**Expected Strategy:**
- Ages 25-30: 100% (build foundation)
- Ages 31-35: 100% (establish career)
- Ages 36-45: 80% (anticipated family)
- Ages 46-55: 90% (peak earning)
- Ages 56-64: 70% (wind down)

### Example B: Parent Trajectory

```bash
./life-optimizer lifetime \
  --salary 100000 \
  --age 35 \
  --married true \
  --children 2 \
  --retirement-age 65
```

**Expected Strategy:**
- Ages 35-40: 80% (young children)
- Ages 41-50: 80% (school age)
- Ages 51-55: 90% (boost pension)
- Ages 56-64: 70% (health priority)

---

## Comparison Scenarios

### Tax Efficiency Comparison

Compare how tax rates change with different work percentages:

```bash
./life-optimizer compare \
  --salary 100000 \
  --age 40 \
  --married true \
  --children 2 \
  --percentages "0.5,0.6,0.7,0.8,0.9,1.0"
```

**Key Insight:** Watch how effective tax rate increases with income

### Life Stage Comparison

Run same salary at different ages:

```bash
# Age 30
./life-optimizer optimize --salary 100000 --age 30 --married false --children 0

# Age 40
./life-optimizer optimize --salary 100000 --age 40 --married true --children 2

# Age 60
./life-optimizer optimize --salary 100000 --age 60 --married true --children 0
```

**Key Insight:** Optimal work percentage changes dramatically with life stage

---

## Special Cases

### Case 1: Insufficient Income

```bash
./life-optimizer optimize \
  --salary 60000 \
  --age 35 \
  --married true \
  --children 2
```

**Result:** No feasible solution at reduced hours
**Recommendation:** 
- 100% work necessary
- Dual income essential
- Move to lower cost area
- Reduce requirements

### Case 2: Financial Independence

```bash
./life-optimizer optimize \
  --salary 150000 \
  --age 50 \
  --married true \
  --children 0
```

**Result:** 50-60% work still meets all needs
**Insight:** Beyond certain income, additional work has very low utility

---

## Canton Differences

### Zürich (High Tax)
```bash
./life-optimizer optimize --salary 100000 --canton ZH
```

### Zug (Low Tax)
```bash
./life-optimizer optimize --salary 100000 --canton ZG
```

**Difference:** Zug residents can work less for same net income due to lower taxes

---

## Conversion Rate (Umwandlungssatz) Examples

The conversion rate is how your accumulated BVG capital becomes a lifelong
pension. The statutory minimum is 6.8%, but **many Swiss pension funds apply
5.0–5.5%**, which is a large difference in monthly income. Every `optimize` and
`pension` run now prints the range across scenarios, so you can see how much of
your pension depends on a rate you should verify with your own fund.

### Example: seeing the range

```bash
./life-optimizer pension \
  --salary 100000 \
  --age 40 \
  --work-pct 0.8
```

Output includes:

```
    MONTHLY PENSION BY CONVERSION RATE (Umwandlungssatz)
  Based on projected BVG capital of CHF 267585 at retirement in 2051

  Conversion rate scenario           Rate      Monthly BVG
  ────────────────────────────────────────────────────────
  Statutory minimum (BVG)           6.80%         CHF 1516
  Typical Swiss fund                5.50%         CHF 1226
  Projected for 2051                5.83%         CHF 1300
  ────────────────────────────────────────────────────────

  Realistic range: CHF 1226 – 1516 per month
```

The same capital supports CHF 1,516/month at the statutory rate but only
CHF 1,226/month at the typical fund rate — a CHF 290/month gap that compound
over a 25-year retirement is roughly CHF 87,000.

### Example: using your fund's actual rate

Ask your pension fund for its current Umwandlungssatz, then pass it directly:

```bash
./life-optimizer pension \
  --salary 100000 \
  --age 40 \
  --work-pct 0.8 \
  --conversion-rate 0.048
```

This makes your fund's rate the headline figure and adds it to the comparison:

```
  Your pension fund                 4.80%         CHF 1070
  Your rate is below the statutory minimum by CHF 446/month
```

Use this when deciding whether a reduced work percentage is affordable — a low
conversion rate can make 80% work materially riskier than the statutory-rate
projection suggests.

### Why the projection matters more for younger people

The projection assumes the rate keeps declining ~0.036 percentage points per
year (rising life expectancy, low rates, demographics), floored at 5.0%:

| Retirement year | Projected rate |
|---|---|
| 2024 | 6.80% |
| 2030 | 6.58% |
| 2040 | 6.22% |
| 2050 | 5.86% |
| 2060 | 5.50% |
| 2074+ | 5.00% (floor binds) |

If you are 40 today, the planning case is roughly 5.8% — closer to the typical
fund rate than to the statutory one. Planning on 6.8% would overstate your
pension.

### Example: using a named pension fund profile

If you know which fund you are in but not its exact rate:

```bash
./life-optimizer pension --salary 100000 --age 40 --work-pct 0.8 --pension-fund publica
```

```
Using pension fund profile: Publica (federal employee fund) at 5.15%
    Rate applied to the mandatory portion in recent years; verify annually.
  Verify this rate with your fund — rates change annually.
```

Available profiles: `publica`, `bvk`, `statutory`, `typical`.

**Treat these as reference points only.** Conversion rates change annually and
often differ between the mandatory and super-mandatory portions of the same
fund. Prefer `--conversion-rate` with the figure from your own fund statement.
`--conversion-rate` takes precedence if you pass both.

### Example: how much does the rate uncertainty cost you?

Every run also prints a distribution over the conversion rate itself, rather
than a single number:

```
CONVERSION-RATE UNCERTAINTY (stochastic)
  Rate drawn ~ N(5.83%, 1.00pp), truncated to [5.00%, 6.80%]

  Monthly BVG pension          CHF
  P10 (bad luck)              2136
  Median                      2492
  P90 (good luck)             2904
  CVaR (worst 10%)            2136

  Probability of falling below CHF 3642/month: 100%
```

Two statistics worth understanding:

- **P10** is the pension you would exceed in 90% of cases.
- **CVaR (worst 10%)** is the *average* pension given that you land in the worst
  decile. Percentiles tell you where the boundary is; CVaR tells you how bad it
  is on the wrong side of it.

The uncertainty band is an explicit assumption (1 percentage point), not a
fitted dispersion — it is centred on the projection so the model does not claim
to know which direction the error runs. It answers "what is at stake if I do not
know my fund's rate", not "what rate will my fund apply".

---

## Consumption Profile and Sparing Examples

The optimizer separates your **mandatory floor** (costs you cannot avoid) from
your **lifestyle basket** (spending you choose). Feasibility is tested against
the floor, so a household is never told it "cannot afford" reduced hours merely
because it would have to trim discretionary spending.

### Example: seeing where the squeeze lands

```bash
./life-optimizer optimize --salary 120000 --age 38 --married true --children 2
```

Output includes:

```
Consumption by Elasticity Tier:
  Inelastic (non-reducible)                CHF 7070
  Quasi-inelastic (locked in)                 CHF 0
  Elastic (sparing-eligible)                CHF 673
  Committed outflows (savings, vacation)   CHF 1350
  ─────────────────────────────────────────────────
  Mandatory floor                          CHF 7070
  Full lifestyle basket                    CHF 9093
```

Rent, food, transport, insurance and Kita cannot be reduced by being frugal.
Only the elastic tier responds to sparing — which is the point of reporting the
split rather than one aggregate number.

Note that the inelastic tier is life-stage adjusted: a `NewParent` household
carries Kita costs and a higher housing figure, so the same salary yields a
larger floor than a childless household would.

### Example: declaring a sparing strategy

```bash
./life-optimizer optimize \
  --salary 120000 --age 38 --married true --children 2 \
  --sparing-ratio 0.6 \
  --utilization-discipline 0.8
```

`--sparing-ratio` is the fraction of elastic spending you source second-hand,
borrowed or shared. `--utilization-discipline` is how strictly you filter
purchases before buying — undiscounted low-utilisation purchases cost more per
unit of real value than a full-price item you actually use.

### Example: lifestyle profiles

```bash
./life-optimizer optimize --salary 120000 --age 38 --consumption-profile extreme-saving
./life-optimizer optimize --salary 120000 --age 38 --consumption-profile luxury
```

| Profile | Discretionary multiplier |
|---|---|
| `extreme-saving` | 0.50 |
| `moderate` | 0.80 |
| `normal` (default) | 1.00 |
| `luxury` | 1.75 |

These are alternative spending profiles, not moral judgments. The same 80% work
schedule can be feasible under one and infeasible under another.

### Example: locked-in spending

```bash
./life-optimizer optimize --salary 120000 --age 38 --quasi-inelastic-share 0.4
```

Use this when part of your nominally discretionary spending is actually locked
in by switching costs — a phone ecosystem, a subscription you cannot practically
leave, a car you need for work. Locked-in spending raises your mandatory floor,
because you cannot flex it.

---

## Employer Achievement Constraint Examples

Your work percentage is not purely your own choice: it is bounded by what your
employer requires you to deliver. Passing `--required-output-index` engages that
constraint.

### Example: can AI justify 80%?

You currently deliver an output index of 1.0 at 100% work, and want to know
whether 80% is credible:

```bash
./life-optimizer optimize --salary 150000 --age 40 --required-output-index 1.0
```

Without AI, the output shows 100% is forced — 80% capacity (0.80) falls short of
the required 1.0.

Now add an assumed AI productivity gain:

```bash
./life-optimizer optimize \
  --salary 150000 --age 40 \
  --required-output-index 1.0 \
  --ai-productivity-gain 0.25
```

A +25% gain makes 80% work deliver exactly the same output 100% used to
(0.80 × 1.25 = 1.0), so reduced hours become feasible on both counts.

### Example: when the goals are simply impossible

```bash
./life-optimizer optimize --salary 200000 --age 40 --required-output-index 2.0
```

```
⚠  NO OPTION MEETS THE REQUIRED OUTPUT
  Status:          MEETS ALL REQUIREMENTS ✓
Employer Achievement Capacity:
  Capacity (A):    1.00
  Required (G):    2.00
  Status:          BELOW REQUIRED OUTPUT ✗ (shortfall 1.00)
```

The tool distinguishes the two failure modes explicitly, because the remedies
are completely different:

- **"no affordable option"** → a budget problem: lower costs, second income, different canton
- **"no option meets the required output"** → a workload problem: renegotiate the portfolio, or model AI leverage

### Modelling employer capture

The optimistic case above assumes your goals stay fixed. If your employer raises
targets in step with your productivity, the constraint binds again — model that
by raising the required index alongside the AI gain:

```bash
./life-optimizer optimize \
  --salary 150000 --age 40 \
  --required-output-index 1.25 \
  --ai-productivity-gain 0.25
```

Now 100% is forced again. AI only buys you time if the gains are not fully
captured as higher output expectations.

---

## Interactive Exploration

For beginners, use interactive mode:

```bash
./life-optimizer interactive
```

This will guide you through all parameters with helpful prompts.
