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

## Interactive Exploration

For beginners, use interactive mode:

```bash
./life-optimizer interactive
```

This will guide you through all parameters with helpful prompts.
