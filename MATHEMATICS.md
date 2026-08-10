# Mathematical Appendix

## Complete Mathematical Formulation

### 1. Optimization Problem

**Maximize:**
```
U = Σ(t=0 to T) β^t × u(c_t, l_t, f_t, h_t, s_t)
```

**Subject to:**
```
1. c_t ≤ w_t × θ_t × (1 - τ(w_t × θ_t))              [Budget constraint]
2. h_work_t = θ_t × H_full                            [Work hours]
3. l_t + f_t + h_work_t + h_sleep = 168              [Time constraint]
4. c_t ≥ R_t                                          [Requirements]
5. Σ P_t ≥ P_min                                      [Pension adequacy]
6. 0 ≤ θ_t ≤ 1                                       [Work percentage bounds]
```

**Where:**
- `U` = Lifetime utility
- `β` = Discount factor (1/(1+r), typically r=0.03)
- `t` = Time period (year)
- `T` = Years until retirement
- `c_t` = Consumption at time t
- `l_t` = Leisure time at time t
- `f_t` = Family time at time t
- `h_t` = Health status at time t
- `s_t` = Security (savings/pension) at time t
- `w_t` = Annual salary at time t
- `θ_t` = Work percentage at time t (decision variable)
- `τ(·)` = Tax function
- `R_t` = Personal requirements at time t
- `P_t` = Pension contributions at time t
- `H_full` = Full-time work hours (42/week)

### 2. Tax Function τ(I)

Swiss tax system (simplified):

```
τ(I) = [T_federal(I) + T_cantonal(I) + T_communal(I) + T_social(I)] / I
```

**Federal Tax (Progressive):**
```
T_federal(I) = Σ max(0, min(I, B_i) - B_{i-1}) × r_i
```

Where brackets B and rates r for single person:
```
B_0 = 0,      r_0 = 0.00
B_1 = 25,000, r_1 = 0.01
B_2 = 50,000, r_2 = 0.05
B_3 = 75,000, r_3 = 0.08
B_4 = 100,000,r_4 = 0.11
B_5 = 150,000,r_5 = 0.13
```

**Cantonal Tax (Zürich):**
```
T_cantonal(I) = I × (0.08 - 0.005 × n_children)
```

**Communal Tax:**
```
T_communal(I) = T_cantonal(I) × m_communal
```
Where `m_communal = 1.19` for Zürich city

**Social Security:**
```
T_social(I) = I × (r_AHV + r_ALV + r_BVG)
            = I × (0.0525 + 0.011 + 0.083)
            = I × 0.1465
```

**Total Effective Tax Rate:**
```
τ(I) = [Federal + Cantonal + Communal + Social] / I
```

### 3. Utility Function u(c, l, f, h, s)

**Weighted sum of component utilities:**
```
u = w_c × u_c(c) + w_l × u_l(l) + w_f × u_f(f) + w_h × u_h(h) + w_s × u_s(s)
```

Where `Σ w_i = 1` (preference weights)

**Component Utilities:**

**3.1 Consumption Utility (Log utility):**
```
u_c(c) = ln(c / R)  if c ≥ R
       = ln(c / R) - 5  if c < R  [penalty for not meeting needs]
```

Exhibits diminishing marginal utility: each additional CHF provides less satisfaction.

**3.2 Leisure Utility (Concave power function):**
```
u_l(l) = (l / l_max)^α × K_l

Where:
- l_max = 80 hours/week (reference)
- α = 0.7 (concavity parameter)
- K_l = 10 (scaling constant)
```

Diminishing returns: 10th hour of leisure worth more than 80th.

**3.3 Family Utility (Life stage dependent):**
```
u_f(f, stage) = (f / f_max)^β × η(stage) × K_f

Where:
- f_max = 80 hours/week
- β = 0.8 (slightly less concave than leisure)
- η(stage) = time value multiplier for life stage
- K_f = 10 (scaling constant)
```

**Time Value Multiplier η(stage):**
```
η = 0.8  for YoungSingle
η = 0.9  for YoungCouple
η = 1.5  for NewParent (young children)
η = 1.3  for SchoolAge
η = 1.0  for Teenagers
η = 1.1  for EmptyNest
η = 1.2  for PreRetirement
```

