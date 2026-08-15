# Mathematical Appendix

## 1. Scope and objective

This appendix formalizes the optimization framework used by Life Optimizer to determine an optimal work percentage over the life cycle. The goal is not to claim a universal truth about household decision-making, but to specify a transparent and auditable model that can be challenged, calibrated, and extended.

The framework combines:

- disposable-income optimization under progressive taxation
- household utility over consumption, leisure, family time, health, and security
- pension dynamics under Swiss BVG / AHV logic
- stochastic macroeconomic scenarios and sequence-of-returns stress tests
- a decision rule that selects a work percentage maximizing expected life utility

The model is designed for explainability and robustness, not for opaque optimization or black-box prediction.

---

## 2. Decision variable and planning horizon

Let:

- $t \in \{0, 1, \dots, T\}$ denote time in annual periods
- $T$ be the number of years until retirement
- $\theta_t \in [0, 1]$ be the work percentage in year $t$
- $w_t$ be gross annual salary in year $t$
- $I_t = w_t \theta_t$ be annual labor income in year $t$
- $\tau(I_t)$ be the effective tax function on labor income

The central decision variable is the annual work percentage $\theta_t$.

The optimization problem is therefore a dynamic multi-objective policy problem over the life cycle, rather than a static one-period choice.

---

## 3. Optimization problem

The objective is to maximize expected lifetime utility:

$$
\max_{\{\theta_t\}_{t=0}^{T}} \; \mathbb{E}\left[\sum_{t=0}^{T} \beta^t \, u(c_t, l_t, f_t, h_t, s_t; z_t)\right]
$$

subject to:

1. **Weekly budget constraint:**

   $$c_t = (1 - \tau(I_t)) I_t + y_t - r_t - p_t$$

   where:
   - $c_t$ = consumption expenditure (£/week)
   - $I_t$ = gross labor income (£/week)
   - $\tau(I_t)$ = effective tax rate on income $I_t$
   - $y_t$ = non-labor income (pension, investments, etc.; £/week)
   - $r_t$ = required recurring expenses (mortgage, utilities, insurance; £/week)
   - $p_t$ = discretionary/period-specific outflows (savings, debt repayment; £/week)

2. **Weekly time budget constraint:**

   $$l_t + f_t + h_t^{\text{work}} + h_t^{\text{sleep}} + h_t^{\text{other}} = 168$$

   where:
   - $l_t$ = leisure time (hours/week)
   - $f_t$ = family/caregiving time (hours/week)
   - $h_t^{\text{work}}$ = paid work time (hours/week)
   - $h_t^{\text{sleep}}$ = sleep (typically 49–56 hours/week)
   - $h_t^{\text{other}}$ = personal care, commute, administration (hours/week)

3. **Work intensity identity:**

   $$h_t^{\text{work}} = \theta_t H_{\text{full}}$$

   where:
   - $\theta_t \in [0,1]$ = employment intensity (fraction of full-time)
   - $H_{\text{full}}$ = standard full-time hours per week (typically 37–40)

4. **Consumption adequacy constraint:**

   $$c_t \geq R_t$$

   where $R_t$ = minimum required consumption to meet basic needs and household obligations at time $t$

5. **Retirement income adequacy:**

   $$P_{T+1} \geq P_{\min}$$

   where:
   - $P_{T+1}$ = total retirement income stream (pension + other sources; £/week)
   - $P_{\min}$ = target retirement income replacement rate (typically 60–80% of working income)

6. **Non-negativity and feasibility bounds:**

   $$0 \leq \theta_t \leq 1, \quad c_t \geq 0, \quad l_t \geq 0, \quad f_t \geq 0, \quad p_t \geq 0$$

### Interpretation

This formulation emphasizes that the decision is not purely about income maximization. The household solves a trade-off between:

- income and consumption
- leisure and family time
- health stress and productivity
- short-term material comfort and long-term pension safety

---

## 4. Utility function

The period utility function is defined as:

$$
 u(c_t, l_t, f_t, h_t, s_t; z_t) =
 w_c u_c(c_t) + w_l u_l(l_t) + w_f u_f(f_t, z_t) + w_h u_h(h_t) + w_s u_s(s_t)
 $$

with weights satisfying:

$$
 w_c + w_l + w_f + w_h + w_s = 1,
 \quad w_i \geq 0
 $$

