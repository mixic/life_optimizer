# Future Work

*A revision of this document's previous version. The earlier version treated
regime-switching, jump risk, dependence modeling, and calibration as entirely
missing. Since then, [`economic_regimes.rs`](src/economic_regimes.rs) shipped a
4-state Markov regime-switching model (Boom/Normal/Recession/Stagflation) with
a calibrated transition matrix, plus a dedicated sequence-of-returns stress
test at retirement (see [`ECONOMIC_SCENARIOS.md`](ECONOMIC_SCENARIOS.md)). This
revision starts from an honest accounting of what actually exists today, rather
than repeating a critique the codebase has partly already addressed, and adds
a second track of work — consumption-side and labor-side realism — that grew
out of [`CRITICS_CURRENT_WORK.md`](CRITICS_CURRENT_WORK.md) and
[`THEORY_OF_SPARING.md`](THEORY_OF_SPARING.md) but has not yet been implemented
in code.*

---

## 1. Honest Assessment of the Current State

Life Optimizer combines four things into one framework: deterministic
Swiss tax/social-security lookup, a multi-objective work-life utility
optimizer, a Monte Carlo pension simulator (log-normal returns under three
fixed scenarios), and a Markov regime-switching model with a forced
sequence-of-returns stress test. That combination is already more structurally
complete than most public retirement calculators, which typically stop at a
single deterministic projection.

What it is not yet is **calibrated**. Every stochastic parameter in the
project — regime transition probabilities, regime-specific return and
volatility, the log-normal scenario parameters — is a reasoned, literature-
informed *assumption*, not a value fitted to historical data with a
documented estimation procedure. This is the single most important gap, and
it is more important than adding further model sophistication: an
uncalibrated four-state regime model is not obviously more trustworthy than a
calibrated single-state one, and right now the project cannot demonstrate
which it has.

A second gap, distinct from calibration, is that the project models markets
and pensions in considerable depth but still models **consumption and labor
supply** in a comparatively primitive way: the tax and pension engines are
detailed, but `PersonalRequirements` still uses flat monthly budget fields
rather than the elasticity-tiered, utilization-weighted consumption model
proposed in `THEORY_OF_SPARING.md` §8, and the work-percentage decision is
still treated as fully discretionary rather than constrained by an employer-
side achievement requirement, as `CRITICS_CURRENT_WORK.md` §1 argues it often
is in practice. Both of these are fully specified in the existing documents —
they are a translation gap from markdown to Rust, not an open research
question.

The strategic objective for this phase of work is therefore not "add more
models." It is: **calibrate what exists, add the dependence structure and
tail risk that a regime model alone does not capture, and close the gap
between the consumption/labor theory already written down in this repository
and the code that currently ignores it.**

---

## 2. Current Weaknesses and Structural Limitations

*This section is preserved verbatim from the original version of this
document. It is kept in full because it is the founding critique the rest of
this revision responds to — some of it has since been partially addressed
(see the status table in §3 and the inline notes below each subsection), and
some of it remains fully open. Removing or paraphrasing it would obscure how
much of the project's direction still traces back to this exact list.*

### 2.1 Limited realism of the underlying stochastic assumptions

The current simulation approach appears to rely primarily on simplified market assumptions. This creates a risk that the model is too smooth and too stable relative to real financial conditions.

Examples of limitations:

- asset returns may be treated as relatively homogeneous despite volatility clustering
- inflation and salary growth may not adequately reflect regime-dependent dynamics
- crisis periods may be underrepresented in the simulated tail distribution
- dependencies between macro variables may be modeled too simplistically

This matters because the retirement decision is highly sensitive to scenarios such as stagflation, prolonged recession, or sudden real-rate shocks.

> **Status note:** regime-dependent dynamics for returns and volatility now
> exist (`economic_regimes.rs`), which addresses the second bullet in part.
> Volatility clustering *within* a regime, and dependence between macro
> variables (fourth bullet), are still open — see §4.2 and §4.3.

### 2.2 Insufficient treatment of dependence structures

