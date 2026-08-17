# Life Optimizer — Design Document

This document describes the architecture, algorithms, data flow, and design rationale behind Life Optimizer. It is intended for contributors, reviewers, and future maintainers.

---

# 1. System Overview

Life Optimizer is a Rust-based decision engine that determines the optimal work percentage (e.g., 60%, 80%, 100%) by modeling:

- Net income after taxes
- Leisure and time utility
- Health and stress
- Long-term pension outcomes (Monte Carlo)
- Life-phase transitions
- Risk preferences and uncertainty

The system combines deterministic models (taxes, BVG rules) with stochastic simulations (market returns) and optimization algorithms.

---

# 2. Architecture

## 2.1 Layered Architecture

+---------------------------+
| Presentation Layer        |
| CLI, reports, visualizations |
+---------------------------+
| Optimization Layer        |
| Pareto front, Bayesian opt |
+---------------------------+
| Simulation Layer          |
| Monte Carlo, scenarios, RL |
+---------------------------+
| Domain Layer              |
| Taxes, BVG, cashflow, life phases |
+---------------------------+
| Infrastructure Layer      |
| Config, logging, data I/O |
+---------------------------+


### Domain Layer
- Swiss tax model (municipality-level)
- BVG accumulation and conversion
- Cashflow model
- Life-phase state machine

### Simulation Layer
- Monte Carlo pension simulation
- Regime-switching economic models
- Deterministic scenarios
- Stress tests

### Optimization Layer
- Grid search (baseline)
- Multi-objective optimization (Pareto front)
- Bayesian optimization (future)
- Robust optimization (CVaR, min-max)

### Presentation Layer
- CLI commands
- Interactive mode
- Scenario comparison tables
- Exportable reports

---

# 3. Algorithms

## 3.1 Tax Calculation
- Deterministic model using municipal tax tables.
- AHV/IV/EO, ALV, BVG contributions.
- Optional user-provided effective tax rate.

## 3.2 BVG Pension Simulation
- Monte Carlo with 10,000+ paths.
- Regime-switching returns:
  - Boom
  - Normal
  - Recession
  - Stagflation
- Transition matrix calibrated from historical data.
- Outputs:
  - Median pension
  - Percentiles (5%, 25%, 75%, 95%)
  - Risk metrics (CVaR)

## 3.3 Utility Modeling
- Leisure utility with diminishing returns.
- Income utility (log or CRRA).
- Health/stress penalty.
- Combined score or multi-objective vector.

## 3.4 Optimization
### Baseline
- Grid search across work percentages.

### Multi-Objective (Future)
- Compute Pareto front using:
  - NSGA-II (evolutionary)
  - MOEA/D
  - Weighted-sum scalarization (fallback)

### Robust Optimization (Future)
- Minimize worst-case outcomes.
- CVaR-based optimization.

### Bayesian Optimization (Future)
- Gaussian Process surrogate model.
- Acquisition functions (EI, UCB).

---

# 4. Life-Phase Modeling

## 4.1 States
- Single
- Young children
- Teenagers
- Eldercare
- Pre-retirement

## 4.2 Transitions
- Probabilistic transitions based on demographic data.
- Each state modifies:
  - Time utility
  - Stress model
  - Financial constraints

## 4.3 Policy Optimization (Future)
- Optimize a policy π(state) → work_percentage.
- RL agent (PPO or DQN) learns long-term strategy.

---

# 5. Data Flow

## 5.1 Input Flow

```
User Input (CLI)
    ↓
├─ Personal Profile
│  ├─ Age, marital status, children
│  ├─ Current salary, canton
│  └─ Life stage classification
│
├─ Preferences
│  ├─ Profile (balanced, family, career)
│  └─ Preference weights (consumption, leisure, family, health, security)
│
└─ Parameters
   ├─ Retirement age
   ├─ Life expectancy
   └─ Pillar 3a contribution
```

## 5.2 Processing Pipeline

