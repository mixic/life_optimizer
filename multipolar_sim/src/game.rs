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

//! A 2x2 game, and its Nash equilibria -- symmetric or not.
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
//! # The two sides need not have the same payoffs
//!
//! The first version of this module solved a *symmetric* game: one matrix described
//! both players, which made the whole model unable to express a claim as basic as
//! "cooperation is worth more to this bloc than to that one". That is not a detail,
//! because it is the claim `MULTIPOLAR_GAME.md` section 4 is organised around -- the
//! realist case has AI as one more axis of zero-sum rivalry, the optimistic case has
//! AI competition that need not be zero-sum, and those are statements about *whose*
//! cooperation payoff is higher. An `--ai` sweep row that could not move the
//! cooperation rate at all is what exposed it.
//!
//! So the solver is now a general 2x2 **bimatrix** solver: [`solve_bimatrix`] takes
//! one matrix per side. [`solve`] remains as the symmetric special case, and when
//! both matrices are equal the general path reproduces the old answer exactly --
//! including the selection rule, the mixing probability and the efficiency loss. A
//! test pins that equivalence, so widening the solver cannot quietly move the
//! results that were already published.
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
//! The mixed equilibrium is what produces the genuinely interesting behaviour:
//! neither side can commit, so each randomises, and the system does not settle. A
//! pure-only solver would miss that entirely. Both are computed, and both sides'
//! mixing probabilities are reported -- they differ as soon as the two matrices do,
//! and it is the *opponent's* matrix that determines how often you mix, because
//! indifference is about what they can make you indifferent to.

/// One player's two options.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Cooperate,
    Compete,
}

/// The four outcomes of a 2x2 game.
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
    /// The two sides play *different* pure actions: one cooperates while the other
    /// competes. Only reachable once the two matrices differ, which is why the
    /// symmetric solver never produced it. It means one side is being exploited and
    /// is choosing that anyway, because its alternative is worse.
    Asymmetric,
}

impl Equilibrium {
    pub fn label(self) -> &'static str {
        match self {
            Equilibrium::Coordination => "coordination",
            Equilibrium::Dilemma => "dilemma",
            Equilibrium::Harmony => "harmony",
            Equilibrium::Cyclical => "cyclical",
            Equilibrium::Asymmetric => "asymmetric",
        }
    }
}

/// The solved game: what actually happens, and what each player gets from it.
///
/// The payoffs reported here are averages across the two sides, which is what makes
/// them comparable with the symmetric solver's single figure. The per-side values
/// are what the simulation actually banks; this type summarises, it does not replace
/// them.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Solution {
    pub equilibrium: Equilibrium,
    /// Probability that the first player cooperates under the equilibrium played.
    ///
    /// `1.0` and `0.0` mean a pure action was selected; anything strictly between is
    /// a mixed equilibrium, where neither side can commit.
    pub cooperate_probability: f64,
    /// Probability that the *second* player cooperates. Equal to
    /// [`Solution::cooperate_probability`] exactly when the two matrices are equal,
    /// which is why the symmetric solver's single figure was enough.
    pub opponent_cooperate_probability: f64,
    /// Expected payoff to each player, averaged across the two sides. The game is
    /// symmetric by default, in which case both sides expect exactly this value.
    pub equilibrium_payoff: f64,
    /// Payoff under mutual cooperation, averaged across the two sides -- the
    /// Pareto-optimal benchmark.
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
    ///
    /// With two matrices this is the mean of the two sides' own losses. Averaging is
    /// a choice, and the alternative -- reporting each side separately -- would
    /// change the scale of a statistic the pension channel is calibrated against, so
    /// the mean is used and the per-side figures stay available at the call site.
    pub efficiency_loss: f64,
}

/// A 2x2 game in which the two sides need not have the same payoffs.
///
/// `mine` is the payoff matrix of the side the caller is solving for, and `theirs`
/// is the opponent's, each in its own "own action first" convention. Setting the two
/// equal recovers the symmetric game, and [`solve_bimatrix`] then reproduces
/// [`solve`] exactly.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bimatrix {
    pub mine: Payoffs,
    pub theirs: Payoffs,
}

