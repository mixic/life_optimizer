# multipolar_sim

A Monte Carlo simulator for a multipolar world: five power blocs, every pair
playing a 2×2 Cooperate/Compete game each year with its Nash equilibrium solved
(pure or mixed), annual shocks, and a pension-security read-out.
Run it with `cargo run -p multipolar_sim`, and with `--sweep` for the run that
actually matters.

```
cargo run -p multipolar_sim -- --compare               # who wins, and who loses
cargo run -p multipolar_sim -- --ai                    # AI as a player, or as a tool?
cargo run -p multipolar_sim -- --sweep                  # the informative mode
cargo run -p multipolar_sim -- --seed 42 --runs 1000    # reproducible ensemble
cargo run -p multipolar_sim -- --bloc "Atlantic:0.50:0.02:0.010:1.0"
cargo run -p multipolar_sim -- --bloc "Antarctic:0.06:0.010:0.020:1.00"
cargo run -p multipolar_sim -- --help
```

`--compare` answers "who wins and who loses" directly. It runs the same years
twice, from the same shock draws, once with a cooperative payoff balance and once
with a non-cooperative one, and prints each bloc's share and dominance probability
in both worlds next to how the world itself fares. It then separates the two claims
worth separating: a cooperative world is better on *every* aggregate while still
redistributing power away from some blocs, which is robust, whereas the *identity*
of the largest bloc is decided by the invented starting shares and growth biases and
should not be read as a prediction.

`--bloc` takes `Name:share:bias:volatility:affinity` with an optional sixth field,
the bloc's **cooperation valuation**. It edits the named bloc if the system has one
and appends it otherwise, and is repeatable — so the five-pole default can be
reshaped a field at a time or replaced outright. The valuation is the field that
makes the two sides of a dyad differ, because it enters that bloc's own payoff
matrix; omitting it leaves the bloc neutral at `1.0`, which is what every bloc was
before the game became a bimatrix. `--seed` makes a whole ensemble reproducible,
which is what lets two parameter settings be compared against the same shock draws
instead of against different luck. `--help` prints every flag with its real default,
derived from the code rather than written out by hand.

## The 2×2 is a bimatrix, and why that mattered

The solver takes **one payoff matrix per side** (`solve_bimatrix`, with `solve_pair`
taking the documented symmetric fast path when the two are equal). This is worth
explaining because the first version of this crate could not express a claim as basic
as *cooperation is worth more to this bloc than to that one* — which is not a detail,
since it is exactly the claim `MULTIPOLAR_GAME.md` §4 is organised around.

Two things follow from it, and both are load-bearing:

* **`cooperation_valuation` was added** to `PowerBloc`: what cooperation is worth
  *when choosing*, entering the payoff matrix. It is deliberately separate from
  `cooperation_affinity`, which is what cooperation *pays once chosen* — downstream of
  the solver, so it cannot change a decision. A test pins both halves: affinity cannot
  move the cooperation rate, valuation can.
* **A symmetric game is never answered asymmetrically.** In the region where the
  sucker's payoff beats mutual competition — which the simulation reaches at high
  tension — a symmetric game has three equilibria: `(C,D)`, `(D,C)` and a mixed one.
  The first two pay more in total, but selecting either would mean deciding which of two
  *identical* players is the exploiter, so the solver searches only `(C,C)` and `(D,D)`
  when the matrices are equal and falls back to the mixed equilibrium. Where the
  matrices differ, all four profiles are candidates and the tie between `(C,D)` and
  `(D,C)` is broken by a stated convention — the side that values cooperation more is
  the one that cooperates — so the answer does not depend on argument order. A test
  checks that invariance across a grid.

**The already-published figures did not move.** `--sweep` and `--compare` outputs are
byte-identical to before the change, because every default bloc is neutral on
valuation and the symmetric fast path therefore runs the original code. That is
verified by construction rather than by a test alone, and the baseline outputs were
diffed to confirm it.

One consequence worth knowing: behaviour is **discontinuous at perfect symmetry**. With
every valuation equal, the asymmetric branch is closed; any nonzero spread opens it. So
a model like this cannot be read as continuous in the spread of valuations, which the
`--ai` cooperation sweep says in its own output.

