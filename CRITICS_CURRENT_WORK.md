# Critics of the Current Work

This document records external criticism and questions raised about the Life Optimizer project. The purpose is not only to defend the current model, but to identify assumptions that should be tested and, where necessary, incorporated into the design.

## 1. Critique from Mr. Gradimir Nikolic

A short Resume: 

Mr. Gradimir Nikolic is a highly knowledgeable expert in economics and statistical risk modeling in the insurance sector. After completing his diploma studies in Agrarian Sciences and Food Technology at the University of Belgrade, he earned a Magister degree from ETH Zurich through a special scientific collaboration program for post-diploma education, led by Prof. Hans Bühlmann (ETH Zurich) and Erwin Straub, two world-renowned experts in insurance mathematics, in cooperation with the Yugoslav insurance company Dunav, which subsidized post-diploma studies in Actuarial Sciences. He went on to develop extensive leadership experience in the insurance industry. His comments therefore provide both a philosophical perspective on work allocation and a professional perspective on economic uncertainty and risk.

Mr. Gradimir Nikolic considers the basic idea of the project valuable: a worker should be able to evaluate the trade-off between income, time, health, leisure, and long-term pension security. However, he raised an important philosophical and practical question about the assumption that a worker can freely choose a work level between 100% and 80%.

### 1.1 Project work is often measured by results

In many companies, success is determined by achievements and project outcomes rather than by the amount of time spent at work. A company may define a strategic annual plan requiring a worker or team to complete five or six projects during the year. These projects are concrete obligations, not merely opportunities for the worker to make a best effort.

Under this structure, the worker may function as a resource in a larger staffing system. The company can assign the worker to a project and expect the required results to be delivered. The worker may therefore be treated as available at either 100% or 80%, depending on the needs of the project and the organization. The decision is not necessarily a personal choice.

If the required goals are not achieved, even with a full-time workload, the worker may be replaced by another worker. This creates a direct tension with an optimizer that treats work percentage as an independent variable under the worker's control.

### 1.2 The central question about AI as a work catalyst

The criticism leads to the following question:

> Can a reduction from 100% work to 80% work be justified by using AI as a catalyst for achieving the same planned project results?

For example, suppose the steering board assigns a worker five or six projects for the year. If the worker can successfully complete all of them while working at 80%, with AI improving productivity, then the reduction from 100% to 80% may be real in terms of time worked while preserving the company's expected output.

In that case, the relevant measure is not simply the worker's time allocation. It is the relationship between:

- Work capacity or availability
- AI-assisted productivity
- Project complexity
- Quality of delivered results
- Deadlines and reliability
- Collaboration and coordination requirements
- The worker's health and sustainability

A nominal 80% schedule is therefore credible only if the worker can continue to meet the organization's required outcomes without shifting hidden work into evenings, weekends, or unpaid availability.

### 1.3 Implication for the Life Optimizer model

The model should distinguish between **contractual work percentage** and **effective achievement capacity**.

A possible formulation is:

$$
A_t = H_t \times P_t \times (1 + \alpha_t),
$$

where:

- $A_t$ is effective achievement capacity in period $t$;
- $H_t$ is paid or scheduled work time;
- $P_t$ is baseline productivity per unit of work time;
- $\alpha_t$ is the productivity effect of AI and other tools.

The worker can reduce scheduled work from 100% to 80% only when effective achievement capacity remains sufficient for the assigned project portfolio:

$$
A_t \geq G_t,
$$

where $G_t$ represents the required project goals for the period.

This condition should also include quality and sustainability constraints. Completing a target by creating excessive stress, hidden overtime, or unacceptable defects should not count as a successful 80% strategy.

### 1.4 Practical tests suggested by the critique

The project could test this issue by adding outcome-based scenarios:

1. **Fixed-goal scenario:** The company assigns a defined number of projects and deadlines, and the optimizer checks whether each work percentage can meet them.
2. **AI productivity scenario:** AI increases productivity by an explicitly modeled range rather than by an assumed constant value.
3. **Quality constraint:** A project counts as successful only when quality, deadline, and collaboration requirements are all met.
4. **Hidden-work constraint:** Work outside the scheduled percentage is counted as additional workload rather than being treated as free productivity.
5. **Replacement-risk scenario:** Failure to meet the required goals creates a probability of job loss or replacement.
6. **Adaptation scenario:** The worker can adjust the work percentage only after demonstrating reliable delivery over a defined evaluation period.