where:

- $u_c$ is consumption utility
- $u_l$ is leisure utility
- $u_f$ is family-time utility
- $u_h$ is health / stress penalty utility
- $u_s$ is security utility
- $z_t$ represents context variables such as life stage, children, and household composition

This structure intentionally makes the trade-offs explicit rather than latent.

### 4.1 Consumption utility

A standard concave specification is used:

$$
 u_c(c_t) = \ln\left(\frac{c_t}{R_t}\right) \quad \text{if } c_t \geq R_t
 $$

and a penalty is applied if household consumption falls below required needs:

$$
 u_c(c_t) = \ln\left(\frac{c_t}{R_t}\right) - \lambda_c \quad \text{if } c_t < R_t
 $$

with $\lambda_c > 0$ capturing severe hardship.

This specification reflects diminishing marginal utility and the fact that a household's marginal value of income falls as consumption rises.

### 4.2 Leisure utility

Leisure utility is modeled as a concave function:

$$
 u_l(l_t) = K_l \left(\frac{l_t}{L_{ref}}\right)^\alpha
 $$

with:

- $L_{ref}$ as a reference leisure level
- $0 < \alpha < 1$ as a concavity parameter

This implies diminishing marginal utility of additional leisure time.

### 4.3 Family utility

Family utility depends on the household life stage and the time available for family care and emotional presence:

$$
 u_f(f_t, z_t) = K_f \eta(z_t) \left(\frac{f_t}{F_{ref}}\right)^\beta
 $$

where:

- $\eta(z_t)$ is a life-stage multiplier
- $0 < \beta < 1$ is the family-time concavity parameter

A reasonable calibration uses higher values of $\eta(z_t)$ during early parenthood and lower values in later life stages with different time demands.

### 4.4 Health utility

Health stress is introduced as a convex penalty on work intensity:

$$
 u_h(h_t) = -\kappa \left(\frac{h_t^{work}}{H_{full}}\right)^\gamma
 $$

with:

- $\gamma > 1$ so that work stress is convex and increasingly costly at higher levels of overwork
- $\kappa > 0$ scaling the health penalty

This allows the model to encode the fact that work stress is not linear: very high work percentages can disproportionately damage health and reduce long-term welfare.

### 4.5 Security utility

Security utility captures pension adequacy and financial resilience:

$$
 u_s(s_t) = \min\left(1, \frac{P_t}{P_{target}}\right) \cdot K_s
 $$

where:

- $P_t$ is projected pension income at time $t$
- $P_{target}$ is a target replacement rate or adequacy threshold
- $K_s$ is a normalization constant

This term prevents the optimizer from selecting a high-leisure option that would create unacceptable pension risk.

---

## 5. Tax function

The tax function is designed to approximate effective tax burden on labor income under Swiss rules.

Let gross taxable income be $I$. Then effective tax burden is:

$$
 \tau(I) = \frac{T_{federal}(I) + T_{cantonal}(I) + T_{communal}(I) + T_{social}(I)}{I}
 $$

where each component is defined separately.

### 5.1 Federal progressive tax

For a progressive tax schedule with brackets $[b_0, b_1, \dots, b_n]$ and marginal rates $r_1, \dots, r_n$,

$$
 T_{federal}(I) = \sum_{i=1}^{n} r_i \cdot \max\{0, \min(I, b_i) - b_{i-1}\}
 $$

This is explicit and economically interpretable.

### 5.2 Cantonal and communal tax

Cantonal and local taxes are approximated as proportional or semi-progressive components:

$$
 T_{cantonal}(I) = \alpha_c(I, z) \cdot I
 $$

$$
 T_{communal}(I) = \alpha_m \cdot T_{cantonal}(I)
 $$

with $z$ capturing local household and tax context such as marital status, canton, and children.

### 5.3 Social security contributions

Mandatory social contributions are modeled as:

$$
 T_{social}(I) = I \cdot (r_{AHV} + r_{ALV} + r_{EO})
 $$

This component is important because it is not merely a tax, but a mandatory social insurance deduction that materially affects labor supply and pension accumulation.

### 5.4 Assumptions and limitations

The tax function is intentionally simplified for transparency. In practice, Swiss taxes vary by canton, municipality, deductions, and household structure. Therefore, the model should allow:

- municipality-specific calibration
- canton-specific tax approximations
- override parameters for empirical income statements
- explicit reporting of assumptions used in each computation

