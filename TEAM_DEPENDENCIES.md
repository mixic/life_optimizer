# Team Dependencies

A chapter on the one assumption that the Life Optimizer has never tested: that a
worker's deliverable capacity is a function of their own hours and nobody else's.

`CRITICS_CURRENT_WORK.md` §7.7 listed team dependencies as the most consequential
omission in the model. This chapter is the response. It does not add a number to the
report. It pins the model, separates what a worker can actually observe from what would
have to be invented, and proposes an implementation whose defaults leave every existing
answer unchanged.

The reader this is written for is a critic, so the claims are stated in a form that can
be attacked, and §8 says which observations would falsify them.

---

## 1. The claim

**A reduced work percentage is not a private transaction.**

The model currently treats it as one. It computes how much output a worker delivers at a
given percentage, compares that to the goals assigned, and reports the difference — all
of it about one person. The implicit assumption is that a shortfall is resolved by the
worker who caused it: through AI leverage, through compression, or through hidden work.
`hidden_work_percentage` exists precisely to price that last option and to refuse to call
it free.

But §1.1 of the critique already says why that cannot be the whole story. The worker "may
function as a resource in a larger staffing system" and the company "can assign the
worker to a project and expect the required results to be delivered". In such a system,
a shortfall does not stay with the person who caused it. It goes somewhere, and where it
goes is a decision that other people live with.

The chapter argues three things:

1. **Output is conserved.** What a worker does not deliver is delivered by someone else,
   absorbed by the worker as hidden work or degraded quality, or not delivered at all.
   There is no fifth option. The current model assumes the third of these is empty and
   the first does not exist; it therefore reports every shortfall as fully internalised.
2. **The externality is a property of the scope, not of the hours.** If an employer
   reduces the assigned portfolio in proportion to the hours, the team channel is empty
   and the current model is correct. If the portfolio is *not* reduced — the fixed-goal
   scenario of §1.4 item 1 — then the team absorbs what the worker does not. So the
   decision variable was never "what percentage do I work", it is "what percentage do I
   work, and how much of the portfolio do I hand back".
3. **Sustainability may be a stock, not a flow.** A worker's ability to run a reduced
   schedule depends on accumulated goodwill that is consumed by drawing on it. A flow
   constraint cannot be exhausted; a stock can. §4 of the critique asks what evidence
   would show that an 80% schedule is sustainable. This chapter proposes that the
   question is about a balance, and that a balance is a different kind of question.

---

## 2. The assumption being broken

The current constraint, from `src/optimizer.rs`, is:

$$
A_t(w) = w \, P \, (1 + \alpha \rho),
\qquad
\text{delivered}(w) = A_t(w) \, q(w),
\qquad
\text{delivered}(w) \geq G .
$$

Every symbol on the right-hand side belongs to the worker: their percentage, their
baseline productivity, their AI gain, their retention, their quality drag. The goal $G$
is an input. Nothing in the expression can represent a second person.

### 2.1 The four channels of a shortfall

Write $G_j = s_j \, G_{\text{full}}$ for the portfolio assigned to member $j$ of a team,
where $s_j \in (0, 1]$ is the **assigned scope** as a fraction of a full-time portfolio
and $G_{\text{full}}$ is a full portfolio normalised to 1. The contractual percentage is
$w_j$. The two are *not* the same thing, and the difference between them is where this
whole chapter lives:

| Case | Meaning | Team channel |
| --- | --- | --- |
| $s_j = w_j$ | The portfolio is scaled with the hours. A genuine proportional part-time arrangement | **Empty.** The current model is right |
| $s_j = 1$ | The full portfolio is retained at reduced hours — §1.4 item 1, the fixed-goal scenario | **Fully exposed.** This is the case worth modelling |
| $w_j < s_j < 1$ | A partial scope reduction, which is what most negotiated arrangements actually are | Partially exposed |

A shortfall $\text{raw}_j = \max(0, G_j - d_j)$ has exactly four destinations:

| # | Destination | Modelled today? | Who bears the cost |
| --- | --- | --- | --- |
| 1 | Absorbed by $j$: hidden work, degraded quality, stress | **Yes** — `hidden_work_percentage`, `quality_factor_at` | The worker |
| 2 | Transferred to colleagues: they cover, hand over, or work harder | **No** | Other people |
| 3 | Not delivered: goals missed, deadlines slip | Partially — replacement risk prices one consequence | The employer, then the worker |
| 4 | Absorbed by the employer: targets cut, deadlines moved | **No** | The employer |

The current model keeps channel 1 and prices a corner of channel 3. Channels 2 and 4 are
absent, and channel 2 is the one the critique's opening paragraph implies.

**A finding that is easy to miss:** the model already contains a team, it just hides it.
When `hidden_work_percentage` reports that a nominal 80% schedule would demand 95% of a
full-timer's capacity, a reader is entitled to ask *why the worker absorbs all of it*.
Sometimes the answer is that they do. Often the answer is that four colleagues each take
a little, nobody records it as overtime, and the schedule looks sustainable from
everywhere except the inside. The model currently cannot tell those two worlds apart, and
they have opposite policy implications: the first is an individual problem with an
individual remedy, the second is an organizational one.

### 2.2 Why the employer's response cannot be assumed away

The chapter does not model the employer, and §7 says why. But one employer behaviour
bounds the whole exercise: whether the portfolio is scaled is the employer's decision,
not the worker's. A reduction is agreed, not declared. That makes $s_j$ the outcome of a
negotiation whose subject is precisely the team impact.

This is the practical reason the channel matters. A worker asking for 80% is not really
asking "may I work 80%". They are asking "may I work 80% and hand back 20% of the
portfolio", and the answer depends on whether anyone else can pick up that 20%. The
optimizer currently answers a question nobody is asked.

---

## 3. The model

### 3.1 Notation

| Symbol | Meaning | Range |
| --- | --- | --- |
| $T$ | The team, $\lvert T \rvert = n$ | — |
| $w_j$ | Contractual work percentage of member $j$ | $(0, 1]$ |
| $s_j$ | Assigned scope as a fraction of a full portfolio | $(0, 1]$ |
| $d_j$ | Delivered, quality-adjusted output (existing model) | $\geq 0$ |
| $h_j$ | Hidden work the shortfall forces on $j$ (existing) | $\geq 0$ |
| $\sigma_{ij}$ | Substitutability: the fraction of $j$'s residual work that member $i$ can actually take on | $[0, 1]$ |
| $\kappa_j$ | Coupling: the share of $j$'s work that requires synchronous presence with others | $[0, 1]$ |
| $A$ | Team absorptive capacity: extra output the rest of the team can supply without degrading its own delivery | $\geq 0$ |
| $K_j$ | Team credit: accumulated goodwill, in FTE-hours | $\mathbb{R}$ |
| $\lambda$ | Conversion of absorbed output into a colleague's leisure and health loss | $\geq 0$ |
| $\beta$ | Normative weight on colleagues' welfare | $[0, 1]$ |

### 3.2 Channel A — spillover

Define $j$'s average substitutability toward the rest of the team as
$\sigma_j = \frac{1}{n-1}\sum_{i \neq j} \sigma_{ij}$. The work transferred is

$$
\text{transferred}_j = \min\bigl(\sigma_j \cdot \text{raw}_j,\ A\bigr),
$$

$$
\text{residual}_j = \text{raw}_j - \text{transferred}_j .
$$

The residual lands back on $j$ as forced hidden work or as a missed goal, which is what
the existing model already computes. Two features of this expression are deliberate:

- **$\sigma$ is not one number per person but one per ordered pair.** $j$'s work may be
  fully fungible within their own sub-team and completely non-fungible outside it. The
  pairwise form is the honest one even though a first implementation would likely take a
  scalar; collapsing it is a modelling choice that should be visible in the code rather
  than buried in a constant.
- **$A$ is a cap, and it is endogenous.** Absorption consumes the rest of the team's
  slack. Once $A$ is exhausted, the next unit of transferred work is not absorbed; it
  degrades the colleagues' own delivery. That is the coupling back to their $q_j(w_j)$
  and $h_j$, and it is why the model cannot be solved member-by-member.