```
CLI Input
    ↓
[Parse & Validate]
    ↓
[Initialize TaxSchedule]  ← loads municipal tax tables
    ↓
[Initialize OptimizerConfig]
    ├─ Personal requirements (housing, food, etc.)
    ├─ Life stage (adjusted consumption)
    └─ Preference weights (utility function parameters)
    ↓
[Grid Search: work_percentage ∈ {50%, 55%, ..., 100%}]
    │
    ├─→ For each work_percentage:
    │   ├─ Calculate gross income
    │   ├─ Apply tax schedule → after-tax income
    │   ├─ Calculate utility components
    │   │   ├─ Consumption utility (log-income)
    │   │   ├─ Leisure utility (diminishing returns)
    │   │   ├─ Family utility (life-stage weighted)
    │   │   ├─ Health utility (stress penalty)
    │   │   └─ Security utility (pension simulation)
    │   ├─ Aggregate utility score (weighted sum)
    │   └─ Store WorkScenario
    │
    └─→ Return best & ranked scenarios
    ↓
[Optional: Monte Carlo Pension Simulation]
    ├─ For each scenario, run 10,000 paths
    ├─ Regime-switching returns (Boom/Normal/Recession/Stagflation)
    ├─ Compute percentiles, CVaR, outcome probabilities
    └─ Attach to WorkScenario
    ↓
[Formatting & Display]
    ├─ CLI table output
    ├─ Breakdown by component
    └─ Recommendations
    ↓
Output to User
```

## 5.3 Tax Calculation Detail

**Input:** Gross income (CHF/year), canton, marital status, children  
**Process:**
1. Load municipal tax tables for canton and income bracket
2. Calculate marginal tax rate (interpolate if needed)
3. Apply federal, cantonal, municipal tax rates
4. Add mandatory social contributions:
   - AHV/IV/EO (employer + employee)
   - ALV (employer + employee)
   - BVG employee contribution (or use work_percentage adjustment)
5. Compute effective tax rate: `total_tax / gross_income`

**Output:** `TaxSchedule` object with lookup functions:
- `after_tax_income(gross: f64) → f64`
- `effective_tax_rate(gross: f64) → f64`
- `marginal_tax_rate(gross: f64) → f64`

## 5.4 Utility Calculation Detail

The core utility function aggregates five dimensions:

$$U_{total} = w_c \cdot U_c + w_l \cdot U_l + w_f \cdot U_f + w_h \cdot U_h + w_s \cdot U_s$$

Where:
- $w_c, w_l, w_f, w_h, w_s$ = preference weights (sum to 1.0)
- $U_c$ = consumption utility
- $U_l$ = leisure utility
- $U_f$ = family utility
- $U_h$ = health utility
- $U_s$ = security utility (pension adequacy)

### Consumption Utility
$$U_c = \log\left(\frac{\text{after-tax income}}{k}\right)$$
where $k$ is a reference income (e.g., CHF 60,000/year). Capped at 0 if income < requirements.

### Leisure Utility
$$U_l = 1 - e^{-\lambda \cdot \text{free hours}}$$
where $\lambda$ controls diminishing returns (e.g., 0.001). Captures that first 4 hours of free time are more valuable than the 20th.

### Family Utility
Depends on life stage:
- **Young single:** 0 (no dependents)
- **Young couple:** 0.2 × work_percentage (some quality time)
- **New parent / School age:** 0.5–0.8 × (1.0 - work_percentage) (high value to family time)
- **Teenager / Pre-retirement:** 0.3 × (1.0 - work_percentage)

### Health Utility
$$U_h = 1.0 - \min(1.0, 0.001 \cdot (1.0 - \text{work}_\% )^2)$$
Penalizes overwork (work_% > 90%) with a convex penalty; rewards time-off with diminishing returns.

### Security Utility
$$U_s = \frac{\text{P}_{50}(\text{retirement income})}{\text{target income}} \text{ (capped at 1.0)}$$
where P50 is the median pension outcome from Monte Carlo simulation.

## 5.5 Monte Carlo Simulation

**When:** Optional; run if `--compare` or detailed pension analysis requested.  
**Input:** Work scenario, age, retirement age, life expectancy, asset model  
**Process:**
1. Initialize pension balance at current age (accumulated BVG + Pillar 3a)
2. For each of 10,000 simulation paths:
   - For each year until retirement:
     - Sample annual return from regime-switching distribution
     - Apply BVG contribution (work_% adjusted)
     - Update balance
   - For each year in retirement (retirement_age to life_expectancy):
     - Sample annual return
     - Compute annuity (or draw from balance)
     - Update balance
     - Record annual retirement income
3. Compute statistics across all paths:
   - Percentiles (5%, 25%, 50%, 75%, 95%)
   - Mean, std dev, CVaR (expected shortfall at 5%)
   - Probability of success (e.g., P(income ≥ target))

