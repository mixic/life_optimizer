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

Mr. Bojan Nedic holds a Master's degree in Electrical Engineering from the University of Belgrade. He is an expert in microelectronics and electrical engineering, as well as software engineering, firmware development, and hardware-near development. He identified a missing dimension in the current model: consumption.

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