**The conservation statement.** This is the chapter's central equation:

$$
\underbrace{s_j G_{\text{full}}}_{\text{assigned}}
\;=\;
\underbrace{d_j}_{\text{delivered}}
\;+\;
\underbrace{h_j}_{\text{absorbed by } j}
\;+\;
\underbrace{\textstyle\sum_{i \neq j} \text{absorbed}_{i \to j}}_{\text{transferred}}
\;+\;
\underbrace{\text{missed}_j}_{\text{not delivered}} .
$$

Every term is observable in principle and only one of them is in the current model's
report. The equation is also the sharpest way to state the critique's concern: the model
has been reporting a world in which the second term is zero and the third does not exist.

### 3.3 Channel B — the coordination tax

Independently of who does the work, distributed availability costs coordination. Each
reduction in availability adds handovers, and each handover consumes time from people who
are not the ones who chose the reduction. The proposed form is

$$
L_h = c_h \sum_{j \in T} (1 - w_j)\,\kappa_j ,
$$

where $L_h$ is the extra coordination load in hours per week and $c_h$ is the one
coefficient. The load reduces $A$ before any work is transferred:

$$
A = A_0 - L_h .
$$

The linear form and the weighting by $\kappa$ are the substance of the model, not
incidental:

- **Linear in the reduction, weighted by coupling.** It says the cost of your 80% depends
  on how coupled your work is, not on your diligence or your intentions. That is a
  falsifiable claim, and §8 states how to test it.
- **Why not superlinear.** Brooks's law and the process-loss literature establish that
  coordination cost grows faster than headcount. But the variable that matters here is
  *availability mismatch*, not the number of people, and no measured exponent relates a
  reduction in someone's week to the team's coordination hours. A linear form is the
  weakest assumption available and it puts the entire magnitude into $c_h$, where a user
  can see it and argue with it. Inventing a quadratic because the neighbouring literature
  is superlinear would be exactly the plausible-narrative failure this project forbids.
- **The cost falls on the team, not on the person who chose.** This yields the chapter's
  most useful structural prediction: **one part-timer in a coupled team is a bottleneck; a
  whole team at 80% is nearly costless.** Two arrangements with identical individual
  hours have very different coordination costs, and no model that looks at one worker at a
  time can express the difference. This is also the mechanism the job-sharing evidence
  points at, in both directions: shared arrangements work when they are designed as such,
  and go wrong when the partner or the split is badly matched.

### 3.4 Channel C — team credit as a stock

The literature has a name for the behaviour that makes absorption possible: discretionary
helping that is not in anyone's job description. It is finite, it is reciprocal, and it
accrues and depletes.

Model it as a stock per member:

$$
K_j(t+1) = (1 - \delta) K_j(t) + \text{deposits}_j(t) - \text{withdrawals}_j(t),
$$

with deposits the occasions on which $j$ covers for someone else or is available when it
matters, and withdrawals the absorbed shortfall that others took on. The constraint that
matters is not a flow:

$$
K_j(t) \geq K_{\min} \quad \text{for all } t .
$$

Why a stock and not a flow matters, in three steps:

1. **A flow constraint can be violated for one period.** A stock constraint is about
   whether the account survives the horizon. A schedule that is comfortable in an average
   month and insolvent after two years is a different object from one that is merely
   tight, and the current model cannot distinguish them.
2. **Credit is insurance.** It is what allows a worker to have a bad month — a child's
   illness, a bereavement, a project that turns — without the shortfall becoming a
   performance issue. A worker with no credit cannot afford a bad month at 80%. This is
   the reciprocal side of the externality, and it is why the sign of the team channel is
   not uniformly negative: the same pool that absorbs my shortfall is the pool that makes
   my reduced schedule survivable in the first place.
3. **It converts a judgement into a quantity.** §4 of the critique asks what evidence
   would show that an 80% schedule is sustainable rather than temporarily achieved. If
   the binding constraint is $K_j \geq K_{\min}$, then the evidence is a balance, and
   the observable questions are concrete: how often in the last year did someone cover
   for you, and how often did you cover for them?

