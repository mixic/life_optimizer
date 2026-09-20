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

### 3.1 Consumption and lifestyle extension

The minimum requirement $R_t$ should be decomposed into unavoidable costs and lifestyle-dependent spending. Let:

- $q_t \in \{E, M, N, L\}$ be the consumption profile: extreme saving, moderate, normal, or luxury;
- $H_t$ be essential household costs, including rent or mortgage, utilities, insurance, food, and debt obligations;
- $D_t$ be a reference level of discretionary spending; and
- $m(q_t)$ be the lifestyle multiplier applied to discretionary spending.

Define total consumption expenditure as:

$$
C_t = H_t + m(q_t)D_t,
$$

with an illustrative ordering:

$$
m(E) < m(M) < m(N) < m(L).
$$

The multiplier should be calibrated from the user's actual budget rather than treated as a universal judgment about lifestyle. Rent is included in $H_t$ because it is usually a recurring cost that does not automatically fall when the work percentage falls.

The household's annual savings capacity is then:

$$
S_t = (1 - \tau(I_t))I_t + y_t - C_t - p_t,
$$

where $p_t$ represents additional planned outflows such as voluntary pension contributions, investments, or reserve funding. The financial feasibility condition becomes:

$$
S_t \geq S_{\min,t},
$$

where $S_{\min,t}$ may be zero for a survival scenario or positive when the household must maintain an emergency reserve and pension contributions.

For a proposed work percentage $\theta_t$, the model should therefore test:

$$
(1 - \tau(I_t))I_t + y_t \geq H_t + m(q_t)D_t + p_t + S_{\min,t}.
$$

This equation makes the effect of consumption explicit. The same 80% work schedule can be feasible under an extreme-saving profile and infeasible under a luxury profile, even when salary, taxes, and market returns are identical.

An example of a simple initial calibration is:

| Profile | Interpretation | Illustrative discretionary multiplier |
|---|---|---:|
| Extreme saving ($E$) | Minimal discretionary spending and strict cost control | 0.50 |
| Moderate ($M$) | Controlled spending with some flexibility | 0.80 |
| Normal ($N$) | Ordinary expected standard of living | 1.00 |
| Luxury ($L$) | Premium services, travel, and high discretionary spending | 1.75 |

These values are starting parameters for scenario analysis, not claims about objectively correct spending levels.

#### 3.1.1 Implemented form: elasticity tiers and the sparing multiplier

The model as implemented splits the basket into three elasticity tiers and
prices the elastic tier at its sparing-adjusted effective cost
(`THEORY_OF_SPARING.md` §7c, §8):

$$
C_t = \underbrace{H_t}_{\text{inelastic}} + \underbrace{Q_t}_{\text{quasi-inelastic}} + \underbrace{L_t}_{\text{elastic}} + \underbrace{D_t}_{\text{committed outflows}}
$$

with the elastic tier multiplied by:

$$
m = m_{\text{profile}} \cdot \bigl[1 - \sigma(1 - \phi)\bigr] \cdot \underbrace{\frac{\rho_{\text{ref}}}{\rho_{\text{use}}(d)}}_{\text{utilization penalty}}
$$

where:

- $m_{\text{profile}}$ is the lifestyle multiplier from the table above
- $\sigma \in [0,1]$ is the sparing ratio
- $\phi = 0.60$ is the second-hand price ratio
- $d \in [0,1]$ is utilization discipline, and
  $\rho_{\text{use}}(d) = 0.65 + (0.85 - 0.65)d$
- $\rho_{\text{ref}} = 0.85$

**Normalization.** The utilization penalty is divided by $\rho_{\text{ref}}$ so
that full discipline yields exactly $1.0$. Without this normalization a
household doing no sparing at all would still be charged a penalty, which
silently inflated the reference basket and made every household look less
affordable than it is. The neutral point is therefore:

$$
m = 1 \iff \text{normal profile} \;\wedge\; \sigma = 0 \;\wedge\; d = 1
$$

**Feasibility uses the mandatory floor, not the full basket.** The quantities
are separated explicitly:

$$
\underbrace{C^{\text{mandatory}}_t = H_t + Q_t}_{\text{feasibility test}}
\qquad\text{vs}\qquad
\underbrace{C^{\text{target}}_t = C^{\text{mandatory}}_t + L_t + D_t}_{\text{consumption-utility ratio}}
$$

