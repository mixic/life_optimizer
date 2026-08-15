<div align="center">

# Life Optimizer

**A Rust-based decision tool for the work-percentage question every Swiss employee eventually faces:
"Should I work 100%, 80%, or something else — and can I actually afford it, now and in retirement?"**

[![Rust](https://img.shields.io/badge/rust-2021-orange.svg)](https://www.rust-lang.org)
[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](#license)

</div>

---

## Why this exists

Most take-home-pay calculators stop at "gross minus tax." That misses almost everything
that actually determines whether reducing your work percentage is a good idea:

- Swiss tax is **progressive**, so cutting your workload often costs less net income
  than the percentage suggests.
- Time has **value that changes with life stage** — an hour with a newborn is not
  worth the same as an hour at 24.
- A pension fund (Pillar 2 / BVG) built on 80% contributions for 30 years looks very
  different from one built on 100% — and that difference doesn't show up until
  you're 65 and it's too late to fix.
- Retirement isn't one number. It's 20-25 years exposed to inflation, market
  volatility, and the possibility of a recession hitting at exactly the wrong time.

**Life Optimizer** models all of it together — taxes, personal budget, time/health/family
utility, and Monte Carlo pension projections — and searches for the work percentage
that maximizes your overall quality of life, not just your paycheck.

---

## What it does

### 1. Work-life balance optimization
Runs a multi-objective utility function across candidate work percentages (50%–100%)
that weighs:
- **Consumption** — after-tax income vs. your personal budget
- **Leisure** — free time, with diminishing returns
- **Family** — weighted by life stage (new parent vs. empty nester)
- **Health** — a convex stress penalty for overwork
- **Security** — long-term pension adequacy

and returns the percentage that maximizes total utility, not just income.

### 2. Accurate Swiss tax modeling
- Official Stadt Bern 2024 tax tables (Kantons-, Gemeinde-, Kirchensteuer), looked up
  and interpolated by income, marital status, and children
- Mandatory social security layered on top (AHV/IV/EO, ALV, BVG)
- `--custom-tax-rate` override so you can plug in your own observed rate from a
  Lohnausweis instead of relying on tables that can't capture every personal deduction

### 3. Monte Carlo pension simulation
Every work-percentage scenario is stress-tested against your actual retirement, not
just projected with a single "expected return" assumption:
- **10,000-path simulation** under Conservative / Base Case / Optimistic market
  assumptions (log-normal returns calibrated to Swiss BVG fund data)
- **Economic regime-switching model** — a Markov chain across Boom / Normal /
  Recession / Stagflation states, so bad years cluster the way they do in reality
  instead of being averaged away
- **Sequence-of-returns stress test** — forces a recession to hit in the 2 years
  before through 1 year after retirement, the single most dangerous timing for a
  pension, and reports how much your income would actually drop
- Supports **deferred retirement up to age 70**, with age-scaled BVG contribution
  rates and the higher conversion rate you earn by working longer
- Outputs a **quality-of-life score (0–10)** for retirement, not just a CHF figure

### 4. Lifetime strategy
Projects the optimal work percentage across your whole career (age 30–65+), since
the right answer at 28 (single, building a foundation) is not the right answer at 38
(kids, time is scarce) or at 60 (health, winding down).

---

## Example output

```
OPTIMAL SOLUTION FOUND!
Work Percentage: 80%
Gross Income:    85,155 CHF/year
After-Tax:       73,386 CHF/year
Monthly Net:     6,115 CHF/month
Total Deduction: 13.8%
Status:          MEETS ALL REQUIREMENTS ✓
TOTAL UTILITY:   31.75

PENSION SUSTAINABILITY ANALYSIS
────────────────────────────────────────
Projected Pension (at age 65):
  AHV (Pillar 1):      CHF 2,129/month
  BVG (Pillar 2):      CHF 3,245/month
  Total Pension:       CHF 5,374/month
Status:              PENSION ADEQUATE ✓
Coverage:            117% of needs

MONTE CARLO PENSION SIMULATION (10,000 paths)
────────────────────────────────────────
  Scenario        P10 (bad)    Median     P90 (good)
  Conservative    CHF 3,200    CHF 4,100   CHF 5,100
  Base Case       CHF 3,800    CHF 5,200   CHF 7,400
  Optimistic      CHF 4,200    CHF 6,800   CHF 11,200

STRESS TEST: RECESSION HITS RIGHT AT RETIREMENT
────────────────────────────────────────
  Normal median pension:    CHF 5,200/month
  Stress-test pension:      CHF 4,100/month  (21% lower)
  Still adequate:           68% of stress scenarios
```

---

## Installation

Requires [Rust](https://www.rust-lang.org/tools/install) (2021 edition).

```bash
git clone https://github.com/yourusername/life-optimizer.git
cd life-optimizer
cargo build --release
```

The binary will be at `target/release/life-optimizer` (or `.exe` on Windows).

---

## Usage

### Find your optimal work percentage

```bash
./life-optimizer optimize \
  --salary 100000 \
  --age 35 \
  --married true \
  --children 2 \
  --retirement-age 65 \
  --life-expectancy 90 \
  --pillar3a 7056
```

### Use your own observed tax rate instead of official tables

```bash
./life-optimizer optimize --salary 100000 --age 35 --custom-tax-rate 0.20
```

### Compare specific scenarios side by side

```bash
./life-optimizer compare --salary 100000 --age 35 --married true --percentages "0.6,0.8,1.0"
```

### Deep-dive pension simulation for a specific work percentage, working to 70

```bash
./life-optimizer pension \
  --salary 100000 \
  --age 35 \
  --work-pct 1.0 \
  --retirement-age 70 \
  --life-expectancy 90 \
  --pillar3a 7056
```

### Lifetime strategy across your whole career

```bash
./life-optimizer lifetime --salary 100000 --age 30 --married true --children 2 --retirement-age 65
```

### Interactive mode

```bash
./life-optimizer interactive
```

---

## CLI reference

| Command | Purpose |
|---|---|
| `optimize` | Find the utility-maximizing work percentage for your situation |
| `compare` | Show gross/net/tax/utility side by side for chosen percentages |
| `pension` | Standalone Monte Carlo + regime-switching pension deep dive |
| `lifetime` | Optimal work percentage trajectory across your career |
| `interactive` | Guided question-by-question setup |

Key flags (`optimize` / `pension`):

| Flag | Description | Default |
|---|---|---|
| `--salary` | Full-time annual gross salary (CHF) | required |
| `--age` | Current age | required |
| `--married` | Marital status | `false` |
| `--children` | Number of children | `0` |
| `--custom-tax-rate` | Override tables with your own observed total deduction rate (decimal) | official tables |
| `--retirement-age` | Planned retirement age (up to 70) | `65` |
| `--life-expectancy` | Planning horizon for retirement | `90` |
| `--pillar3a` | Annual Pillar 3a contribution (CHF, max 7,056) | `0` |
| `--profile` | `balanced` / `family` / `career` preference weighting | `balanced` |

---

## How the math works

**Objective function:**

```
U = Σ β^t · u(consumption, leisure, family, health, security)
```

subject to budget, time (168 hrs/week), personal-requirement, and pension-adequacy
constraints. Full derivation, calibration notes, and the utility component formulas
are in [`MATHEMATICS.md`](MATHEMATICS.md).

**Tax engine:** progressive lookup tables (Stadt Bern 2024 official rates) with
linear interpolation between income brackets, layered with mandatory Swiss social
security (AHV/IV/EO 5.3%, ALV 1.1%, BVG ~6.5%).

**Pension engine:** age-scaled BVG contributions compounded annually under either
(a) a fixed log-normal return distribution per scenario, or (b) a 4-state Markov
regime-switching model (Boom/Normal/Recession/Stagflation), converted to an annuity
at retirement using an age-dependent conversion rate, then drawn down against
inflation-adjusted needs through your planning horizon. See
[`ECONOMIC_SCENARIOS.md`](ECONOMIC_SCENARIOS.md) and
[`PENSION_OPTIMIZATION.md`](PENSION_OPTIMIZATION.md) for details.

---

## Project structure

```
life-optimizer/
├── Cargo.toml
├── src/
│   ├── main.rs               CLI, command dispatch, orchestration
│   ├── tax.rs                Swiss progressive tax lookup tables
│   ├── requirements.rs       Personal budget, life stages, preference weights
│   ├── optimizer.rs          Multi-objective utility optimization engine
│   ├── monte_carlo.rs        Pension Monte Carlo + regime-switching simulation
│   ├── economic_regimes.rs   Markov chain economic regime model
│   ├── display.rs            Work-life balance result formatting
│   └── mc_display.rs         Pension simulation result formatting
├── MATHEMATICS.md            Full mathematical formulation
├── ECONOMIC_SCENARIOS.md     Regime-switching model & stress test details
├── PENSION_OPTIMIZATION.md   Pension sustainability methodology
├── EXAMPLES.md                Worked usage examples
├── QUICKSTART.md              Getting-started guide
└── PROJECT_SUMMARY.md         Executive overview
```

---

## Assumptions & limitations

- Tax tables are calibrated to **Stadt Bern, 2024**. Other cantons/municipalities
  will differ — use `--custom-tax-rate` with your own observed rate for accuracy
  anywhere else.
- Pension projections are **estimates**, not financial advice. Real BVG plans vary
  by employer (contribution rates, conversion rates, buy-in options).
- Market return assumptions are calibrated to historical Swiss BVG fund performance
  and general developed-market business-cycle statistics — not a guarantee of future
  returns.
- The retirement "needs" figure defaults to 75% of current net income, a common
  rule of thumb, not a personalized budget.

This tool is meant to inform a conversation with a financial advisor or pension
fund — not replace one.

---

## License

This project is licensed under the **GNU General Public License v3.0 (GPL-3.0)**. 

* **Why the change?** I've transitioned from the MIT License to GPL v3.0 to ensure that this standalone application remains free and open-source for everyone, and that all future derivative works and improvements are contributed back to the community.
* **Previous Versions:** Legacy versions remain available under the terms of the MIT License.

For more details, see the [LICENSE](LICENSE) file.


## Contributing

Issues and PRs welcome, particularly:
- Tax tables for other Swiss cantons
- Refinements to the regime-switching calibration
- Additional life-stage/preference profiles