One caution, stated because it limits the model: $\delta$ and $K_{\min}$ are properties of
a team's culture and of a manager's memory. They are not measurable in the sense that a
tax rate is measurable. §8 treats that as a reason to bound the model's claims, not as a
reason to pick a number.

### 3.5 Channel D — the externality, and the normative choice

The cost to a colleague $i$ who absorbs work from $j$ is

$$
\text{cost}_{i \leftarrow j} = \lambda \cdot \text{absorbed}_{i \to j} + \text{coordination share}_i ,
$$

in units of $i$'s own time, which converts into $i$'s own hidden work and quality drag
through the machinery the model already has. The optimizer therefore can and should
report: *this schedule costs your two colleagues roughly X hours a week each.*

Whether that cost should count against the recommendation is **not an empirical
question**. It is a choice about whose welfare the objective function is allowed to see.
The proposal is to expose it as one declared weight:

$$
U_{\text{total}} = U_{\text{worker}} + \beta \sum_{i \in T \setminus \{j\}} U_i ,
\qquad \beta \in [0, 1] .
$$

- $\beta = 0$ is the current behaviour, and the default. The optimizer answers the
  worker's question and prints the effects on everyone else without aggregating them.
- $\beta = 1$ is the utilitarian sum.
- Intermediate values are a concern parameter, and there is no honest way to name a
  "correct" one.

Two limits are worth stating on the page rather than leaving to be discovered:

- **$\beta$ does not include the employer.** The employer has an objective function too —
  profitability, delivery risk, the cost of replacing a worker — and it is out of scope.
  This chapter is about the people who did not choose the arrangement, not about the firm.
- **Because $\beta$ is normative, the more defensible output may be a frontier rather
  than a point.** Rather than asking the user to name $\beta$, the tool could report the
  set of work percentages that are not dominated *for the worker and the team jointly*,
  and let the user choose. That is the same treatment `multipolar_sim` gives to Pareto
  outcomes, and it avoids the model making an ethical decision on the user's behalf.

---

## 4. What the model would then be able to answer

Four questions, of which the current model answers one:

| Question | Status |
| --- | --- |
| 1. Can I deliver my assigned goals at 80%? | **Answered today** — the individual channel |
| 2. What does my 80% cost the people I work with? | **New, reporting only** — channels A, B and D |
| 3. Would the team be better off if I went to 80%? | **New, normative** — requires $\beta$, or a frontier |
| 4. Is the arrangement sustainable, or will the account run out? | **New, dynamic** — the stock in channel C |

Question 4 is the one that most changes what the tool is for. The current optimizer
produces an answer for a point in time. A stock constraint produces an answer about a
duration, which is the form the question takes in real life: not "is 80% feasible" but
"is 80% still feasible in the third year".

**And a fifth, which is arguably the whole point:**

| Question | Status |
| --- | --- |
| 5. Should I be asking for 80% of the hours, or for 80% of the portfolio? | **The reframing** |

The team externality is zero when $s_j = w_j$. It is maximal when $s_j = 1$. So the
optimizer's recommendation should probably be a **pair** $(w_j, s_j)$ rather than a single
percentage — or equivalently, 80% of the hours *with* a specified handback. That is a
change to the shape of the output, not an extra coefficient, and it follows from the
conservation equation alone. It also makes the tool's advice negotiable in a way that
"work 80%" never was: a worker who walks in with a scope reduction to propose is asking a
question the employer can answer.

---

## 5. What is measurable, and what would be invented

The project's rule is that a parameter may be illustrative but may not be presented as
measured. Applied here, that splits the inputs cleanly.

### 5.1 Observable by the worker, with no estimate

| Quantity | How the worker knows it |
| --- | --- |
| $w_j$ | Their contract |
| $s_j$ | Their portfolio: how many projects, how much of each they own. This is the input that matters most and it is currently a single number on the command line |
| $n$, and each colleague's $w_i$ | Visible in any team |
| Whether a skill is uniquely held | The bus-factor question: who else can do this |
| Whether work requires synchronous presence ($\kappa$) | Whether the job is meetings-and-decisions or artefacts-and-deadlines |
| How often a colleague covered for you last quarter | Recoverable from memory or a calendar, and the number that anchors $K_j$ |