This distinction is what makes the §3.1 claim true — that the same work
percentage "can be feasible under an extreme-saving profile and infeasible under
a luxury profile". It also corrects an earlier conflation in which the savings
goal and discretionary spending were treated as unavoidable obligations, causing
ordinary households (e.g. CHF 120k with two children) to be reported as having
no affordable option at any work percentage.

The breakdown by tier is reported in the CLI so the person can see *where* the
squeeze is landing, as §7c argues: a household with rising inelastic costs can
practice maximum sparing on the elastic tier and still see little movement in
total consumption.

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

#### 6.2.1 Conversion rate scenarios

$\gamma$ — the *Umwandlungssatz* — is not a single known number. The statutory
BVG minimum is $\gamma = 6.8\%$, but many Swiss pension funds apply materially
less because the same capital must fund a longer retirement. Reporting a single
pension figure therefore overstates precision, so $\gamma$ is treated as a
scenario parameter with three reference values plus an optional user-supplied rate:

| Scenario | $\gamma$ | Meaning |
|---|---|---|
| Statutory | 6.8% | BVG Art. 14 minimum |
| Fund-typical | 5.5% | Rate commonly applied by Swiss Pensionskassen |
| Future projection | $\gamma(t)$ | Forward projection, see below |
| Custom | user input | The person's actual fund rate |

The forward projection follows a linear reduction, floored at 5.0%:

$$
 \gamma(t) = \max\Big(\gamma_0 - (t - t_0)\,\Delta\gamma,\ \gamma_{min}\Big)
 $$

with $\gamma_0 = 0.068$, $t_0 = 2024$, $\Delta\gamma = 0.00036$ (0.036 percentage
points per year), and $\gamma_{min} = 0.05$. This yields 6.58% by 2030, 6.22% by
2040, and 5.86% by 2050; the floor binds from about 2074.

Deferred or early retirement scales whichever base rate applies, because
annuitizing over fewer expected remaining years raises the annually payable
rate independently of the base:

$$
 \gamma_{\text{eff}}(a, t) = \gamma(t) \cdot \phi(a),
 \qquad
 \phi(65) = 1,\quad \phi(70) = \tfrac{0.078}{0.068},\quad \phi(62) = \tfrac{0.050}{0.068}
 $$

The monthly pension used for display and adequacy checks is:

$$
 P_{\text{monthly}} = \frac{\gamma_{\text{eff}} \cdot K_{retirement}}{12}
 $$

Because several scenarios are computed simultaneously, the reported outcome is a
range rather than a point estimate:

$$
 \big[\min_s P_{\text{monthly}}(s),\ \max_s P_{\text{monthly}}(s)\big]
 \quad\text{over all scenarios } s
$$

The statutory-vs-typical gap $P_{BVG}(6.8\%) - P_{BVG}(5.5\%)$ is reported
explicitly, since it is the amount by which the statutory rate flatters the
outcome for someone whose fund applies the typical rate.

Both the optimizer's security-utility term and the Monte Carlo projection call
the same `effective_conversion_rate()` function, so the two engines cannot
disagree about which $\gamma$ is assumed.

#### 6.2.2 Stochastic conversion rate and downside risk

Because a single point value for $\gamma$ implies a precision the input does not
have, the rate is also modelled as a random variable centred on the projection:

$$
\gamma \sim \mathcal{N}\!\left(\gamma(t),\ \sigma_\gamma^2\right)
\quad\text{truncated to}\quad
\bigl[\gamma_{\min},\ \gamma_0\bigr]
$$

with $\sigma_\gamma = 0.010$, $\gamma_{\min} = 0.05$ and $\gamma_0 = 0.068$.
Truncation is deliberate: an untruncated normal would generate conversion rates
above the statutory minimum or below the projection floor, which no Swiss fund
applies. Because $P = \gamma K/12$ is linear in $\gamma$, uncertainty in the rate
translates one-for-one into proportional uncertainty in the pension.

Downside risk is reported as conditional value-at-risk over the worst decile:

$$
\mathrm{CVaR}_{10\%} = \mathbb{E}\bigl[P \mid P \leq q_{10\%}\bigr]
$$