This would make the model more realistic for modern project-based employment. It would also clarify that the recommendation is not simply “work less,” but rather “work less when tools and working methods preserve the outcomes that the employer requires.”

## 2. Critique from Mr. Bojan Nedic

A short Resume: 

Mr. Bojan Nedic holds a Master's degree in Electrical Engineering from the University of Belgrade. He is an expert in electrical engineering, microelectronics and signal processing as well as software engineering, firmware development, and hardware-near development. He identified a missing dimension in the current model: consumption.

The optimizer considers income, work, leisure, health, and pension security, but it does not yet adequately represent how much money a person actually consumes during the working period. Without a consumption dimension, the model cannot distinguish between a person who lives extremely frugally and a person who maintains a normal or luxury lifestyle. It also cannot properly represent rent as a major recurring expense.

### 2.1 Consumption should be an explicit input

The model should allow the user to select or define a consumption level, for example:

- **Extreme saving:** minimal discretionary spending and strict cost control;
- **Moderate:** controlled spending with some flexibility for leisure and goals;
- **Normal:** an ordinary expected standard of living;
- **Luxury:** high discretionary spending, premium services, travel, and lifestyle costs.

These categories should not be treated as moral judgments. They are alternative spending profiles that allow the optimizer to show how lifestyle choices change the feasibility of reduced work.

Rent or housing cost should be modeled separately because it is often the largest fixed expense and may not decrease when work percentage decreases. A useful baseline is:

$$
C_t = R_t + L_t + D_t,
$$

where:

- $C_t$ is total consumption in period $t$;
- $R_t$ is rent or housing cost;
- $L_t$ is the selected lifestyle consumption level; and
- $D_t$ is debt repayment or other unavoidable expenditure.

The household's remaining resources can then be expressed as:

$$
S_t = Y_t - T_t - C_t,
$$

where $S_t$ is savings or investment capacity, $Y_t$ is income, and $T_t$ is tax and social-security expenditure.

### 2.2 Implication for work-percentage decisions

Consumption changes the answer to the central optimization question. An 80% work schedule may be feasible for a person with moderate consumption and low rent, but infeasible for a person with luxury consumption, high rent, or large debt obligations. Conversely, an extreme-saving profile may allow a worker to reduce working time earlier, although it may also reduce current quality of life.

The model should therefore calculate whether a proposed work percentage can simultaneously:

1. Cover rent and other essential expenses;
2. Cover the selected lifestyle level;
3. Maintain an emergency reserve;
4. Continue required pension and investment contributions; and
5. Preserve the desired future pension outcome.

This makes consumption a direct part of the trade-off rather than an implicit assumption hidden inside a generic requirement value.

### 2.3 Suggested implementation

Consumption could initially be implemented as a configurable monthly profile with separate values for essential and discretionary spending:

```text
consumption_level = extreme_saving | moderate | normal | luxury
monthly_rent = user-defined amount
monthly_essential_costs = profile-dependent amount
monthly_discretionary_costs = profile-dependent amount
monthly_debt_costs = user-defined amount
```

The optimizer can then compare work percentages under identical income and market assumptions while varying only consumption. This would reveal whether a recommendation is robust or depends on an unrealistically low spending level.

His critique should be understood as a request to make the model financially complete: income determines what enters the household, while consumption and rent determine what remains available for saving, investing, and future security.

## 3. Resulting Design Principle

The criticism suggests that work percentage should not be modeled as a purely free personal choice. It should be treated as a decision constrained by the employment environment.

A more complete optimization problem would therefore ask:

> What is the lowest sustainable work percentage at which the worker can reliably meet the employer's required goals, with or without AI assistance, while preserving health, income, leisure, and pension adequacy?

This reframing preserves the original purpose of the Life Optimizer while adding an important real-world condition: personal freedom over work time exists only within the boundaries of contractual obligations and measurable results.

## 4. Open Research Questions