**Output:** `PensionOutcome` with percentiles and risk metrics

---

# 6. Module Interfaces

## 6.1 `tax.rs` — Swiss Tax Model

**Responsibility:** Compute after-tax income given gross income and personal circumstances.

**Key Types:**
- `TaxSchedule`: Encapsulates all tax rules for a specific canton, marital status, child count.
- `TaxBracket`: Marginal rate and threshold for income bracket.

**Main Functions:**
```rust
impl TaxSchedule {
    pub fn after_tax_income(&self, gross: f64) -> f64;
    pub fn effective_tax_rate(&self, gross: f64) -> f64;
    pub fn tax_only_rate(&self, gross: f64) -> f64;  // excludes BVG/social
}
```

**Dependencies:** None (pure calculation).

**Test Coverage:** Unit tests for known tax calculations (e.g., sample Swiss incomes from Lohnausweis examples).

---

## 6.2 `requirements.rs` — Personal Basket & Life Stage

**Responsibility:** Define personal consumption needs and adjust them by life stage.

**Key Types:**
- `PersonalRequirements`: Monthly budget categories (housing, food, transport, etc.)
- `LifeStage`: Enum with variants (YoungSingle, NewParent, SchoolAge, etc.) carrying age/child data.
- `PreferenceWeights`: Weights for utility function components (sum to 1.0).

**Main Functions:**
```rust
impl PersonalRequirements {
    pub fn total_monthly(&self) -> f64;
    pub fn adjusted_for_life_stage(&self, stage: &LifeStage) -> Self;
}

impl LifeStage {
    pub fn age(&self) -> u32;
    pub fn youngest_child_age(&self) -> Option<u32>;
}
```

**Design Rationale:**
- Life stage adjustments allow the same person at different life stages (e.g., 30-year-old single vs. 35-year-old parent) to have different consumption and utility profiles without redefining the entire requirement struct.
- Preference weights are user-configurable via CLI profiles (e.g., `--profile family` sets high `w_f`, lower `w_c`).

---

## 6.3 `optimizer.rs` — Core Optimization Loop

**Responsibility:** Evaluate work scenarios and compute utility scores.

**Key Types:**
- `OptimizerConfig`: Bundle of inputs (salary, tax schedule, requirements, preferences).
- `LifeOptimizer`: Evaluates scenarios; holds the optimization algorithm.
- `WorkScenario`: Output of one evaluation (income, utility, components, feasibility).
- `UtilityBreakdown`: Decomposition of total utility into components.

**Main Functions:**
```rust
impl LifeOptimizer {
    pub fn evaluate_scenario(&self, work_percentage: f64) -> WorkScenario;
    pub fn optimize(&self) -> Vec<WorkScenario>;  // Grid search
    pub fn calculate_utility(&self, ...) -> UtilityBreakdown;
}
```

**Algorithm:**
1. Grid search work percentages (50%, 55%, ..., 100%) or user-specified range.
2. For each percentage:
   - Compute gross, after-tax income, time allocation.
   - Calculate 5-component utility.
   - Aggregate to total score.
   - Store result.
3. Sort results by utility; return top N and ranked list.

**Design Notes:**
- Grid search is simple, deterministic, and explainable. Future versions may add Bayesian or evolutionary optimization.
- Utility components are computed independently; weights are applied at aggregation.

---

## 6.4 `monte_carlo.rs` — Pension Simulation

**Responsibility:** Stochastic projection of pension outcomes under market uncertainty.

**Key Types:**
- `MonteCarlo`: Simulation engine; holds economic regime model and RNG.
- `PensionOutcome`: Statistics across all simulation paths (percentiles, CVaR, success probability).
- `SimulationPath`: Single realized trajectory (annual balances, returns, retirement income).

**Main Functions:**
```rust
impl MonteCarlo {
    pub fn simulate_pension(&self, scenario: &WorkScenario) -> PensionOutcome;
    pub fn run_paths(&self, num_paths: usize) -> Vec<SimulationPath>;
}
```

**Design Notes:**
- Uses a Markov regime-switching model (Boom, Normal, Recession, Stagflation) calibrated to Swiss historical data.
- Supports deferred retirement (up to age 70) with age-scaled contribution rates and BVG conversion rules.
- Can simulate forced drawdowns or annuity models at retirement.

---

## 6.5 `economic_regimes.rs` — Regime-Switching Model

