# multipolar_sim

A Monte Carlo simulator for a multipolar world: five power blocs, every pair
playing a 2×2 Cooperate/Compete game each year with its Nash equilibrium solved
(pure or mixed), annual shocks, and a pension-security read-out.

Run it with `cargo run -p multipolar_sim`, and with `--sweep` for the run that
actually matters.

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
| `simulation.rs` | The Monte Carlo: one run, and the ensemble over many |
| `pension.rs` | The AHV/pension channels the simulated world implies |
| `report.rs` | Terminal presentation |
| `main.rs` | CLI, argument parsing, `--sweep` |