### 5.2 Declared, with one visible coefficient each

| Coefficient | What it controls | Why it cannot be estimated here |
| --- | --- | --- |
| $c_h$ | Coordination hours per unit of reduced availability in a coupled role | No measured relation between an individual's hours and the team's coordination load |
| $\sigma_{ij}$ | How much of $j$'s work $i$ can take on | Task-specific, tacit, and known to the team rather than derivable |
| $A_0$ | Team slack before any transfer | The employer's staffing decision, not the worker's information |
| $\delta$, $K_{\min}$ | How fast goodwill decays and how little of it is survivable | Properties of a team's culture and a manager's memory |
| $\lambda$ | Absorbed hours converted into a colleague's welfare | Requires a welfare comparison the model has no basis for |
| $\beta$ | How much colleagues' welfare counts | A normative choice, not a measurement |

### 5.3 Nothing is taken from the literature

This is the part that has to be said plainly, because the temptation is strongest here.

**No coefficient in this chapter is taken from any source.** The sources in §9 establish
that the mechanisms exist and are studied:

- coordination cost grows with the number of communication paths, and people
  systematically underestimate it;
- groups produce less than the sum of their members' individual capacities;
- individuals exert less effort in a group than alone;
- discretionary helping that nobody is obliged to provide is what makes a team absorb
  variation, and it is governed by reciprocity;
- firms deliberately hold slack as a buffer, which is why the buffer is finite;
- flexibility arrangements carry a career penalty, which is the informal version of the
  replacement risk the model already prices;
- job-sharing arrangements, when badly designed or badly matched, raise work intensity and
  overtime rather than lowering them.

Every one of those is a *mechanism*, and a mechanism is what a model needs to be
structurally right. None of them supplies a number that could be transferred to
$c_h$, $\sigma$, $A_0$, $\delta$ or $\lambda$, and pretending otherwise would produce a
figure with the authority of a citation and the substance of a guess.

### 5.4 A calibration path, instead of a calibrated value

The honest alternative to inventing $c_h$ is to say how a user with a real team could
measure it, and to make the tool accept the measurement when it exists:

1. **Baseline the coordination load.** Count recurring meetings and handover events in a
   normal month, and the hours they consume, for the team rather than for one person.
2. **Reduce and re-count.** When someone does reduce, re-count after a quarter. The
   difference, divided by the reduction in availability, is $c_h$ for that team.
3. **Record the coverage.** Every occasion on which someone covered for someone else,
   with the hours. The ratio of absorbed hours to shortfall hours is $\sigma$ for that
   task family.
4. **Watch the balance.** Deposits and withdrawals over a year give $\delta$ and an
   order of magnitude for $K_{\min}$.

None of this is available to a standalone binary, and all of it is available to a team
that decides to look. The tool's job is to hold the coefficient that the team measures.

---

## 6. Why the tool cannot estimate any of this

Worth stating separately, because "not modelled" and "not measurable by this program" are
different claims and only the second is a permanent limit.

- **The data belongs to the employer.** Coverage, handovers and slack are visible in
  calendars, ticket systems and rosters, none of which the worker controls or can export
  reliably.
- **Substitutability is tacit.** A team knows who can take over what; there is no
  document that says so, which is the same reason the bus-factor question is asked in
  interviews rather than computed.
- **The counterfactual is unobservable.** Nobody can observe what their colleagues would
  have done had they stayed at 100%. Any estimate of the externality is a comparison
  against a world that did not happen.
- **The population is selected.** Workers who reduce are not a random sample of workers,
  and the ones who reduce successfully are the ones who had the slack, the credit, or the
  employer's goodwill to begin with. Estimates from observed reductions are biased in a
  direction that is hard even to sign.

The consequence is a design rule rather than a limitation: **every team input must be
declared by the user, with its provenance visible, and the report must say which parts of
its answer came from declarations rather than from measurement.** That is the same rule
the tax tables already follow, applied to a quantity that is genuinely softer.

---