**Responsibility:** Model correlated economic states and return distributions.

**Key Types:**
- `EconomicRegime`: Enum (Boom, Normal, Recession, Stagflation).
- `TransitionMatrix`: Markov chain; state probabilities and transitions.
- `RegimeReturns`: Distribution of returns (mean, vol) for each regime.

**Main Functions:**
```rust
impl TransitionMatrix {
    pub fn next_state(&self, current: EconomicRegime) -> EconomicRegime;
}

impl RegimeReturns {
    pub fn sample_return(&self, regime: EconomicRegime) -> f64;
}
```

**Design Rationale:**
- Regime-switching improves realism: market downturns cluster (multi-year recessions) rather than being random.
- Calibration uses Swiss BVG fund historical returns and macro economic data.

---

## 6.6 `display.rs` & `mc_display.rs` — Output Formatting

**Responsibility:** Pretty-print results to CLI.

**Main Functions:**
```rust
pub fn display_scenarios(scenarios: &[WorkScenario]);
pub fn display_pension_outcomes(outcomes: &[PensionOutcome]);
pub fn format_table_header(...);
pub fn format_currency(amount: f64) -> String;
```

**Design Notes:**
- Uses the `colored` crate for terminal colors.
- Formats currency in CHF with thousands separators.
- Presents utility breakdown and key percentiles for readability.

---

## 6.7 `main.rs` — CLI Entry Point

**Responsibility:** Parse arguments, orchestrate workflows, handle error reporting.

**Commands:**
1. `optimize`: Find optimal work percentage for current situation.
2. `compare`: Analyze specific scenarios (e.g., 80%, 90%, 100%).
3. `scenario`: Deep dive into one work percentage with full Monte Carlo output.

**Error Handling:**
- Clap handles argument parsing and validation.
- Runtime errors (invalid age, negative salary, etc.) are caught and reported with context.

---

# 7. Error Handling & Validation

## 7.1 Input Validation

- **Salary:** Must be positive (CHF > 0).
- **Age:** Must be between 18 and 100; retirement_age must be ≥ current age and ≤ 75.
- **Life expectancy:** Must be ≥ retirement_age.
- **Work percentage:** Constrained to [0.5, 1.0] (no part-time below 50% or above 100%).
- **Preference weights:** Must sum to 1.0; no negative weights.
- **Canton code:** Must be a valid Swiss canton (ZH, BE, GE, etc.).

**Implementation:** Validation logic in `main.rs` before creating `OptimizerConfig`. Early exit with clear error messages.

## 7.2 Numerical Edge Cases