**3.4 Health Utility (Stress penalty):**
```
u_h(h_work, stage) = K_h - σ(h_work) / ρ(stage)

Where:
σ(h_work) = (h_work / H_full)^2 × 5  [stress factor, convex]
ρ(stage) = stress tolerance for life stage
K_h = 10
```

**Stress Tolerance ρ(stage):**
```
ρ = 1.2  for YoungSingle (can handle more)
ρ = 0.6  for NewParent (already stressed)
ρ = 0.7  for PreRetirement (health concerns)
```

**3.5 Security Utility (Pension adequacy):**
```
u_s(P_total) = min(10, 10 × R_replace / R_target)

Where:
R_replace = P_total / I_current  [replacement rate]
R_target = 0.60                   [target 60% replacement]
```

### 4. Pension Calculation

**Swiss 3-Pillar System:**

**Pillar 1 (AHV - State pension):**
```
P_AHV = min(max_AHV, 0.30 × I_avg)

Where:
max_AHV = 2,450 CHF/month (single, 2024)
I_avg = average indexed income over career
```

**Pillar 2 (BVG - Occupational pension):**
```
P_BVG_total = Σ C_t × (1 + r)^(T-t)

Where:
C_t = I_t × θ_t × r_BVG  [annual contribution]
r_BVG = 0.083             [contribution rate]
r = 0.02                  [assumed return]

P_BVG_annual = P_BVG_total × γ

Where:
γ = 0.068  [conversion rate at age 65]
```

**Total Pension:**
```
P_total = P_AHV + P_BVG_annual
```

### 5. Requirements Function R_t

**Base Requirements:**
```
R_t = R_housing + R_food + R_transport + R_insurance + 
      R_childcare + R_healthcare + R_education + 
      R_vacation + R_savings + R_discretionary
```

**Life Stage Adjustments:**
```
R_t(stage) = R_base × Φ(stage)

Where Φ(stage) is adjustment matrix:
```

| Category   | Single | Couple | NewParent | Teenagers | EmptyNest |
|------------|--------|--------|-----------|-----------|-----------|
| Housing    | 0.7    | 1.0    | 1.0       | 1.0       | 0.8       |
| Food       | 0.7    | 1.0    | 1.2       | 1.3       | 0.7       |
| Childcare  | 0.0    | 0.0    | 1.5       | 0.5       | 0.0       |
| Education  | 0.0    | 0.0    | 1.0       | 1.5       | 0.0       |

### 6. Solution Methods

**6.1 Grid Search (Current Implementation):**
```
θ* = argmax{θ ∈ Θ} U(θ)

Where:
Θ = {0.5, 0.6, 0.7, 0.8, 0.9, 1.0}  [candidate set]
```

**Complexity:** O(|Θ| × T)

**Advantages:**
- Simple, guaranteed to find solution in discrete set
- Easy to visualize and compare

**Disadvantages:**
- May miss optimal between grid points
- Not scalable to many dimensions

**6.2 Gradient-Based Optimization:**

For continuous θ ∈ [0, 1]:

```
∇U(θ) = ∂U/∂θ = Σ β^t × ∂u/∂θ_t

∂u/∂θ_t = (∂u/∂c_t × ∂c_t/∂θ_t) + 
          (∂u/∂l_t × ∂l_t/∂θ_t) + 
          (∂u/∂h_t × ∂h_t/∂θ_t) + 
          (∂u/∂s_t × ∂s_t/∂θ_t)
```

Could use gradient descent, Newton's method, or L-BFGS.

**6.3 Dynamic Programming:**

Bellman equation:
```
V(t, W) = max{θ_t} {u(c_t, l_t, ...) + β × V(t+1, W')}

Where:
W = wealth state
W' = W + (c_t - R_t)  [updated wealth]
```

**State space:** (age, wealth)
**Action:** work percentage θ
**Transition:** deterministic given θ

### 7. Comparative Statics

**Effect of income increase:**
```
∂θ*/∂w > 0  or  < 0 ?

Competing effects:
(+) Income effect: Can afford to work less
(-) Substitution effect: Opportunity cost of leisure increases
```

