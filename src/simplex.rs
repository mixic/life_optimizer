// Life Optimizer
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

//! A small dense simplex solver, so that "solved by linear programming" is a claim
//! with a solver behind it rather than a description of the shape of a problem.
//!
//! # Why this exists rather than a crate
//!
//! The early-retirement model in [`crate::early_retirement`] allocates withdrawals
//! across years against a banded tax schedule. The deterministic version of that is a
//! linear program, and the dual variables are the thing worth reading: each one is the
//! shadow price of a year's tax-free room, which is exactly the quantity that says
//! whether another franc of withdrawal belongs in that year. A boxed solver would
//! compute the same numbers and hide the reason they are the numbers.
//!
//! It is also a *cross-check*. The same allocation has a closed-form greedy solution
//! and a KKT characterisation (equalised marginal tax across years), so this file is a
//! third, independent route to the same answer. Three methods agreeing is worth more
//! than one method being fast.
//!
//! # What it solves
//!
//! ```text
//!   minimise    c'x
//!   subject to  A_ub x <= b_ub
//!               A_eq x  = b_eq
//!               x >= 0
//! ```
//!
//! By the **two-phase** method with **Bland's rule**. Bland's rule is chosen over
//! Dantzig's deliberately: it is slower and it cannot cycle. A solver that returns a
//! wrong optimum because it looped is worse than one that takes more iterations.
//!
//! # The tableau, stated once so the signs cannot drift
//!
//! Columns are `[ x | slacks | artificials ]`. The last tableau row is the objective
//! row, holding reduced costs in the convention **`r_j = c_j - z_j`**, so the entering
//! column is the lowest-indexed one with `r_j < 0` and the row's rightmost entry holds
//! `-z`. Getting this convention wrong inverts every sign in the answer while leaving
//! the arithmetic self-consistent, which is why it is written down here and why the
//! tests check the objective against hand-computed values rather than against the
//! solver's own output.
//!
//! Artificial columns are kept out of phase 2 by a **forbidden mask** rather than by a
//! large cost. Big-M would perturb the optimum, and an infinite cost would poison the
//! reduced costs of every row that still holds an artificial — which is how the dual
//! values of equality constraints get read in the first place.
//!
//! # Limits, stated rather than discovered
//!
//! Dense, `f64`, no sparsity, no presolve, and an iteration cap so a pathological input
//! terminates instead of hanging. It is sized for the problems in this repository —
//! tens of variables — and would be the wrong tool for tens of thousands.

use std::fmt;

/// What happened when the solver ran.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LpStatus {
    /// An optimal vertex was reached.
    Optimal,
    /// No feasible point exists.
    Infeasible,
    /// The objective is unbounded below on the feasible set.
    Unbounded,
    /// The iteration cap was hit. Reported rather than papered over: a truncated
    /// simplex returns a *feasible but non-optimal* vertex, and a caller that could not
    /// tell the difference would read it as an answer.
    IterationLimit,
}

impl LpStatus {
    pub fn label(self) -> &'static str {
        match self {
            LpStatus::Optimal => "optimal",
            LpStatus::Infeasible => "infeasible",
            LpStatus::Unbounded => "unbounded",
            LpStatus::IterationLimit => "iteration limit reached",
        }
    }
}

/// The result of one solve.
#[derive(Debug, Clone)]
pub struct LpSolution {
    pub status: LpStatus,
    /// Optimal objective, or `NaN` when no optimum was reached.
    pub objective: f64,
    /// The decision variables, one per entry of the cost vector.
    pub x: Vec<f64>,
    /// Shadow price of each `<=` constraint, in the order given.
    ///
    /// The rate at which the optimal objective changes per unit increase of that
    /// right-hand side. Non-positive for a minimisation, because relaxing a `<=`
    /// constraint cannot make the minimum worse; its **magnitude** is the value of one
    /// more unit of that right-hand side.
    pub dual_ub: Vec<f64>,
    /// Shadow price of each `=` constraint, in the order given.
    pub dual_eq: Vec<f64>,
    pub iterations: usize,
}

impl LpSolution {
    fn empty(status: LpStatus, n: usize, m_ub: usize, m_eq: usize, iterations: usize) -> Self {
        LpSolution {
            status,
            objective: f64::NAN,
            x: vec![f64::NAN; n],
            dual_ub: vec![f64::NAN; m_ub],
            dual_eq: vec![f64::NAN; m_eq],
            iterations,
        }
    }
}