- **Near-zero income:** Utility function clips consumption utility to prevent log(0).
- **Very high income:** No caps on utility (it scales with income log-linearly).
- **Work percentage = 0%:** Not allowed (doesn't make sense in context). Grid starts at 50%.
- **Negative after-tax income:** Caught in tax module; returns 0 with warning.

## 7.3 Monte Carlo Convergence

- Default path count: 10,000. Tested for convergence; results stable above 5,000 paths.
- If requested paths > 100,000, warn user about runtime (simulation can take minutes).
- Seed RNG with user-provided value (or time-based default) for reproducibility.

---

# 8. Testing Strategy

## 8.1 Unit Tests

- **Tax calculations:** Known inputs from official Swiss Lohnausweis examples.
- **Utility functions:** Verify formulas (e.g., log-utility is monotonically increasing).
- **Life stage adjustments:** Check that consumption adjustments are reasonable (e.g., newborn stage increases childcare).
- **Regime transitions:** Verify Markov chain sums to 1.0 for each state.

## 8.2 Integration Tests

- **End-to-end optimize:** Run full pipeline from CLI input to result. Verify result is sensible (best utility is at highest work% if preferences favor income, lower if favor leisure).
- **Determinism:** Same input produces same output (critical for reproducibility).
- **Monte Carlo stability:** Increase path count and verify percentiles converge.

## 8.3 Benchmarks

- **Tax lookup:** Should be < 1µs per call.
- **Utility calculation:** < 10µs per scenario.
- **Full 10-scenario grid search:** < 1ms (without Monte Carlo).
- **Monte Carlo 10,000 paths:** < 2 seconds.

Run with: `cargo bench`.

---

# 9. Configuration & Assumptions

## 9.1 Hardcoded Defaults

- **Work hours:** 42 hours/week (Swiss full-time standard).
- **Sleep:** 8 hours/day.
- **Discount rate:** 3% per year (time preference).
- **Leisure utility decay:** λ = 0.001 (controls rate of diminishing returns).
- **Health stress penalty:** Convex in (1.0 - work_%), impacts > 90% work.

## 9.2 Calibrated Parameters

- **Regime transition matrix:** Calibrated from Swiss economic data (SNB historical database).
- **Return distributions by regime:** Fitted to BVG composite fund returns (2000–2024).
- **Inflation:** Assumed 1.5% per year (adjustable via Pillar 3a indexing).

## 9.3 User-Overridable Parameters

- `--custom-tax-rate`: Override municipal tax tables with single effective rate (e.g., 0.1382).
- `--profile`: Choose preference preset (balanced, family, career).
- `--retirement_age`: Target retirement age (default 65, up to 70).
- `--pillar3a`: Annual Pillar 3a contribution (max CHF 7,056).

---



# 10. Design Rationale & Philosophy

## 10.1 Why Rust?

- **Performance:** Monte Carlo simulation with 10,000 paths must complete in seconds, not minutes. Rust's zero-cost abstractions (SIMD, no GC) deliver this.
- **Correctness:** Financial calculations demand precision. Rust's strict type system and memory safety catch errors at compile time.
- **Determinism:** No garbage collector means no unpredictable pauses during simulation. Results are reproducible.
- **Auditability:** Compiled binary is reviewable; no hidden dependencies or runtime surprises.

## 10.2 Why Layered Architecture?

- **Domain layer first:** Taxes, BVG rules, and life stages are the core problem. Encode them transparently in domain types, not buried in callbacks.
- **Modularity:** Each layer (domain, simulation, optimization, presentation) can be tested independently. A tax bug is isolated; it doesn't break the optimizer.
- **Explainability:** An output can be traced back to its inputs. "Best work% is 80%" is backed by a utility breakdown (60% consumption, 20% leisure, etc.), not a black-box neural network.
- **Maintainability:** Future contributors can modify the tax model without touching the optimization algorithm.

## 10.3 Why Grid Search, Not Gradient Descent?

- **Discrete problem:** Work percentages are discrete (50%, 55%, 60%, ..., 100%), not continuous. Gradient descent assumes continuity.
- **Non-smooth utility:** Life-stage transitions or tax bracket changes create discontinuities. Grid search handles them naturally.
- **Interpretability:** 11 evaluated scenarios is explainable. A gradient-based optimizer produces a magic number (e.g., 73.5%) that's hard to justify.
- **Extensibility:** Grid search is trivially parallelizable (each scenario is independent). Gradient descent adds complexity without much benefit in this problem space.

**Future:** May add Bayesian optimization or evolutionary algorithms once the core model is stable and user-validated.

## 10.4 Why Monte Carlo, Not Deterministic Projection?

- **Sequence-of-returns risk:** A 50-year retirement is not a single "average" return. Bad returns at the start devastate outcomes, even if the long-term average is good.
- **Tail risk:** Users care about whether they have money at age 95, not whether the median is high. Monte Carlo computes percentiles (5th, 25th, etc.).
- **Regime clustering:** Real markets are regime-switching (multi-year recessions, multi-year booms). A Markov chain capture this better than IID returns.
- **Explainability:** "80% of scenarios yield CHF 50k/year in retirement" is more actionable than "expected retirement income is CHF 52k."

## 10.5 Why Five Utility Dimensions?

Research in life satisfaction (Cantril scale, Gallup surveys) shows that humans value:
1. **Consumption** — basic needs and discretionary spending (income).
2. **Leisure** — free time, hobbies, rest.
3. **Family** — time with spouse, children, parents.
4. **Health** — freedom from stress, burnout, illness.
5. **Security** — peace of mind about retirement and emergencies.

The model doesn't claim to capture all human flourishing, but these five are evidence-backed and roughly independent (orthogonal in preference space). Weighting them allows users to express their own priorities.

---

# 11. Performance & Scalability

## 11.1 Computational Complexity

| Operation | Time | Notes |
|-----------|------|-------|
| Single tax lookup | O(1) | Binary search on bracket + linear interpolation |
| Single scenario evaluate | O(1) | ~10 arithmetic operations |
| Grid search (11 scenarios) | O(11) | ~0.5ms |
| Monte Carlo 10k paths | O(10,000 × years) | ~2s for typical 50-year horizon |
| Full optimize command | O(11 + 10,000×50) | ~2s (grid search dominates) |

## 11.2 Memory Usage

- **TaxSchedule:** ~1 KB (handful of tax rates and thresholds).
- **OptimizerConfig:** ~1 KB (structs and scalars).
- **LifeOptimizer:** ~1 MB (when storing 10,000 simulation paths in memory).
- **Overall:** Negligible; single invocation uses < 10 MB.

## 11.3 Scaling Bottlenecks

- **Number of scenarios:** Grid is fixed at 11 (50%–100% in 5% increments). If finer granularity is needed (1% steps), cost grows linearly.
- **Monte Carlo paths:** Main bottleneck. 100k paths takes ~20s; 1M paths takes ~3 minutes. Mitigated by:
  - Adaptive sampling (start with 5k, increase if variance is high).
  - Parallelization (rayon crate for multi-threaded path generation).
  - GPU option (future, using wgpu or similar).

## 11.4 Determinism

All operations are deterministic given fixed random seed. Monte Carlo uses PCG or XORShift RNG, both with deterministic seeding. This allows reproducible results for testing and user-facing scenarios.

---

# 12. Security & Privacy

## 12.1 Data Sensitivity

The tool processes personal financial data:
- Salary, canton, family status, ages, preferences.
- **Threat model:** Local CLI tool; assumes trusted execution environment.
- **No network communication:** All computation happens locally. No data is sent to cloud or third parties.
- **No logging:** Results are printed to stdout; not persisted unless user redirects output.

## 12.2 Numerical Precision

- All money amounts are `f64` (IEEE 754 double precision).
- Sufficient for CHF to the centimes (10^-2); carries ~15 significant digits.
- Tax calculations are accurate to ± 0.01 CHF; acceptable for guidance (not official tax return).

## 12.3 Assumptions & Disclaimers

- Assumes Swiss tax law static over simulation horizon (no legislative changes).
- Assumes BVG fund returns follow historical regime-switching distribution (not predictive).
- Inflation is assumed constant; doesn't account for stagflation scenarios (future work).
- Pension conversion rates and rules may change; tool uses current law.

**User-facing disclaimer:** Tool is for planning and education, not official tax or retirement advice. Users should consult a financial advisor or tax professional for binding decisions.

---

# 13. Building & Development

## 13.1 Build System

```bash
# Debug build (fast compile, slow runtime)
cargo build

# Release build (slow compile, fast runtime; recommended for users)
cargo build --release

# Run directly
cargo run --release -- optimize --salary 150000 --age 40 --canton ZH

# Run tests
cargo test

# Benchmark
cargo bench

# Format
cargo fmt

# Lint
cargo clippy
```

## 13.2 Dependencies

Key crates:
- **clap:** CLI argument parsing.
- **serde / serde_json:** Data serialization (for config files, future web API).
- **colored:** Terminal colors.
- **rand / pcg:** Random number generation for Monte Carlo.
- **rayon:** (Optional) Parallelization for path simulation.

Minimal external dependencies to avoid supply-chain risk and keep build time short.

## 13.3 Code Style

- Follow Rust idioms (clippy-approved).
- Comments on algorithms and tricky logic.
- Module-level documentation (examples in docstrings).
- No unsafe code except in performance-critical inner loops (with comments and justification).

---

# 14. Contributing Guidelines for Future Developers

## 14.1 Before Starting

1. **Read this design doc.** Understand the layered architecture and module responsibilities.
2. **Understand the domain.** Familiarize yourself with Swiss taxes, BVG rules, and life-phase modeling.
3. **Check existing issues & roadmap.** See what's planned (RL, behavioral economics, multi-currency).
4. **Write an issue first.** Propose changes before diving into code.

## 14.2 Code Review Checklist

- **Correctness:** Do the financial calculations match the design doc formulas?
- **Testing:** Are edge cases (zero income, 100% work) handled?
- **Explainability:** Is the logic clear? Are utility components decomposed for transparency?
- **Performance:** Does it add new dependencies or slow down the grid search significantly?
- **Scope:** Does it fit the layered architecture? Or does it blur boundaries (e.g., mixing tax logic into optimizer)?

## 14.3 Common Modifications

### Adding a New Tax Feature
1. Extend `TaxSchedule` with a new field (e.g., `child_tax_credit: f64`).
2. Implement logic in `tax.rs` to apply it.
3. Add test case with known tax outcome.
4. Update README with the new capability.

### Adjusting Utility Weights
1. Modify `PreferenceWeights` struct in `requirements.rs`.
2. Add new profile preset in `main.rs` (e.g., `--profile health-first`).
3. Document weights in `README.md`.

### Changing the Simulation Horizon
1. Update `current_age`, `retirement_age`, `life_expectancy` in config.
2. Run tests to ensure no off-by-one errors in accumulation loops.
3. Update benchmarks if horizon significantly changes (e.g., 100-year projections).

---

# 15. Future Work & Research Directions

## 15.1 Short Term (Next 1–2 Releases)

- **Behavioral economics:** Add loss-aversion (Prospect Theory) utility function.
- **Portfolio optimization:** Allow user to choose asset allocation (conservative, balanced, aggressive) and simulate accordingly.
- **Sensitivity analysis:** Chart how utility changes with tax rate ±2%, inflation ±1%, return volatility ±5%.
- **Web UI:** Wrap CLI in a simple web interface (Rust + WASM, or TypeScript frontend + Rust backend API).

## 15.2 Medium Term (2–3 Years)

- **Reinforcement learning:** Train a policy π(state, age) → work_percentage using historical outcomes. Allows adaptation to personal preferences over time.
- **Cross-country support:** Extend tax module to Germany, France, US. Reuse domain layer; plug in country-specific tax & pension rules.
- **Behavioral feedback:** Integrate with personal finance apps (YNAB, Personal Capital) to validate assumptions against real spending.
- **Interactive scenario builder:** "What if" UI for climate shocks, career disruption, health events.

## 15.3 Long Term (3+ Years)

- **AI co-pilot:** Integrate LLM to explain results in natural language. ("You'd be happier at 80% because you'd gain 20 hours/week of family time, offsetting 5% less income, and your pension is still 95th percentile.")
- **Evolutionary algorithms:** Search for Pareto-optimal policies across entire career (vs. static single-scenario optimization).
- **Quantum computing:** (Speculative) Use quantum annealing for high-dimensional portfolio + work-percentage co-optimization.

---

# 16. Conclusion

Life Optimizer is built on a foundation of transparent mathematics, modular architecture, and respect for user agency. It's designed to answer a complex, personal question with rigor and explainability, not to replace human judgment but to inform it.

The layered design allows contributors to extend the model (new taxes, new life stages, new utility dimensions) without destabilizing the core. The focus on determinism and testability means results are reproducible and auditable. And the commitment to explainability ensures that users can understand and critique the reasoning behind recommendations.

Whether you're a contributor, a user, or a researcher building on this work, this design document provides the foundation for that collaboration.

---

## Appendix A: Glossary

- **BVG:** Berufliche Vorsorge (Swiss occupational pension, "Pillar 2").
- **AHV:** Alters- und Hinterlassenenversicherung (state pension, "Pillar 1").
- **Pillar 3a:** Voluntary private savings for retirement (max CHF 7,056/year in 2024).
- **CVaR:** Conditional Value-at-Risk (expected loss in worst 5% of scenarios).
- **Regime-switching:** Markov chain where economic returns depend on current state (Boom, Normal, Recession, Stagflation).
- **Utility function:** Mathematical function mapping outcomes (income, time, etc.) to happiness/satisfaction scores.
- **Effective tax rate:** Total tax / gross income (includes all federal, cantonal, municipal, and social contributions).
- **Work percentage:** Fraction of full-time equivalent (FTE); 80% = 4 days/week on 5-day calendar.

---

## Appendix B: References & Further Reading

1. **Swiss Tax Law:**
   - Federal Tax Administration (FTA) — official tax tables
   - Cantonal tax offices — municipal surcharges

2. **Pension Rules:**
   - BVG/LPP — Swiss occupational pension law
   - OAK BVG — BVG umbrella organization guidelines

3. **Economics & Optimization:**
   - Kahneman & Tversky (1979) — Prospect Theory
   - Markowitz (1952) — Modern Portfolio Theory
   - Mossin (1966) — Capital Asset Pricing Model (CAPM)
   - Merton & Samuelson (1974) — Portfolio selection with logarithmic utility

4. **Life Satisfaction Research:**
   - Diener et al. (2003) — Subjective well-being and life satisfaction scales
   - Gallup World Poll — cross-country life satisfaction data

---




