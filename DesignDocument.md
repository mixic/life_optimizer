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


---

# 6. Design Rationale

- **Rust** chosen for performance, safety, and deterministic behavior.
- **Layered architecture** ensures modularity and testability.
- **Monte Carlo + deterministic scenarios** provide realism and robustness.
- **Multi-objective optimization** reflects real human trade-offs.
- **Explainability tools** increase trust and transparency.

---

# 7. Future Extensions

- Reinforcement learning for adaptive policies.
- Behavioral economics models.
- Cross-country tax/pension modules.
- Cloud-based API for integrations.

---

# 8. Conclusion

This design provides a scalable, extensible foundation for Life Optimizer. It supports rigorous modeling, transparent decision-making, and future expansion into advanced research areas.



