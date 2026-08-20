# Philosophical & Sociological Aspects of Life Optimizer

*Or: what a work-percentage calculator cannot tell you, and why it's worth saying so.*

---

## Why this document exists

Every other document in this repository treats the model as a tool: inputs go in,
a utility score comes out, and the math is transparent enough that you can check
the reasoning yourself. That transparency is the project's main design commitment
(see the [README](README.md)'s "Motivation and design philosophy").

But transparency about the *math* is not the same as transparency about the
*assumptions underneath the math*. A utility function has to decide, in advance,
what counts as a good life — and that decision is philosophical, not computational.
This document tries to make those assumptions visible instead of quietly encoding
them in weights and formulas.

Sections 3 and 8 below build directly on two outside critiques recorded in
[`CRITICS_CURRENT_WORK.md`](CRITICS_CURRENT_WORK.md) — one on whether "work
percentage" is really a free choice at all, one on the missing consumption side of
the ledger. Both turn out to be philosophical questions as much as modeling gaps,
which is why they belong here as well as there.

---

## 1. The problem with optimizing a life

The tool computes:

$$
U = \sum_{t} \beta^t \cdot u(\text{consumption}, \text{leisure}, \text{family}, \text{health}, \text{security})
$$

This is a **utilitarian** framing: reduce a life to a small set of measurable
components, assign weights, sum them up, maximize. It's the same logic economics
has used since Bentham and Jevons, and it is genuinely useful for the narrow
question the tool asks ("60%, 80%, or 100%?"). But it is worth being honest about
what gets lost:

- **Not everything that matters is measurable.** Meaning, purpose, dignity,
  creative fulfillment, and love do not compress cleanly into a consumption term
  and a leisure term. The model's "family" and "health" components are proxies —
  useful ones, but proxies nonetheless.
- **Averaging obscures texture.** A life with intense highs and lows can produce
  the same expected utility as a flat, comfortable one. Whether that equivalence
  is meaningful depends on values the model cannot supply.
- **The weights are a moral choice disguised as a parameter.** Setting
  `family = 0.3, security = 0.2` is not a neutral technical decision — it's a
  stance on what a good life consists of, made explicit here specifically so it
  can be interrogated rather than accepted by default.

This is not a flaw to be fixed with a better formula. It is a structural limit of
any optimization approach to human life, and the honest response is to say so
clearly rather than to imply the model has resolved it.

---

## 2. Whose optimum is it, anyway? Individual choice inside a fixed system

The tool treats work percentage as a free individual choice: you pick 60%, 80%,
or 100%, and the model tells you the consequence. Sociologically, this framing is
incomplete in at least three ways.

**a) The menu of options is not neutral.** Whether 80% is *available* to you at all
depends on your employer, your profession, your negotiating power, and your
country's labor law — none of which the model touches. A calculator that
optimizes within a menu can make the menu itself look natural, when it is in fact
a social and institutional artifact.

**b) Care work is unevenly distributed.** In Switzerland, part-time work remains
strongly gendered: the reduction from 100% to 80% is disproportionately taken up
by women, often coupled with unpaid childcare and household labor that does not
appear anywhere in a Lohnausweis. A model that only sees gross salary and BVG
contributions will systematically underweight the true cost of reduced paid work
for the person who also absorbs the unpaid work — and can inadvertently make a
structurally unequal arrangement look like an optimal personal choice.

**c) "Optimal" for the individual is not "optimal" for the collective.** The
Swiss AHV system, in particular, is a **pay-as-you-go intergenerational contract**:
today's contributions fund today's retirees, not your own future self. A
population that collectively chooses lower work percentages changes the ratio
this system depends on. The model, by design, only ever answers "what's best for
you" — it has no voice for "what happens if everyone does this."

None of this means individual optimization is illegitimate. It means the tool
answers a narrower question than "how should I live," and conflating the two is
the actual risk.

---

## 3. Two critiques that sharpen this problem

The point above — that work percentage is not simply a free individual choice —
is not just a theoretical concern. It was raised directly, and independently, by
two outside readers of this project (documented in full in
[`CRITICS_CURRENT_WORK.md`](CRITICS_CURRENT_WORK.md)). Both critiques deserve to
be taken seriously here, because they are really the same philosophical objection
approached from two different directions: **the model treats "80%" as if it were
a self-contained number, when in reality it is embedded in an employment contract
on one side and a lifestyle on the other.**