This is reported alongside the percentiles rather than instead of them, because
they answer different questions: $q_{10\%}$ is the level exceeded in 90% of
draws, while $\mathrm{CVaR}_{10\%}$ is the average outcome *given* that the
household lands in the worst decile.

Note that $\gamma_{\text{eff}}$ is excluded from this term: the band describes
dispersion in the *projected* rate, so it is centred on $\gamma(t)$ regardless
of which scenario the user selected for the headline figure. The two are
independent inputs — the user's own fund rate changes the headline, not the
uncertainty band, whose purpose is to show what is at stake if that rate is
unknown.

**Limitation.** Truncation compresses both tails, which pulls
$\mathrm{CVaR}_{10\%}$ toward $q_{10\%}$ and understates the severity a true
fund-level distribution would show. $\sigma_\gamma$ is an assumption, not a
fitted dispersion; estimating it from observed fund rates is the calibration work
described in §11.5.

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
V(t, x_t) = \max_{\theta_t \in [0,1]} \bigl\lbrace u_t + \beta \mathbb{E}\bigl[V(t+1, x_{t+1})\bigr] \bigr\rbrace
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

## 14. AI-driven productivity and the future of work

The critique from Mr. Gradimir Nikolic can be extended into a forward-looking AI scenario. AI should not be modeled only as a reason to reduce work hours. It may increase output expectations, change employment risk, alter the value of human skills, and create new psychological and social constraints.

### 14.1 AI-adjusted achievement capacity

Let $a_t \geq 0$ represent the productivity contribution of AI, and let $\rho_t$ represent the worker's baseline productivity. Effective project achievement capacity is:

$$
A_t = \theta_t H_{full} \rho_t (1 + a_t).
$$

The worker can satisfy the employer's required project output $G_t$ only when:

$$
A_t \geq G_t.
$$

This captures the main philosophical point: a reduction in scheduled work is justified only when AI-assisted capacity still meets the required outcomes. However, employers may respond to higher productivity by increasing $G_t$. Therefore, AI does not automatically translate into shorter work.

#### 14.1.1 Implemented form and what it adds to the decision

The constraint is implemented in `optimizer.rs` and applied **in addition to**
financial feasibility. A candidate work percentage is feasible only when both
hold:

$$
\theta \text{ is feasible} \iff \underbrace{(1-\tau(I))I/12 \geq C^{\text{mandatory}}}_{\text{budget}} \;\wedge\; \underbrace{A(\theta) \geq G}_{\text{achievement}}
$$

Two derived quantities answer the critique's actual questions:

**What is the lowest credible work percentage?** Inverting the linear capacity
identity gives the §3a answer directly:

$$
\theta_{\min} = \frac{G}{P(1+\alpha)} \quad \text{when } P(1+\alpha) \geq G
$$

When $P(1+\alpha) < G$ the goals exceed even full-time capacity, and the model
returns *no* viable percentage rather than a number. This is a genuine finding:
the assigned portfolio is infeasible as stated, and the CLI says so explicitly
("It's a workload problem, not a budget problem") instead of recommending
something undeliverable.

**How much AI gain would justify reducing to $\theta$?** From §1.2:

$$
\alpha_{\text{required}}(\theta) = \frac{G}{\theta P} - 1
$$

Together these let the model represent all three §14.2 outcomes without assuming
which applies: *shared productivity gain* (α rises, $G$ constant → reduced hours
become feasible), *employer capture* ($G$ rises with α → the constraint binds
again), and *labour substitution* (capacity parameters fall, raising the required
$\theta$).

Both constraints are reported independently, because the remedies differ
entirely. An infeasible search now distinguishes `Unaffordable`,
`AchievementUnreachable`, and `Both`, and the CLI prints the corresponding
advice.

### 14.2 A possible 10-year, 40% work scenario

In a scenario where AI allows a worker to operate at 40% of the current scheduled time, the model should compare at least three cases:

1. **Shared productivity gain:** project goals remain approximately constant and the worker receives more leisure or family time.
2. **Employer capture:** productivity rises, but project goals increase so that the worker remains under similar performance pressure.
3. **Labor substitution:** some tasks disappear, increasing replacement or unemployment risk for workers whose skills are no longer demanded.