## AI in the game: player, or tool?

`MULTIPOLAR_GAME.md` §7 lists four live answers to "who captures the gains from
AI" and declines to pick one. `--ai` builds the two that are structurally
different, plus the control, and runs all three from the same shock draws:

| World                    | What it assumes                                                              | The question it poses                         |
| ------------------------ | ---------------------------------------------------------------------------- | --------------------------------------------- |
| `AI AS A SIXTH POWER`  | AI is a player: it holds power of its own and plays every dyad               | Does it end a hegemon, or is it held down?    |
| `AI AS A WIELDED TOOL` | AI is owned: no new player, but each bloc's power grows with its own AI lead | Does uneven ownership concentrate the system? |
| `NO AI LAYER`          | The existing five-bloc model, unchanged                                      | The control both are measured against         |

They are not two answers to one question, so each is scored on its own terms
rather than on one shared metric. The layer is a *transformation of the bloc
list* — one world appends an actor, the other adds a term to `growth_bias` — which
keeps the Monte Carlo, the solver and the pension channel untouched, and makes the
control the existing model rather than a reimplementation of it that could drift.

What the default run says, and the honest limits of it:

* **As a player, AI is absorbed, not ascendant.** Grown at exactly the
  fastest-growing conventional bloc's rate, the actor ends *below* where it
  started. The mechanism is monetary: an actor issuing no reserve currency holds
  no leverage over anyone while every issuer holds maximum leverage over it, so in
  a mostly uncooperative world it absorbs the full sanction drag from every
  direction and applies none. Growing faster than every bloc is not by itself
  enough to hold position.
* **It becomes hegemon only on a large growth advantage, and the sweep names it.**
  The fate column turns from `ABSORBED` through `ASCENDANT` to `HEGEMON` between
  roughly 8% and 11% a year of compounded growth against a field whose weighted mean
  is about 2%. That is the number worth arguing about, not the share printed beside
  it. The headline figure is robust to the growth-bias correction described in
  limitation 3: the hinge sat at 8–11% a year *effective* before it, and at 8–11% a
  year nominal after, which is what the re-statement was for.
* **As a tool, uneven ownership concentrates the system a lot; equal ownership
  does nothing.** In the default run the two frontier leaders gain and all three
  laggards lose, and the control — the same lead for everyone — moves the top share
  by under half a point. A general-purpose capability that lifts every bloc equally
  is very nearly invisible in a model of *relative* power. The *sizes* do not order
  by lead, though, and the mode does not claim they do: the lead term sits on top of
  each bloc's pre-existing starting share and growth bias, so a bloc with a smaller
  lead can gain more than one with a larger lead.
* **Which bloc owns AI is not a prediction.** The lead vector's *shape* follows
  widely reported capability concentration; its magnitudes are invented, and the
  ranking it produces is a property of the parameterisation.

Three limitations the mode prints rather than hides:

1. **The cooperation channel now works, and it took the bimatrix to get there.** The
   third sweep row was flat at every value in the first version of this feature,
   because `--ai-cooperation` raised `cooperation_affinity` — a term applied
   *downstream* of the solver — and so could not change a decision. It now raises the
   owning bloc's `cooperation_valuation`, which enters the payoff matrix, and the row
   moves: `-0.15` leaves cooperation at `0.334` and `+0.15` at `0.422`, which is the
   realist case against the optimistic one, measured. The default row is a knife-edge
   (see above), and the sweep says so.
2. **Adding any sixth actor changes the field, not only an AI one.** One more member
   means one more dyad for every existing bloc, and a dyad carries tension, energy
   interdependence and sanction drag. What it no longer changes is anybody's growth
   rate — see the next item. The verdict is unaffected — it is measured on the actor
   itself — but a per-bloc comparison against the five-bloc control is not a clean
   isolation and is not presented as one.
