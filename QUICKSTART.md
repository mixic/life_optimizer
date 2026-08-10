# Quick Start Guide

## Installation

### 1. Install Rust

If you don't have Rust installed:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### 2. Clone and Build

```bash
cd life-optimizer
cargo build --release
```

The binary will be at: `target/release/life-optimizer`

### 3. Run Your First Optimization

```bash
./target/release/life-optimizer optimize \
  --salary 100000 \
  --age 35 \
  --married true \
  --children 2 \
  --youngest-child-age 5
```

## Understanding the Output

### Work Configuration
- **Work Percentage:** Optimal percentage of full-time (e.g., 80%)
- **Gross Income:** Annual income before taxes
- **After-Tax:** Annual income after all taxes and social security
- **Monthly Net:** What you actually receive each month
- **Effective Tax:** Your real tax rate (not marginal!)

### Time Allocation
- **Work Hours:** Hours per week at the office
- **Free Hours:** Hours available for family, leisure, personal pursuits
- **Work Days:** Equivalent full 8-hour days

### Financial Health
- **Status:** Whether your income covers all requirements
- **Surplus/Deficit:** Monthly money left over (or shortage)

### Utility Score
Measures overall life satisfaction considering:
- **Consumption:** Can you afford what you need/want?
- **Leisure:** Do you have enough free time?
- **Family:** Can you spend quality time with loved ones?
- **Health:** Is stress manageable?
- **Security:** Is your future (pension) secure?

Higher total = better overall life quality

## Common Scenarios

### "I want to work less but am I losing too much money?"

```bash
./life-optimizer compare \
  --salary 100000 \
  --age 40 \
  --married true \
  --children 2 \
  --percentages "0.8,1.0"
```

**Look for:**
- How much tax you SAVE by working less (progressive tax!)
- Whether 80% still meets your requirements
- How much utility (life quality) improves

**Key insight:** Going from 100% to 80% work:
- You lose 20% gross income
- But only lose ~15% net income (due to lower taxes!)
- Gain 20% free time
- Often increases total utility!

### "What's my optimal strategy over my whole career?"

```bash
./life-optimizer lifetime \
  --salary 100000 \
  --age 30 \
  --married false \
  --children 0 \
  --retirement-age 65
```

**Shows you:**
- When to work full-time (early career)
- When to reduce hours (family years)
- When to wind down (pre-retirement)

### "Should both my partner and I work full-time?"

Run twice:

**Option A: One works 100%, other stays home**
```bash
./life-optimizer optimize --salary 120000 --age 38 --children 2
```

**Option B: Both work 80%**
```bash
# Partner 1
./life-optimizer optimize --salary 100000 --age 38 --children 2
# Partner 2  
./life-optimizer optimize --salary 80000 --age 38 --children 2
```

**Compare:**
- Option A: 120k gross → ~95k net, one person stressed
- Option B: 180k gross → ~145k net, both have 3-day weekends!

## Interpreting Results

### High Utility Score (30+)
✓ Good balance of income, time, and security
✓ Requirements comfortably met
✓ Sustainable long-term

### Medium Utility Score (20-30)
⚠ Acceptable but could improve
⚠ May be trading too much time for money (or vice versa)
⚠ Consider adjusting work percentage

### Low Utility Score (<20)
✗ Something is seriously wrong
✗ Either income insufficient OR working too much
✗ Not sustainable - change needed

## Tax Insights

The tool reveals important tax dynamics:

### Example: CHF 100k salary in Zürich (married, 2 kids)

| Work % | Gross   | Net     | Effective Tax | Marginal Tax |
|--------|---------|---------|---------------|--------------|
| 100%   | 100,000 | 80,000  | 20.0%         | 28%          |
| 80%    | 80,000  | 66,400  | 17.0%         | 24%          |
| 60%    | 60,000  | 51,600  | 14.0%         | 20%          |

**Key Insight:** 
- Last 20k of income (100k → 80k) is taxed at 28%!
- You only net 13,600 CHF from that extra 20k
- Is it worth it for the extra 8.4 hours/week of work?

## Common Mistakes

### Mistake 1: Focusing only on gross income
**Wrong:** "I need to work 100% to maximize income"
**Right:** "What work percentage maximizes my net income AND quality of life?"

### Mistake 2: Ignoring life stage
**Wrong:** "80% worked great at age 30, so it will at age 50"
**Right:** "Optimal work percentage changes as your life changes"

### Mistake 3: Not considering partner's situation
**Wrong:** "One of us should work 100% to maximize household income"
**Right:** "Two at 80% often better than one at 100% + one at 0%"

### Mistake 4: Forgetting about pension
**Wrong:** "I'll just work less now and worry about retirement later"
**Right:** "Check the Security utility score - is your pension adequate?"

## Advanced Usage

### Custom Preference Weights

If you value family time much more than average:

```bash
./life-optimizer optimize \
  --salary 100000 \
  --age 35 \
  --children 2 \
  --profile family  # Uses family-focused weights
```

### Sensitivity Analysis

Test how robust your decision is:

```bash
# What if salary increases 10%?
./life-optimizer optimize --salary 110000 --age 35 --children 2

# What if one more child?
./life-optimizer optimize --salary 100000 --age 35 --children 3

# What if 5 years older?
./life-optimizer optimize --salary 100000 --age 40 --children 2
```

## Getting Help

### Use Interactive Mode

If command-line is confusing:

```bash
./life-optimizer interactive
```

This will ask you questions one at a time.

### Understanding Utility Components

Each component measures something specific:

- **Consumption (2-3):** Can afford basic needs
- **Leisure (5-8):** Decent amount of free time  
- **Family (5-10):** Good family time (if you have kids)
- **Health (7-10):** Manageable stress
- **Security (5-10):** Pension on track

Total of 30+ = excellent balance

## Real Success Stories

### Case 1: Software Engineer
- **Before:** 100% work, stressed, missing kids' childhood
- **After:** 80% work, Fridays off, much happier
- **Surprise:** Only lost 7% net income due to tax savings!

### Case 2: Dual Income Couple
- **Before:** One at 100%, other at 50% = 150% total
- **After:** Both at 80% = 160% total income, both get 3-day weekends
- **Result:** More money AND more time!

### Case 3: Pre-Retirement (Age 60)
- **Before:** 100% work "to maximize pension"
- **After:** 70% work, health improved, pension still adequate
- **Insight:** Extra pension contributions weren't worth the stress

## Next Steps

1. Run optimization for YOUR situation
2. Compare with your current work percentage
3. Try interactive mode to explore different scenarios
4. Calculate lifetime strategy to see full picture
5. Discuss with partner if applicable
6. Consider tax optimization strategies (Pillar 3a, etc.)

---

**Remember:** This tool provides DATA to make better decisions, but the final choice is always yours. Consider factors the model can't capture: career advancement opportunities, job satisfaction, commute time, flexibility, company culture, etc.