- How should AI productivity gains be estimated without assuming that every task benefits equally?
- Does AI reduce total effort, or does it increase expected output and therefore raise the number of assigned projects?
- How should team dependencies be represented when one worker's reduced availability affects other workers?
- What evidence is sufficient to show that an 80% schedule is sustainable rather than temporarily achieved through hidden overtime?
- How should the model balance employer risk, worker replacement risk, and the value of additional leisure?
- Should the optimizer recommend a work percentage only after evaluating both financial outcomes and the probability of meeting project goals?

## 5. AI and the Future Reduction of Work

Mr. Gradimir Nikolic's perspective becomes even more important when considering a future in which AI may allow some people to achieve current project results with substantially less paid work. A possible 40% work schedule in ten years should not be interpreted as a guaranteed prediction. It is a scenario that raises economic, psychological, sociological, and philosophical questions.

AI could produce several different outcomes:

- Workers may keep the same goals and receive more leisure, family time, and recovery time.
- Employers may increase project targets so that higher productivity does not reduce performance pressure.
- Some jobs may disappear or become less secure, especially where AI can perform most required tasks.
- The benefits of AI may flow mainly to companies and capital owners unless institutions distribute them more broadly.

Reduced work could improve health and quality of life, but people may also lose routine, professional identity, social contact, or opportunities to experience achievement. This does not mean that society must force people to work unnecessarily. It means that a future with less paid work should provide meaningful alternatives such as education, caregiving, volunteering, creative work, community participation, and lifelong learning.

The Life Optimizer should therefore distinguish between **paid work** and **meaningful activity**. A person working 40% but spending the remaining time learning, caring for family, contributing to the community, or building a creative project may have a very different outcome from a person who is isolated and inactive.

The social impact of AI-driven work reduction should be evaluated through at least four dimensions:

1. **Economic security:** Can people pay for housing, consumption, healthcare, and taxes when labor income declines?
2. **Distribution:** Who receives the gains from AI productivity: workers, employers, or capital owners?
3. **Human development:** How are cognitive skills, purpose, social relationships, and learning maintained?
4. **Public finance:** How are pensions, healthcare, education, and other public services financed when labor is a smaller share of total production?

This is an interdisciplinary research problem. Economists can study productivity, wages, taxes, and distribution. Sociologists can study institutions, inequality, and social cohesion. Psychologists can study motivation, identity, cognition, and well-being. Philosophers can study the meaning of work, fairness, freedom, and the responsibilities created by powerful AI.

The model should treat these outcomes as alternative scenarios and sensitivity parameters, not as a single certain forecast. The key question is not simply whether AI permits people to work 40%, but whether society can convert increased productivity into secure income, meaningful activity, preserved human capability, and a fair distribution of time and wealth.

## 6. Selected Current Publications

The following publications provide useful evidence and frameworks for extending this project. They cover AI productivity, labor-market exposure, job quality, inequality, and the social organization of work.