Empirically: Often ∂θ*/∂w < 0 for high earners (income effect dominates)

**Effect of tax increase:**
```
∂θ*/∂τ < 0  [unambiguous]

Higher taxes reduce after-tax income, making work less attractive.
```

**Effect of children:**
```
∂θ*/∂n_children < 0  [typically]

Children increase:
1. Requirements R ↑
2. Time value η ↑

First effect pushes toward more work, second toward less.
Empirically, time value dominates → work less.
```

### 8. Sensitivity Analysis

**Taylor expansion around optimal:**
```
U(θ* + ε) ≈ U(θ*) + ∇U(θ*) × ε + (1/2) × ε^T × H × ε

Where:
H = Hessian matrix (second derivatives)
```

If H is negative definite, θ* is local maximum.

**Elasticity of utility w.r.t. work percentage:**
```
E_θ = (∂U/∂θ) × (θ/U)
```

### 9. Extensions

**9.1 Uncertainty:**

Stochastic income:
```
w_t = w̄ × exp(σ × ε_t)

Where ε_t ~ N(0,1)
```

Requires dynamic programming with expectation:
```
V(t, W) = max{θ_t} E[u(...) + β × V(t+1, W')]
```

**9.2 Career Effects:**

Promotion probability:
```
P(promote | θ_t) = p_0 × θ_t^γ

Where γ > 1 [superlinear: part-time hurts career]
```

**9.3 Health Dynamics:**

Health stock evolution:
```
H_{t+1} = (1 - δ) × H_t + φ(l_t) - ψ(h_work_t)

Where:
δ = depreciation rate
φ = health investment function (leisure)
ψ = health cost function (work stress)
```

### 10. Calibration

**Preference weights estimated from:**
- Revealed preference (observed choices)
- Stated preference (surveys)
- Life satisfaction studies

**Typical values (literature):**
```
w_consumption ≈ 0.25-0.35
w_leisure ≈ 0.15-0.25
w_family ≈ 0.20-0.30
w_health ≈ 0.10-0.20
w_security ≈ 0.10-0.20
```

**Discount rate β:**
```
β = 1/(1+ρ)

Where ρ ≈ 0.03 (3% annual time preference)
```

From studies: ρ ranges 0.01-0.05

**Utility function parameters:**
```
α (leisure concavity) ≈ 0.6-0.8
β (family concavity) ≈ 0.7-0.9
stress_exponent ≈ 1.5-2.5
```

### 11. Validation

**Cross-validation approaches:**

1. **Out-of-sample prediction:**
   - Estimate on subset of population
   - Predict choices for holdout set
   - Compare predicted vs actual

2. **Life satisfaction regression:**
   ```
   LS_i = α + β_1 × U_i + β_2 × X_i + ε_i
   
   Where:
   LS_i = reported life satisfaction
   U_i = model-predicted utility
   X_i = controls
   ```
   
   Expect β_1 > 0 and significant.

3. **Behavioral consistency:**
   - Do people who follow recommendations report higher satisfaction?
   - Longitudinal studies

### 12. Computational Complexity

**Current implementation:**
- Grid points: |Θ| = 6
- Time periods: T = 35 (age 30-65)
- Evaluations: O(|Θ| × T) = O(210)
- Very fast: < 1ms

**Full dynamic programming:**
- State space: age × wealth × health
- Dimensions: 35 × 100 × 10 = 35,000 states
- Complexity: O(|Θ| × |S|^2) with value iteration
- Still tractable

**Continuous optimization:**
- Variables: θ_t for t=1..T = 35 variables
- Nonlinear, non-convex problem
- Gradient descent: O(iterations × T)
- Typically converges in 10-50 iterations

### References

1. Layard, R. (2005). Happiness: Lessons from a New Science.
2. Blanchflower, D., & Oswald, A. (2004). Well-being over time in Britain and the USA.
3. Swiss Federal Tax Administration (ESTV). Federal Tax Rates.
4. Swiss Federal Social Insurance Office. AHV/IV contribution rates.
5. Mas-Colell, A., Whinston, M., & Green, J. (1995). Microeconomic Theory.
6. Ljungqvist, L., & Sargent, T. (2018). Recursive Macroeconomic Theory.
