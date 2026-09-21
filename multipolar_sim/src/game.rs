// Multipolar World Simulator
// Copyright (C) 2026 MILAN NIKOLIC
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

//! A symmetric 2x2 game, and its Nash equilibria.
//!
//! Every year, every pair of power blocs plays a one-shot game: each side chooses
//! **Cooperate** or **Compete**, and the payoffs depend on how evenly matched the
//! two are and how tense the system has become.
//!
//! This module is the intellectual core. The security dilemma that
//! `MULTIPOLAR_GAME.md` section 4 describes is a claim about *this* structure, and
//! the Pareto-inferior Nash equilibrium that `THEORY_OF_SPARING.md` section 7d
//! formalises for producers and consumers is the same shape. Everything downstream
//! -- conflict frequency, efficiency loss, polarity -- is a consequence of solving
//! this game rather than assuming its outcome, so the solver is the part worth
//! being careful about.
//!
//! # Why the equilibrium is not simply "compete"
//!
//! The textbook security dilemma has `Compete` strictly dominant, and a solver that
//! returned that every time would be both correct and useless: it would reproduce
//! one narrative and never show the conditions under which cooperation survives --
//! which is exactly the question the literature cited in `MULTIPOLAR_GAME.md`
//! section 4 is split on. The payoff structure is therefore built so that **mutual
//! cooperation can be a Nash equilibrium** when the gains from cooperation are
//! large relative to the cost of being exploited, and so that arms-race dynamics
//! emerge from parameters rather than from a script.
//!
//! # Mixed equilibria matter
//!
//! For a 2x2 symmetric game the mixed equilibrium is what produces the genuinely
//! interesting behaviour: neither side can commit, so each randomises, and the
//! system does not settle. A pure-only solver would miss that entirely. Both are
//! computed, and the mixing probability is reported.

/// One player's two options.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Cooperate,
    Compete,
}

/// The four outcomes of a symmetric 2x2 game.
///
/// Names follow the Prisoner's-Dilemma convention:
/// * `cc` -- both cooperate (the Pareto-optimal outcome)
/// * `cd` -- this player cooperates, the other competes (the sucker's payoff)
/// * `dc` -- this player competes, the other cooperates (the temptation)
/// * `dd` -- both compete (the arms-race outcome)
///
/// The ordering `cd < dd < cc < dc` makes this a Prisoner's Dilemma, but nothing
/// here *requires* it: with large enough cooperation gains `cc` can exceed `dc`,
/// which is what lets mutual cooperation become an equilibrium.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Payoffs {
    pub cc: f64,
    pub cd: f64,
    pub dc: f64,
    pub dd: f64,
}

impl Payoffs {
    /// Payoff to `mine`, given the opponent's action.
    pub fn payoff(&self, mine: Action, theirs: Action) -> f64 {
        match (mine, theirs) {
            (Action::Cooperate, Action::Cooperate) => self.cc,
            (Action::Cooperate, Action::Compete) => self.cd,
            (Action::Compete, Action::Cooperate) => self.dc,
            (Action::Compete, Action::Compete) => self.dd,
        }
    }

    /// Payoff to cooperating against a given opponent action.
    pub fn cooperate_against(&self, theirs: Action) -> f64 {
        self.payoff(Action::Cooperate, theirs)
    }

    /// Payoff to competing against a given opponent action.
    pub fn compete_against(&self, theirs: Action) -> f64 {
        self.payoff(Action::Compete, theirs)
    }
}

/// Which kind of equilibrium the game has.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Equilibrium {
    /// Both `(C,C)` and `(D,D)` are equilibria; the solver selects the
    /// Pareto-superior `(C,C)`.
    Coordination,
    /// Compete strictly dominates: `(D,D)` is the unique outcome. The security
    /// dilemma in its pure form, and the case `MULTIPOLAR_GAME.md` describes as
    /// the arms-race dynamic.
    Dilemma,
    /// Cooperate is the best response whatever the other does: `(C,C)` is reached
    /// because defection is not profitable. The optimistic case the literature
    /// holds out when cooperation gains are large.
    Harmony,
    /// Neither pure outcome is an equilibrium; the only equilibrium is mixed.
    Cyclical,
}

impl Equilibrium {
    pub fn label(self) -> &'static str {
        match self {
            Equilibrium::Coordination => "coordination",
            Equilibrium::Dilemma => "dilemma",
            Equilibrium::Harmony => "harmony",
            Equilibrium::Cyclical => "cyclical",
        }
    }
}