3. **Growth biases are annual rates, and that took a correction.** `simulation.rs`
   used to add `power * growth_bias` inside the dyad loop *as well as* in the annual
   drift step, so a nominal bias compounded once per bloc and its real meaning
   depended on how many blocs existed. It now applies the bias once a year, and the
   default biases were re-stated in annual terms at the same time
   (`(1 + b)^5 - 1`, the old five-bloc convention), so that the model's *behaviour*
   stayed as it was while the *parameter's meaning* became clear. A test pins the
   invariant: the same bias must buy the same annual growth in a seven-bloc world as
   in a five-bloc one. The correction is a prerequisite for comparing systems of
   different sizes — which is what adding a region to this model is — because a
   bloc-count artefact is exactly what such a comparison would otherwise have
   measured. The published figures moved by about a percentage point when the
   correction and the re-statement are taken together, which is the check that the
   two were not compensating errors.

## Read this before using any number it prints

**Nothing in this crate is calibrated.** The growth rates, volatilities,
cooperation gains, conflict costs and pension elasticities are chosen to be
plausible in sign and rough magnitude so the *mechanism* can be explored. They are
not measurements, and no output here is a forecast.

The reason is the one `MULTIPOLAR_GAME.md` section 5 states and `FutureWork.md`
section 7 warns about: there is no general theorem guaranteeing that a multipolar
system converges at all, and a simulator that produced confident-looking 50-year
numbers from unfitted parameters would be exactly the "plausible narrative wrapped
around unfitted parameters" that section 7 rejects.

So the honest use of this tool is **sensitivity**, not prediction:

* A quantity that barely moves across a parameter's plausible range is one the
  conclusion does not depend on, and is safe to be wrong about.
* A quantity that flips across that range is one the conclusion *is* conditional
  on, and any claim resting on it has to say so.

`--sweep` prints exactly that comparison, and the binary prints the caveat before
any result rather than after it.

## Why this is a separate crate

This crate is a workspace member of `life_optimizer`, not a module of it, and the
boundary is deliberate.

`life-optimizer`'s figures are either traced to an official source or explicitly
flagged as unsourced — see the `Provenance` enum in `src/cantons.rs` and section 7
of `FutureWork.md`. It has no fallback path by construction. This crate's figures
are illustrative by construction. Those are opposite epistemic standards, and each
is right for its own subject.

Keeping them in separate crates means an illustrative number cannot reach a
reported tax or pension figure by accident: the boundary is a `use` statement that
does not exist. If the two are ever connected, the connection should be an
explicit, opt-in interface — a scenario the simulator *emits* and the tax tool
*reads*, with the borrowed figures labelled as illustrative in the output — rather
than shared code.

A build failure in this crate must also never be able to block `life-optimizer`.

## Where this sits in the wider project

The model is the formal counterpart of three documents that already live in the
repository root:

| Document                                       | What it claims                                                                                                        | What this crate does with it                                                                                        |
| ---------------------------------------------- | --------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------- |
| `MULTIPOLAR_GAME.md` §4                     | The security dilemma is the characteristic risk of a multipolar transition                                            | `blocks.rs` builds the payoff matrix that produces it, and `game.rs` solves it rather than assuming the outcome |
| `MULTIPOLAR_GAME.md` §8                     | Imperial overstretch: an arms race eventually stops paying for itself                                                 | `GameParams::conflict_wear` — the only channel by which conflict becomes self-limiting                           |
| `THEORY_OF_SPARING.md` §7d                  | Engineered obsolescence as a 2×2 game with a Pareto-inferior Nash equilibrium                                        | The same solver;`efficiency_loss` is the operational measure of that inferiority                                  |
| `PHILOSOPHICAL_SOCIOLOGICAL_ASPECTS.MD` §2c | AHV is a pay-as-you-go intergenerational contract, and "optimal for the individual is not optimal for the collective" | `pension.rs` — the voice the household tool has no way to represent                                              |

## Layout

| Module            | Responsibility                                                                  |
| ----------------- | ------------------------------------------------------------------------------- |
| `game.rs`       | 2×2 bimatrix game, Nash equilibrium selection (pure or mixed), efficiency loss |
| `blocks.rs`     | Power blocs and the payoff structure their interactions produce                 |
| `economy.rs`    | Monetary standing, energy trade, financial conditions — with provenance        |
| `ai.rs`         | AI as a player and AI as a tool: the two rival hypotheses, and the verdicts     |
| `simulation.rs` | The Monte Carlo: one run, and the ensemble over many                            |
| `pension.rs`    | The AHV/pension channels the simulated world implies                            |
| `report.rs`     | Terminal presentation                                                           |
| `main.rs`       | CLI, argument parsing,`--sweep`, `--compare`, `--ai`                      |