/// A malformed problem, as distinct from an infeasible one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LpError {
    /// A row's length does not match the number of columns.
    RaggedRow {
        row: usize,
        expected: usize,
        got: usize,
    },
    /// A right-hand side is negative.
    ///
    /// Not handled by negating the row here, because negating a row flips its
    /// inequality and the caller's `dual_ub` sign would silently change meaning. A
    /// caller with a negative right-hand side should write it as the constraint it is.
    NegativeRhs {
        row: usize,
    },
    /// Nothing to solve.
    Empty,
}

impl fmt::Display for LpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LpError::RaggedRow {
                row,
                expected,
                got,
            } => write!(f, "row {row} has {got} entries, expected {expected}"),
            LpError::NegativeRhs { row } => write!(
                f,
                "row {row} has a negative right-hand side; write it as the constraint it is \
                 rather than having this solver negate the row and change the dual's sign"
            ),
            LpError::Empty => write!(f, "the problem has no columns"),
        }
    }
}

/// Cap on pivots, far above what any problem here needs, so that hitting it means
/// something is wrong rather than that the model grew.
const MAX_ITERATIONS: usize = 20_000;
/// Numerical zero.
const EPSILON: f64 = 1e-9;

/// Solve a linear program by two-phase simplex with Bland's rule.
pub fn solve(
    objective: &[f64],
    a_ub: &[Vec<f64>],
    b_ub: &[f64],
    a_eq: &[Vec<f64>],
    b_eq: &[f64],
) -> Result<LpSolution, LpError> {
    let n = objective.len();
    let m_ub = a_ub.len();
    let m_eq = a_eq.len();

    if n == 0 {
        return Err(LpError::Empty);
    }
    for (row, entries) in a_ub.iter().enumerate() {
        if entries.len() != n {
            return Err(LpError::RaggedRow {
                row,
                expected: n,
                got: entries.len(),
            });
        }
    }
    for (row, entries) in a_eq.iter().enumerate() {
        if entries.len() != n {
            return Err(LpError::RaggedRow {
                row: m_ub + row,
                expected: n,
                got: entries.len(),
            });
        }
    }
    if b_ub.len() != m_ub || b_eq.len() != m_eq {
        return Err(LpError::Empty);
    }
    for (row, rhs) in b_ub.iter().chain(b_eq.iter()).enumerate() {
        if *rhs < 0.0 {
            return Err(LpError::NegativeRhs { row });
        }
    }

    // ---- standard form: [ x | slack (m_ub) | artificial (m_ub + m_eq) ] ----
    let total_columns = n + m_ub + m_ub + m_eq;
    let slack_start = n;
    let artificial_start = n + m_ub;
    let rows = m_ub + m_eq;
    let rhs = total_columns;

    // `rows + 1` rows: the constraints, then the objective row.
    let mut tableau = vec![vec![0.0_f64; total_columns + 1]; rows + 1];
    let mut basis = vec![0usize; rows];

    for (row, entries) in a_ub.iter().enumerate() {
        tableau[row][..n].copy_from_slice(entries);
        tableau[row][slack_start + row] = 1.0;
        tableau[row][artificial_start + row] = 1.0;
        tableau[row][rhs] = b_ub[row];
        basis[row] = artificial_start + row;
    }
    for (row, entries) in a_eq.iter().enumerate() {
        let target = m_ub + row;
        tableau[target][..n].copy_from_slice(entries);
        tableau[target][artificial_start + target] = 1.0;
        tableau[target][rhs] = b_eq[row];
        basis[target] = artificial_start + target;
    }

    // ---- phase 1: minimise the sum of the artificials ----------------------
    let mut cost = vec![0.0_f64; total_columns];
    for value in cost.iter_mut().skip(artificial_start) {
        *value = 1.0;
    }
    build_objective_row(&mut tableau, &cost, &basis, rows, total_columns);
    let mut iterations = 0;
    let mut status = run_phase(
        &mut tableau,
        &mut basis,
        rows,
        total_columns,
        rhs,
        &mut iterations,
    );
    if status != LpStatus::Optimal {
        return Ok(LpSolution::empty(
            status,
            n,
            m_ub,
            m_eq,
            iterations,
        ));
    }

    // Phase-1 objective is the sum of the artificials still basic; a positive value
    // means some row could not be satisfied by any feasible point.
    let phase_one = -tableau[rows][rhs];
    if phase_one > 1e-7 {
        return Ok(LpSolution::empty(
            LpStatus::Infeasible,
            n,
            m_ub,
            m_eq,
            iterations,
        ));
    }

    // ---- drive any remaining artificials out of the basis ------------------
    //
    // A row whose basic variable is artificial at zero is degenerate. If no real column
    // can enter it, the row is redundant and is left holding its artificial at zero,
    // contributing nothing to the objective row. This flag records that, so phase 2
    // does not try to price a variable that is not part of the problem.
    let mut redundant = vec![false; rows];
    for row in 0..rows {
        if basis[row] < artificial_start {
            continue;
        }
        match (0..artificial_start).find(|column| tableau[row][*column].abs() > EPSILON) {
            Some(column) => {
                pivot(&mut tableau, row, column, rows, rhs);
                basis[row] = column;
            }
            None => redundant[row] = true,
        }
    }

    // ---- phase 2: the real objective --------------------------------------
    let mut cost = vec![0.0_f64; total_columns];
    cost[..n].copy_from_slice(objective);
    // Artificials stay in the tableau so that the dual of an equality row can be read
    // from its reduced cost, but they are masked out of the entering rule.
    let mut forbidden = vec![false; total_columns];
    for value in forbidden.iter_mut().skip(artificial_start) {
        *value = true;
    }
    build_objective_row_masked(&mut tableau, &cost, &basis, &redundant, rows, total_columns);

    status = run_phase_masked(
        &mut tableau,
        &mut basis,
        rows,
        total_columns,
        rhs,
        &forbidden,
        &mut iterations,
    );
    if status != LpStatus::Optimal {
        return Ok(LpSolution::empty(
            status,
            n,
            m_ub,
            m_eq,
            iterations,
        ));
    }

    // ---- read the answer back ---------------------------------------------
    let mut x = vec![0.0_f64; n];
    for row in 0..rows {
        if basis[row] < n {
            x[basis[row]] = tableau[row][rhs];
        }
    }
    let objective_value: f64 = (0..n).map(|column| objective[column] * x[column]).sum();

    // `r_j = c_j - z_j`, so for a slack column `r = -y` and for an artificial column on
    // an equality row `r = -y` as well. Both dual vectors are therefore negatives of
    // reduced costs, which the tests check against hand-computed values.
    let dual_ub = (0..m_ub).map(|row| -tableau[rows][slack_start + row]).collect();
    let dual_eq = (0..m_eq)
        .map(|row| -tableau[rows][artificial_start + m_ub + row])
        .collect();

    Ok(LpSolution {
        status: LpStatus::Optimal,
        objective: objective_value,
        x,
        dual_ub,
        dual_eq,
        iterations,
    })
}