### 3a. Work is often judged by output, not hours — so what does "80%" even mean?

Mr. Gradimir Nikolic's critique is, at bottom, a critique of the model's unit of
analysis. Many employment relationships — especially project-based, knowledge, or
engineering work — are governed not by a clock but by a **deliverable**: a
steering committee assigns five or six projects for the year, and the worker is
expected to deliver them, whether that takes 80% of a working week or 110%. Under
that regime, "reducing to 80%" is not a private lifestyle adjustment; it is a
renegotiation of a performance contract, with the underlying, and very real, risk
of being judged as underdelivering and replaced.

This reframes the model's central question. It is not:

> *What work percentage maximizes my utility?*

but rather:

> *What is the lowest work percentage at which I can still reliably deliver what
> is expected of me — with or without the help of tools like AI — while
> preserving my health, income, leisure, and long-term security?*

The second question is philosophically more honest, because it acknowledges that
**freedom over time exists only inside the boundaries of an obligation to
someone else.** This is an old idea, not a new one — it is close to what
sociologist Harry Braverman called the *indeterminacy of labor* in
*Labor and Monopoly Capital* (1974): what an employer actually buys is not a
fixed quantity of "labor time," but a *capacity* to produce results, which must
then be extracted, measured, and enforced. A calculator that only prices hours
is quietly assuming that the indeterminacy problem has already been solved in the
worker's favor — which, for many jobs, it has not.

The critique's proposed formalization —

$$
A_t = H_t \cdot P_t \cdot (1 + \alpha_t) \qquad \text{subject to} \qquad A_t \geq G_t
$$

(effective achievement capacity must meet required project goals, where `α_t` is
an AI-driven productivity gain) — is worth sitting with philosophically as well as
mathematically. It implies that **AI does not, by itself, buy back anyone's
time.** It only creates the *possibility* of buying back time, and whether that
possibility is realized depends on a second, entirely social question: does the
employer bank the productivity gain as more free time for the worker, or does it
simply raise `G_t` — the target — to match the new capacity? That is not a
technical question the model can answer; it is a question about bargaining power,
industry norms, and whether productivity gains are shared or captured. (See
Section 5 below.)

### 3b. Income without consumption tells you nothing about freedom

Mr. Bojan Nedic's critique attacks the model from the other end: even if income
and tax are modeled perfectly, the model still cannot say whether an 80% schedule
is *livable*, because it has no explicit representation of what a person actually
spends — rent, lifestyle, debt. Two people with identical after-tax income at 80%
can be in completely different positions: one comfortable, one unable to cover
rent.

This is philosophically adjacent to a distinction economist and philosopher
Amartya Sen draws between **income** and **capability** (*Development as Freedom*,
1999, already cited above): income is only instrumentally valuable — what matters
is what a person is actually able to *do and be* with it. A model that stops at
after-tax income and calls it "consumption utility" is implicitly assuming a
uniform, unstated lifestyle. Making rent, debt, and a lifestyle tier
(`extreme_saving | moderate | normal | luxury`) explicit inputs, as the critique
proposes, does not just improve realism — it removes a hidden and unexamined
value judgment about what "enough" looks like, and hands that judgment back to
the person using the tool, where it belongs.

It is also worth naming, gently, what this critique implies about the earlier
discussion of positional goods (Section 5 below, and Veblen): a "luxury" lifestyle
tier is not a fixed, objective category. What counts as normal consumption is
itself socially constructed and shifts with reference group — which is exactly
why the model should expose the *assumption* as a parameter the user sets,
rather than baking one lifestyle level silently into "personal requirements."

### 3c. Whose side is Section 3a actually argued from?

There is one more layer worth separating out, because it changes the character
of the critique entirely. Mr. Nikolic's observation — that a worker is a resource
assigned to deliverables, and that "80%" is only credible if `A_t ≥ G_t` still
holds — is, on inspection, **an employer's-eye view of the problem.** It answers
the question "can the organization still get what it needs?" It does not yet ask
"does the worker, as a person, actually want to be working 100% on those five
projects, or would they be better off, and *better at their job*, working less?"