The model should not assume that one case will apply to the entire economy. Results should be reported as scenario distributions rather than as a single prediction.

### 14.3 Cognitive engagement and meaningful activity

Reduced paid work can improve health and leisure, but complete disengagement from demanding activity may reduce skill maintenance, identity, social connection, or perceived purpose for some people. A simple cognitive-engagement index can be defined as:

$$
Q_t = q_0 + q_w \theta_t + q_l E_t^{learning} + q_s E_t^{social} - q_d D_t^{disengagement},
$$

where $E_t^{learning}$ and $E_t^{social}$ measure purposeful learning and social activity outside paid work. This avoids assuming that paid employment is the only source of cognitive sharpness or happiness.

The utility function can then include a separate purpose and engagement term:

$$
u_{purpose}(Q_t) = K_q \ln(1 + Q_t).
$$

This allows the model to compare a 40% work strategy with a balanced strategy combining part-time work, education, care, volunteering, entrepreneurship, or other meaningful activity.

### 14.4 Income, taxation, and social distribution

If AI reduces the labor income of a large share of the population, individual income may no longer be sufficient to support a stable tax base. A macro-level extension could define total tax revenue as:

$$
\mathcal{T}_t = \mathcal{T}^{labor}_t + \mathcal{T}^{capital}_t + \mathcal{T}^{consumption}_t,
$$

and test how revenue changes when labor income falls but capital income and AI-generated output rise. The model should also track the distribution of gains between workers, firms, and capital owners, because unequal access to AI can produce different outcomes for otherwise identical workers.

### 14.5 Advanced model families

Several established approaches could represent this future more realistically:

- **Dynamic stochastic general equilibrium models:** represent households, firms, wages, capital, taxation, and technology shocks at the macroeconomic level.
- **Overlapping-generations models:** compare how AI-driven changes affect young workers, mid-career workers, and retirees differently.
- **Agent-based models:** represent heterogeneous workers, employers, skills, firms, and job transitions rather than assuming one average worker.
- **Real-options models:** value the option to reduce work, retrain, defer retirement, or return to employment when AI adoption changes.
- **System-dynamics models:** study feedback loops between productivity, wages, consumption, tax revenue, public services, and social stability.
- **Markov decision processes or partially observable decision models:** represent changing employment states, uncertain AI capability, retraining, and replacement risk over time.

The most practical research path is a hybrid: retain the current household optimizer, add AI-adjusted achievement and cognitive-engagement variables, and connect it later to an agent-based or overlapping-generations macro model.

### 14.6 Social and psychological research questions

The model cannot resolve these questions from economics alone. It should expose them for interdisciplinary research:

- How much paid work is necessary for social participation and personal identity?
- Does reduced work increase happiness when people have meaningful alternatives?
- How should education, volunteering, caregiving, and creative work be valued?
- Who owns and receives the benefits of AI productivity?
- How can society preserve cognitive development without forcing unnecessary employment?
- How should taxes and public services be financed if labor becomes a smaller part of total production?

The central conclusion is conditional: AI may make 40% work economically possible, but its social effect depends on distribution, purpose, education, health, and whether productivity gains are converted into shared time or merely into higher output expectations.

## 15. Current limitations and project risks

The project currently has several risks that should be acknowledged explicitly:

1. Parameter choices may be too heuristic.
2. Tax modeling may be insufficiently local or institution-specific.
3. Utility assumptions may not be empirically grounded.
4. Stochastic processes may underrepresent tail risk and dependence.
5. Validation may be weaker than a serious quantitative framework requires.
6. The model may appear more precise than it actually is.

These limitations are not fatal, but they do mean that the project should be framed as a transparent research and planning framework, not as a definitive financial adviser.

---

## 16. Summary

The formal structure of Life Optimizer can be read as a constrained intertemporal utility optimization problem with a stochastic pension environment and explicit household preferences. This is a coherent and defensible framework for decision support.

However, the model must be strengthened in four areas to become genuinely credible:

1. stronger calibration logic
2. more realistic macroeconomic dynamics
3. explicit validation and sensitivity analysis
4. disciplined model selection and benchmarking

The project is strongest when it is framed as transparent and explainable quantitative planning, rather than as a black-box optimization engine. That is the right strategic position for a tool intended to support major life decisions.