impl Bimatrix {
    /// The symmetric game: one matrix describing both sides.
    pub fn symmetric(payoffs: Payoffs) -> Self {
        Bimatrix {
            mine: payoffs,
            theirs: payoffs,
        }
    }
}

/// Whether `mine` is a best response to `theirs` in this matrix.
///
/// Weak, with a tolerance: a tie counts as a best response, because a player who is
/// indifferent may play either action.
fn is_best_response(payoffs: &Payoffs, mine: Action, theirs: Action) -> bool {
    const EPS: f64 = 1e-12;
    let chosen = match mine {
        Action::Cooperate => payoffs.cooperate_against(theirs),
        Action::Compete => payoffs.compete_against(theirs),
    };
    let other = match mine {
        Action::Cooperate => payoffs.compete_against(theirs),
        Action::Compete => payoffs.cooperate_against(theirs),
    };
    chosen + EPS >= other
}

/// Whether `action` strictly dominates the other action in this matrix.
fn strictly_dominates(payoffs: &Payoffs, action: Action) -> bool {
    const EPS: f64 = 1e-12;
    let other = match action {
        Action::Cooperate => Action::Compete,
        Action::Compete => Action::Cooperate,
    };
    payoffs.payoff(action, Action::Cooperate) > payoffs.payoff(other, Action::Cooperate) + EPS
        && payoffs.payoff(action, Action::Compete) > payoffs.payoff(other, Action::Compete) + EPS
}

/// One side's efficiency loss: how far its equilibrium payoff falls short of its own
/// mutual-cooperation payoff, as a fraction of the range its own matrix offers.
fn side_loss(payoffs: &Payoffs, payoff: f64) -> f64 {
    const EPS: f64 = 1e-12;
    let worst = payoffs.cd.min(payoffs.dd).min(payoffs.cc).min(payoffs.dc);
    let span = payoffs.cc - worst;
    if span <= EPS {
        0.0
    } else {
        ((payoffs.cc - payoff) / span).clamp(0.0, 1.0)
    }
}

/// Assemble a `Solution` from a profile's mixing probabilities and per-side payoffs.
fn finish(
    equilibrium: Equilibrium,
    mine_cooperates: f64,
    theirs_cooperates: f64,
    mine_payoff: f64,
    theirs_payoff: f64,
    game: &Bimatrix,
) -> Solution {
    Solution {
        equilibrium,
        cooperate_probability: mine_cooperates,
        opponent_cooperate_probability: theirs_cooperates,
        equilibrium_payoff: (mine_payoff + theirs_payoff) / 2.0,
        cooperation_payoff: (game.mine.cc + game.theirs.cc) / 2.0,
        efficiency_loss: (side_loss(&game.mine, mine_payoff)
            + side_loss(&game.theirs, theirs_payoff))
            / 2.0,
    }
}

/// Solve a symmetric 2x2 game.
///
/// Kept as its own entry point because it is the common case and because the
/// symmetric result is the one the rest of the project's published figures were
/// produced with. It is exactly `solve_bimatrix` on two identical matrices, and a
/// test pins that equivalence rather than trusting the wrapper.
pub fn solve(payoffs: &Payoffs) -> Solution {
    solve_bimatrix(&Bimatrix::symmetric(*payoffs))
}

/// Solve a pair of matrices, taking the symmetric path when the two sides are
/// identical.
///
/// # Why this is a function and not an inline `if`
///
/// It is not an optimisation. It means the *default* model -- every bloc valuing
/// cooperation neutrally, so both matrices are equal -- runs through exactly the code
/// it ran through before the solver was widened. The project's already-published
/// `--sweep` and `--compare` figures are therefore preserved *by construction* rather
/// than by an equivalence test that a later edit could weaken without anyone noticing.
///
/// The equivalence test still exists, because a fast path that disagreed with the
/// general one would be a defect either way.
pub fn solve_pair(mine: &Payoffs, theirs: &Payoffs) -> Solution {
    if mine == theirs {
        solve(mine)
    } else {
        solve_bimatrix(&Bimatrix {
            mine: *mine,
            theirs: *theirs,
        })
    }
}