## The economic layer, and what is measured versus invented

`economy.rs` supplies the economics that decides what the strategic structure
*costs*: who issues the money others hold, who depends on whose energy, and how
strained the financial system is.

Every quantity carries a `Provenance`, either `Sourced { source, vintage }` or
`Illustrative { rationale }`, and the report prints the table so a reader can see
which is which without reading the prose. The split is:

* **Sourced**: reserve-currency composition (IMF COFER, 2025 Q4 — USD 56.77%, EUR
  20.25%, CNY 1.95%, 6.13% in currencies COFER does not identify, a category that
  has more than doubled since 2021); the direction and aggregate size of energy
  flows (about 80% of Russian oil exports went to China and India in 2025; Russian
  gas was 16.1% of EU LNG and 16.3% of EU pipeline-gas imports, against a US share
  of 52.5% of LNG).
* **Illustrative**: *every* transmission elasticity, without exception. Nobody can
  measure how a one-point shift in reserve share changes sanction leverage, because
  the counterfactual does not exist. Also illustrative: the split of a published
  aggregate across this model's blocs, because the blocs are political groupings
  that no statistical agency reports as a unit.

That second category is the honest part. A test enforces that no parameter ships
without a provenance that actually says something, and that every `Sourced` entry
names both a source and a vintage.

### Two doors into the model

Bloc-specific economics are partly about what a bloc **decides** and partly about what
it **pays**, and the layer enters through a different door for each:

* **Into the game**: the pair's interdependence raises what mutual cooperation is
  worth. Both sides feel it, so it belongs in both matrices. Alongside it sits each
  bloc's own cooperation valuation, which is what makes the two matrices differ. The
  interdependence term deliberately does *not* touch the temptation to defect: a richer
  relationship is worth more to capture as well as more to sustain, and picking a sign
  for that would be hiding a judgement.
* **Into the simulation**: energy disruption and monetary leverage hit individual
  blocs differently, so they act on that bloc's own power — the importer loses supply,
  the exporter loses revenue, and the bloc with less monetary leverage absorbs more of
  an adversarial turn.

The distinction matters for what each door can express. Capability asymmetries act on
power, so they belong downstream of the solver; value asymmetries act on the choice, so
they belong in the matrix. A test asserts that interdependence stays pair-symmetric,
because it describes the relationship rather than either side of it.

### Outstanding

An item surfaced by `--ai`, and now **fixed** rather than merely reported: a bloc's
growth bias used to be applied once per dyad as well as once per year, so a bloc's
growth rate depended on how many other blocs existed. It is applied once a year now,
and the default biases were re-stated as annual rates so the model's behaviour stayed
where it was — see limitation 3 above. The AI report used to print an *effective*
annual growth beside the nominal one precisely because the two differed; that column
is gone, because the two are the same number.

The reserve shares are a fixed endowment held constant for all 50 years, so
de-dollarisation remains a change in the *level* of leverage rather than a drift in
it. De-dollarisation also cannot yet enter as a bloc-specific payoff: it is an
asymmetry in *capability*, which the bimatrix deliberately does not carry — the
matrices hold what a bloc values, not what it can do. A test asserts that
interdependence stays pair-symmetric for the same reason.

Not yet built: European deindustrialisation, and the crypto channel. That last one
is planned as a *falsifiable* test rather than an assumed buffer, because the
observed record runs the other way: in the March 2020 panic bitcoin fell about 56%,
worse than the S&P 500, and its correlation to equities **rose above 0.5 during
turbulence** while decaying toward zero in calm markets — the correlation
strengthens exactly when a buffer would be needed. (Sources: IMF COFER for
reserves; Deputy PM Novak via OilPrice and Eurostat via TASS for energy; The Block
via ForkLog for the crypto record.)