## 7. Implementation proposal

Not implemented. This section is the design so that it can be reviewed before it is coded.

### 7.1 Shape

A new module `src/team.rs`, deliberately separate from `AchievementConstraint`:

```text
pub struct TeamMember   { label, work_percentage, scope, coupling, substitutability, credit }
pub struct TeamLink     { from, to, substitutability }
pub struct TeamConfig   { members, links, slack, handover_cost, credit_decay, credit_min,
                          colleague_weight, worker_index }
pub struct TeamOutcome  { transferred, coordination_load, internalised, missed,
                          credit_by_year, colleague_cost, feasible, sustainability }
```

It is a separate type rather than more fields on `AchievementConstraint` because it
answers a different question. A team is not another property of the worker's capacity.

### 7.2 Wiring and the neutrality requirement

```text
OptimizerConfig.team: Option<TeamConfig>     // default None
```

With `team: None` the optimizer's behaviour, its report, and its numbers must be
byte-identical to today's. That is not a courtesy to existing users; it is the property
that makes the feature reviewable, and it is checkable by running the existing suites
unchanged.

A first CLI cut should be three flags, because they determine the *shape* of the answer:

```text
--colleague-work-percentages 100,100,80,100   the team around you
--task-substitutability 0.4                   how much of your work others can absorb
--team-coupling 0.6                           how much of it needs you present
```

and a second tier for the magnitudes, which scale the result rather than changing its
direction:

```text
--assigned-scope 1.0        s_j: how much of a full portfolio you still hold
--team-slack 0.5            A_0, in FTE-months of spare capacity
--team-credit 1.0           K_j at the start
--team-credit-min 0.2       K_min
--handover-cost 0.05        c_h
--team-weight 0.0           beta
```

`--assigned-scope` is the flag this chapter most wants, and it is also the one that can
be added to the model *before* any of the others, because it is a single number the
worker already knows and it determines whether the team channel is even open.

### 7.3 Report

A `TEAM IMPACT` block beside the achievement block, in the same voice as the existing
one: state what was declared, what follows, and what the number cannot see.

```text
Team Impact:
  Assigned scope:  100% of a full portfolio at 80% hours
  Substitutability:40% of the shortfall can be absorbed
  Transferred:     0.08 FTE  (12.6 h/week across 3 colleagues)
  Coordination:    +2.1 h/week team-wide, from your coupling
  Internalised:    0.04 FTE  (already counted as hidden work above)
  Team credit:     1.00 → 0.31 over 5 years (floor 0.20)
    Sustainable at this rate, but only just: the account is
    exhausted in year 7 if the current draw continues.
```

### 7.4 Tests to add

- **Conservation.** The four terms of §3.2 sum to the assigned scope, exactly, for every
  scenario — including the degenerate ones where substitutability is zero or the team is
  empty.
- **Neutrality.** `team: None` produces output identical to the pre-change binary.
- **The structural result.** One part-timer in a fully coupled team fares worse than the
  same total reduction spread across the team. This is the prediction that distinguishes
  the model from a per-worker one and it should fail loudly if the implementation loses it.
- **The scope identity.** With $s_j = w_j$ the team channel contributes exactly nothing,
  for any values of the other parameters. This is the §1 claim and it is cheap to pin.
- **Credit exhaustion.** A draw that is feasible on a one-year flow becomes infeasible
  over the horizon when the stock binds.
- **$\beta$ monotonicity.** Raising $\beta$ never makes a schedule with a larger
  colleague cost look better.

### 7.5 What to do first

If only one thing is built, build `--assigned-scope` and report the conservation identity.
It requires no new coefficient, it makes the existing `hidden_work_percentage` honest by
naming what it assumes (that the worker absorbs everything), and it converts the tool's
recommendation from a percentage into a percentage *and* a handback. The rest of the
chapter scales a result whose direction is already fixed by that one input.

---

## 8. What would falsify this

The chapter is only worth the space if it can be wrong in a way that could be observed.

