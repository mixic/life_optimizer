# Economic Regime-Switching & Recession Stress Testing

## What's New

Two additional layers of realism on top of the base Monte Carlo simulation:

1. **Regime-switching model** — instead of assuming returns are drawn independently
   each year from one fixed distribution, this models the economy moving between
   **Boom / Normal / Recession / Stagflation** states using a Markov chain. Bad years
   cluster together (as they do in reality — a recession rarely lasts exactly one year).

2. **Sequence-of-returns stress test** — deliberately forces a recession or
   stagflation shock to hit in the 2 years before and 1 year after your retirement
   date. This is historically the single most damaging risk to a pension: the same
   average return over a career is far less dangerous than a crash right when you
   stop contributing and start withdrawing.

3. **Deferred retirement to age 70** — contribution rates and the BVG conversion
   rate now scale correctly for working past 65 (up to 70), including the higher
   conversion rate you earn for deferring retirement.

## The Four Economic Regimes

| Regime | Real Return | Volatility | Inflation | Long-run Frequency |
|---|---|---|---|---|
| Boom | +8.0% | 12% | 1.0% | ~19% |
| Normal | +3.2% | 8% | 1.5% | ~62% |
| Recession | -8.0% | 18% | 0.5% | ~14% |
| Stagflation | -3.0% | 14% | 5.0% | ~5% |

The transition matrix is calibrated loosely to post-WWII developed-market business
cycles: recessions occur roughly once every 8-10 years and last 1-2 years;
stagflation (1970s-style — poor growth *and* high inflation) is rarer.

## Usage

```powershell
# Full optimize + regime-switching + stress test, retiring at 70
.\life-optimizer.exe optimize --salary 100000 --age 35 --married true --retirement-age 70 --life-expectancy 90 --pillar3a 7056

# Pension-only deep dive, working to 70
.\life-optimizer.exe pension --salary 100000 --age 35 --work-pct 1.0 --retirement-age 70 --life-expectancy 90 --pillar3a 7056
```

Both commands now print three additional sections after the standard Monte Carlo
summary:

### 1. Regime-Switching Model
Shows median/P10/P90 capital, pension, and quality-of-life score under realistic
clustered economic cycles — generally a bit more pessimistic than the static
"base case" scenario because it captures the damage multi-year recessions do.

### 2. Stress Test: Recession at Retirement
Shows what happens in the worst-case timing scenario — a downturn hitting exactly
when you stop working. Compares the resulting pension against the regime-switching
median so you can see the percentage drop, and reports how often the outcome is
still adequate.

### 3. Mitigation suggestions
Practical steps to reduce sequence-of-returns risk: de-risking the portfolio a few
years before retirement, keeping a cash buffer, flexible retirement timing, and
staggered Pillar 3a withdrawals.

## Why This Matters More When Working to 70

A longer accumulation period (e.g. age 35 → 70, 35 years) means more compounding
years, but it also means more *exposure* to the possibility that a recession lands
right before you retire. The regime-switching model and stress test specifically
quantify that risk rather than averaging it away, which a single fixed-return
assumption cannot do.

Deferred retirement also raises your BVG conversion rate (roughly +0.2%/year from
65 to 70 in this model), which partially offsets sequence-of-returns risk since
each franc of capital converts to a larger pension.