/// The solved game: what actually happens, and what each player gets from it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Solution {
    pub equilibrium: Equilibrium,
    /// Probability that a player cooperates under the equilibrium played.
    ///
    /// `1.0` and `0.0` mean a pure equilibrium was selected; anything strictly
    /// between is a mixed equilibrium, where neither side can commit.
    pub cooperate_probability: f64,
    /// Expected payoff to each player. The game is symmetric, so both sides expect
    /// the same value.
    pub equilibrium_payoff: f64,
    /// Payoff under mutual cooperation -- the Pareto-optimal benchmark.
    pub cooperation_payoff: f64,
    /// Efficiency loss: how far the equilibrium falls short of mutual cooperation,
    /// as a fraction of the range the game actually offers.
    ///
    /// `0.0` means the equilibrium delivered at least as much as mutual
    /// cooperation; `1.0` means none of the cooperative surplus was captured. The
    /// reference is mutual cooperation and *not* the best outcome available, so a
    /// game in which cooperation is itself the worst outcome reports `0.0` -- see
    /// `efficiency_loss_is_positive_exactly_when_equilibrium_falls_short_of_cooperation`
    /// for why that is the honest reading rather than a bug. This is the quantity
    /// the simulator accumulates to answer "how much is being lost to the security
    /// dilemma", and it is the operational version of the Pareto-inferiority claim.
    pub efficiency_loss: f64,
}