1. **If substitutability is near zero in practice**, the spillover channel collapses and
   the model reduces to the current one. The test is direct: in one real team, measure how
   many hours of a reduced member's work were actually taken on by others. If the answer
   is "almost none, the work simply stopped", then a reduced schedule is a scope
   negotiation or it is a missed goal, and team absorption is a story rather than a
   mechanism.
2. **If coordination cost scales with the *count* of part-timers rather than with
   availability mismatch**, the $\kappa$-weighted form is wrong and a team of five at 80%
   costs what five separate part-timers cost. The Eurofound reading on job sharing points
   the other way — that well-designed shared arrangements absorb the cost of the split —
   but that is one secondary source about a different question.
3. **If $K_{\min}$ is not a real constraint** — if teams in practice absorb indefinitely
   because the absorbers have no way to refuse — then credit is not a stock that can run
   out, and the sustainability question stays a flow question. This is a claim about
   workplace power and it is testable by asking people who have run reduced schedules for
   years.
4. **If the informal penalty is the binding constraint rather than the workload**, then
   the important parameter is not $c_h$ or $\sigma$ but the career cost the model keeps
   out of scope. The flexibility-stigma literature suggests this may be the dominant
   channel, and if so, a model of hours and output is the wrong instrument and the honest
   output is a warning rather than a recommendation.

Point 4 is the one that should make a reader uneasy, and it is the reason §7.3 prints the
numeration of what was *declared* rather than presenting a single number. The tool can
price the workload. It cannot price what a manager concludes about someone who is not
there on Fridays.

---

## 9. Sources

Each entry says what it supports and, in the same line, that no number was taken from it.
The list is short on purpose: mechanisms that are established, not coefficients.

1. **Brooks, F. P. (1975). _The Mythical Man-Month: Essays on Software Engineering_. Addison-Wesley.**
   Communication paths in a team grow as $n(n-1)/2$, and adding people to a late project
   delays it. Supports the existence of coordination cost and its superlinearity in
   *headcount* — which is deliberately **not** carried over to availability mismatch in
   §3.3. No coefficient taken.

2. **Steiner, I. D. (1972). _Group Process and Productivity_. Academic Press.**
   Process loss: a group's output falls short of its members' potential, and the shortfall
   is attributable to coordination and motivation rather than to ability. Supports the
   framing of channels A and B as distinct losses. No coefficient taken.

3. **Latané, B., Williams, K., & Harkins, S. (1979). "Many hands make light the work: The
   causes and consequences of social loafing." _Journal of Personality and Social
   Psychology_, 37(6), 822–832.**
   Individual effort falls as group size rises when contributions are not individually
   identifiable. This is the mirror image of the chapter's externality — the absorbers may
   themselves exert less when the shortfall is diffuse — and it is a reason $\sigma$ is
   not a constant. No coefficient taken.

4. **Organ, D. W. (1988). _Organizational Citizenship Behavior: The Good Soldier
   Syndrome_. Lexington Books.**
   The discretionary, unrewarded helping behaviour that is not in anyone's job
   description, and the reason absorption is possible at all. Support for channel C being
   a stock of discretionary effort rather than a duty. No coefficient taken.

5. **Gouldner, A. W. (1960). "The norm of reciprocity: A preliminary statement."
   _American Sociological Review_, 25(2), 161–178.**
   Reciprocity as a governing norm: helping creates an obligation to return it. This is
   the mechanism by which credit depletes and why the externality is not simply a transfer
   that everyone ignores. No coefficient taken.

6. **Cyert, R. M., & March, J. G. (1963). _A Behavioral Theory of the Firm_.
   Prentice-Hall.**
   Organizational slack as a deliberate buffer that absorbs variation. Supports treating
   the team's absorptive capacity as finite and held on purpose, rather than as an
   unlimited pool of goodwill. No coefficient taken.