A major weakness in many retirement and pension models is the tendency to simulate variables independently or with simplistic correlation assumptions. In practice, macro variables are strongly dependent.

Examples:

- inflation and interest rates often move in a non-linear relationship
- salary growth may vary with unemployment and inflation cycles
- asset returns and macro conditions are not independent over longer horizons
- pension outcomes can be jointly driven by market stress, inflation, labor income, and timeline assumptions

Without a proper dependency structure, the model can underestimate the frequency and severity of adverse outcomes.

> **Status note:** still fully open. The regime-switching model ties returns
> and inflation together *through the shared regime state* (see §4, Track A
> above), but salary growth is not yet part of that coupling, and there is no
> continuous dependence structure within a regime. See §4.2.

### 2.3 Missing regime-aware dynamics

A one-state or static-parameter model is usually insufficient for long-term planning. The project should explicitly model macroeconomic regimes such as expansion, contraction, inflationary shocks, or stagflation.

Without regime switching:

- projections may be too optimistic in calm periods
- volatile periods may be underweighted
- long-term retirement outcomes may display unrealistic stability
- scenario analysis may fail to capture regime transitions that materially affect pension security

> **Status note:** this is the one item substantially addressed since this
> critique was first written. `economic_regimes.rs` implements a 4-state
> Markov regime-switching model (Boom/Normal/Recession/Stagflation) with a
> transition matrix, and `monte_carlo.rs` uses it for both a full-career
> simulation and a dedicated sequence-of-returns stress test at retirement.
> What remains open is *calibration* of that model's parameters against
> historical data (§4.1) and validation of its implied regime frequencies
> against real business-cycle dating (§4.4).

### 2.4 Weak validation and calibration culture

The project needs a stronger validation framework. Without disciplined calibration and comparative benchmarking, a model can appear numerically stable while still being economically misleading.

Examples of missing validation:

- no clear comparison against historical distributions
- no stress-test benchmarks for crisis scenarios
- no calibration targets for inflation, salary, and market volatility
- no formal evaluation of model error against observed data

This is one of the most important gaps: a model that is computationally efficient but not calibrated to empirical data can be worse than a simpler but more realistic model.

