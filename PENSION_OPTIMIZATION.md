# Pension Sustainability Analysis

## New Feature: Live Comfortably Until Age 90

The optimizer now checks if your pension will be adequate to maintain your lifestyle through retirement!

## How It Works

The calculator projects your pension based on:

### Pillar 1 (AHV - State Pension)
- Maximum: CHF 29,400/year (single) or CHF 44,100/year (couple)
- Calculation: ~30% of average working income (capped at maximum)

### Pillar 2 (BVG - Occupational Pension)  
- Contributions: ~8.3% of salary per year
- Growth: 2% annual return (conservative estimate)
- Conversion: 6.8% conversion rate at age 65
- Total capital accumulated over working years

### Retirement Needs
- Estimated as 75% of your current after-tax income
- Adjusted for typical retirement expenses (less work costs, no childcare)

## Usage

### Basic Example:

```powershell
.\life-optimizer.exe optimize `
  --salary 100000 `
  --age 35 `
   `
  --retirement-age 65 `
  --life-expectancy 90
```

### Custom Retirement Planning:

```powershell
# Early retirement at 60, live until 95
.\life-optimizer.exe optimize `
  --salary 100000 `
  --age 35 `
   `
  --retirement-age 60 `
  --life-expectancy 95

# Standard retirement, conservative planning
.\life-optimizer.exe optimize `
  --salary 100000 `
  --age 35 `
   `
  --retirement-age 65 `
  --life-expectancy 90
```

## Output Example

```
PENSION SUSTAINABILITY ANALYSIS
============================================================

Current Work Scenario:
  Work Percentage:     80%
  Working Income:      CHF 85,155/year
  Years Until Retire:  30 years

Projected Pension (at age 65):
  AHV (Pillar 1):      CHF 2,129/month
  BVG (Pillar 2):      CHF 3,245/month
  Total Pension:       CHF 5,374/month

Retirement Needs:
  Estimated Monthly:   CHF 4,586
  Years in Retirement: 25 years (age 65-90)

Sustainability Check:
  Status:              PENSION ADEQUATE ✓
  Coverage:            117% of needs
  Monthly Surplus:     CHF +788
```

## Scenarios

### Adequate Pension (Good!)

```
Status:              PENSION ADEQUATE ✓
Coverage:            117% of needs
Monthly Surplus:     CHF +788
```

**Meaning:** Your projected pension covers your estimated retirement needs. You can work the desired percentage!

### Insufficient Pension (Warning!)

```
Status:              PENSION INSUFFICIENT 
Coverage:            73% of needs
Monthly Shortfall:   CHF -1,240
Total Shortfall:     CHF -372,000 over 25 years

Solutions:
  • Increase work percentage to boost BVG contributions
  • Contribute to Pillar 3a (CHF 7,056/year tax deductible)
  • Build private savings/investments  
  • Consider working past age 65
  • Working 95% would provide adequate pension
```

**Meaning:** Your current work percentage won't generate enough pension. The calculator shows you what % is needed.

## Key Factors

### 1. Work Percentage Impact

| Work % | Annual Income | BVG Capital @ 65 | Monthly Pension |
|--------|---------------|------------------|-----------------|
| 60%    | CHF 63,866    | CHF 273,000      | CHF 1,856       |
| 80%    | CHF 85,155    | CHF 364,000      | CHF 2,475       |
| 100%   | CHF 100,000   | CHF 455,000      | CHF 3,093       |

**Lower work % = Lower pension!**

### 2. Years Until Retirement

Starting at age:
- **25:** 40 years of contributions → Excellent pension
- **35:** 30 years of contributions → Good pension
- **45:** 20 years of contributions → Reduced pension
- **55:** 10 years of contributions → Significantly reduced pension

**The earlier you start, the better!**

### 3. Life Expectancy

Living until:
- **85 years:** 20 years in retirement → More affordable
- **90 years:** 25 years in retirement → Standard planning
- **95 years:** 30 years in retirement → Conservative planning

**Longer life = more savings needed**

## Understanding the Numbers

### Pillar 1 (AHV)
- **Flat benefit** based on contribution years
- Most people get ~CHF 2,000-2,500/month
- Maximum: CHF 2,450 (single) or CHF 3,675 (couple)
- Indexed to inflation

### Pillar 2 (BVG)
- **Depends directly on your income**
- Contributions accumulate with interest
- Converted to pension at retirement
- Higher income = higher pension

### Example Calculation:

```
Age 35, retire 65, CHF 85,155/year @ 80% work:

BVG annual contribution: CHF 85,155 × 8.3% = CHF 7,068
Over 30 years @ 2% growth: CHF 7,068 × 49.0 = CHF 346,332
Pension @ 6.8% conversion: CHF 346,332 × 6.8% = CHF 23,550/year
Monthly BVG pension: CHF 1,963/month

Plus AHV: ~CHF 2,129/month
Total: CHF 4,092/month
```

## Optimization Strategy

### Scenario 1: Pension is Adequate
**You have flexibility!**
- Can reduce work % for better work-life balance
- Pension will still cover retirement needs
- Focus on maximizing current quality of life

### Scenario 2: Pension is Borderline
**Balance is key!**
- Working 80-90% might be optimal
- Get better life quality now
- While maintaining adequate pension
- Consider Pillar 3a contributions

### Scenario 3: Pension is Insufficient
**Need to boost savings!**
- May need to work 90-100%
- OR add Pillar 3a (CHF 7,056/year)
- OR build private investment portfolio
- OR plan to work a few extra years

## Pillar 3a Boost

The calculator doesn't include Pillar 3a, but you can add it:

**Maximum contribution:** CHF 7,056/year
**Tax benefit:** Reduces taxable income (saves 13-30% in taxes)
**At retirement:** Additional capital

**Example boost:**
```
CHF 7,056/year × 30 years @ 3% return = CHF 340,000
As annuity @ 4%: +CHF 1,134/month extra
```

This can make a HUGE difference!

## Conservative vs Optimistic

The calculator uses **conservative assumptions**:

### Conservative (Default):
- 2% BVG return
- 6.8% conversion rate
- 75% of working income needed

### Optimistic Alternative:
- 4% BVG return (better investment)
- 75% of working income needed
- Additional Pillar 3a savings
- Part-time work in early retirement

Your actual situation may be better!

## Recommendations by Age

### Age 25-35: Build Foundation
- Work 90-100% to maximize pension contributions
- Start Pillar 3a early (compound interest!)
- High risk tolerance for investments

### Age 35-50: Balance Period
- Can reduce to 80% if pension projection good
- Keep contributing to Pillar 3a
- Reassess every 5 years

### Age 50-65: Final Push
- Check if on track for adequate pension
- May need to boost to 90-100% final years
- Reduce investment risk
- Plan transition to retirement

## Quick Check

**Am I on track?**

Run the optimizer and check the pension section:
- Green "ADEQUATE" → You're good!
- Yellow "INSUFFICIENT" → Need to adjust

**Rule of thumb:**
Total pension should be ≥75% of current net income

**Example:**
- Current net: CHF 6,000/month
- Needed pension: CHF 4,500/month
- AHV: ~CHF 2,200/month
- BVG needed: ≥CHF 2,300/month

## Windows Usage

```powershell
# Standard check (default: retire 65, live to 90)
.\life-optimizer.exe optimize --salary 100000 --age 35 

# Early retirement planning
.\life-optimizer.exe optimize --salary 100000 --age 35 --retirement-age 60 --life-expectancy 90

# Conservative long-life planning  
.\life-optimizer.exe optimize --salary 100000 --age 35 --retirement-age 65 --life-expectancy 95
```

---

## Summary

**Pension check included automatically**
**Shows if your work % is sustainable long-term**
**Calculates exact % needed for adequate pension**
**Plans until age 90 (or custom)**

The optimizer now ensures you can **live well both now AND in retirement!** 
