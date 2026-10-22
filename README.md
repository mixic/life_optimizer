<div align="center">

# Life Optimizer

**A Rust-based decision tool for the work-percentage question every Swiss employee eventually faces:
"Should I work 100%, 80%, or something else — and can I actually afford it, now and in retirement?"**

[![CI](https://github.com/mixic/life_optimizer/actions/workflows/ci.yml/badge.svg)](https://github.com/mixic/life_optimizer/actions/workflows/ci.yml)
[![Rust](https://img.shields.io/badge/rust-2021-orange.svg)](https://www.rust-lang.org)
[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](#license)

</div>

---

## Why this exists

Most take-home-pay calculators stop at "gross minus tax." That misses almost everything
that actually determines whether reducing your work percentage is a good idea:

- Swiss tax is **progressive**, so cutting your workload often costs less net income
  than the percentage suggests.
- Time has **value that changes with life stage** — an hour with a newborn is not
  worth the same as an hour at 24.
- A pension fund (Pillar 2 / BVG) built on 80% contributions for 30 years looks very
  different from one built on 100% — and that difference doesn't show up until
  you're 65 and it's too late to fix.
- Retirement isn't one number. It's 20-25 years exposed to inflation, market
  volatility, and the possibility of a recession hitting at exactly the wrong time.

**Life Optimizer** models all of it together — taxes, personal budget, time/health/family
utility, and Monte Carlo pension projections — and searches for the work percentage
that maximizes your overall quality of life, not just your paycheck.

**A second question, and a model built to answer it.** The same apparatus is applied to a question
about AI-assisted development: for a fixed body of code, does the authoring time that machine
generation saves survive the comprehension and maintenance cost of code the engineer did not write?
`comprehension_cost` estimates the net effect on cumulative engineering hours over a 36-month
horizon, decomposed into authoring, review, comprehension and maintenance. The grid, the decision
rule and the falsification condition were pre-registered before the model was run, and every
comprehension-side parameter is marked `Declared` rather than measured.

In the central parameterisation the net effect is negative — cumulative benefit turns negative in
month 3 and reaches −247 hours by month 36 — but the decomposition is more informative than the
sign. The authoring saving contributes **+13 h**. The mechanism the hypothesis proposes, a faster
decay of the mental model for code the engineer did not author, contributes **−39 h** and ranks
seventh of sixteen in a one-at-a-time sensitivity analysis, at 7% of the total variation. The
largest single channel is review leakage (**−127 h**): defects that a low-comprehension review
fails to catch generate their own maintenance stream. The ranking is led by the diagnosis-cost
terms (18% and 13%), the initial comprehension gap (11% for human-authored code, 8% for AI-authored)
and the leakage hazard (11%).

The inference available here is narrower than the effect size suggests. Because the load-bearing
parameters are unmeasured, the result identifies which assumptions the conclusion depends on — and
is a negative result about the model's power to settle the question — rather than evidence that
AI-authored code is net-negative in practice. What would test it is a task-level measurement of
diagnosis time on AI-authored versus human-authored modules; refining the decay rate would leave
the conclusion where it is. Pre-registration, decision rule, sensitivity tables and figures are in
[`AI_AUTHORING_AND_COMPREHENSION.md`](AI_AUTHORING_AND_COMPREHENSION.md).

### Design philosophy

A decision this consequential deserves a model you can argue with. So every formula is written
out, every input is either sourced or labelled as declared, and each assumption is placed where
you can see it move the answer:

- nothing important hides inside a black box;
- the trade-offs are stated as formulas rather than as weights chosen behind a slider;
- each variable — taxes, inflation, pension contributions, utility, health stress, and regime
  dependence — is visible, and the ones that cannot be measured are marked as such;
- the model supports reasoning rather than only prediction, which means it should tell you what it
  cannot establish.

Two things follow from that and are worth stating plainly, because they are the model's honesty
rather than its limitations:

**It reports a range where it cannot report a point.** Retirement needs are reported across 60/75/90
with the level at which the plan stops working; the recommendation comes with the margin to the
runner-up and with a robustness block showing how far it moves when a declared preference is moved;
and when the optimum sits at the edge of the percentages tested, the tool says so instead of calling
it stable.

**It names what it does not know.** `VALIDATION.md` records what has been checked against an outside
source and what has not. `CRITIQUE_RESPONSE.md` records an external review of this repository claim
by claim, including the places the review was wrong and the places it found a real defect.

The goal is not only to pick a number. It is to leave you able to say why the number is what it is,
and where it would stop being right.

---

## What it does

### 1. Work-life balance optimization
Runs a multi-objective utility function across candidate work percentages (50%–100%)
that weighs:
- **Consumption** — after-tax income vs. your personal budget
- **Leisure** — free time, with diminishing returns
- **Family** — weighted by life stage (new parent vs. empty nester)
- **Health** — a convex stress penalty for overwork
- **Security** — long-term pension adequacy

and returns the percentage that maximizes total utility, not just income.

### 2. Accurate Swiss tax modeling
- Official Stadt Bern 2024 tax tables (Kantons-, Gemeinde-, Kirchensteuer), looked up
  and interpolated by income, marital status, and children
- Mandatory social security layered on top (AHV/IV/EO, ALV, BVG)
- `--custom-tax-rate` override so you can plug in your own observed rate from a
  Lohnausweis instead of relying on tables that can't capture every personal deduction

### 3. Monte Carlo pension simulation
Every work-percentage scenario is stress-tested against your actual retirement, not
just projected with a single "expected return" assumption:
- **10,000-path simulation** under Conservative / Base Case / Optimistic market
  assumptions (log-normal returns calibrated to Swiss BVG fund data)
- **Economic regime-switching model** — a Markov chain across Boom / Normal /
  Recession / Stagflation states, so bad years cluster the way they do in reality
  instead of being averaged away
- **Sequence-of-returns stress test** — forces a recession to hit in the 2 years
  before through 1 year after retirement, the single most dangerous timing for a
  pension, and reports how much your income would actually drop
- Supports **deferred retirement up to age 70**, with age-scaled BVG contribution
  rates and the higher conversion rate you earn by working longer
- Outputs a **quality-of-life score (0–10)** for retirement, not just a CHF figure

### 4. Lifetime strategy
Projects the optimal work percentage across your whole career (age 30–65+), since
the right answer at 28 (single, building a foundation) is not the right answer at 38
(kids, time is scarce) or at 60 (health, winding down).

### 5. Employer achievement capacity, honestly constrained

A work percentage is not purely yours to choose: it is bounded by what your employer
requires you to deliver. Pass `--required-output-index` and the optimizer only
recommends percentages that actually deliver the assigned goals, with or without AI
leverage:

- **AI as a range, not a point** — feasibility is judged at the *pessimistic* end of
  the declared productivity range, so a schedule that only works if AI performs at
  the top is labelled `ONLY IF AI DELIVERS at the optimistic end` — a bet on the tool
  rather than a credible reduction
- **Quality channels** — the share of the AI gain that survives verification and
  rework, and the defects induced by compressing the same output into fewer hours
- **Hidden work reported, not absorbed** — if the pessimistic AI outcome would demand
  evenings or weekends, that is printed in h/week beside the contract. It is not
  credited as leisure, and it is not added to the hours either: working the cover and
  missing the goal are two answers to the same shortfall, not two costs to add up, and
  the model prices the second through the replacement risk
- **Replacement risk** — a missed goal can be priced as a probability of job loss
  (`--enforcement risk-weighted`, which requires `--replacement-risk`), or refused
  outright (`strict`, the default)
- **Two-level consumption** — rent, debt instalments and other unavoidable costs form
  a mandatory floor that does not shrink when hours do, while lifestyle profiles
  (extreme-saving / moderate / normal / luxury) show what the same schedule costs at
  each living standard

The reasoning, the provenance of every coefficient, and an explicit list of what is
*not* modelled are in [`CRITICS_CURRENT_WORK.md`](CRITICS_CURRENT_WORK.md) §7, with
worked commands in [`EXAMPLES.md`](EXAMPLES.md).

---

## Companion tool: the multipolar world simulator

This repository contains a second and independent tool. `multipolar_sim` is a
Monte Carlo simulator for a multipolar world: five power blocs, every pair of them
playing a 2×2 Cooperate/Compete game each year with its Nash equilibrium solved (pure
or mixed), random annual shocks, and what the resulting world implies for pension
security.

```bash
cargo run -p multipolar_sim -- --sweep     # the informative mode
cargo run -p multipolar_sim -- --psyche    # the psychological layer
cargo run -p multipolar_sim -- --help
```

It also carries a second, independent model: a psychological layer of the sort
[`THE_PSYCHOLOGICAL_LAYER.md`](documents/THE_PSYCHOLOGICAL_LAYER.md) documents, in which
assistance is *solved for* rather than chosen, and what it costs is the practice that
builds the capacity to judge. `--psyche` runs a 2,000-person cohort twice on the same
seed — once tracking the machine's tempo, once holding assistance still — and prints
the gap. Its parameters are declared for the same reason the world simulator's are
invented: no measurement exists for them.

It exists because the optimizer above answers "what is best for *you*" and has no way
to represent "what happens if everyone does this" — the question
[`PHILOSOPHICAL_SOCIOLOGICAL_ASPECTS.MD`](documents/PHILOSOPHICAL_SOCIOLOGICAL_ASPECTS.MD)
section 2c raises about AHV as a pay-as-you-go intergenerational contract. The theory
it operationalises is in [`MULTIPOLAR_GAME.md`](documents/MULTIPOLAR_GAME.md) section 4 and
[`THEORY_OF_SPARING.md`](documents/THEORY_OF_SPARING.md) section 7d.

> **Its parameters are illustrative, not calibrated, and nothing it prints is a
> forecast.** Run `--sweep`, which shows which conclusions hold across parameter
> ranges and which flip on small changes; that comparison is the result, not any
> single 50-year trajectory.

It is a **separate crate** rather than a module of `life-optimizer`, and that is
deliberate. Every figure the optimizer reports is either traced to an official source
or explicitly flagged as unsourced, with no fallback path by construction; the
simulator's figures are invented by construction. Keeping them in separate crates
means an illustrative number cannot reach a reported tax or pension figure by
accident — the boundary is a `use` statement that does not exist. The same separation
means a build failure in the simulator cannot block the optimizer. See
[`multipolar_sim/README.md`](multipolar_sim/README.md) for the model and
its module map.

---

## Third question: is AI authoring time real time?

`comprehension_cost` is a small, deliberately quarantined crate that asks whether the hours AI
saves writing code are given back understanding it. It is in its own crate for the same reason
`multipolar_sim` is: its parameters are hypotheses, and a `use` statement that does not exist is
the strongest guarantee that they cannot reach a reported tax or pension figure.

The design is a state-transition model. Modules carry an authoring mode, a comprehension level
that decays while untouched, a Poisson maintenance hazard, a resolution cost that rises as
comprehension falls, and a review gate whose catch probability is proportional to comprehension —
so what a low-comprehension review misses becomes a further stream of events.

It was **pre-registered**: the parameter ranges, the decision rule and the falsification condition
were committed before the model existed, so the verdict could not be chosen after seeing the
numbers. Two things came out of running it.

- **The verdict is SUPPORTED, and that is the least interesting part.** The central cell crosses
  over in month 3 and ends 247 hours worse off. But the decomposition says the authoring saving is
  worth **+13 hours over three years**, against −61 for review, −39 for slower retention, −34 for
  the defect multiplier and −127 for review leakage. The scale asymmetry is the argument: authoring
  savings are tens of hours, comprehension and maintenance costs are hundreds.
- **The answer does not depend on the parameter the hypothesis names.** The retention asymmetry
  ranks seventh of sixteen in a one-at-a-time sweep, at 7% of the variation. The top two are how
  much longer *diagnosis* takes when comprehension is low. That inverts the research priority and
  is a negative result about the model's power as much as about AI: the anchors in the literature
  do not constrain the parameters the conclusion actually rests on.

Two safeguards are worth naming. Every comprehension-side parameter is marked `Declared` — invented,
plausible in sign, unmeasured — and the CLI prints the provenance table on its own output. And there
is a control: with the comprehension asymmetry removed, the same model reports **+13 h, +29 h and
exactly 0 h** at 0%, 15% and 33% authoring savings. A model that could not report a win would make
the loss meaningless; that it can is the only reason the result means anything.

```bash
cargo run -p comprehension_cost -- --sweep        # the grid, verdict, sensitivity, break-even
cargo run -p comprehension_cost -- --verbose      # month by month, central cell
cargo test -p comprehension_cost                  # eleven tests, including both controls
```

Figures are in [`figures/`](figures) and are produced from the model's own CSV export by
`tools/plot_comprehension_cost.py`, which refuses to draw if the export is inconsistent.

---

## Example output

Abridged from a real run of `./life-optimizer optimize --salary 150000 --age 40`. Every block below
appears in an ordinary `optimize` invocation; sections that are not about the decision are elided
with `[...]` so the output can be read rather than scrolled. Nothing here is hand-written.

```
🎯 OPTIMAL SOLUTION FOUND!

Work Configuration:
  Work Percentage: 40%
  Gross Income:    60000 CHF/year
  After-Tax:       46148 CHF/year
  Monthly Net:     3846 CHF/month
  Tax Rate:        12.1% (official Bern table)

Financial Health:
  Status:          MEETS ALL REQUIREMENTS ✓
  Monthly Surplus: +86 CHF
    (tested against the mandatory floor of CHF 3760/month, not the full lifestyle basket)

Utility Score Breakdown:
  Consumption:     -0.06
  Leisure:         2.26
  Family:          2.59
  Health:          1.39
  Security:        1.20
  Weights:  consumption 0.25  leisure 0.20  family 0.25  health 0.15
            security 0.15
  TOTAL UTILITY:   7.34

  Colleagues:
    !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
    WARNING: THE EFFECT ON COLLEAGUES IS NOT MODELLED
    !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
    A reduced schedule usually moves work onto colleagues. This result does
    not account for that, so it cannot tell you whether the reduction is fair
    to your team, and it will favour such a schedule over one that
    genuinely produces less.

    This reduction is 25.2 hours a week if a team absorbs it, and those
    hours are currently attributed to nobody.

    Re-run with --team-size <n> (and --redistribution <0..1>) to see how many
    hours a week would land on each colleague.

[... pension projection, Monte Carlo and stress test ...]

Retirement needs, as an input rather than an assumption:
  You are evaluated at CHF 5590/month, from this household's modelled budget
  Projected income:  CHF 2207/month (BVG + AHV, real)
  -> BREAK-EVEN: this plan supports up to CHF 2207/month. You assume more than that.

   of that    needs/month   coverage     verdict
       60%           4472        49%       short
       75%           5590        39%       short <==
       90%           6708        33%       short

  The plan supports CHF 2207/month against a need of CHF 5590/month.
  Spending is not modelled as changing with age, health, or a partner's death.

Is the leader distinguishable from the runner-up?
      work    utility     margin    verdict
       40%       7.34       0.00     leader
       50%       7.17       0.17     behind
       60%       6.97       0.37     behind

  SEPARATED: 40% leads 50% by 0.17 utility points (2% of the leading score).
  The band is declared at 2% of the leading score: the utility scale is
  arbitrary, so a margin in points carries no meaning on its own.

Robustness: does the answer survive its own assumptions?
  Weights in use (declared, not measured):  consumption 0.25  leisure 0.20
  family 0.25  health 0.15  security 0.15

   input moved        by    optimal     shift
   consumption     x0.50        40%      none
   [... the other four weights, all "none" ...]
  consumption basket     x0.50        30%   -10 pts
  consumption basket     x2.00        90%   +50 pts

  SENSITIVE: The answer is 40%, but it moves between 30% and 90% when a single
  declared input is moved (widest shift 50 points). The table says which one.
  Treat the margin, not the leader, as the finding.
```

Read that output as an argument rather than a verdict. The weights do not move the answer at all;
the consumption basket moves it by fifty points. So the number rests on the budget model and not on
the preferences — the opposite of what most people would guess, and the kind of thing only a printed
sensitivity table can tell you.```

---

## Installation

Requires [Rust](https://www.rust-lang.org/tools/install) (2021 edition).

```bash
git clone https://github.com/mixic/life_optimizer.git
cd life-optimizer
cargo build --release
```

The binary will be at `target/release/life-optimizer` (or `.exe` on Windows).

This is a Cargo workspace. A bare `cargo build --release` builds only the
`life-optimizer` package, which is what the commands below use; the companion
simulator is a separate crate, so build both with `cargo build --release --workspace`
or just run it with `cargo run -p multipolar_sim`.

---

## Usage

### The interactive mode, as a transcript

Twelve flags are not an entry point. `interactive` asks the questions instead, and this is a real
session — the answers were piped in, so they do not echo inline in the captured output:

```
$ ./life-optimizer interactive

What is your full-time annual salary (CHF)? 120000
What is your age? 40
Are you married? (y/n) n
How many children do you have? 0

What are your priorities?
1. Balanced (equal weight to all factors)
2. Family-focused (prioritize time with family)
3. Career-focused (prioritize income and security)
Choose (1/2/3): 1

What will you need per month in retirement, in CHF?
  Leave blank to use this household's modelled budget.
  The result reports a 60/75/90 range and the level at which the plan stops
  working either way, so this is a starting point rather than a commitment.
Monthly need (CHF, blank for the modelled budget): 4500
  Using your figure.

Analyzing your situation...
```

Blank answers are taken as "use the model": a blank salary falls back to CHF 80,000, a blank age to
35, and a blank retirement need to this household's modelled budget rather than to a rule of thumb.

The interactive mode does **not** ask about your canton, your team, or the AI-productivity
coefficients. Canton defaults to Bern; the team externality is reported as not modelled, with the
warning that implies; the AI model is off. Each is available as a flag on the non-interactive
command.

### Find your optimal work percentage

```bash
./life-optimizer optimize \
  --salary 100000 \
  --age 35 \
  --married true \
  --children 2 \
  --retirement-age 65 \
  --life-expectancy 90 \
  --pillar3a 7056
```

### Use your own observed tax rate instead of official tables

```bash
./life-optimizer optimize --salary 100000 --age 35 --custom-tax-rate 0.20
```

### Compare specific scenarios side by side

```bash
./life-optimizer compare --salary 100000 --age 35 --married true --percentages "0.6,0.8,1.0"
```

### Deep-dive pension simulation for a specific work percentage, working to 70

```bash
./life-optimizer pension \
  --salary 100000 \
  --age 35 \
  --work-pct 1.0 \
  --retirement-age 70 \
  --life-expectancy 90 \
  --pillar3a 7056 \
  --retirement-needs 4200
```

`--retirement-needs` is **required**, and the omission is an error rather than a default. This
command used to apply 75% of net income — the conventional rule of thumb, and a measurement of
nobody's life — which meant its headline "are you covered?" answer was produced by an invented
number. `optimize` needs no such flag: it uses your own modelled budget, and reports the answer
across a 60/75/90 range either way.

### Lifetime strategy across your whole career

```bash
./life-optimizer lifetime --salary 100000 --age 30 --married true --children 2 --retirement-age 65
```

### Interactive mode

```bash
./life-optimizer interactive
```

---

## CLI reference

| Command | Purpose |
|---|---|
| `optimize` | Find the utility-maximizing work percentage for your situation |
| `compare` | Show gross/net/tax/utility side by side for chosen percentages |
| `pension` | Standalone Monte Carlo + regime-switching pension deep dive. Requires `--retirement-needs` |
| `lifetime` | Optimal work percentage trajectory across your career |
| `interactive` | Guided question-by-question setup |

Key flags (`optimize` / `pension`):

| Flag | Description | Default |
|---|---|---|
| `--salary` | Full-time annual gross salary (CHF) | required |
| `--age` | Current age | required |
| `--married` | Marital status | `false` |
| `--children` | Number of children | `0` |
| `--custom-tax-rate` | Override tables with your own observed total deduction rate (decimal) | official tables |
| `--retirement-age` | Planned retirement age (up to 70) | `65` |
| `--life-expectancy` | Planning horizon for retirement | `90` |
| `--pillar3a` | Annual Pillar 3a contribution (CHF, max 7,056) | `0` |
| `--profile` | `balanced` / `family` / `career` preference weighting | `balanced` |
| `--consumption-profile` | `extreme-saving` / `moderate` / `normal` / `luxury` lifestyle baseline | `normal` |
| `--monthly-debt` | Debt repayment or other unavoidable monthly outflow (CHF) — joins the mandatory floor | `0` |
| `--ai-scenario` | Named AI-adoption position: `sceptical`, `neutral` or `optimistic`. Sets the productivity gain, its optimistic end, the quality retention and the required output together, prints what it set, and is anchored to the published range in `documents/AI_AS_CATALYST.md` §4 | unset |

The employer-side achievement model — eight coefficients for what an employer demands, how much of
the AI gain survives review, and what missing the bar costs — is not on `optimize`. Every one of
those coefficients is illustrative, and a table of eight unmeasured numbers with defaults invites
the reading that they were measured. It is a separate verb:

```bash
./life-optimizer stress-ai --salary 120000 --age 40 \
  --required-output-index 1.2 --ai-productivity-gain 0.25 --ai-quality-retention 0.85 \
  --replacement-risk 2.5 --enforcement risk-weighted
```

`stress-ai` accepts every `optimize` flag plus the eight employer coefficients:
`--required-output-index`, `--ai-productivity-gain`, `--ai-productivity-gain-high`,
`--ai-quality-retention`, `--compression-quality-sensitivity`, `--replacement-risk`,
`--evaluation-period-years` and `--enforcement`. It runs the same search as `optimize`; the only
difference is where those eight numbers came from.

---

## How the math works

**Objective function:**

```
U = Σ β^t · u(consumption, leisure, family, health, security)
```

subject to budget, time (168 hrs/week), personal-requirement, and pension-adequacy
constraints. Full derivation, calibration notes, and the utility component formulas
are in [`MATHEMATICS.md`](MATHEMATICS.md).

**Tax engine:** progressive lookup tables (Stadt Bern 2024 official rates) with
linear interpolation between income brackets, layered with mandatory Swiss social
security (AHV/IV/EO 5.3%, ALV 1.1%, BVG ~6.5%).

**Pension engine:** age-scaled BVG contributions compounded annually under either
(a) a fixed log-normal return distribution per scenario, or (b) a 4-state Markov
regime-switching model (Boom/Normal/Recession/Stagflation), converted to an annuity
at retirement using an age-dependent conversion rate, then drawn down against
inflation-adjusted needs through your planning horizon. See
[`ECONOMIC_SCENARIOS.md`](documents/ECONOMIC_SCENARIOS.md) and
[`PENSION_OPTIMIZATION.md`](PENSION_OPTIMIZATION.md) for details.

---

## Project structure

```
life-optimizer/
├── Cargo.toml                    Workspace manifest (root package: life-optimizer)
├── human_capacity/               The psychological layer, as a library: practice,
│                                 capacity, the tempo that forces assistance. Used by
│                                 multipolar_sim and by knowledge_work
├── knowledge_work/               Just-in-time logic in AI-assisted knowledge work, as a
│                                 library plus CLI: the operating envelope, the rework loop,
│                                 and the direction framework
├── comprehension_cost/           Whether AI authoring time survives comprehension, as a
│                                 library plus CLI: pre-registered, Declared parameters,
│                                 a control that lets it report the opposite conclusion
├── multipolar_sim/               Companion Monte Carlo world simulator (see above)
├── src/
│   ├── main.rs                   CLI, command dispatch, orchestration
│   ├── lib.rs                    Library surface and module map
│   ├── tax.rs                    Swiss tax: federal + cantonal + social security
│   ├── federal_tax.rs            Federal tariff, including closed-form formulas
│   ├── cantons.rs                Cantonal schedules, Steuerfuss, provenance rules
│   ├── deductions.rs             Deduction engine (rules, applicability, scales)
│   ├── requirements.rs           Personal budget, life stages, preference weights
│   ├── consumption.rs            Consumption basket and cost model
│   ├── optimizer.rs              Multi-objective utility optimization engine
│   ├── monte_carlo.rs            Pension Monte Carlo + regime-switching simulation
│   ├── economic_regimes.rs       Markov chain economic regime model
│   ├── display.rs                Work-life balance result formatting
│   ├── mc_display.rs             Pension simulation result formatting
│   └── *_data.rs                 GENERATED tables (ESTV scales, deductions, Steuerfuss)
├── source-documents/             The external sources themselves: ESTV workbooks,
│                                 ESTV "Tarife" exports, and the DBG / tariff PDFs,
│                                 with a manifest recording what each one feeds
├── MATHEMATICS.md                Full mathematical formulation
├── PENSION_OPTIMIZATION.md       Pension sustainability methodology
├── SWISS_TAX_DATA.md             Tax data provenance, coverage and known defects
├── VALIDATION.md                 What has been checked against an outside source, what has
│                                 not, and the BVG defect the check found
├── AI_AUTHORING_AND_COMPREHENSION.md
│                                 The pre-registered AI-authoring model: what the anchors
│                                 actually say, the decision rule, and the result that
│                                 argues against its own hypothesis
├── CRITIQUE_RESPONSE.md          An external review of this repository, audited claim by claim
├── CHANGELOG.md                  What changed, when, and why
├── TEAM_DEPENDENCIES.md          Why a reduced schedule is not a private transaction
├── CRITICS_CURRENT_WORK.md       External criticism, and §7: what is implemented, what is not
├── documents/                    The arguments this tool came out of; see documents/README.md
│                                 for the index. They are the same question at different scales:
│                                 what a working life is worth, what is owed for the
│                                 qualification behind it, and what the work does to the person
├── figures/                      Figures drawn by tools/plot_ai_catalyst.py,
│                                 tools/plot_information_warfare.py, tools/plot_spoiler.py,
│                                 tools/plot_strategies.py, tools/plot_early_retirement.py,
│                                 tools/plot_psyche.py and tools/plot_workflow.py
├── EXAMPLES.md                   Worked usage examples
├── QUICKSTART.md                 Getting-started guide
└── PROJECT_SUMMARY.md            Executive overview
```

---

## Assumptions & limitations

- **Cantonal tax: 23 of 26 cantons are priced.** Bern uses its own official Stadt Bern 2024
  table (cantonal, municipal and church). Every other priced canton uses the ESTV Tarife export
  scale multiplied by that canton's Steuerfuss, both from the same year, and the output names
  which basis it used. Three cantons cannot be priced — **Fribourg, Geneva and Valais** — because
  their cantonal Steuerfuss cell in the official export is blank or ambiguous (`147.5%9)` for
  Geneva). The tool refuses rather than guessing, and the refusal names the cantons that work.
  For those three, use `--custom-tax-rate` with your own observed effective rate.
- Pension projections are **estimates**, not financial advice. Real BVG plans vary
  by employer (contribution rates, conversion rates, buy-in options).
- Market return assumptions are calibrated to historical Swiss BVG fund performance
  and general developed-market business-cycle statistics — not a guarantee of future
  returns.
- The retirement "needs" figure defaults to 75% of current net income, a common
  rule of thumb, not a personalized budget.
- The achievement-capacity coefficients (AI productivity gain, quality retention,
  compression sensitivity, replacement risk) are **illustrative sensitivities, not
  estimates**. Nothing in the tool predicts your AI productivity; it exposes the
  coefficients so you can test what your answer depends on. All default to neutral.
- Team effects are **reported rather than silently ignored**. The model cannot tell whether a
  reduced schedule's work is redistributed or dropped, so it asks: `--team-size` and
  `--redistribution` compute the hours that land on each colleague, every reduction result carries
  a warning, and a reduction with no team parameters is reported as **not modelled** rather than as
  having no effect. The reasoning is in [`TEAM_DEPENDENCIES.md`](TEAM_DEPENDENCIES.md).

This tool is meant to inform a conversation with a financial advisor or pension
fund — not replace one.

---

## License

This project is licensed under the **GNU General Public License v3.0 (GPL-3.0)**.

**What the licence is meant to prevent.** The tool exists so a person can see how their own
retirement and work-percentage numbers are produced. A closed-source service that took this code,
wrapped it, and sold the answers back to the same people — without their being able to check the
assumptions, or to fork a version that told the truth about them — would defeat that purpose while
being perfectly legal under MIT. GPL v3 is what stops it: anyone who distributes a derivative must
distribute its source under the same terms.

It is not meant to stop anyone from *using* the tool, from forking it to model their own canton, or
from quoting and criticising the code. Running it, reading it, and publishing about it are all
unaffected.

* **Previous versions** remain available under the terms of the MIT License.

For more details, see the [LICENSE](LICENSE) file.

## Maintenance

This is a personal project, maintained by one person, and it is better to say so than to imply
otherwise.

**What that means in practice.** Issues and pull requests are welcome and are read, but there is no
response-time commitment and no release cadence. The bus factor is one. If you need a guaranteed
support relationship, this is not it, and you should fork it — the licence permits that, and the
code is organised for it.

**What is maintained carefully.** The verification work is not decorative: the test suite, the CI
configuration, and the documents that record what has and has not been checked (`VALIDATION.md`,
`CRITIQUE_RESPONSE.md`) are kept honest, because a financial-adjacent tool that lets its validation
drift is worse than one that never claimed any.

**What ages.** Bern's table is Stadt Bern 2024 and the other 22 priced cantons use ESTV imports of
the same vintage. Swiss tax law changes annually, so the tables drift out of date and
`--custom-tax-rate` is the mitigation.
`SWISS_TAX_DATA.md` records the provenance and the known defects. `CHANGELOG.md` records what
changed and when.

## Contributing

Issues and PRs welcome, particularly:
- The three unpriced cantonal Steuerfuss values, resolved against the official source
  (Fribourg, Geneva, Valais — see the assumptions block)
- Refinements to the regime-switching calibration
- Additional life-stage/preference profiles