These are not the same question, and conflating them hides a genuine empirical
and philosophical debate. A large body of organizational psychology argues that
the causal arrow is not "the worker chooses less work, therefore output drops."
It is closer to the reverse: **a satisfied worker is frequently a more productive
one.** Meta-analyses since the 1990s (see, e.g., Judge et al., 2001) find a real,
if moderate, positive correlation between job satisfaction and job performance.
If that relationship holds for a given individual, then an employer who insists
on 100% out of the belief that hours alone secure `G_t` may be optimizing the
wrong variable — burning out precisely the capacity, `α_t` and `P_t`, that the
formula depends on. Section 3a's model, read only from the employer's side, is
therefore missing its own feedback loop: work percentage does not just consume
capacity, over time it also *shapes* it.

But this only pushes the real question back one level, and it is the one worth
sitting with, because the model cannot answer it for anyone: **what actually
makes a worker happy — delivering the big project and feeling the achievement of
having contributed real expertise, or having more hours with family and life
outside work?** This is not a question the tool, or this document, can resolve
in general, because the honest answer is: *it depends on the person.*

Psychology offers useful vocabulary for the split without resolving it.
Self-determination theory (Deci & Ryan, *Self-Determination Theory*, 2017)
argues that wellbeing depends on three needs — autonomy, competence, and
relatedness — and work can supply all three just as much as family life can. A
demanding project that stretches real expertise and is delivered successfully
can be a genuine source of *competence* satisfaction, not merely an obstacle to
leisure. Mihaly Csikszentmihalyi's concept of *flow* (*Flow: The Psychology of
Optimal Experience*, 1990) makes a related point empirically: people frequently
report their most engaged, most positive psychological states not during
leisure, but during demanding, skill-matched work. And Amy Wrzesniewski's
research on **work orientation** (Wrzesniewski & Rosso, *Job, Career, and
Calling*, 1997 onward) distinguishes people who relate to their work as a *job*
(a means to income), a *career* (a path to advancement), or a *calling* (a source
of identity and meaning in itself) — the same 100%-vs-80% choice can be a relief
for the first person and a real loss for the third.

This reframes what the model's `--profile career` weighting is actually doing.
As implemented, "career-focused" mostly re-weights toward income and long-term
security — it does not represent the person for whom the *work itself*, not just
its paycheck or its safety margin, is a primary source of wellbeing. That is a
real gap, and it is not a gap that more tax accuracy or a better Monte Carlo
model can close, because it is not a financial question. Aristotle's distinction
between *hedonia* (wellbeing as pleasant, restful experience) and *eudaimonia*
(wellbeing as flourishing through activity that exercises one's capacities well,
*Nicomachean Ethics*, Book I) is the oldest version of exactly this fork: for
some people, being fully engaged in a demanding project *is* the flourishing, not
the thing traded away to get to it.

The honest position, then, is not that the model should assume more leisure is
always better — it already avoids that by making leisure's utility concave with
diminishing returns rather than unbounded. The honest position is that the model
has no way of knowing, and cannot be made to know from salary and tax data alone,
*which kind of person is using it.* That has to come from the person, as an input
as real as salary or age — closer to a question the person needs to sit with
than a parameter that can be inferred.

---

## 4. Quantifying wellbeing: Goodhart's Law and the map/territory problem

> "When a measure becomes a target, it ceases to be a good measure."
> — commonly attributed to Goodhart's Law

The quality-of-life score (0–10) produced by the Monte Carlo pension module is a
compressed proxy for something much richer: whether retirement will actually feel
adequate, dignified, and secure. The moment that number becomes the thing you
optimize for — rather than a rough signal to sanity-check your intuition against —
it risks distorting the very thing it was meant to describe. A retirement that
scores 7/10 on this scale but is spent isolated, purposeless, or estranged from
family is not a good outcome, even though the model would call it "adequate."

This is the general map/territory problem applied to personal finance tools: the
model is not your life, and a good decision is one that uses the map without
mistaking it for the terrain.

---

## 5. Work, time, and the Protestant ethic

Max Weber's *The Protestant Ethic and the Spirit of Capitalism* (1905) traced how
diligent, continuous work came to be seen not just as economically useful but as a
sign of virtue and even salvation. That inheritance is still visible in how many
people relate to the number "100%": working less can feel, independent of any
financial analysis, like a moral failing rather than a neutral allocation of time.

The model's utility function pushes back against this by explicitly pricing
leisure and family time as *positive* contributors to a good life, with
diminishing marginal utility on both income and hours worked. That is itself a
philosophical stance — closer to Aristotle's view that leisure (*scholē*), not
labor, is the precondition for the highest human activities, than to a purely
productivist ethic. Making this explicit matters, because a person raised inside
the productivist frame may find the model's recommendation to work 80% instinctively
uncomfortable even when the numbers clearly support it — and it is worth knowing
that discomfort has a cultural history, rather than mistaking it for a private flaw.