/// Solve a symmetric 2x2 game.
///
/// Selection follows a documented rule rather than a preference:
///
/// 1. **Dominance**, if either action strictly dominates -- the opponent's choice
///    cannot change it, so it is what self-interested play produces.
/// 2. Otherwise **mutual cooperation**, if `(C,C)` is an equilibrium. This is the
///    cooperative outcome being *stable*, not optimism: in a coordination game
///    both `(C,C)` and `(D,D)` are equilibria and this picks the Pareto-superior
///    one. A model that always picked `(D,D)` would be assuming the worst case
///    rather than deriving it, and the point of reporting efficiency loss is to
///    see when the good equilibrium is available and when it is not.
/// 3. Otherwise **mutual competition**, if `(D,D)` is an equilibrium.
/// 4. Otherwise **mixed**.
pub fn solve(payoffs: &Payoffs) -> Solution {
    let cc = payoffs.cc;
    let cd = payoffs.cd;
    let dc = payoffs.dc;
    let dd = payoffs.dd;

    let cooperation_payoff = cc;

    // Strict dominance, with a tolerance so floating-point noise cannot manufacture
    // a dilemma. Cooperate dominates if cooperating beats competing against BOTH
    // opponent actions; likewise for competing.
    const EPS: f64 = 1e-12;
    let cooperate_dominates = payoffs.cooperate_against(Action::Cooperate)
        > payoffs.compete_against(Action::Cooperate) + EPS
        && payoffs.cooperate_against(Action::Compete)
            > payoffs.compete_against(Action::Compete) + EPS;
    let compete_dominates = payoffs.compete_against(Action::Cooperate)
        > payoffs.cooperate_against(Action::Cooperate) + EPS
        && payoffs.compete_against(Action::Compete)
            > payoffs.cooperate_against(Action::Compete) + EPS;

    // Pure equilibria: is each action a best response to itself?
    let cc_is_equilibrium = payoffs.cooperate_against(Action::Cooperate)
        >= payoffs.compete_against(Action::Cooperate) - EPS;
    let dd_is_equilibrium = payoffs.compete_against(Action::Compete)
        >= payoffs.cooperate_against(Action::Compete) - EPS;

    let (equilibrium, cooperate_probability, equilibrium_payoff) = if cooperate_dominates {
        (Equilibrium::Harmony, 1.0, cc)
    } else if compete_dominates {
        (Equilibrium::Dilemma, 0.0, dd)
    } else if cc_is_equilibrium {
        // Includes the coordination game, where both are equilibria. The
        // Pareto-superior one is selected deliberately; see the doc comment.
        (Equilibrium::Coordination, 1.0, cc)
    } else if dd_is_equilibrium {
        (Equilibrium::Dilemma, 0.0, dd)
    } else {
        // No pure equilibrium: the mixed one is all there is.
        //
        // In a symmetric 2x2 game a player is indifferent between their two
        // actions at the mixed equilibrium. Let q be the probability the
        // *opponent* cooperates. Indifference requires:
        //
        //   q * cc + (1-q) * cd  =  q * dc + (1-q) * dd
        //
        // Rearranged:
        //
        //   q * (cc - cd - dc + dd) = dd - cd
        //   q = (dd - cd) / (cc - cd - dc + dd)
        //
        // The denominator is zero exactly when one action dominates, which the
        // branches above have excluded -- but it is still guarded, because a
        // degenerate game would divide by zero here.
        let denominator = cc - cd - dc + dd;
        if denominator.abs() < EPS {
            // Degenerate: fall back to the better pure outcome.
            let (q, value) = if cc >= dd { (1.0, cc) } else { (0.0, dd) };
            (Equilibrium::Cyclical, q, value)
        } else {
            let q = ((dd - cd) / denominator).clamp(0.0, 1.0);
            // Expected payoff at indifference: evaluating either bracket gives the
            // same value, which is what indifference means.
            let value = q * cc + (1.0 - q) * cd;
            (Equilibrium::Cyclical, q, value)
        }
    };

    // Efficiency loss, normalised against the range the game offers. Without
    // normalisation a game with large payoffs would look "more wasteful" than one
    // with small payoffs at identical structure. The reference is the worst
    // outcome available, so 1.0 means no surplus captured at all.
    let worst = cd.min(dd).min(cc).min(dc);
    let span = cc - worst;
    let efficiency_loss = if span <= EPS {
        0.0
    } else {
        ((cc - equilibrium_payoff) / span).clamp(0.0, 1.0)
    };

    Solution {
        equilibrium,
        cooperate_probability,
        equilibrium_payoff,
        cooperation_payoff,
        efficiency_loss,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The textbook Prisoner's Dilemma: competing dominates, and the equilibrium
    /// is the outcome both players would rather avoid. This is the structure
    /// `THEORY_OF_SPARING.md` section 7d describes for engineered obsolescence.
    #[test]
    fn prisoners_dilemma_is_solved_as_a_dilemma() {
        let pd = Payoffs { cc: 3.0, cd: 0.0, dc: 5.0, dd: 1.0 };
        let solution = solve(&pd);
        assert_eq!(solution.equilibrium, Equilibrium::Dilemma);
        assert_eq!(solution.cooperate_probability, 0.0);
        assert_eq!(solution.equilibrium_payoff, 1.0);
        // Cooperation was worth 3 and the worst outcome 0, so 2/3 of the span is
        // lost.
        assert!(
            (solution.efficiency_loss - 2.0 / 3.0).abs() < 1e-9,
            "expected 2/3, got {}",
            solution.efficiency_loss
        );
    }

    /// A coordination game -- the structure `MULTIPOLAR_GAME.md` section 4 invokes
    /// via the Concert of Europe, where a good equilibrium exists but is not
    /// forced. Both mutual cooperation and mutual competition are equilibria; the
    /// solver selects the Pareto-superior one as a *documented rule*, not an
    /// assumption that actors behave well.
    #[test]
    fn coordination_selects_the_pareto_superior_equilibrium() {
        let hunt = Payoffs { cc: 4.0, cd: 0.0, dc: 3.0, dd: 2.0 };
        let solution = solve(&hunt);
        assert_eq!(solution.equilibrium, Equilibrium::Coordination);
        assert_eq!(solution.cooperate_probability, 1.0);
        assert_eq!(solution.equilibrium_payoff, 4.0);
        assert!(
            solution.efficiency_loss.abs() < 1e-12,
            "the good equilibrium was reached, so nothing is lost"
        );
        // Both must genuinely be equilibria, or this is not a coordination game.
        assert!(hunt.cc >= hunt.dc, "C must be a best response to C");
        assert!(hunt.dd >= hunt.cd, "D must be a best response to D");
    }

    /// Harmony: cooperating is best whatever the other side does, so the dilemma
    /// does not exist. The optimistic case the cited literature holds out when AI
    /// competition is not zero-sum.
    #[test]
    fn harmony_has_no_dilemma() {
        let harmony = Payoffs { cc: 5.0, cd: 2.0, dc: 4.0, dd: 1.0 };
        let solution = solve(&harmony);
        assert_eq!(solution.equilibrium, Equilibrium::Harmony);
        assert_eq!(solution.cooperate_probability, 1.0);
        assert!(solution.efficiency_loss.abs() < 1e-12);
    }

    /// A game with no pure equilibrium must return a genuinely mixed one, with a
    /// probability strictly between the two pure actions. This is the case a
    /// pure-only solver silently gets wrong, so it is pinned -- including the
    /// indifference condition that *defines* the mixed equilibrium.
    #[test]
    fn games_without_a_pure_equilibrium_are_solved_mixed() {
        // Matching-pennies-like: neither pure profile is an equilibrium.
        let cyclical = Payoffs { cc: 0.0, cd: 2.0, dc: 1.0, dd: 0.0 };
        let solution = solve(&cyclical);
        assert_eq!(solution.equilibrium, Equilibrium::Cyclical);
        assert!(
            solution.cooperate_probability > 0.0 && solution.cooperate_probability < 1.0,
            "a mixed equilibrium must mix: got {}",
            solution.cooperate_probability
        );

        // Verify indifference directly, which is what defines the mixed
        // equilibrium: at q, the opponent cannot gain by switching.
        let q = solution.cooperate_probability;
        let if_i_compete = q * cyclical.dc + (1.0 - q) * cyclical.dd;
        let if_i_cooperate = q * cyclical.cc + (1.0 - q) * cyclical.cd;
        assert!(
            (if_i_compete - if_i_cooperate).abs() < 1e-9,
            "at the mixed equilibrium the player must be indifferent: \
             {if_i_compete} vs {if_i_cooperate}"
        );
    }

    /// Efficiency loss is positive *exactly* when the equilibrium falls short of
    /// mutual cooperation, and zero otherwise.
    ///
    /// The first version of this test asserted the weaker "cooperation was not
    /// reached, so surplus was lost", and that claim is false. In the cyclical game
    /// `{cc: 0, cd: 2, dc: 1, dd: 0}` mutual cooperation is *itself* the worst
    /// outcome, so there is no cooperative surplus to lose and the loss is
    /// legitimately `0.0` even though the solver reached a mixed equilibrium. Since
    /// the loss is `(cc - payoff) / (cc - worst)`, the exact invariant is a
    /// comparison against `cooperation_payoff`, not against whether cooperation was
    /// reached.
    #[test]
    fn efficiency_loss_is_positive_exactly_when_equilibrium_falls_short_of_cooperation() {
        let games = [
            Payoffs { cc: 3.0, cd: 0.0, dc: 5.0, dd: 1.0 },   // dilemma
            Payoffs { cc: 4.0, cd: 0.0, dc: 3.0, dd: 2.0 },   // coordination
            Payoffs { cc: 5.0, cd: 2.0, dc: 4.0, dd: 1.0 },   // harmony
            Payoffs { cc: 0.0, cd: 2.0, dc: 1.0, dd: 0.0 },   // cyclical: cooperation is the worst outcome
            Payoffs { cc: 10.0, cd: -5.0, dc: 12.0, dd: -2.0 },
        ];

        for payoffs in games {
            let solution = solve(&payoffs);
            if solution.cooperation_payoff > solution.equilibrium_payoff {
                assert!(
                    solution.efficiency_loss > 0.0,
                    "cooperation beats the equilibrium, so surplus was lost: {payoffs:?}"
                );
            } else {
                assert!(
                    solution.efficiency_loss.abs() < 1e-12,
                    "nothing was lost relative to cooperation, so loss must be zero: \
                     {payoffs:?}"
                );
            }
            assert!(
                (0.0..=1.0).contains(&solution.efficiency_loss),
                "loss out of range for {payoffs:?}"
            );
        }
    }

    /// Degenerate and extreme games must not panic, divide by zero, or produce
    /// non-finite values.
    #[test]
    fn degenerate_games_are_handled() {
        let flat = Payoffs { cc: 1.0, cd: 1.0, dc: 1.0, dd: 1.0 };
        let solution = solve(&flat);
        assert!(solution.efficiency_loss.abs() < 1e-12);
        assert!(solution.equilibrium_payoff.is_finite());
        assert!(solution.cooperate_probability.is_finite());

        let no_span = Payoffs { cc: 2.0, cd: 2.0, dc: 2.0, dd: 2.0 };
        assert!(solve(&no_span).efficiency_loss.is_finite());

        let extreme = Payoffs { cc: 1e6, cd: -1e6, dc: 1e9, dd: -1e9 };
        let solution = solve(&extreme);
        assert!(solution.efficiency_loss.is_finite());
        assert!((0.0..=1.0).contains(&solution.efficiency_loss));
    }
}