/// Solve a 2x2 game in which the two sides may face different payoffs.
///
/// Selection follows a documented rule rather than a preference:
///
/// 1. **Strict dominance on both sides**, if it holds -- neither side's choice can
///    change the outcome, so it is what self-interested play produces.
/// 2. Otherwise **the pure equilibrium with the highest joint payoff**, preferring
///    the profile with more cooperating sides on a tie. With identical matrices this
///    reproduces the old "prefer mutual cooperation when it is stable" rule exactly,
///    because `cc >= dc > dd` whenever `(C,C)` and `(D,D)` are both equilibria, so
///    joint payoff already ranks `(C,C)` first.
/// 3. Otherwise the **mixed** equilibrium, if its probabilities are interior.
/// 4. Otherwise a degenerate fallback.
///
/// The preference for a high joint payoff is a welfare criterion, and it is worth
/// naming as one: a game with two equilibria is genuinely underdetermined, and
/// picking the better one is a *choice* the model makes rather than a consequence of
/// rationality. Step 3 is the case where no such choice exists.
pub fn solve_bimatrix(game: &Bimatrix) -> Solution {
    const EPS: f64 = 1e-12;
    let a = &game.mine;
    let b = &game.theirs;

    // --- 1. Strict dominance on both sides ---------------------------------
    if strictly_dominates(a, Action::Cooperate) && strictly_dominates(b, Action::Cooperate) {
        return finish(Equilibrium::Harmony, 1.0, 1.0, a.cc, b.cc, game);
    }
    if strictly_dominates(a, Action::Compete) && strictly_dominates(b, Action::Compete) {
        return finish(Equilibrium::Dilemma, 0.0, 0.0, a.dd, b.dd, game);
    }

    // --- 2. Pure equilibria -------------------------------------------------
    //
    // A profile is a Nash equilibrium exactly when each side's action is a best
    // response to the other's.
    //
    // # Why identical matrices only consider the symmetric profiles
    //
    // When the two sides are interchangeable, the answer has to be interchangeable
    // too: reporting "the first side cooperates and the second competes" for a game
    // in which the first and second side are the same player is not a result, it is
    // an arbitrary pick dressed as one. And it is not hypothetical -- in the region
    // where the sucker's payoff exceeds mutual competition, `(C,D)` and `(D,C)` are
    // both equilibria of a symmetric game, so an unrestricted search would have to
    // choose between two mirror images with identical joint payoffs.
    //
    // So a symmetric game is searched only over `(C,C)` and `(D,D)`. When neither is
    // an equilibrium the mixed one is used, and that is the *unique exchangeable*
    // equilibrium of such a game: it is the only one under which both sides play the
    // same strategy. The asymmetric pure equilibria that also exist pay more in
    // total, which is worth knowing, but neither can be selected without deciding
    // which of two identical players is the exploiter.
    let profiles: &[(Action, Action)] = if a == b {
        &[
            (Action::Cooperate, Action::Cooperate),
            (Action::Compete, Action::Compete),
        ]
    } else {
        &[
            (Action::Cooperate, Action::Cooperate),
            (Action::Compete, Action::Compete),
            (Action::Cooperate, Action::Compete),
            (Action::Compete, Action::Cooperate),
        ]
    };

    let mut best: Option<((f64, u32, f64), Action, Action)> = None;
    for &(mine, theirs) in profiles {
        if !is_best_response(a, mine, theirs) || !is_best_response(b, theirs, mine) {
            continue;
        }

        let joint = a.payoff(mine, theirs) + b.payoff(theirs, mine);
        let cooperating =
            u32::from(mine == Action::Cooperate) + u32::from(theirs == Action::Cooperate);
        // Third key: when exactly one side cooperates, the cooperation payoff of the
        // side doing it. This is what makes the choice between `(C,D)` and `(D,C)`
        // *order-independent*.
        //
        // Those two profiles pay the same in total whenever both are equilibria, which
        // the model reaches often, so joint payoff cannot separate them. Breaking the
        // tie by list order -- the first version of this code did exactly that -- made
        // the answer depend on which bloc happened to have the lower index, which is
        // not a property of the situation. Preferring the profile in which the side
        // that values cooperation more is the one cooperating is a *convention*, and
        // it is named as one: the pair is genuinely indeterminate, and this settles it
        // in a way that does not depend on how the caller ordered the arguments.
        let enthusiast_cooperates = match (mine, theirs) {
            (Action::Cooperate, Action::Compete) => a.cc,
            (Action::Compete, Action::Cooperate) => b.cc,
            _ => 0.0,
        };
        let key = (joint, cooperating, enthusiast_cooperates);

        let better = match best {
            None => true,
            Some((best_key, _, _)) => {
                if key.0 > best_key.0 + EPS {
                    true
                } else if (key.0 - best_key.0).abs() <= EPS {
                    if key.1 != best_key.1 {
                        key.1 > best_key.1
                    } else {
                        key.2 > best_key.2 + EPS
                    }
                } else {
                    false
                }
            }
        };
        if better {
            best = Some((key, mine, theirs));
        }
    }

    if let Some((_, mine, theirs)) = best {
        let label = match (mine, theirs) {
            (Action::Cooperate, Action::Cooperate) => Equilibrium::Coordination,
            (Action::Compete, Action::Compete) => Equilibrium::Dilemma,
            // One side cooperates while the other competes, and neither can improve
            // by moving. That is the security dilemma's *asymmetric* form: the
            // exploiter has no reason to stop and the exploited side has no better
            // option. Reachable only when the two sides' payoffs differ.
            _ => Equilibrium::Asymmetric,
        };
        let p = if mine == Action::Cooperate { 1.0 } else { 0.0 };
        let q = if theirs == Action::Cooperate {
            1.0
        } else {
            0.0
        };
        return finish(
            label,
            p,
            q,
            a.payoff(mine, theirs),
            b.payoff(theirs, mine),
            game,
        );
    }

    // --- 3. The mixed equilibrium -------------------------------------------
    //
    // Let `p` be the probability the first side cooperates and `q` the second's.
    // Each side's mixing is pinned by the *other's* indifference:
    //
    //   side A indifferent:  q * (a.cc - a.cd - a.dc + a.dd) = a.dd - a.cd
    //   side B indifferent:  p * (b.cc - b.cd - b.dc + b.dd) = b.dd - b.cd
    //
    // so `q` comes from A's matrix and `p` from B's. With identical matrices the two
    // expressions coincide and reduce to the symmetric formula this module used
    // before.
    let denominator_a = a.cc - a.cd - a.dc + a.dd;
    let denominator_b = b.cc - b.cd - b.dc + b.dd;

    if denominator_a.abs() < EPS || denominator_b.abs() < EPS {
        // Degenerate: the indifference condition carries no information. Fall back to
        // whichever mutual outcome is better, which is at least a defensible answer
        // rather than a division by zero.
        let joint_cooperate = a.cc + b.cc;
        let joint_compete = a.dd + b.dd;
        return if joint_cooperate >= joint_compete {
            finish(Equilibrium::Cyclical, 1.0, 1.0, a.cc, b.cc, game)
        } else {
            finish(Equilibrium::Cyclical, 0.0, 0.0, a.dd, b.dd, game)
        };
    }

    let q = ((a.dd - a.cd) / denominator_a).clamp(0.0, 1.0);
    let p = ((b.dd - b.cd) / denominator_b).clamp(0.0, 1.0);

    // At indifference each side's payoff is the same whichever action it takes, so
    // evaluating one bracket gives that side's value.
    let mine_payoff = q * a.cc + (1.0 - q) * a.cd;
    let theirs_payoff = p * b.cc + (1.0 - p) * b.cd;

    finish(
        Equilibrium::Cyclical,
        p,
        q,
        mine_payoff,
        theirs_payoff,
        game,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The textbook Prisoner's Dilemma: competing dominates, and the equilibrium
    /// is the outcome both players would rather avoid. This is the structure
    /// `THEORY_OF_SPARING.md` section 7d describes for engineered obsolescence.
    #[test]
    fn prisoners_dilemma_is_solved_as_a_dilemma() {
        let pd = Payoffs {
            cc: 3.0,
            cd: 0.0,
            dc: 5.0,
            dd: 1.0,
        };
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
        let hunt = Payoffs {
            cc: 4.0,
            cd: 0.0,
            dc: 3.0,
            dd: 2.0,
        };
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
        let harmony = Payoffs {
            cc: 5.0,
            cd: 2.0,
            dc: 4.0,
            dd: 1.0,
        };
        let solution = solve(&harmony);
        assert_eq!(solution.equilibrium, Equilibrium::Harmony);
        assert_eq!(solution.cooperate_probability, 1.0);
        assert!(solution.efficiency_loss.abs() < 1e-12);
    }

    /// A game with no pure equilibrium must return a genuinely mixed one, with a
    /// probability strictly between the two pure actions. This is the case a
    /// pure-only solver silently gets wrong, so it is pinned -- including the
    /// indifference condition that *defines* the mixed equilibrium.
    ///
    /// # Why this is now a bimatrix and not a symmetric game
    ///
    /// The first version of this test used the symmetric game
    /// `{cc: 0, cd: 2, dc: 1, dd: 0}` and described it as having no pure
    /// equilibrium. That description was false, and generalising the solver is what
    /// exposed it: in that game `(Cooperate, Compete)` satisfies both best-response
    /// conditions, because the cooperator earns `cd = 2` -- the best payoff either
    /// side can obtain -- and the competitor earns `dc = 1`. Its mirror is an
    /// equilibrium too.
    ///
    /// In fact a symmetric 2x2 game essentially *cannot* lack a pure equilibrium:
    /// having none requires `cc < dc` and `dd < cd` to rule out `(C,C)` and `(D,D)`,
    /// and those two inequalities are exactly what makes `(C,D)` and `(D,C)`
    /// equilibria. So "no pure equilibrium" is only reachable once the two sides face
    /// *different* payoffs, which is what this test now uses.
    ///
    /// The old solver's answer for that game was nonetheless the right one, for a
    /// reason its own comment did not give: the two asymmetric equilibria are mirror
    /// images, so a symmetric game cannot select either without deciding which of two
    /// identical players is the exploiter. See
    /// `a_symmetric_game_is_never_answered_asymmetrically`.
    #[test]
    fn games_without_a_pure_equilibrium_are_solved_mixed() {
        // Matching-pennies-like across two *different* matrices: neither pure profile
        // is an equilibrium for both sides at once.
        let game = Bimatrix {
            mine: Payoffs {
                cc: 1.0,
                cd: 0.0,
                dc: 0.0,
                dd: 1.0,
            },
            theirs: Payoffs {
                cc: 0.0,
                cd: 1.0,
                dc: 1.0,
                dd: 0.0,
            },
        };
        let solution = solve_bimatrix(&game);
        assert_eq!(solution.equilibrium, Equilibrium::Cyclical);
        assert!(
            solution.cooperate_probability > 0.0 && solution.cooperate_probability < 1.0,
            "a mixed equilibrium must mix: got {}",
            solution.cooperate_probability
        );
        assert!(
            solution.opponent_cooperate_probability > 0.0
                && solution.opponent_cooperate_probability < 1.0,
            "both sides must mix: got {}",
            solution.opponent_cooperate_probability
        );

        // Verify indifference directly, which is what defines the mixed
        // equilibrium: at this q, the first side cannot gain by switching.
        let q = solution.opponent_cooperate_probability;
        let if_i_compete = q * game.mine.dc + (1.0 - q) * game.mine.dd;
        let if_i_cooperate = q * game.mine.cc + (1.0 - q) * game.mine.cd;
        assert!(
            (if_i_compete - if_i_cooperate).abs() < 1e-9,
            "at the mixed equilibrium the player must be indifferent: \
             {if_i_compete} vs {if_i_cooperate}"
        );
    }

    /// A symmetric game must never be answered asymmetrically.
    ///
    /// In the region where the sucker's payoff beats mutual competition -- which the
    /// simulation reaches at high tension -- a symmetric game has *three*
    /// equilibria: `(C,D)`, `(D,C)`, and a mixed one. The first two pay more in total
    /// (2 and 1, against 2/3 each) and it is tempting to prefer them. Doing so would
    /// be wrong: the two sides are the same player, so the model would be deciding
    /// which of two identical actors is the exploiter. The mixed equilibrium is the
    /// only exchangeable one, so it is the one reported.
    ///
    /// This is the property that keeps the simulator from manufacturing structure out
    /// of an arbitrary pick, so it is pinned directly rather than left implicit.
    #[test]
    fn a_symmetric_game_is_never_answered_asymmetrically() {
        let game = Payoffs {
            cc: 0.0,
            cd: 2.0,
            dc: 1.0,
            dd: 0.0,
        };

        // Both asymmetric profiles really are Nash equilibria of this game -- the
        // reason a naive "prefer the pure one" rule would select them.
        assert!(is_best_response(&game, Action::Cooperate, Action::Compete));
        assert!(is_best_response(&game, Action::Compete, Action::Cooperate));

        let solution = solve(&game);
        assert_eq!(
            solution.equilibrium,
            Equilibrium::Cyclical,
            "the exchangeable answer here is the mixed equilibrium, not one of the \
             two mirror-image pure ones"
        );
        assert_eq!(
            solution.cooperate_probability, solution.opponent_cooperate_probability,
            "identical sides must mix identically"
        );
    }

    /// Unequal cooperation payoffs must actually change what the sides do.
    ///
    /// Without this the asymmetric solver would be machinery that never bites, and
    /// the claim `--ai` exists to test -- that AI ownership makes cooperation worth
    /// more to the bloc that owns it -- would still be inexpressible.
    #[test]
    fn unequal_cooperation_payoffs_change_the_cooperation_rate() {
        // One side already wants to cooperate against cooperation; the other's
        // cooperation payoff is too low, so `(C,C)` is not an equilibrium and the
        // dilemma's `(D,D)` is. Both compete.
        let keen = Payoffs {
            cc: 5.5,
            cd: 0.0,
            dc: 5.0,
            dd: 1.0,
        };
        let cool = Payoffs {
            cc: 4.0,
            cd: 0.0,
            dc: 5.0,
            dd: 1.0,
        };
        let before = solve_bimatrix(&Bimatrix {
            mine: keen,
            theirs: cool,
        });
        assert_eq!(
            before.equilibrium,
            Equilibrium::Dilemma,
            "with the second side's cooperation payoff low, both compete"
        );
        assert_eq!(before.opponent_cooperate_probability, 0.0);

        // Now raise only the second side's cooperation payoff past its temptation.
        // Cooperation becomes its best response to cooperation, `(C,C)` becomes an
        // equilibrium, and it pays better in total than `(D,D)`.
        let after = solve_bimatrix(&Bimatrix {
            mine: keen,
            theirs: Payoffs { cc: 5.5, ..cool },
        });
        assert!(
            after.opponent_cooperate_probability > before.opponent_cooperate_probability,
            "raising a side's cooperation payoff must raise how often it cooperates: \
             {} vs {}",
            after.opponent_cooperate_probability,
            before.opponent_cooperate_probability
        );
        assert!(
            after.cooperation_payoff > before.cooperation_payoff,
            "and must raise the reported average cooperation payoff"
        );
    }

    /// Every solved profile must actually be a Nash equilibrium: no side can gain by
    /// deviating unilaterally. Checked across a grid of bimatrix games rather than on
    /// hand-picked cases, because "the solver returns *an* equilibrium" is the one
    /// property that must never quietly fail.
    #[test]
    fn every_solved_profile_is_a_nash_equilibrium() {
        let values = [0.0, 1.0, 2.0, 3.5];
        for &a_cc in &values {
            for &a_cd in &values {
                for &a_dc in &values {
                    for &a_dd in &values {
                        for &b_dc in &values {
                            let game = Bimatrix {
                                mine: Payoffs {
                                    cc: a_cc,
                                    cd: a_cd,
                                    dc: a_dc,
                                    dd: a_dd,
                                },
                                // Vary the opponent's matrix too, so the asymmetric
                                // equilibria this solver can now reach are exercised.
                                theirs: Payoffs {
                                    cc: a_cc,
                                    cd: a_cd,
                                    dc: b_dc,
                                    dd: a_dd,
                                },
                            };
                            let solution = solve_bimatrix(&game);
                            let p = solution.cooperate_probability;
                            let q = solution.opponent_cooperate_probability;

                            // Expected payoff to each side at the reported profile.
                            let mine = p * (q * game.mine.cc + (1.0 - q) * game.mine.cd)
                                + (1.0 - p) * (q * game.mine.dc + (1.0 - q) * game.mine.dd);
                            let theirs = q * (p * game.theirs.cc + (1.0 - p) * game.theirs.cd)
                                + (1.0 - q) * (p * game.theirs.dc + (1.0 - p) * game.theirs.dd);

                            // Deviating to the pure actions must not help either side.
                            let mine_if_cooperate = q * game.mine.cc + (1.0 - q) * game.mine.cd;
                            let mine_if_compete = q * game.mine.dc + (1.0 - q) * game.mine.dd;
                            let theirs_if_cooperate =
                                p * game.theirs.cc + (1.0 - p) * game.theirs.cd;
                            let theirs_if_compete = p * game.theirs.dc + (1.0 - p) * game.theirs.dd;

                            assert!(
                                mine + 1e-9 >= mine_if_cooperate,
                                "{game:?} at p={p} q={q}: first side gains by deviating \
                                 to Cooperate ({mine_if_cooperate} > {mine})"
                            );
                            assert!(
                                mine + 1e-9 >= mine_if_compete,
                                "{game:?} at p={p} q={q}: first side gains by deviating \
                                 to Compete ({mine_if_compete} > {mine})"
                            );
                            assert!(
                                theirs + 1e-9 >= theirs_if_cooperate,
                                "{game:?} at p={p} q={q}: second side gains by deviating \
                                 to Cooperate ({theirs_if_cooperate} > {theirs})"
                            );
                            assert!(
                                theirs + 1e-9 >= theirs_if_compete,
                                "{game:?} at p={p} q={q}: second side gains by deviating \
                                 to Compete ({theirs_if_compete} > {theirs})"
                            );

                            assert!(
                                (0.0..=1.0).contains(&p) && (0.0..=1.0).contains(&q),
                                "{game:?}: probabilities out of range: {p}, {q}"
                            );
                            assert!(
                                (0.0..=1.0).contains(&solution.efficiency_loss),
                                "{game:?}: loss out of range"
                            );
                        }
                    }
                }
            }
        }
    }

    /// The answer must not depend on the order the two sides were passed in.
    ///
    /// Swapping `mine` and `theirs` swaps the two cooperation probabilities and must
    /// change nothing else -- not the equilibrium selected, not the payoffs, not the
    /// loss. This is the property that catches a solver quietly deciding an
    /// asymmetric equilibrium by list position, which the first version of this code
    /// did: `(C,D)` and `(D,C)` pay the same in total whenever both are equilibria,
    /// so whichever came first in the iteration won, and the answer depended on which
    /// bloc had the lower index rather than on anything about the situation.
    ///
    /// Checked across a grid, because the failure is invisible for most inputs and
    /// only shows up where two asymmetric profiles tie.
    #[test]
    fn the_solution_does_not_depend_on_the_order_of_the_two_sides() {
        let values = [0.0, 1.0, 2.0, 3.5];
        for &cc in &values {
            for &cd in &values {
                for &dc in &values {
                    for &dd in &values {
                        for &other_cc in &values {
                            let mine = Payoffs { cc, cd, dc, dd };
                            let theirs = Payoffs {
                                cc: other_cc,
                                ..mine
                            };

                            let forward = solve_bimatrix(&Bimatrix { mine, theirs });
                            let reversed = solve_bimatrix(&Bimatrix {
                                mine: theirs,
                                theirs: mine,
                            });

                            assert_eq!(
                                forward.equilibrium, reversed.equilibrium,
                                "{mine:?} vs {theirs:?}: the equilibrium label must not \
                                 depend on argument order"
                            );
                            assert!(
                                (forward.cooperate_probability
                                    - reversed.opponent_cooperate_probability)
                                    .abs()
                                    < 1e-12,
                                "{mine:?} vs {theirs:?}: swapping the sides must swap \
                                 the cooperation probabilities"
                            );
                            assert!(
                                (forward.opponent_cooperate_probability
                                    - reversed.cooperate_probability)
                                    .abs()
                                    < 1e-12,
                                "{mine:?} vs {theirs:?}: and only swap them"
                            );
                            assert!(
                                (forward.equilibrium_payoff - reversed.equilibrium_payoff).abs()
                                    < 1e-12,
                                "{mine:?} vs {theirs:?}: the averaged payoff is \
                                 order-independent by construction"
                            );
                            assert!(
                                (forward.efficiency_loss - reversed.efficiency_loss).abs() < 1e-12,
                                "{mine:?} vs {theirs:?}: so is the averaged loss"
                            );
                        }
                    }
                }
            }
        }
    }

    /// With identical matrices the general solver must reproduce the symmetric one
    /// exactly, on every field.
    ///
    /// This is the guarantee that widening the solver did not quietly move figures
    /// that were already reported. `solve` delegates, so the interesting part is that
    /// the *derived* fields agree -- the mixing probability, the efficiency loss and
    /// the two per-side probabilities -- and that the two sides come out equal.
    #[test]
    fn the_general_solver_reduces_to_the_symmetric_one() {
        let games = [
            Payoffs {
                cc: 3.0,
                cd: 0.0,
                dc: 5.0,
                dd: 1.0,
            },
            Payoffs {
                cc: 4.0,
                cd: 0.0,
                dc: 3.0,
                dd: 2.0,
            },
            Payoffs {
                cc: 5.0,
                cd: 2.0,
                dc: 4.0,
                dd: 1.0,
            },
            Payoffs {
                cc: 0.0,
                cd: 2.0,
                dc: 1.0,
                dd: 0.0,
            },
            Payoffs {
                cc: 10.0,
                cd: -5.0,
                dc: 12.0,
                dd: -2.0,
            },
            Payoffs {
                cc: 1.0,
                cd: 1.0,
                dc: 1.0,
                dd: 1.0,
            },
        ];

        for payoffs in games {
            let symmetric = solve(&payoffs);
            let general = solve_bimatrix(&Bimatrix::symmetric(payoffs));
            assert_eq!(
                symmetric, general,
                "the symmetric entry point and the general solver must agree on \
                 {payoffs:?}"
            );
            assert_eq!(
                general.cooperate_probability, general.opponent_cooperate_probability,
                "with identical matrices both sides must mix identically: {payoffs:?}"
            );
        }
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
            Payoffs {
                cc: 3.0,
                cd: 0.0,
                dc: 5.0,
                dd: 1.0,
            }, // dilemma
            Payoffs {
                cc: 4.0,
                cd: 0.0,
                dc: 3.0,
                dd: 2.0,
            }, // coordination
            Payoffs {
                cc: 5.0,
                cd: 2.0,
                dc: 4.0,
                dd: 1.0,
            }, // harmony
            Payoffs {
                cc: 0.0,
                cd: 2.0,
                dc: 1.0,
                dd: 0.0,
            }, // cyclical: cooperation is the worst outcome
            Payoffs {
                cc: 10.0,
                cd: -5.0,
                dc: 12.0,
                dd: -2.0,
            },
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
        let flat = Payoffs {
            cc: 1.0,
            cd: 1.0,
            dc: 1.0,
            dd: 1.0,
        };
        let solution = solve(&flat);
        assert!(solution.efficiency_loss.abs() < 1e-12);
        assert!(solution.equilibrium_payoff.is_finite());
        assert!(solution.cooperate_probability.is_finite());

        let no_span = Payoffs {
            cc: 2.0,
            cd: 2.0,
            dc: 2.0,
            dd: 2.0,
        };
        assert!(solve(&no_span).efficiency_loss.is_finite());

        let extreme = Payoffs {
            cc: 1e6,
            cd: -1e6,
            dc: 1e9,
            dd: -1e9,
        };
        let solution = solve(&extreme);
        assert!(solution.efficiency_loss.is_finite());
        assert!((0.0..=1.0).contains(&solution.efficiency_loss));
    }
}