---

## 6. Status, comparison, and the limits of "enough"

Economist Richard Easterlin observed that beyond a certain income threshold,
further income gains produce steadily diminishing returns to reported happiness —
partly because much of consumption is **positional**: a bigger apartment or newer
car matters largely relative to what peers have, not in absolute terms (Veblen's
"conspicuous consumption," 1899, made a related point about visible spending as
social signaling).

The model's consumption utility already reflects diminishing returns
mathematically (`u_c = ln(income/requirements)`), which happens to align with this
literature. But it cannot capture the *relational* dimension: whether reducing
your work percentage will feel like freedom or like falling behind depends heavily
on your social reference group, not just your own budget. Two people with
identical financial situations can experience the same 80% decision completely
differently depending on what their friends, colleagues, and family are doing.

---

## 7. Mortality, horizon, and why the pension module exists at all

Why simulate outcomes out to age 90 or later at all? Because the honest answer to
"how should I spend my working years" cannot avoid the fact that those years are
finite and that they end in a period — retirement, old age — where earning power
is gone and only what was built earlier remains.

This is close to Seneca's argument in *De Brevitate Vitae* ("On the Shortness of
Life," c. 49 CE): life is not short, we simply spend much of it as though it were
infinite, deferring what matters. The Monte Carlo and regime-switching modules in
this project are, underneath the statistics, a technical answer to a very old
question: *given that time and resources are finite and the future is uncertain,
how much of the present should be spent securing it?* Modeling recession risk at
retirement is not just financial prudence — it is an attempt to take the finitude
and uncertainty of a life seriously enough to plan around it, rather than assuming
away the bad scenarios because they are uncomfortable to model.

---

## 8. If AI buys back work-time, who actually receives the gain?

The critique summarized in Section 3a extends naturally into a larger question
that the [`CRITICS_CURRENT_WORK.md`](CRITICS_CURRENT_WORK.md) document raises
explicitly: if AI tools genuinely let a worker deliver the same output in 80% of
the time, is that a *personal* gain in leisure, or does the employer simply raise
the target and absorb the gain as extra output? This is not a new question —
Keynes asked a version of it in *Economic Possibilities for our Grandchildren*
(1930), predicting a 15-hour work week within a century on the strength of
compounding productivity. A century later, average working hours in most rich
countries have fallen only modestly, even as productivity has risen many times
over. The gap between Keynes's prediction and reality is itself the answer: **productivity
gains do not automatically become leisure.** Whether they do depends on
bargaining power, social norms, and institutions — not on the existence of the
technology.

Economist Thomas Piketty's central empirical claim in *Capital in the Twenty-First
Century* (2013) — that returns to capital have tended to outpace returns to
labor — is directly relevant here: if AI-driven productivity gains flow
disproportionately to capital owners rather than to the workers whose tasks are
augmented, then a model that quietly assumes "AI lets you work 80% for the same
pay" is assuming away the actual distributive question. The honest version of the
model's job-security constraint (`A_t ≥ G_t` from Section 3a) is therefore not
just an engineering detail — it is a small, local instance of a much larger
political-economy question about who captures the surplus from automation.

There is a second, quieter dimension to this that the critique also names: the
distinction between **paid work** and **meaningful activity**. Hannah Arendt, in
*The Human Condition* (1958), distinguishes *labor* (the cyclical, consumption-driven
activity of maintaining life), *work* (the durable creation of things that outlast
us), and *action* (participation in a shared public life). A future in which paid
labor genuinely shrinks is not automatically a good outcome — it is a good
outcome only if the freed time is convertible into work and action in Arendt's
sense: building, caregiving, learning, participating — rather than into isolation.
A work-percentage optimizer that only measures "leisure hours" cannot distinguish
between these outcomes, because from the outside they look identical: fewer paid
hours. The lived difference between them is enormous, and it lies entirely
outside what any calculator can see.

None of this is a reason to abandon the model. It is a reason to treat any
scenario where AI lowers the sustainable work percentage as an open empirical and
political question, not a settled input — exactly the posture the critique's
own "Open Research Questions" section takes.

---

## 9. The paradox of the tool itself