> **Status note:** still fully open, and — per §4 below — this is judged the
> single highest-priority remaining gap. A stress-test *mechanism* now exists
> (§2.3's status note), but that is not the same as validating it, or the
> rest of the model, against empirical data. See §4.1.

### 2.5 Inadequate benchmark architecture

The project currently appears to lack a systematic benchmark suite that compares model families under identical conditions. This is essential if the objective is to move from a prototype into a credible quantitative framework.

Without a benchmark layer:

- it is difficult to compare simulation strategies fairly
- it is hard to distinguish model realism from computational convenience
- there is no evidence-based basis for selecting the production model
- stress testing and decision support remain ad hoc

> **Status note:** still fully open. See §4.5 and §4.6.

### 2.6 Limited strategic differentiation

The project is at risk of becoming a generic Monte Carlo pension tool without a distinctive academic or quantitative edge. Many retirement tools can simulate scenarios, but fewer offer a robust hybrid framework with scenario-aware modeling, calibration logic, and benchmark-based selection of methods.

A project becomes strategically relevant only when it can justify why a specific model family is chosen for a given task, and when that choice is supported by empirical evidence.

> **Status note:** the regime-switching addition (§2.3) is a step toward
> differentiation on the modeling side. The Track B work in §5 — an
> elasticity-tiered consumption model and an employer-side achievement
> constraint, neither of which is common in retirement calculators — is
> arguably now a second, independent axis of differentiation this project has
> that most Monte Carlo pension tools do not, once implemented in code rather
> than left as documentation.

---

## 3. What Already Exists (do not re-build this)

For clarity, since §2 above does not by itself distinguish shipped work from
still-open work:

| Capability | Status | Location |
|---|---|---|
| Progressive Swiss tax (Stadt Bern, lookup + interpolation) | Shipped | `tax.rs` |
| Custom observed tax rate override | Shipped | `tax.rs`, CLI `--custom-tax-rate` |
| Multi-objective work-life utility optimizer | Shipped | `optimizer.rs` |
| Monte Carlo pension simulation (log-normal, 3 scenarios) | Shipped | `monte_carlo.rs` |
| 4-state Markov regime-switching (Boom/Normal/Recession/Stagflation) | Shipped | `economic_regimes.rs` |
| Sequence-of-returns stress test at retirement | Shipped | `monte_carlo.rs::run_retirement_shock_stress_test` |
| Deferred retirement to age 70, age-scaled BVG contribution/conversion | Shipped | `monte_carlo.rs` |
| Regime transition matrix / return parameters calibrated to data | Not done | — |
| Correlation/dependence between inflation, returns, salary growth | Not done | — |
| Jump-diffusion or discrete tail-shock component | Not done | — |
| Historical backtesting / calibration pipeline | Not done | — |
| Benchmark suite comparing model families | Not done | — |
| Elasticity-tiered, utilization-weighted consumption model | Documented only | `THEORY_OF_SPARING.md` §8 |
| Employer-side achievement-capacity constraint on work % | Documented only | `CRITICS_CURRENT_WORK.md` §1.3 |
| Production / stress-test / research model separation | Not done | — |

---

## 4. Track A — Quantitative Realism

### 4.1 Calibration is the priority, not additional model families

An uncalibrated model with more free parameters is not an improvement; it is
a larger surface for unverified assumptions. Before any new stochastic
component is added, the existing regime-switching parameters need a
documented calibration procedure:

- Fit regime-specific return and volatility parameters to historical Swiss
  BVG fund performance data and/or a broader developed-market equity/bond
  blend, rather than the current literature-informed point estimates in
  `MarketAssumptions`.
- Estimate the regime transition matrix from an actual business-cycle dating
  method (e.g., a Hamilton-style Markov-switching model fit to historical
  GDP growth or a recession-indicator series) rather than the currently
  hand-specified `TransitionMatrix::calibrated()` probabilities, which are
  reasoned but not fitted.
- Publish the calibration inputs, method, and resulting parameter table in a
  new `CALIBRATION.md`, so every number in `economic_regimes.rs` is traceable
  to a data source and an estimation method rather than to a design choice.

### 4.2 Dependence structure between macro variables

Currently, returns, inflation, and (implicitly) salary growth move together
only through the shared regime state — within a regime, they are otherwise
independent draws. Real macro variables have residual dependence beyond what
a shared discrete regime captures (e.g., inflation and interest rates move in
a related but non-linear way even within a single business-cycle phase).

Proposed addition:

$$
(R_t, I_t, W_t) \sim C\big(F_R(R_t), F_I(I_t), F_W(W_t)\big)
$$

where $R_t$, $I_t$, $W_t$ are returns, inflation, and salary growth in period
$t$, $F_{(\cdot)}$ are their regime-conditional marginal distributions
(already implicitly defined by `MarketAssumptions`), and $C$ is a copula (a
Gaussian or Student-$t$ copula is a reasonable starting point) capturing
residual dependence *within* a regime that the discrete state alone misses.
This should be layered on top of the existing regime model, not replace it —
regime-switching captures large discrete shifts; a copula captures the
smaller, continuous co-movement within a regime.

### 4.3 Jump-diffusion for tail events sharper than a regime transition

A regime transition captures a *sustained* change in market conditions (a
multi-year recession, for example). It does not capture a genuine one-day/
one-week shock — a market crash or a sudden inflation surprise — that is
sharper and shorter than a regime shift. A Merton-style jump-diffusion
component, layered on top of the existing regime-switching returns, would
capture this:

$$
dS_t = \mu(regime_t)\,S_t\,dt + \sigma(regime_t)\,S_t\,dW_t + S_t\,dJ_t
$$

where $J_t$ is a compound Poisson jump process with regime-dependent jump
intensity — jumps should be more frequent and more severe during Recession
and Stagflation regimes than during Normal or Boom, which the current model
does not distinguish (a recession-regime year and a Boom-regime year currently
differ only in mean and volatility, not in tail shape).

### 4.4 Historical backtesting and validation

None of the above is credible without a validation step. Proposed minimum
validation suite:

- Compare simulated return/inflation distributions against the empirical
  historical distribution (Swiss and/or broader developed-market data) for
  moments beyond the mean and variance — skewness and kurtosis in particular,
  since the current log-normal-per-regime approach understates fat tails
  relative to what jump-diffusion (§4.3) would add.
- Backtest the regime-switching model's implied recession frequency and
  duration against actual post-WWII business-cycle dating.
- Report CVaR (conditional value-at-risk) and downside-percentile pension
  outcomes explicitly, not only the median/P10/P90 currently reported in
  `mc_display.rs`.

### 4.5 Benchmark suite

A comparison harness that runs the same scenario (same salary, age, work
percentage, retirement age) through each available model configuration —
static log-normal, regime-switching, regime-switching + copula, regime-
switching + copula + jump-diffusion — and reports the resulting pension
distributions side by side. This is what makes the added complexity in
§4.2–4.4 justifiable rather than decorative: if a more complex model does not
materially change the decision-relevant output (the recommended work
percentage, or the P10 pension outcome) for realistic parameter ranges, that
is itself a useful, reportable finding, not a wasted effort.

### 4.6 Production / stress-test / research model separation

Once §4.1–4.5 exist, the CLI should expose three explicit modes rather than
one fixed pipeline:

- **Production mode** — the calibrated regime-switching model, fast enough
  for interactive CLI use, used by `optimize` and `pension` by default.
- **Stress-test mode** — the existing sequence-of-returns shock test, extended
  to also run the jump-diffusion tail scenarios from §4.3, used explicitly
  when the person asks "how bad could this get."
- **Research mode** — the full copula + jump-diffusion + regime-switching
  stack, computationally heavier, used for calibration validation and for
  anyone who wants the most complete (if slower) simulation.

---

## 5. Track B — Closing the Theory-to-Code Gap

This track has no open research question attached to it — every formula below
already exists in a markdown document in this repository. The work is
translating it into `PersonalRequirements` and `OptimizerConfig`.

### 5.1 Elasticity-tiered consumption model

`THEORY_OF_SPARING.md` §7c and §8 propose splitting the flat `discretionary`
field into elasticity tiers:

$$
C_t = \underbrace{R_t + E_t}_{\text{inelastic}} + \underbrace{Q_t}_{\text{quasi-inelastic}} + \underbrace{L_t}_{\text{elastic, sparing-eligible}} + D_t
$$

with $L_t$ further decomposed by a sparing ratio $\sigma$, second-hand price
ratio $\phi$, and a utilization-rate penalty $\rho_{\text{use}}^{-1}$ (full
formula in `THEORY_OF_SPARING.md` §8). Concrete implementation:

- Add `rent`, `essential_inelastic`, `quasi_inelastic`, and
  `sparing_eligible` fields to `PersonalRequirements`, replacing the current
  flat `discretionary` field.
- Add CLI flags `--sparing-ratio`, `--utilization-discipline`, and
  `--quasi-inelastic-share` as specified in `THEORY_OF_SPARING.md` §8.
- Report the elasticity-tier breakdown in `display.rs` rather than a single
  aggregate discretionary number, so the person can see where budget pressure
  is actually landing (as argued in `THEORY_OF_SPARING.md` §7c).

### 5.2 Employer-side achievement-capacity constraint

`CRITICS_CURRENT_WORK.md` §1.3 proposes that a reduction in work percentage is
only credible when effective achievement capacity still meets the
organization's required output:

$$
A_t = H_t \cdot P_t \cdot (1 + \alpha_t) \qquad \text{subject to} \qquad A_t \geq G_t
$$

Concrete implementation:

- Add an optional `--required-output-index` and `--ai-productivity-gain`
  (`α_t`) pair of CLI flags.
- When both are supplied, `optimizer.rs` should mark a candidate work
  percentage as infeasible if it fails $A_t \geq G_t$, in addition to the
  existing budget-feasibility check — the model already has a "feasible: ✓/✗"
  concept for budget adequacy (see `WorkScenario`); this extends the same
  mechanism to job-security adequacy.
- This directly operationalizes the reframed question from
  `PHILOSOPHICAL_SOCIOLOGICAL_ASPECTS.MD` §3a: not "what work percentage
  maximizes my utility," but "what is the lowest work percentage at which I
  can still reliably deliver what's expected of me."

### 5.3 Satisfaction–performance feedback (exploratory)

`PHILOSOPHICAL_SOCIOLOGICAL_ASPECTS.MD` §3c and `HAPPINESS_OR_FEAR_WORK_LIFE.md`
§2–4 argue that $P_t$ (baseline productivity) is not actually independent of
work percentage and job satisfaction — chronic overwork degrades the
productivity term itself (Hobfoll's Conservation of Resources, Yerkes-Dodson).
This is a genuinely open modeling question rather than a ready-to-implement
formula, and belongs in Track A's research mode once §4.6 exists: a
feedback term $P_t = P_0 \cdot f(\text{sustained work \%}, \text{time})$ where
$f$ declines under prolonged high work percentage, calibrated against the
burnout and engagement literature cited in `HAPPINESS_OR_FEAR_WORK_LIFE.md`,
would let the optimizer discover — rather than assume — cases where 100% work
is self-defeating even under the pure achievement-capacity constraint of §5.2.

---

## 6. Strategic Priorities, Ranked

1. **Calibration discipline for the model that already exists** (§4.1) — before
   any new stochastic machinery, make the current regime-switching model's
   parameters traceable to data.
2. **Dependence and tail risk layered on the existing regime model** (§4.2–4.3)
   — copula dependence and jump-diffusion, not a replacement architecture.
3. **Consumption-model code integration** (§5.1) — the highest-value, lowest-
   research-risk item on this list, since the formula is already fully
   specified; it is also the piece most directly usable by anyone running the
   tool today.
4. **Employer-side achievement constraint** (§5.2) — second-highest value for
   the same reason: fully specified, addresses a critique already validated
   by an outside domain expert (`CRITICS_CURRENT_WORK.md` §1).
5. **Benchmark suite and production/stress/research separation** (§4.5–4.6) —
   necessary for credibility once §1–4 exist, but depends on them being done
   first.
6. **Satisfaction–productivity feedback** (§5.3) — genuinely exploratory;
   correctly belongs last, in research mode, once the rest of the pipeline is
   trustworthy enough to layer a speculative mechanism on top of.

---

## 7. The Central Strategic Question

The question this project has to keep answering honestly is not whether it
can simulate a plausible-looking pension outcome — it already can, and did
before the regime-switching model existed. The question is whether the
numbers it produces are **traceable**: to a data source, to a named
assumption, to a documented estimation method — versus being a plausible
narrative wrapped around unfitted parameters.

The same discipline applies to Track B: it is not enough that the sparing
ratio and achievement-capacity constraint are *mathematically* well-specified
in the accompanying markdown documents. Until they exist in `optimizer.rs` and
`monte_carlo.rs`, they are philosophy the tool does not yet act on — and a
project that writes rigorous critiques of its own assumptions but does not
close the gap between the critique and the code is not meaningfully more
credible than one that never wrote the critique at all.

---

## 8. Roadmap

### Phase 1 — Calibration (Track A foundation)
- Fit regime-specific return/volatility parameters to historical data
- Estimate the transition matrix from a business-cycle dating method
- Publish `CALIBRATION.md` with sources, method, and resulting parameters

### Phase 2 — Theory-to-code closure (Track B, can run in parallel with Phase 1)
- Implement the elasticity-tiered consumption model (§5.1) in
  `requirements.rs` and `display.rs`
- Implement the achievement-capacity constraint (§5.2) in `optimizer.rs`

### Phase 3 — Dependence and tail risk (Track A, depends on Phase 1)
- Add copula-based dependence between returns, inflation, salary growth (§4.2)
- Add regime-dependent jump-diffusion (§4.3)
- Extend the historical validation suite (§4.4) to cover both additions

### Phase 4 — Benchmarking and model governance
- Build the benchmark harness comparing model configurations (§4.5)
- Implement production/stress-test/research mode separation (§4.6)

### Phase 5 — Research extensions
- Satisfaction–productivity feedback term (§5.3)
- Scenario-tree or robust optimization for the work-percentage decision itself,
  evaluated against the existing grid-search optimizer as a baseline
- Exploratory reinforcement-learning approach to adaptive work-percentage
  strategy over a career, benchmarked against Phase 1–4 methods rather than
  presented as a replacement for them

---

## 9. Expected Impact

Phases 1–2 alone would close the largest credibility gap the project
currently has: a regime-switching model whose parameters cannot yet be traced
to data, and a consumption/labor theory that exists only in prose. Phases 3–4
would bring the quantitative core closer to the standard a serious retirement-
planning framework should be held to — dependence structure, tail risk,
benchmarked model selection — rather than a Monte Carlo tool that merely looks
sophisticated. Phase 5 is explicitly speculative and should be evaluated
against, not substituted for, the more transparent baseline methods built in
the earlier phases.

The throughline across both tracks is the same: **this project should not
claim more rigor than it can currently show its work for** — whether that
work is a fitted transition matrix or an implemented consumption formula
sitting, right now, only in a markdown file.


## 10. Conversion Rate Analysis: From Capital to Actual Pension

### 10.1 Problem Statement

Most pension simulations, including earlier versions of the Life Optimizer, focus on the accumulation of pension capital. However, the crucial question for retirement is: **How much monthly pension actually results from this capital?**

The conversion rate is the central lever that bridges the gap between accumulated capital and lifelong pension. The large discrepancy between the statutory rate (6.8%) and the rates actually applied by many pension funds (often 5.0% – 5.5%) means that the actual pension is significantly lower than often assumed. This effect is amplified by the trend towards further declining conversion rates in the future, driven by:

- **Increasing life expectancy** – pensions must be paid for longer periods
- **Persistently low interest rates** – lower returns on pension fund assets
- **Demographic shifts** – fewer active contributors per retiree

### 10.2 Core Issues in the Current Model

1. **Static Conversion Rate**: Only the statutory minimum rate of 6.8% is used, which is often unrealistic in practice.
2. **Missing Scenario Analysis**: The user does not see a range of possible pension amounts based on different conversion rates.
3. **No Future Projection**: The foreseeable trend towards lower rates (due to increasing life expectancy and low interest rates) is not modelled.
4. **Overestimated Pension Amounts**: The output suggests a precision that is not justified due to the uncertainty in the conversion rate.

### 10.3 Mathematical Formulation

The monthly pension is calculated as:

$$
P_{\text{monthly}} = \frac{C \times r}{12}
$$

Where:
- $C$ = Pension capital at retirement
- $r$ = Conversion rate (decimal)

The dynamic conversion rate projection over time:

$$
r(t) = r_0 - (t - t_0) \times \Delta r
$$

With:
- $r_0 = 0.068$ (2024 statutory rate)
- $t_0 = 2024$
- $\Delta r = 0.00036$ (annual reduction of 0.036 percentage points)

This leads to projected rates of:

| Year | Projected Rate |
|------|---------------|
| 2024 | 6.80% |
| 2030 | 6.58% |
| 2040 | 6.22% |
| 2050 | 5.86% |
| 2060 | 5.50% |

The rate is bounded at a minimum of 5.0% (lower threshold based on expert projections).

### 10.4 Impact Analysis: From Capital to Monthly Pension

The following table shows the impact of different conversion rates on the monthly pension for a given pension capital of CHF 500,000:

| Scenario | Conversion Rate | Annual Pension | Monthly Pension | Difference from Statutory |
|----------|----------------|----------------|-----------------|---------------------------|
| **Statutory (BVG)** | 6.8% | CHF 34,000 | CHF 2,833 | CHF 0 |
| **Typical Fund Rate** | 5.5% | CHF 27,500 | CHF 2,292 | -CHF 541 |
| **Future Projection** | 5.0% | CHF 25,000 | CHF 2,083 | -CHF 750 |
| **Conservative Estimate** | 4.5% | CHF 22,500 | CHF 1,875 | -CHF 958 |

**Example Output Format:**

MONTHLY PENSION BY CONVERSION RATE
--------------------------------------
Statutory Rate (6.8%): CHF 5'374
Typical Fund Rate (5.5%): CHF 4'345
Future Projection (5.0%): CHF 3'950
Actual Range: CHF 3'950 - 5'374

Note: The actual pension depends on your pension fund's conversion rate.
Many funds apply a lower rate. Use --conversion-rate for a precise calculation.

### 10.5 Planned Improvements

#### Short-Term (Next Release)
- **Three-Scenario Display**: Side-by-side presentation of pension amounts at 6.8% (statutory), 5.5% (fund-typical), and 5.0% (future projection)
- **New CLI Parameter `--conversion-rate`**: Enables precise input of the actual fund rate
- **Transparent Disclaimer**: Clear indication of the discrepancy between statutory and actual rates in the output

#### Medium-Term (Next 2-3 Releases)
- **Dynamic Conversion Rate Modelling**: Linear reduction of the rate over time based on demographic and economic trends
- **Fund-Specific Profiles**: Integration of standard profiles for major Swiss pension funds (Publica, BVK, etc.)
- **Pension Range as Standard Output**: Display of the possible range instead of a single value

#### Long-Term (Roadmap 2027+)
- **Stochastic Modelling**: Monte Carlo simulation of the conversion rate based on interest rate and life expectancy scenarios
- **Historical Analysis**: Display of conversion rate development over the last 30 years with trend projections
- **Personalized Fund Database**: Building a community-based database with actual conversion rates of various pension funds
- **Capital Withdrawal Optimisation**: Simulation of the tax implications of capital withdrawal vs. pension withdrawal

### 10.6 Technical Implementation

#### Data Structure

```rust
pub enum ConversionRateScenario {
    Statutory,      // 6.8%
    FundTypical,    // 5.5%
    FutureProjection, // Dynamic based on retirement year
    Custom(f64),    // User-provided rate
}

pub struct PensionRange {
    pub statutory: f64,      // Monthly pension at 6.8%
    pub typical: f64,        // Monthly pension at 5.5%
    pub future: f64,         // Monthly pension at projected rate
    pub custom: Option<f64>, // Monthly pension at user rate
    pub range: (f64, f64),   // (minimum, maximum) monthly pension
}

```

####  CLI Integration

```bash
# Use default scenarios
./life-optimizer optimize --salary 100000 --age 35

# Override with custom conversion rate
./life-optimizer optimize --salary 100000 --age 35 --conversion-rate 0.055

# Display all scenarios
./life-optimizer optimize --salary 100000 --age 35 --show-conversion-scenarios
```

Implementation Location
Component	File	Description
Conversion rate enum	src/monte_carlo.rs	Define ConversionRateScenario
Dynamic rate projection	src/monte_carlo.rs	project_conversion_rate(year) function
Pension range calculation	src/monte_carlo.rs	calculate_pension_range() function
Output formatting	src/mc_display.rs	Display scenarios in results
CLI parameter	src/main.rs	Add --conversion-rate flag
Documentation	MATHEMATICS.md	Update with conversion rate formulas
10.7 Expected Benefits
Realistic Expectations: Users see that the actual pension is often significantly below the statutory maximum.

Better Decision Basis: The range makes the uncertainty transparent and prevents poor decisions based on overly optimistic assumptions.

Future Awareness: Younger users understand the trend towards lower conversion rates and can plan accordingly.

Precision When Needed: With --conversion-rate, the user can input the exact rate of their pension fund.

Transparency: The tool shows the range of possible outcomes rather than a single, potentially misleading number.

10.8 Next Steps
□ Implementation of the --conversion-rate parameter in CLI
□ Extension of output in mc_display.rs to include three scenarios
□ Integration of dynamic model (linear reduction) in monte_carlo.rs
□ Creation of fund-specific profiles for the 5 largest Swiss pension funds
□ Documentation of new features in MATHEMATICS.md
□ Update EXAMPLES.md with conversion rate usage examples