/// Build the objective row: `r_j = c_j - sum over rows of cost(basis[row]) * a[row][j]`,
/// and the rightmost entry holds `-z`.
fn build_objective_row(
    tableau: &mut [Vec<f64>],
    cost: &[f64],
    basis: &[usize],
    rows: usize,
    total_columns: usize,
) {
    let redundant = vec![false; rows];
    build_objective_row_masked(tableau, cost, basis, &redundant, rows, total_columns);
}

/// The same, treating redundant rows as unpriced so a held artificial cannot leak a
/// cost into the reduced costs of a column it does not belong to.
fn build_objective_row_masked(
    tableau: &mut [Vec<f64>],
    cost: &[f64],
    basis: &[usize],
    redundant: &[bool],
    rows: usize,
    total_columns: usize,
) {
    // Accumulated in a local vector rather than written straight into `tableau[rows]`,
    // because that row cannot be borrowed at the same time as the rows being read.
    let mut reduced = vec![0.0_f64; total_columns + 1];
    reduced[..total_columns].copy_from_slice(cost);
    let mut z = 0.0;
    for (row, line) in tableau.iter().enumerate().take(rows) {
        if redundant[row] {
            continue;
        }
        let coefficient = cost[basis[row]];
        for (value, entry) in reduced.iter_mut().zip(line.iter()) {
            *value -= coefficient * entry;
        }
        z += coefficient * line[total_columns];
    }
    reduced[total_columns] = -z;
    tableau[rows].copy_from_slice(&reduced);
}