There is a final, self-referential tension worth naming. A tool like this can
reduce decision paralysis — but it can also become a new source of it, by
implying that a "correct" answer exists and can be computed, when in fact many
reasonable people with identical numbers would reasonably choose differently. The
philosopher Barry Schwartz's *The Paradox of Choice* (2004) argues that more
information and more optimization can increase anxiety rather than reduce it, if
it creates the expectation of a single right answer.

The intended use of this project is therefore not "trust the number," but closer
to what the README already states in its design philosophy: make the trade-offs
visible enough that *you* can reason about them, disagree with a weight, change an
assumption, and arrive at a decision you actually understand and can stand behind
— rather than one you outsourced to a formula.

---

## 10. Summary: what this model is, and is not

**It is:**
- A structured way to make implicit trade-offs (tax, time, pension risk) explicit
- A transparent, auditable, and challengeable set of assumptions
- A genuinely useful sanity check against purely intuitive decisions

**It is not:**
- A definition of a good life
- Neutral with respect to gender, labor-market structure, or intergenerational
  fairness
- Neutral with respect to *who* controls the "80%" decision — the employment
  contract and job-security risk sit outside the model entirely (Section 3a)
- A representation of what a person actually needs to spend to live at 80% —
  rent, lifestyle, and debt are not yet explicit inputs (Section 3b)
- A settled position on whether AI-driven productivity gains become leisure or
  simply raise expectations (Section 8)
- Aware of whether *you* are the kind of person for whom demanding work itself is
  a source of flourishing, or the kind for whom it is a cost traded against
  family and rest — that distinction (Section 3c) has to come from you, not from
  the model
- A replacement for judgment, values, relationships, or conversation with the
  people the decision actually affects

The most defensible use of this tool is the one implied by the rest of this
repository's philosophy: as an instrument for *reasoning together with*, not
*deferring to*. The critiques recorded in
[`CRITICS_CURRENT_WORK.md`](CRITICS_CURRENT_WORK.md) are themselves an example of
that posture working as intended — external readers stress-testing the model's
hidden assumptions rather than accepting its output.

---

## Further reading

- Max Weber, *The Protestant Ethic and the Spirit of Capitalism* (1905)
- Thorstein Veblen, *The Theory of the Leisure Class* (1899)
- Seneca, *De Brevitate Vitae* ("On the Shortness of Life," c. 49 CE)
- Barry Schwartz, *The Paradox of Choice* (2004)
- Richard Easterlin, "Does Economic Growth Improve the Human Lot?" (1974)
- Amartya Sen, *Development as Freedom* (1999) — on capabilities vs. utility as
  the right unit of measure for wellbeing
- Harry Braverman, *Labor and Monopoly Capital* (1974) — on the indeterminacy of
  labor and the gap between contracted time and required output
- Hannah Arendt, *The Human Condition* (1958) — on the distinction between labor,
  work, and action, relevant to what "freed time" is actually for
- John Maynard Keynes, "Economic Possibilities for our Grandchildren" (1930) —
  on why productivity gains have not historically translated into proportional
  leisure
- Thomas Piketty, *Capital in the Twenty-First Century* (2013) — on the
  distribution of returns between capital and labor
- Aristotle, *Nicomachean Ethics*, Book I — on *eudaimonia* (flourishing through
  activity) vs. *hedonia* (wellbeing as pleasant experience)
- Edward Deci & Richard Ryan, *Self-Determination Theory* (2017) — autonomy,
  competence, and relatedness as the basis of wellbeing, applicable to work as
  much as to leisure
- Mihaly Csikszentmihalyi, *Flow: The Psychology of Optimal Experience* (1990) —
  on engaged, skill-matched work as a source of peak psychological experience
- Amy Wrzesniewski & Kathryn Rosso, "Job, Career, and Calling" (in *The Oxford
  Handbook of Positive Psychology*, and related work from 1997 onward) — on the
  three orientations people hold toward their work
- Judge, T. A. et al., "The Job Satisfaction–Job Performance Relationship: A
  Qualitative and Quantitative Review" (*Psychological Bulletin*, 2001) — on the
  empirical link between worker satisfaction and performance
- [`CRITICS_CURRENT_WORK.md`](CRITICS_CURRENT_WORK.md) — the two critiques this
  document responds to directly, plus a curated reading list on AI, productivity,
  and the future of work (IMF, ILO, OECD, NBER, WEF)
- Charles Goodhart, on the measure-target distinction (1975)