This is preferable to pretending a single formula accurately represents all Swiss tax contexts.

---

## 6. Pension model

The model combines state and occupational pension elements.

### 6.1 AHV / Pillar 1

The state pension is approximated as:

$$
 P_{AHV} = \min\{P_{AHV}^{max}, \lambda_{AHV} \cdot \bar{I}_{career}\}
 $$

where:

- $P_{AHV}^{max}$ is a capped pension level
- $\bar{I}_{career}$ is average indexed annual income over the working life
- $\lambda_{AHV}$ is a pension replacement factor

### 6.2 BVG / Pillar 2

Occupational pension contributions are modeled as:

$$
 C_t = I_t \cdot \theta_t \cdot r_{BVG}
 $$

with annual capital accumulation:

$$
 K_{t+1} = K_t (1 + r_t) + C_t
 $$

where:

- $r_t$ is the annual return on pension assets
- $r_{BVG}$ is the contribution rate

At retirement, capital is converted into an annual pension using a conversion coefficient $\gamma$:

$$
 P_{BVG} = \gamma K_{retirement}
 $$

The actual value of $r_t$ is stochastic and should be modeled under a regime-aware or Monte Carlo framework.

### 6.3 Total retirement income

Total pension income is:

$$
 P_{total} = P_{AHV} + P_{BVG}
 $$

The adequacy constraint is then checked against a target replacement ratio or minimum required retirement needs.

---

## 7. Requirements and living-cost function

The household requirement function is defined as:

$$
 R_t = R_{housing} + R_{food} + R_{transport} + R_{insurance} + R_{health} + R_{childcare} + R_{education} + R_{discretionary}
 $$

This requirement is life-stage dependent:

$$
 R_t = R_{base} \cdot \phi(z_t)
 $$

where $\phi(z_t)$ is a multiplier depending on family composition and life stage.

This is essential because the same income level does not imply the same standard of living across different life phases.

---

## 8. Stochastic macroeconomic environment

The projection framework includes regime-dependent returns and macroeconomic states. Let the state variable be:

$$
 X_t \in \{Boom, Normal, Recession, Stagflation\}
 $$

with transition matrix:

$$
 P(X_{t+1} = j \mid X_t = i) = p_{ij}
 $$

The annual return process is then modeled as:

$$
 r_{t+1} = \mu_{X_t} + \sigma_{X_t} \epsilon_t,
 \quad \epsilon_t \sim \mathcal{N}(0,1)
 $$

This approach allows shocks to cluster and regimes to persist, which is more realistic than a single constant-volatility process.

The important point is not just that volatility exists, but that economic states are persistent and regime-dependent.

---

## 9. Sequence-of-returns risk

A key retirement risk is sequence-of-returns risk. Let retirement begin at year $T_R$.

The model should evaluate outcomes under a stress scenario in which a negative macroeconomic regime occurs in the years directly before and after retirement:

$$
 r_{T_R-2}, r_{T_R-1}, r_{T_R}, r_{T_R+1} \text{ are negative or unusually weak}
 $$

This timing matters because withdrawal risk is highest precisely when the portfolio has not yet had time to recover.

This is one of the clearest examples of why a simple expected-return model is insufficient.

---

## 10. Solution methods

### 10.1 Grid search

The current implementation uses a discrete search over candidate work percentages:

$$
 \Theta = \{0.5, 0.6, 0.7, 0.8, 0.9, 1.0\}
 $$

and selects:

$$
 \theta^* = \arg\max_{\theta \in \Theta} \mathbb{E}[U(\theta)]
 $$

This method is transparent and simple, but it has limitations:

- it only identifies the optimum within a grid
- it may miss local optima in a richer utility surface
- it scales poorly when decision variables become multidimensional

### 10.2 Continuous optimization

For a continuous formulation, one may use:

$$
 \nabla U(\theta) = \frac{\partial U}{\partial \theta}
 $$

and apply methods such as:

- gradient descent
- quasi-Newton methods
- L-BFGS
- dynamic programming for richer state spaces

This is mathematically valid, but the model must still be constrained by economic realism and interpretability.

### 10.3 Dynamic programming

A richer formulation is a Bellman equation of the form:

$$
V(t, x_t) = \max_{\theta_t \in [0,1]} \left\{ u_t + \beta \mathbb{E}\left[V(t+1, x_{t+1})\right] \right\}
$$

where $x_t$ is the state vector containing relevant household and financial conditions.

This is more general and can handle dynamic decision making, but it is also more computationally demanding and requires stronger calibration discipline.

---

## 11. Calibration logic

The model is only useful if its parameters are interpretable and defensible. Calibration should be conducted in a transparent manner.

### 11.1 Utility weights

Preference weights should be informed by:

- revealed preference data
- life-satisfaction studies
- behavioral studies on work-life trade-offs
- region-specific household data

A plausible structure is:

$$
 w_c, w_l, w_f, w_h, w_s \in [0,1]
 $$

with the constraint that their sum is one.

### 11.2 Discount factor

The discount factor is usually specified as:

$$
 \beta = \frac{1}{1+\rho}
 $$

with $\rho$ between 1% and 5% depending on the decision context.

### 11.3 Health and family parameters

Parameters for health stress and family multipliers should be estimated or at least justified using empirical household evidence, not chosen arbitrarily.

### 11.4 Tax parameters

Tax schedules should be calibrated to official cantonal and municipal rules, and with explicit ability to override using observed personal tax filings.

### 11.5 Macro parameters

Regime transition matrices and return distributions should be estimated from historical Swiss or developed-market data, with robustness checks across different windows.

This is essential: a regime model without calibration is merely a stylized narrative, not a model with decision relevance.

---

## 12. Validation strategy

Validation is critical. A model can be elegant and still be wrong.

### 12.1 Internal model checks

The model should be checked for:

- monotonicity of utility functions
- feasibility of budget and time constraints
- feasibility of pension adequacy thresholds
- numerical stability of optimization across parameter values

### 12.2 Historical backtesting

The macro model should be compared against historical data for:

- inflation paths
- salary growth
- return distributions
- crisis episodes
- growth and recession clusters

### 12.3 Stress testing

A credible model must evaluate adverse scenarios such as:

- prolonged recession
- high inflation
- sudden rate shifts
- early retirement under poor sequence-of-returns conditions

### 12.4 Sensitivity analysis

The project should quantify how output changes when key parameters are perturbed.

For example:

- small changes in discount rate
- moderate changes in tax rates
- alternative family utility weights
- different pension adequacy targets

This reveals whether the recommendation is robust or fragile.

### 12.5 Model comparison

Different model families should be compared under the same conditions:

- GBM baseline
- OU/Vasicek dynamics
- regime-switching model
- jump-diffusion model
- hybrid model

The objective is not simply to select the most complex model, but the one that provides the best balance of realism, calibration quality, and computational tractability.

---

## 13. Strategic framing

This project should be interpreted as an explicit decision-support system rather than a purely predictive model. Its central value is that it makes assumptions visible and contestable.

This matters because modern AI and black-box optimization tools can generate attractive outputs quickly, but they often do so without revealing:

- which assumptions matter most
- how sensitive the recommendation is to those assumptions
- whether the model remains robust under stress
- whether the result is economically plausible

A transparent model, even if simpler, can still be more valuable than a complex but opaque one.

The core strategic objective is therefore not to maximize complexity, but to maximize credibility, interpretability, and decision quality.

---

## 14. Current limitations and project risks

The project currently has several risks that should be acknowledged explicitly:

1. Parameter choices may be too heuristic.
2. Tax modeling may be insufficiently local or institution-specific.
3. Utility assumptions may not be empirically grounded.
4. Stochastic processes may underrepresent tail risk and dependence.
5. Validation may be weaker than a serious quantitative framework requires.
6. The model may appear more precise than it actually is.

These limitations are not fatal, but they do mean that the project should be framed as a transparent research and planning framework, not as a definitive financial adviser.

---

## 15. Summary

The formal structure of Life Optimizer can be read as a constrained intertemporal utility optimization problem with a stochastic pension environment and explicit household preferences. This is a coherent and defensible framework for decision support.

However, the model must be strengthened in four areas to become genuinely credible:

1. stronger calibration logic
2. more realistic macroeconomic dynamics
3. explicit validation and sensitivity analysis
4. disciplined model selection and benchmarking

The project is strongest when it is framed as transparent and explainable quantitative planning, rather than as a black-box optimization engine. That is the right strategic position for a tool intended to support major life decisions.