/// Pivot until optimal, unbounded, or the cap is hit, ignoring no columns.
fn run_phase(
    tableau: &mut [Vec<f64>],
    basis: &mut [usize],
    rows: usize,
    total_columns: usize,
    rhs: usize,
    iterations: &mut usize,
) -> LpStatus {
    let forbidden = vec![false; total_columns];
    run_phase_masked(
        tableau,
        basis,
        rows,
        total_columns,
        rhs,
        &forbidden,
        iterations,
    )
}

/// Pivot with Bland's rule: entering column is the lowest-indexed one with a negative
/// reduced cost that is not forbidden; leaving row is the lowest-indexed row achieving
/// the minimum ratio. Both choices by index, which is what makes cycling impossible.
fn run_phase_masked(
    tableau: &mut [Vec<f64>],
    basis: &mut [usize],
    rows: usize,
    total_columns: usize,
    rhs: usize,
    forbidden: &[bool],
    iterations: &mut usize,
) -> LpStatus {
    loop {
        if *iterations >= MAX_ITERATIONS {
            return LpStatus::IterationLimit;
        }

        let entering = (0..total_columns)
            .find(|column| !forbidden[*column] && tableau[rows][*column] < -EPSILON);
        let Some(entering) = entering else {
            return LpStatus::Optimal;
        };

        let mut leaving: Option<usize> = None;
        let mut best_ratio = f64::INFINITY;
        for (row, line) in tableau.iter().enumerate().take(rows) {
            let coefficient = line[entering];
            if coefficient > EPSILON {
                let ratio = line[rhs] / coefficient;
                if ratio < best_ratio - EPSILON {
                    best_ratio = ratio;
                    leaving = Some(row);
                }
            }
        }

        let Some(leaving) = leaving else {
            return LpStatus::Unbounded;
        };

        pivot(tableau, leaving, entering, rows, rhs);
        basis[leaving] = entering;
        *iterations += 1;
    }
}