1. **Cazzaniga, M. et al. (International Monetary Fund, 2024), _Gen-AI: Artificial Intelligence and the Future of Work_.** [IMF Staff Discussion Note](https://www.imf.org/en/Publications/Staff-Discussion-Notes/Issues/2024/01/14/Gen-AI-Artificial-Intelligence-and-the-Future-of-Work-542379)

	Useful for analyzing occupational exposure to generative AI, productivity effects, labor-income distribution, and the risk that AI benefits may be distributed unevenly.

2. **International Labour Organization (2023), _Generative AI and Jobs: A Global Analysis of Potential Effects on Job Quantity and Quality_.** [ILO publication](https://www.ilo.org/publications/major-publications/generative-ai-and-jobs-global-analysis-potential-effects-job-quantity-and-quality)

	Useful for distinguishing job transformation from full job replacement and for considering job quality, autonomy, and different effects across groups of workers.

3. **OECD (2023), _OECD Employment Outlook 2023: Artificial Intelligence and the Labour Market_.** [OECD publication](https://www.oecd.org/en/publications/oecd-employment-outlook-2023_08785bba-en.html)

	Useful for evidence on AI adoption, worker experiences, training, workplace risks, and the role of public policy.

4. **Brynjolfsson, E., Li, D., and Raymond, L. R. (2023), _Generative AI at Work_, NBER Working Paper 31161.** [NBER publication](https://www.nber.org/papers/w31161)

	Useful for empirical evidence that generative AI can affect worker productivity, especially through the transfer of knowledge and practices from more experienced workers.

5. **Acemoglu, D. and Restrepo, P. (2018), _Artificial Intelligence, Automation and Work_, NBER Working Paper 24196.** [NBER publication](https://www.nber.org/papers/w24196)

	Useful for modeling the difference between automation, new tasks, productivity, wages, and employment. This is particularly relevant to the question of whether AI creates leisure or simply changes the demand for labor.

6. **International Labour Organization (2019), _Working on a Warmer Planet: The Impact of Heat Stress on Labour Productivity and Decent Work_.** [ILO publication](https://www.ilo.org/publications/major-publications/working-warmer-planet-impact-heat-stress-labour-productivity-and-decent)

	Useful as a reminder that long-term work and productivity scenarios should include health, environmental, and labor-capacity risks rather than focusing on technology alone.

7. **World Economic Forum (2025), _The Future of Jobs Report 2025_.** [World Economic Forum report](https://www.weforum.org/publications/the-future-of-jobs-report-2025/)

	Useful for current employer expectations about changing skills, job creation, job displacement, and reskilling needs. It should be treated as a survey-based scenario source, not as a precise forecast.

These sources support a research position rather than a predetermined conclusion. They justify modeling several AI adoption, employment, productivity, and distribution scenarios and reporting the uncertainty around each one. 

## 7. Implementation Status in This Repository

This section is the response to §1 to §5. It records what the code now does, what it deliberately refuses to do, and where every new coefficient comes from. The rule is unchanged from `FutureWork.md` §7: a parameter may be illustrative, but it may not be presented as measured, and no figure may be invented to make a narrative close.

Every input added here defaults to the *neutral* value, so the pre-critique answers are reproducible and the new machinery is inert unless it is declared.

### 7.1 The achievement-capacity constraint (§1.2 and §1.3)

Implemented in `src/optimizer.rs` (`AchievementConstraint`, `AchievementStatus`, `Robustness`, `Enforcement`), reachable from `src/main.rs`, and reported by `src/display.rs`.

$$
A_t(w) = w \, P \, (1 + \alpha \rho),
\qquad
\text{compression}(w) = \max\left(\frac{G}{w P},\, 1\right),
$$

$$
q(w) = \mathrm{clamp}\Bigl(1 - \rho_{\text{comp}}\,\bigl(\text{compression}(w) - 1\bigr),\ 0,\ 1\Bigr),
\qquad
\text{delivered}(w) = A_t(w)\, q(w),
$$

$$
A_t \geq G \iff \text{delivered}(w) \geq G ,
$$

where $w$ is the contractual work percentage (with $H_t$ normalized so that full time is 1), $P$ is baseline productivity per unit of work time, $\alpha$ is the AI productivity gain, $\rho \in [0,1]$ is the share of that gain which survives verification and rework, $\rho_{\text{comp}}$ is the delivered quality lost per unit of pace above the sustainable rate, and $G$ is the required output index for the period.

Two decisions have to be stated explicitly, because otherwise the equations can be read as claiming more than they do:

1. **The quality drag is measured against baseline pace, not AI-assisted pace.** `compression` divides $G$ by $w P$, so a given portfolio stretches the worker by the same amount whether or not AI is in use. Crediting AI with *relieving* the stretch as well as *raising* output would assume a substitution elasticity between human and machine effort that nobody has measured, and it would take two benefits out of one gain. The asymmetry is deliberate, and it disappears when the user declares no drag: with `--compression-quality-sensitivity 0` (the default), $q(w) \equiv 1$ and the model collapses exactly onto the critique's linear formulation $A_t = H_t P_t (1 + \alpha_t)$.
2. **A goal is met only at the pessimistic end of the declared range.** Feasibility is judged by `delivered_pessimistic`, which makes the recommendation a claim about the worker rather than about the tool. `Robustness` reports the difference, and a report quotes the label verbatim: `ROBUST across the AI range` (delivers across the whole declared range), `ONLY IF AI DELIVERS at the optimistic end` (delivers only if AI lands at the top of the range — a bet on the tool, not a credible reduction), and `UNREACHABLE at any point in the AI range` (not even full time delivers it, which is a finding about the assignment rather than an error in the run).

The derived quantities, and the critique question each one answers:

| Quantity | Question it answers |
| --- | --- |
| `minimum_viable_work_percentage` | §3: the lowest percentage at which the required goals are still delivered |
| `required_ai_gain_for` | §1.2 directly — how much AI gain would be needed to justify a given reduction. Returns `None` when the declared retention is zero: if none of the gain survives verification, a larger gain buys nothing |
| `hidden_work_percentage` | §1.4 item 4 — the cover the pessimistic AI outcome would demand, as a fraction of full time |
| `replacement_risk_at` | §1.4 item 5 — shortfall mapped to a probability of replacement through one linear coefficient |
| `amortised_work_percentage` | §1.4 item 6 — workload averaged over the remaining career, including the years served before a reduction is granted |

### 7.2 The six practical tests of §1.4

| §1.4 test | What the code does | Knob | Neutral default |
| --- | --- | --- | --- |
| 1. Fixed-goal scenario | The optimizer searches only over work percentages that deliver the assigned goals; if none does, it reports a workload problem and names the AI gain that would be required | `--required-output-index` | absent: work percentage stays fully discretionary, exactly as before |
| 2. AI productivity scenario | Feasibility is tested across a declared range rather than at a point; `Robustness` classifies each schedule | `--ai-productivity-gain` (pessimistic), `--ai-productivity-gain-high` (optimistic) | `0.0` / unset: no AI gain is assumed |
| 3. Quality constraint | Two channels: the share of the AI gain that survives verification, and the defects induced by compressing the same output into fewer hours. Both act on *delivered* output, not on capacity | `--ai-quality-retention`, `--compression-quality-sensitivity` | `1.0` / `0.0`: delivered equals capacity, no drag |
| 4. Hidden-work constraint | Work beyond the contract that the pessimistic outcome requires is reported beside the contract, in h/week, and is never credited as leisure or folded into the hours — its cost arrives through the refusal or the risk pricing | `--required-output-index` (the shortfall is derived; there is no separate knob) | no constraint, hence no hidden work |
| 5. Replacement-risk scenario | Failure to deliver raises a replacement probability, which reduces the security component of utility | `--replacement-risk`, `--enforcement` | `0.0` and `strict` |
| 6. Adaptation scenario | A required probationary period is averaged into the workload, so the leisure gain is not overstated | `--evaluation-period-years` | `0.0`: the declared percentage is the average |

Two of these need a word of explanation, because the implementation takes a position rather than merely exposing a number.

**Hidden work (item 4).** A nominal 80% schedule that needs 95% of a full-timer's capacity is worked at 95%, whatever the contract says, and the difference arrives as evenings, weekends, or unpaid availability. The critique's instruction is that this be counted as workload rather than treated as free productivity, so the report prints it beside the contract, in hours per week, with the wording that the hours above are the contract and this is carried on top of them.

What it deliberately does **not** do is add those hours to the effective workload. Working the cover and missing the goal are two *answers* to the same shortfall, not two costs to be added together: the model already judges delivery at the pessimistic end, so a schedule that needs the cover is exactly a schedule that fails on that scenario, and its cost arrives through the strict refusal or through the replacement-risk pricing. Adding the cover to the hours as well would charge for both resolutions at once. Free hours are computed from the amortised workload rather than the nominal one, so the evaluation period of item 6 does reach the leisure figures. By construction hidden work is zero for any schedule that is `ROBUST` — if the goals are delivered at the pessimistic end, no cover is needed.

**Enforcement (item 5).** `--enforcement strict` (the default) keeps the critique's own reading of the constraint: a percentage that does not deliver is not offered at all. `--enforcement risk-weighted` offers it, marks it "OFFERED BUT NOT DELIVERED", and prices the replacement risk as $security = \beta_{security} \cdot pension\_value \cdot (1 - r)$, so the shortfall becomes a trade-off rather than a hidden failure. The trade-off is real and not decorative, which is testable: with a punitive sensitivity the search returns to full time on its own.

### 7.3 Consumption (§2)

The critique's decomposition is implemented as stated:

$$C_t = R_t + L_t + D_t, \qquad S_t = Y_t - T_t - C_t,$$

where $R_t$ is `housing`, $L_t$ is the profile-scaled elastic tier plus the quasi-inelastic share the household declares, and $D_t$ is debt repayment. Debt repayment is new (`--monthly-debt`) and is placed in the *inelastic* tier, joined to the mandatory floor: it is not trimmable, and it does not shrink when hours do, which is exactly why the critique wants it visible rather than folded into a generic requirement. The profile also carries committed outflows (savings goal, vacation sinking fund) which stay in the target basket.

Savings capacity is reported as $S_t = Y_t - T_t - C_t$ with the savings goal removed from $C_t$ first, so that the same money is not counted both as consumption and as capacity to save.

The lifestyle profiles use discretionary multipliers of 0.50 (extreme saving), 0.80 (moderate), 1.00 (normal), and 1.75 (luxury), taken from this project's own calibration table (`MATHEMATICS.md` §3.1) and themselves illustrative rather than measured.

Feasibility is tested against the **mandatory floor**, not the full lifestyle basket. The reason is stated in the code: a household is not unable to afford 80% merely because it would have to trim discretionary spending. The full lifestyle-inclusive target is still reported beside it, so the squeeze is visible rather than hidden — reporting only the floor would have been the mirror-image mistake.

The five checks of §2.2 map onto the code as follows:

| §2.2 check | Status |
| --- | --- |
| 1. Cover rent and other essential expenses | Implemented: the mandatory floor includes housing and debt repayment |
| 2. Cover the selected lifestyle level | Reported: the lifestyle-inclusive basket is shown against income, but it is a target, not a feasibility test |
| 3. Maintain an emergency reserve | Partial: carried inside `savings_goal` and reported; there is no separate reserve-balance check, so the model does not verify that the reserve is actually maintained |
| 4. Continue required pension and investment contributions | Partial: the savings goal and pillar 3a contributions feed the pension layer, but the floor does not enforce them — they can be flexed |
| 5. Preserve the desired future pension outcome | Yes, in a separate layer: the pension and Monte Carlo modules consume $S_t$ and report the resulting pension quality |

### 7.4 Backwards compatibility

All new knobs are neutral by default. With $\alpha_{high} = \alpha$, $\rho = 1$, $\rho_{comp} = 0$, zero replacement risk, a zero evaluation period, zero debt, and strict enforcement, `delivered` is identically equal to `capacity` and the constraint degenerates to the original linear test. This is not an assertion but a checked property: the pre-existing suites (`optimizer_behavior`, `cli`, `cli_canton`, `pension_fund_and_stochastic`, `conversion_rate`, and the deduction suites) continue to pass unchanged, together with the tests added for the critique.

### 7.5 Provenance of the new coefficients

| Input | Symbol | Default | Status |
| --- | --- | --- | --- |
| Baseline productivity per unit of work time | $P$ | 1.0 | Normalization, not a measurement |
| AI productivity gain, pessimistic end | $\alpha$ | 0.0 | User-declared and illustrative. The model never estimates it |
| AI productivity gain, optimistic end | $\alpha_{high}$ | = $\alpha$ | User-declared and illustrative |
| Share of the AI gain surviving verification and rework | $\rho$ | 1.0 | Unmeasured. 1.0 is the assumption-free default ("output usable as delivered"); departing from it is the user's judgement, not the model's |
| Quality lost per unit of pace above the sustainable rate | $\rho_{comp}$ | 0.0 | Unmeasured. No published coefficient maps schedule compression onto defect rates for knowledge work. Exposed so a user can test the sensitivity instead of having one invented for them |
| Replacement probability per unit of relative shortfall | — | 0.0 | Unmeasured. Deliberately one linear coefficient rather than a curve, because nobody has measured how a 10% delivery gap maps onto a dismissal probability |
| Required evaluation period | — | 0.0 | A policy choice, not a measurement |
| Monthly debt repayment | $D_t$ | 0.0 | The user's own contract value |
| Lifestyle discretionary multipliers | — | see §7.3 | From this project's own calibration table, itself illustrative |

**No figure from the publications in §6 is encoded anywhere in this model.** They are a reading list: they indicate which questions have a literature, and the model exposes the coefficients that literature would have to supply. Encoding an IMF, ILO, OECD, or WEF aggregate as a Swiss household coefficient would be exactly the "plausible narrative wrapped around unfitted parameters" that `FutureWork.md` §7 forbids.

### 7.6 Worked example

```text
cargo run -- optimize --salary 150000 --age 40 --required-output-index 1.0 \
  --ai-productivity-gain 0.5 --ai-productivity-gain-high 0.8 \
  --compression-quality-sensitivity 0.5 --evaluation-period-years 2 --monthly-debt 300
```

```text
Employer Achievement Capacity:
  AI gain range:   50% pessimistic … 80% optimistic
  Quality factor:  0.88   usable output per unit of capacity
  Robustness:      ROBUST across the AI range
  Status:          MEETS REQUIRED OUTPUT ✓ (margin +0.05)
  Average workload:81.6%   over the years to retirement, including
Consumption by Elasticity Tier:
  Mandatory floor                          CHF 4060
  Saving capacity (Y − T − C)              CHF 2150
```

How to read it: the goals are still delivered even if AI returns only half of the optimistic gain, so the reduction is robust rather than a bet; compressing the portfolio into the reduced week costs about 12% of the output per hour, which is why the margin is thin; and the average workload is 81.6% rather than the contractual 80% because the first two years are served at full time. A reader who thinks the quality coefficient or the evaluation period is wrong can change it and see the answer move — which is the point of exposing them.

### 7.7 What is deliberately not modelled

- **Team dependencies (§4, third question).** Not modelled at all, and this is the most consequential gap. The model treats the worker as a resource whose reduced availability has no effect on anyone else, while a real 80% schedule may push work onto colleagues. Representing that needs an explicit team-coupling model and data this project does not have; asserting a coupling coefficient would be invention. The omission should be read as a known limitation of every result here, not as a finding that the effect is zero. The full design for that channel — the conservation identity that breaks the assumption, the reason sustainability is a stock rather than a flow, and the provenance rule for every coefficient it needs — is in [`TEAM_DEPENDENCIES.md`](TEAM_DEPENDENCIES.md). It is designed and deliberately not implemented; the limitation stands exactly as stated.
- **Per-task AI gains (§4, first question).** One scalar with a range, not a task-level decomposition. A single gain also cannot represent the plausible case where AI helps with some of the portfolio and not with the rest.
- **The employer's response (§4, second question, and §5).** $G_t$ is an input. The model can be *run* with a higher required output to represent an employer who raises targets after a productivity gain, but it does not endogenise that response. The second bullet of §5 is therefore a scenario the user can construct, not something the model predicts.
- **An evidence standard for sustainability (§4, fourth question).** Proxies only: amortised workload, hidden work, and the evaluation period. No empirical evidence that an 80% schedule is sustainable is encoded, and none is claimed. The model reports what the schedule demands; it cannot certify the worker's endurance.
- **The joint employer-worker problem (§4, fifth question).** Partial. Replacement risk is priced on the worker's side; there is no employer-side objective, so the employer bears no cost when a worker is lost. The model answers the worker's question, not the joint one.
- **Deciding only after weighing finance and goal probability (§4, sixth question).** Yes, this is what the strict/risk-weighted split and the report exist to do.
- **Meaningful activity and the paid-work/meaningful-activity distinction (§5).** Not modelled. The optimizer reports free hours and says nothing about what fills them: it cannot distinguish a 60% schedule spent learning, caring, or contributing from the same schedule spent isolated and inactive. Of the four dimensions §5 lists, only economic security is addressed, and only as income, tax, and pension arithmetic; distribution, human development, and public finance are outside the present scope. The one place the model does take a position on human capability is the hidden-work constraint: it refuses to book unverified AI output as free time.
- **The 40% schedule in ten years.** Reachable as a scenario through the AI-gain and required-output inputs, and still not a forecast. Nothing in the model fits a curve to it, and nothing in it should be quoted as a projection.

### 7.8 Reproduction

`cargo test --workspace --all-targets` runs 284 tests (55 in the library, plus the integration suites, plus the simulator's own). The tests that pin the behaviour described above are `tests/achievement_constraint.rs` (18), `tests/consumption_model.rs` (16), and `tests/cli_consumption_achievement.rs` (16), which exercise the critique's items through both the library and the command line.
