# multipolar_sim

A Monte Carlo simulator for a multipolar world: five power blocs, every pair
playing a 2×2 Cooperate/Compete game each year with its Nash equilibrium solved
(pure or mixed), annual shocks, and a pension-security read-out.

Run it with `cargo run -p multipolar_sim`, and with `--sweep` for the run that
actually matters.

```
cargo run -p multipolar_sim -- --compare               # who wins, and who loses
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

`--bloc` takes `Name:share:bias:volatility:affinity`, edits the named bloc if the
system has one and appends it otherwise, and is repeatable — so the five-pole
default can be reshaped a field at a time or replaced outright. `--seed` makes a
whole ensemble reproducible, which is what lets two parameter settings be compared
against the same shock draws instead of against different luck. `--help` prints
every flag with its real default, derived from the code rather than written out by
hand.

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

| Document | What it claims | What this crate does with it |
|---|---|---|
| `MULTIPOLAR_GAME.md` §4 | The security dilemma is the characteristic risk of a multipolar transition | `blocks.rs` builds the payoff matrix that produces it, and `game.rs` solves it rather than assuming the outcome |
| `MULTIPOLAR_GAME.md` §8 | Imperial overstretch: an arms race eventually stops paying for itself | `GameParams::conflict_wear` — the only channel by which conflict becomes self-limiting |
| `THEORY_OF_SPARING.md` §7d | Engineered obsolescence as a 2×2 game with a Pareto-inferior Nash equilibrium | The same solver; `efficiency_loss` is the operational measure of that inferiority |
| `PHILOSOPHICAL_SOCIOLOGICAL_ASPECTS.MD` §2c | AHV is a pay-as-you-go intergenerational contract, and "optimal for the individual is not optimal for the collective" | `pension.rs` — the voice the household tool has no way to represent |

## Layout

| Module | Responsibility |
|---|---|
| `game.rs` | Symmetric 2×2 game, Nash equilibrium selection (pure or mixed), efficiency loss |
| `blocks.rs` | Power blocs and the payoff structure their interactions produce |
| `economy.rs` | Monetary standing, energy trade, financial conditions — with provenance |
| `simulation.rs` | The Monte Carlo: one run, and the ensemble over many |
| `pension.rs` | The AHV/pension channels the simulated world implies |
| `report.rs` | Terminal presentation |
| `main.rs` | CLI, argument parsing, `--sweep` |

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

The 2×2 is *symmetric* — one payoff matrix describes both sides — but bloc-specific
economics are inherently asymmetric, since the issuer of a reserve currency is not
in the same position as a bloc that issues none. So the layer enters twice:

* **Into the game**: the pair's interdependence raises what mutual cooperation is
  worth. Both sides feel it, so it belongs in the matrix. It deliberately does *not*
  touch the temptation to defect: a richer relationship is worth more to capture as
  well as more to sustain, and picking a sign for that would be hiding a judgement.
* **Into the simulation**: energy disruption and monetary leverage hit individual
  blocs differently, so they act on that bloc's own power — the importer loses
  supply, the exporter loses revenue, and the bloc with less monetary leverage
  absorbs more of an adversarial turn.

### Outstanding

Making the 2×2 *itself* asymmetric is the next structural step. Until then,
de-dollarisation enters through pair averages and through power, not through a
bloc-specific payoff matrix. The reserve shares are also a fixed endowment held
constant for all 50 years, so de-dollarisation is a change in the *level* of
leverage rather than a drift in it. Both are noted in the report's own output.

Not yet built, in the agreed order: AI's effect on growth, European
deindustrialisation, and the crypto channel. That last one is planned as a
*falsifiable* test rather than an assumed buffer, because the observed record runs
the other way: in the March 2020 panic bitcoin fell about 56%, worse than the S&P
500, and its correlation to equities **rose above 0.5 during turbulence** while
decaying toward zero in calm markets — the correlation strengthens exactly when a
buffer would be needed. (Sources: IMF COFER for reserves; Deputy PM Novak via
OilPrice and Eurostat via TASS for energy; The Block via ForkLog for the crypto
record.)