7. **Heath, C., & Staudenmayer, N. (2000). "Coordination neglect: How lay theories of
   organizing complicate coordination in organizations." _Research in Organizational
   Behavior_, 22, 153–191.** ([record](https://www.semanticscholar.org/paper/Coordination-Neglect%3A-How-Lay-Theories-of-in-Heath-Staudenmayer/4dfc056865608ce0be809c42a1d2bb6988e9e3ee))
   People systematically underestimate coordination requirements when they redesign work.
   Directly relevant to why a worker asking for 80% does not anticipate channel B, and why
   the tool should show it rather than assume it was considered. No coefficient taken.

8. **Coltrane, S., Miller, E. C., DeHaan, T., & Stewart, L. (2013). "Fathers and the
   flexibility stigma." _Journal of Social Issues_, 69(2), 279–302.
   doi:10.1111/josi.12015** ([record](https://openurl.ebsco.com/EPDB%3Agcd%3A1%3A19075889/detailv2?sid=ebsco%3Aocu_results%3Asitemap&id=ebsco%3Adoi%3A10.1111%2Fjosi.12015&bquery=DE%20%22UNEMPLOYED%20people%22&page=1&crl=f&link_origin=none))
   Workers who use flexibility arrangements are read as less committed and pay a career
   penalty. This is the informal counterpart of the replacement risk the model already
   prices, and the reason §8 point 4 is a live worry. No coefficient taken.

9. **Correll, S. J., Benard, S., & Paik, I. (2007). "Getting a job: Is there a motherhood
   penalty?" _American Journal of Sociology_, 112(5), 1297–1338.**
   Experimental evidence that caregiving status produces a measurable evaluative penalty
   independent of productivity. Supports the claim that the cost of a reduced schedule can
   fall through a channel the workload model does not see. No coefficient taken.

10. **Eurofound (2015). _New forms of employment_. Publications Office of the European
    Union, Luxembourg.**
    Job sharing as an employment form, and the finding that poorly designed or badly
    matched sharing raises work intensity, overtime demands and work-related stress rather
    than lowering them. This is the closest thing to direct evidence for §3.3's structural
    prediction. **Secondary citation:** the specific finding was read through a national
    research institute's summary rather than the primary report, and should be checked
    against the original before being quoted anywhere that matters. No coefficient taken.

11. **Stadt Zürich, _Merkblatt zu Teilzeitarbeit_.** ([PDF](https://www.stadt-zuerich.ch/content/dam/web/de/lebenslagen/unterstuetzung-und-beratung/dokumente/gleichstellung/vereinbarkeit-von-familie-und-beruf/2023-merkblatt-teilzeitarbeit.pdf))
    A practical Swiss source on how part-time arrangements are actually agreed. Listed as a
    **lead, not as a source read here**: this chapter makes no claim about Swiss
    employment law, and any legal statement about a right to reduce one's percentage
    should be checked against primary law rather than against this chapter. No coefficient
    taken.

### A note on how this list was assembled

These are established references, and the titles and identifiers were checked against
indexing records rather than taken from memory alone. They were **not** read in full for
this chapter, and no claim above depends on a figure, a page number, or a specific
estimate from any of them. Each is cited for the existence of a mechanism. If any of these
citations is to appear in a publication rather than in a repository document, it should be
checked against the original first — the same rule the tax tables follow, applied to a
softer literature.

---

## 10. Open questions this chapter does not answer

- **How to price a colleague's welfare at all.** $\lambda$ converts absorbed hours into
  someone else's loss, and there is no defensible exchange rate between one person's
  leisure and another's. This is why the proposal reports rather than aggregates by
  default.
- **Whether $\beta$ should exist.** A Pareto frontier avoids the normative choice
  entirely, at the cost of a harder output to read. Which is better is an open design
  question, not a settled one.
- **The second mover's problem.** If one colleague reduces, the next reduction is cheaper
  because the team is already reorganised around reduced availability. That makes going
  last a subsidy, and it is a coordination game the single-worker model cannot see.
- **The employer's side.** Whether $s_j < w_j$ is granted is the actual decision, and the
  employer's costs — replacement, retraining, delivery risk — are outside this model
  entirely. Until that is modelled, every recommendation here is advice to one party in a
  two-party negotiation.
- **Whether this should be a model at all.** If the binding constraint is what a manager
  concludes rather than what a schedule costs, the honest instrument is a conversation
  protocol rather than a parameterised optimizer. §8 point 4 is the test, and this chapter
  does not pretend to have run it.
