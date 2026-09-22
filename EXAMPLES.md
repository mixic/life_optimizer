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

These are alternative spending profiles, not moral judgments. They change what a
schedule *leaves you* — the target basket, the saving capacity, and the
consumption term of the utility score — but they cannot change whether you can
afford it. Feasibility is tested against the mandatory floor of rent, essentials
and debt, and the profile multiplier is applied only to the discretionary tier,
which is deliberately treated as trimmable. So `extreme-saving` and `luxury`
return the same work percentage and the same affordability verdict; what moves is
the basket and the saving capacity beside them. Only the inelastic items — rent
and contractual debt — can make a schedule unaffordable.

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

### Refining the AI assumption

The plain `--ai-productivity-gain` assumes one number, usable as delivered, for
every hour you compress into. Five optional flags let you attack that
assumption instead of accepting it. All of them are off by default, and with the
defaults the constraint behaves exactly as in the examples above.

| Flag | Meaning | Default |
|---|---|---|
| `--ai-productivity-gain-high` | Optimistic end of a productivity *range*. `--ai-productivity-gain` becomes the pessimistic end | unset (= pessimistic) |
| `--ai-quality-retention` | Share of the AI gain that survives verification and rework | `1.0` |
| `--compression-quality-sensitivity` | Output per hour lost per unit of pace above your sustainable rate | `0.0` |
| `--replacement-risk` | Replacement probability per unit of relative goal shortfall | `0.0` |
| `--enforcement` | `strict` refuses a schedule that does not deliver; `risk-weighted` offers it and prices the risk | `strict` |
| `--evaluation-period-years` | Years at full time before the reduction becomes credible | `0.0` |
| `--monthly-debt` | Debt repayment or other unavoidable contractual outflow | `0.0` |

Feasibility is always judged at the **pessimistic** end of the range, so the
report tells you which kind of claim the recommendation is:

- `ROBUST across the AI range` — delivers even if AI only returns the low gain;
- `ONLY IF AI DELIVERS at the optimistic end` — a bet on the tool: the goals hold
  only if AI lands at the top of the range;
- `UNREACHABLE at any point in the AI range` — even full time falls short.

```bash
./life-optimizer optimize \
  --salary 150000 --age 40 \
  --required-output-index 1.0 \
  --ai-productivity-gain 0.5 --ai-productivity-gain-high 0.8 \
  --compression-quality-sensitivity 0.5 \
  --evaluation-period-years 2 \
  --monthly-debt 300
```

```text
Employer Achievement Capacity:
  AI gain range:   50% pessimistic … 80% optimistic
  Capacity (A):    1.20
  Quality factor:  0.88   usable output per unit of capacity
  Delivered:       1.05
  Required (G):    1.00
  Robustness:      ROBUST across the AI range
  Status:          MEETS REQUIRED OUTPUT ✓ (margin +0.05)
  Average workload:81.6%   over the years to retirement, including
                     the evaluation period worked at full time
```

Read it as: the goals still hold up if AI returns only half the optimistic
gain, so 80% is robust rather than a bet — but compressing the same portfolio
into the reduced week costs about 12% of the output per hour, which is why the
margin is only 0.05. The average workload is 81.6% rather than the contractual
80% because the first two years are served at full time.

### Hidden work and replacement risk

`--replacement-risk` prices a missed goal as a probability of losing the job.
On its own it does not change what strict mode will offer — strict simply
declines to recommend a schedule that does not deliver:

```bash
./life-optimizer optimize --salary 150000 --age 40 \
  --required-output-index 1.0 --replacement-risk 0.5
```

With `--enforcement risk-weighted`, the same shortfall is offered instead, and
the workload it hides is printed rather than absorbed:

```bash
./life-optimizer optimize --salary 150000 --age 40 \
  --required-output-index 1.0 --replacement-risk 0.5 \
  --enforcement risk-weighted
```

```text
Employer Achievement Capacity:
  Capacity (A):    0.50
  Required (G):    1.00
  Robustness:      UNREACHABLE at any point in the AI range
  Status:          OFFERED BUT NOT DELIVERED (50% short)
  Hidden work:     21.0 h/week if AI lands at the pessimistic end
    (work the contract does not mention. At that outcome the load is a
    full-time one whatever the hours above say, and it is not credited as leisure.)
  Replacement risk:  25%   from the goal shortfall
```

A 50% work contract here would really be a full-time job with a half-time
salary. Those hours are not added to the free hours above either: working the
cover and missing the goal are two answers to the same shortfall, not two costs
to add up, and the model prices the second one. A large enough
`--replacement-risk` makes the optimizer return to 100% on its own — the risk is
a real trade-off, not a decoration. Note that `--enforcement risk-weighted` will
not run without a `--replacement-risk`: the mode exists to price the shortfall,
and with nothing declared it would silently stop the required output from
constraining the search at all.

The reasoning behind these knobs, and an explicit list of what is *not*
modelled, is in `CRITICS_CURRENT_WORK.md` §7.

---

## Interactive Exploration

For beginners, use interactive mode:

```bash
./life-optimizer interactive
```

This will guide you through all parameters with helpful prompts.