/// Gauss–Jordan elimination about the pivot, applied to the constraints **and** the
/// objective row, which is what keeps the reduced costs consistent without recomputing
/// them from the basis each iteration.
fn pivot(
    tableau: &mut [Vec<f64>],
    pivot_row: usize,
    pivot_column: usize,
    _rows: usize,
    rhs: usize,
) {
    let pivot_value = tableau[pivot_row][pivot_column];
    debug_assert!(pivot_value.abs() > EPSILON, "pivot on a zero element");
    for value in tableau[pivot_row].iter_mut().take(rhs + 1) {
        *value /= pivot_value;
    }
    // The normalized pivot row is copied out so the other rows can be updated without
    // holding two borrows of the tableau at once.
    let pivot_values: Vec<f64> = tableau[pivot_row][..=rhs].to_vec();
    for (row, line) in tableau.iter_mut().enumerate() {
        if row == pivot_row {
            continue;
        }
        let factor = line[pivot_column];
        if factor.abs() < EPSILON {
            continue;
        }
        for (value, pivot_entry) in line.iter_mut().zip(pivot_values.iter()) {
            *value -= factor * pivot_entry;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64, tolerance: f64) -> bool {
        (a - b).abs() < tolerance
    }

    /// The objective and the vertex have to match hand-computed values, not the
    /// solver's own output. A tableau with an inverted sign convention still produces
    /// self-consistent arithmetic, so a test that only checked the arithmetic would
    /// pass on a solver that returns every answer negated.
    ///
    /// The first version of this test asserted the optimum of a different problem: it
    /// checked the *intersection* vertex without asking whether a better corner existed,
    /// and the solver was right to keep pivoting past it. The constraints below put the
    /// best vertex at a genuine intersection of two binding constraints, so the test is
    /// about the solver rather than about which corner happens to be best.
    #[test]
    fn a_two_variable_lp_matches_its_hand_solution() {
        // maximise 3x + 2y  ==  minimise -3x - 2y
        //   x + y <= 4
        //  2x + y <= 6
        //   x, y >= 0
        // Vertices: (0,0)=0, (3,0)=9, (2,2)=10, (0,4)=8. The optimum is (2,2), where
        // both constraints bind, giving 3*2 + 2*2 = 10.
        let solution = solve(
            &[-3.0, -2.0],
            &[vec![1.0, 1.0], vec![2.0, 1.0]],
            &[4.0, 6.0],
            &[],
            &[],
        )
        .expect("well-formed");
        assert_eq!(solution.status, LpStatus::Optimal);
        assert!(close(solution.objective, -10.0, 1e-9), "{:?}", solution);
        assert!(close(solution.x[0], 2.0, 1e-9), "{:?}", solution.x);
        assert!(close(solution.x[1], 2.0, 1e-9), "{:?}", solution.x);
    }

    /// The shadow price has to be the derivative it claims to be. Checked by actually
    /// perturbing the right-hand side and re-solving: a dual that is off by a sign or a
    /// factor is the single easiest thing to get wrong and the hardest to notice, since
    /// the primal answer looks perfect either way.
    #[test]
    fn dual_values_are_the_derivative_of_the_optimum() {
        // minimise -x - y subject to x <= 2, x + y <= 3.
        let base = solve(
            &[-1.0, -1.0],
            &[vec![1.0, 0.0], vec![1.0, 1.0]],
            &[2.0, 3.0],
            &[],
            &[],
        )
        .expect("well-formed");
        assert_eq!(base.status, LpStatus::Optimal);
        assert!(close(base.objective, -3.0, 1e-9), "{:?}", base);

        // Relaxing the binding constraint by one unit improves the minimum from -3 to
        // -4, so the derivative is -1 and the dual must equal the observed change rather
        // than its negation.
        let perturbed = solve(
            &[-1.0, -1.0],
            &[vec![1.0, 0.0], vec![1.0, 1.0]],
            &[2.0, 4.0],
            &[],
            &[],
        )
        .expect("well-formed");
        let observed = perturbed.objective - base.objective;
        assert!(
            close(observed, base.dual_ub[1], 1e-6),
            "observed change {observed}, dual {}",
            base.dual_ub[1]
        );
        assert!(
            close(base.dual_ub[0], 0.0, 1e-9),
            "a non-binding constraint must have a zero shadow price, got {}",
            base.dual_ub[0]
        );
    }

    /// An equality constraint's dual is the marginal value of the right-hand side, and
    /// in the withdrawal model that number *is* the answer: the value of one more franc
    /// to allocate. Checked by perturbation for the same reason as above.
    #[test]
    fn equality_duals_are_the_marginal_value_of_the_requirement() {
        // minimise x + 2y subject to x + y = 3, x <= 2.
        // Cheapest is to use as much x as allowed: x = 2, y = 1, cost 4.
        let solution = solve(&[1.0, 2.0], &[vec![1.0, 0.0]], &[2.0], &[vec![1.0, 1.0]], &[3.0])
            .expect("well-formed");
        assert_eq!(solution.status, LpStatus::Optimal);
        assert!(close(solution.objective, 4.0, 1e-9), "{:?}", solution);

        let raised = solve(&[1.0, 2.0], &[vec![1.0, 0.0]], &[2.0], &[vec![1.0, 1.0]], &[4.0])
            .expect("well-formed");
        let observed = raised.objective - solution.objective;
        assert!(
            close(observed, solution.dual_eq[0], 1e-6),
            "one more unit cost {observed}, dual says {}",
            solution.dual_eq[0]
        );
    }

    /// Infeasibility must be reported as infeasibility. A solver that returned a
    /// feasible-looking vertex for an impossible problem would be worse than one that
    /// crashed, because the caller would believe it.
    #[test]
    fn an_impossible_problem_is_reported_as_infeasible() {
        // x >= 5 written as -x <= -5 is refused (negative right-hand side), so the
        // infeasible case is built from two constraints that cannot both hold.
        let solution = solve(
            &[1.0],
            &[vec![1.0], vec![-1.0]],
            &[1.0, -3.0],
            &[],
            &[],
        );
        // -3 on the right-hand side of a <= row is refused up front.
        assert!(matches!(solution, Err(LpError::NegativeRhs { .. })));

        // With non-negative right-hand sides: x <= 1 and x >= 3 is x <= 1 and -x <= -3,
        // so the infeasibility has to come through an equality instead.
        let solution = solve(&[1.0], &[], &[], &[vec![1.0]], &[2.0]).expect("well-formed");
        assert_eq!(solution.status, LpStatus::Optimal);

        // x + 0*y = 2 with x <= 1: feasible set empty.
        let solution = solve(&[1.0], &[vec![1.0]], &[1.0], &[vec![1.0]], &[2.0])
            .expect("well-formed");
        assert_eq!(solution.status, LpStatus::Infeasible, "{:?}", solution);
    }

    /// Unboundedness must be reported, not returned as a large number. The early
    /// retirement model would otherwise read an unbounded withdrawal programme as a
    /// very profitable one.
    #[test]
    fn an_unbounded_problem_is_reported_as_unbounded() {
        // minimise -x subject to nothing but x >= 0.
        let solution = solve(&[-1.0], &[], &[], &[], &[]).expect("well-formed");
        assert_eq!(solution.status, LpStatus::Unbounded, "{:?}", solution);
    }

    /// A malformed problem is a different thing from an infeasible one, and conflating
    /// them would send a caller looking for a modelling error in the wrong place.
    #[test]
    fn malformed_problems_are_rejected_before_any_pivoting() {
        assert!(matches!(
            solve(&[1.0, 1.0], &[vec![1.0]], &[1.0], &[], &[]),
            Err(LpError::RaggedRow { .. })
        ));
        assert!(matches!(solve(&[], &[], &[], &[], &[]), Err(LpError::Empty)));
        assert!(matches!(
            solve(&[1.0], &[vec![1.0]], &[-1.0], &[], &[]),
            Err(LpError::NegativeRhs { row: 0 })
        ));
    }

    /// Degeneracy is where simplex implementations break: a naive pivot rule can cycle
    /// forever on a problem whose optimum is perfectly well defined. This is Beale's
    /// example, the classic cycling case, and Bland's rule is what gets through it. The
    /// test asserts termination and the known optimum, because a solver that hung here
    /// would be caught by neither a timeout nor a wrong-answer check in normal use.
    #[test]
    fn a_degenerate_problem_terminates_at_the_right_optimum() {
        // minimise -0.75 x4 + 150 x5 - 0.02 x6 + 6 x7
        //   0.25 x4 - 60 x5 - 0.04 x6 + 9 x7 <= 0
        //   0.5  x4 - 90 x5 - 0.02 x6 + 3 x7 <= 0
        //   x6 <= 1
        // Optimum is -0.05 at x = (0, 0, 1/3, 1, 0, 1, 0) in the original labelling;
        // the value is what matters here.
        let solution = solve(
            &[0.0, 0.0, 0.0, -0.75, 150.0, -0.02, 6.0],
            &[
                vec![0.0, 0.0, 0.0, 0.25, -60.0, -0.04, 9.0],
                vec![0.0, 0.0, 0.0, 0.5, -90.0, -0.02, 3.0],
                vec![0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0],
            ],
            &[0.0, 0.0, 1.0],
            &[],
            &[],
        )
        .expect("well-formed");
        assert_eq!(
            solution.status,
            LpStatus::Optimal,
            "Bland's rule must terminate on the classic cycling example"
        );
        assert!(
            close(solution.objective, -0.05, 1e-6),
            "objective {} is not the known optimum -0.05",
            solution.objective
        );
        assert!(
            solution.iterations < MAX_ITERATIONS,
            "the cap was hit, so the rule did not converge"
        );
    }

    /// The solver is used to allocate a total across years, so the archetypal problem it
    /// must get right is: split a fixed sum across buckets with different costs, fill the
    /// cheap bucket first. That greedy answer is known in closed form, which makes it an
    /// independent check on the whole pipeline.
    #[test]
    fn filling_the_cheapest_bucket_first_is_reproduced() {
        // x1 costs 1 per unit up to 3, x2 costs 2 up to 3, x3 costs 5 up to 3.
        // Total to place: 7. Cheapest-first: 3 + 3 + 1, cost 3*1 + 3*2 + 1*5 = 14.
        let solution = solve(
            &[1.0, 2.0, 5.0],
            &[vec![1.0, 0.0, 0.0], vec![0.0, 1.0, 0.0], vec![0.0, 0.0, 1.0]],
            &[3.0, 3.0, 3.0],
            &[vec![1.0, 1.0, 1.0]],
            &[7.0],
        )
        .expect("well-formed");
        assert_eq!(solution.status, LpStatus::Optimal);
        assert!(close(solution.objective, 14.0, 1e-9), "{:?}", solution);
        assert!(close(solution.x[0], 3.0, 1e-9), "{:?}", solution.x);
        assert!(close(solution.x[1], 3.0, 1e-9), "{:?}", solution.x);
        assert!(close(solution.x[2], 1.0, 1e-9), "{:?}", solution.x);
    }
}

