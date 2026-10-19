// knowledge_work -- just-in-time logic in AI-assisted knowledge work
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

//! The direction framework's contract: it reports who wins under which priorities, and it never
//! reports a winner.

use knowledge_work::direction::*;

fn option(
    throughput: f64,
    cycle_time: f64,
    escaped_defects: f64,
    autonomy_years: f64,
    backlog: f64,
) -> Outcome {
    Outcome { throughput, cycle_time, escaped_defects, autonomy_years, backlog }
}

/// Two configurations of the same workflow, in the shape the model produces: one that ships a
/// lot and one that ships less of a better thing. Neither dominates, which is the point.
fn competing_options() -> Vec<Outcome> {
    vec![
        // Lean: everything generated is pushed through.
        option(30.0, 12.0, 5.0, 6.0, 0.0),
        // Buffered: less throughput, faster, fewer escapes, and more years spent judging.
        option(20.0, 3.0, 2.0, 14.0, 6.0),
    ]
}

/// An option worse on every criterion must never come first, under any weighting. If the grid
/// can rescue a dominated option the aggregation is broken.
#[test]
fn a_dominated_option_never_wins() {
    let options = vec![option(10.0, 8.0, 6.0, 4.0, 8.0), option(20.0, 4.0, 3.0, 9.0, 2.0)];
    let result = acceptance(&options, 8);
    assert_eq!(
        result.shares[0],
        0.0,
        "the dominated option won {:.0}% of the weightings",
        result.shares[0] * 100.0
    );
    assert!(result.never_first.contains(&0));
    assert!((result.shares[1] - 1.0).abs() < 1e-12);
}

/// Weighting a single criterion must rank by that criterion, in the right direction: the
/// framework must know that a shorter cycle time is better and that more escaped defects is not.
///
/// The fixture has to *disagree* for this to test anything. A first version used a dominated
/// pair, where one option was better on every criterion, so every weighting returned the same
/// winner and the test could not tell a correct direction from an inverted one.
#[test]
fn a_single_criterion_weighting_ranks_by_that_criterion() {
    let options = competing_options();
    // lean: (30, 12, 5, 6, 0); buffered: (20, 3, 2, 14, 6). The winner under each criterion
    // alone, worked out by hand so the test is not reading the implementation back to itself.
    let expected = [
        0, // throughput: 30 beats 20
        1, // cycle time: 3 days beats 12
        1, // escaped defects: 2 beats 5
        1, // autonomy years: 14 beats 6
        0, // backlog: 0 beats 6
    ];
    for (criterion, expected_winner) in expected.iter().enumerate() {
        let mut values = [0.0; CRITERIA];
        values[criterion] = 1.0;
        let winner = rank(&options, &Weights::new(values))[0];
        assert_eq!(
            winner,
            *expected_winner,
            "under a weighting on {} alone the winner was {winner}, not {expected_winner}",
            criterion_name(criterion)
        );
    }
    // Reversing the options must reverse the answer for every criterion that discriminates,
    // which proves the direction is applied to the value and not to the position.
    let reversed = vec![options[1], options[0]];
    for (criterion, expected_winner) in expected.iter().enumerate() {
        let mut values = [0.0; CRITERIA];
        values[criterion] = 1.0;
        let winner = rank(&reversed, &Weights::new(values))[0];
        assert_eq!(
            winner,
            1 - *expected_winner,
            "reversing the options did not reverse the ranking on {}",
            criterion_name(criterion)
        );
    }
}

/// The shares must be shares: non-negative, summing to one.
#[test]
fn the_shares_are_shares() {
    let options = competing_options();
    let result = acceptance(&options, 6);
    assert!(result.weightings > 100, "the grid is too coarse: {}", result.weightings);
    let total: f64 = result.shares.iter().sum();
    assert!((total - 1.0).abs() < 1e-12, "the shares sum to {total}, not one: {:?}", result.shares);
    assert!(result.shares.iter().all(|share| *share >= 0.0));
}

/// The grid must be the simplex: every weight vector non-negative and summing to one.
#[test]
fn the_grid_is_the_simplex() {
    let grid = weight_simplex(4);
    // Five-part compositions of four: C(8, 4) = 70.
    assert_eq!(grid.len(), 70);
    for weights in &grid {
        let total: f64 = (0..CRITERIA).map(|index| weights.get(index)).sum();
        assert!((total - 1.0).abs() < 1e-12, "a weight vector sums to {total}, not one");
        assert!((0..CRITERIA).all(|index| weights.get(index) >= 0.0));
    }
    // A normalised vector is normalised whatever it was built from.
    let lopsided = Weights::new([3.0, 1.0, 0.0, 0.0, 0.0]);
    assert!((lopsided.throughput - 0.75).abs() < 1e-12);
    assert!((lopsided.cycle_time - 0.25).abs() < 1e-12);
}

/// The framework's central claim: on a genuine trade-off, the answer depends on the priorities,
/// and the declared weighting is one point on the surface rather than the truth.
#[test]
fn the_declared_weighting_does_not_settle_the_question() {
    let options = competing_options();
    let result = acceptance(&options, 8);
    println!(
        "acceptance shares under {} weightings: lean {:.0}%, buffered {:.0}%; declared ranking {:?}",
        result.weightings,
        result.shares[0] * 100.0,
        result.shares[1] * 100.0,
        result.declared_ranking
    );
    for (index, share) in result.shares.iter().enumerate() {
        assert!(
            *share > 0.05,
            "option {index} wins under only {:.1}% of weighting vectors, so this fixture is not \
             a trade-off",
            share * 100.0
        );
    }
    assert!(result.never_first.is_empty());
}

/// The direction of the result must be explicable: the option that wins when autonomy is
/// weighted must be the one with more autonomy years. A framework whose disagreement cannot be
/// traced to a criterion is reporting noise.
#[test]
fn the_disagreement_can_be_traced_to_a_criterion() {
    let options = competing_options();
    let mut autonomy_only = [0.0; CRITERIA];
    autonomy_only[3] = 1.0;
    let autonomy_winner = rank(&options, &Weights::new(autonomy_only))[0];
    assert_eq!(
        autonomy_winner, 1,
        "the higher-autonomy option should win under an autonomy weighting"
    );
    let mut throughput_only = [0.0; CRITERIA];
    throughput_only[0] = 1.0;
    let throughput_winner = rank(&options, &Weights::new(throughput_only))[0];
    assert_eq!(throughput_winner, 0);
    assert_ne!(
        autonomy_winner, throughput_winner,
        "the two criteria did not disagree, so the fixture proves nothing"
    );
}

/// Every criterion the framework scores must have a declared provenance, or the axes of the
/// simplex would be undocumented assumptions.
#[test]
fn every_criterion_is_declared() {
    let table = Weights::provenance_table();
    assert_eq!(table.len(), CRITERIA);
    for (name, rationale) in &table {
        assert!(!name.is_empty());
        assert!(
            rationale.starts_with("declared:"),
            "the rationale for {name} does not say it is declared"
        );
        assert!(rationale.len() > 40, "the rationale for {name} is not a rationale");
    }
    // And every criterion is reachable by index, which is what the scoring loop relies on.
    for index in 0..CRITERIA {
        assert_ne!(criterion_name(index), "unknown");
        let mut values = [0.0; CRITERIA];
        values[index] = 1.0;
        assert!(Weights::new(values).get(index) > 0.99);
    }
}
