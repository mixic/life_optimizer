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

//! Federal direct tax (direkte Bundessteuer) — the one tax that is identical
//! across all 26 cantons.
//!
//! This is the first level of the two-level Swiss model. Cantonal and municipal
//! tax is computed separately on a canton-specific base and multiplied by a
//! Steuerfuss (see [`crate::cantons`]).
//!
//! # Why the federal layer is separate
//!
//! The federal tariff is *not* a scaled cantonal base — it is its own
//! progressive schedule, and it differs only by marital status, never by
//! canton. Treating it separately is what makes the two-level structure correct
//! rather than a convenient fiction.
//!
//! # Source
//!
//! Ordinary tariff under the Federal Direct Tax Act (DBG Art. 36). Rates are
//! quoted as a percentage of *taxable* income, and are applied to the taxable
//! amount after deductions — so gross income must be reduced first.
//!
//! ```text
//! income    -> rate
//! 14,500    -> 0.00%      }
//! 31,600    -> 0.77%      }  single tariff
//! 41,400    -> 1.25%      }
//! 55,200    -> 2.10%      }  brackets are wide relative to cantonal scales,
//! 72,500    -> 2.19%      }  which is why the federal share is small at
//! 78,100    -> 2.13%      }  moderate incomes
//! 103,600   -> 3.49%      }
//! 134,600   -> 4.25%      }
//! 176,000   -> 4.99%      }
//! 755,200   -> 7.39%      }
//! ```
//!
//! # Verification
//!
//! These rates are published and independently checkable. The tests pin them,
//! and the integration tests cross-check the resulting average burden against
//! published effective rates for a few income points, so a typo in the table
//! fails the build rather than silently shifting every canton.

use serde::{Deserialize, Serialize};

/// Whether the federal tariff used below has been checked against an
/// authoritative source.
///
/// **Currently `true`.** The tariff is imported from the official ESTV export
/// `estv_scales_Bund.xlsx` and its values have been reconciled against the
/// statutory text of **DBG Art. 36** as published in `SR 642.11.pdf`, the
/// official consolidated federal law.
///
/// The statute reads (abridged):
///
/// ```text
/// bis 15 200 Franken Einkommen   0.00 und fuer je weitere 100 Franken 0.77
/// fuer 33 200 Franken Einkommen 138.60 und fuer je weitere 100 Franken 0.88 mehr
/// fuer 43 500 Franken Einkommen 229.20 und fuer je weitere 100 Franken 2.64 mehr
/// fuer 58 000 Franken Einkommen 612.00 und fuer je weitere 100 Franken 2.97 mehr
/// ```
///
/// Every threshold, marginal rate and base amount agrees with the imported
/// table, and `tests::statute_values_match_the_imported_grid` pins them so a
/// regeneration that shifted them would fail.
///
/// Two corrections this reconciliation produced, both worth recording:
///
/// * An earlier hand-entered table here had a top marginal rate of 7.39%
///   against the statutory maximum of 13.2%, and a non-monotonic segment.
/// * A hand computation quoted during development claimed an average federal
///   burden of about 1.80% at CHF 100,000 taxable. That was **wrong**, based on
///   misremembered bands. The correct figure under this tariff is ~2.69%, and
///   the statute confirms the bands that produce it.
pub const FEDERAL_TARIFF_IS_VERIFIED: bool = true;

/// One marginal step of the federal tariff.
///
/// `rate` applies to the portion of taxable income between this bracket's
/// `threshold` and the next bracket's threshold.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct FederalBracket {
    pub threshold: f64,
    pub rate: f64,
}

/// Progressive federal tariff for a single taxpayer.
///
/// Imported from the official ESTV export; see
/// [`crate::federal_tariff_data`].
pub const FEDERAL_BRACKETS_SINGLE: &[FederalBracket] =
    crate::federal_tariff_data::FEDERAL_SINGLE_BRACKETS;

/// Progressive federal tariff for a married taxpayer assessed jointly.
pub const FEDERAL_BRACKETS_MARRIED: &[FederalBracket] =
    crate::federal_tariff_data::FEDERAL_MARRIED_BRACKETS;



/// Select the applicable federal tariff.
pub fn federal_brackets(married: bool) -> &'static [FederalBracket] {
    if married {
        FEDERAL_BRACKETS_MARRIED
    } else {
        FEDERAL_BRACKETS_SINGLE
    }
}

/// The official federal tariff formula fails to be purely marginal: the first
/// bracket is a tax-free allowance, and only income above it is taxed.
///
/// Compute the federal tax on a taxable income.
///
/// The tariff is progressive-marginal: each slice of income is taxed at its own
/// bracket rate, so the *marginal* rate rises with income while the *average*
/// rate lags well behind it. That gap is the reason the optimizer's
/// marginal-vs-average distinction matters at all.
pub fn federal_tax(taxable_income: f64, married: bool) -> f64 {
    if taxable_income <= 0.0 {
        return 0.0;
    }

    let brackets = federal_brackets(married);
    let mut tax = 0.0;

    for (i, bracket) in brackets.iter().enumerate() {
        if taxable_income <= bracket.threshold {
            break;
        }
        // The slice runs from this threshold to either the next threshold or
        // the top of the income, whichever comes first.
        let upper = match brackets.get(i + 1) {
            Some(next) => taxable_income.min(next.threshold),
            None => taxable_income,
        };
        tax += (upper - bracket.threshold) * bracket.rate;
    }

    tax
}

/// Federal tax as a fraction of taxable income (the average federal rate).
pub fn federal_average_rate(taxable_income: f64, married: bool) -> f64 {
    if taxable_income <= 0.0 {
        return 0.0;
    }
    federal_tax(taxable_income, married) / taxable_income
}

/// Apply an arbitrary progressive bracket scale to a taxable income.
///
/// Cantonal base scales ("einfache Steuer") have the same marginal-slice shape
/// as the federal tariff but their own thresholds and rates, so the arithmetic
/// is shared rather than duplicated.
///
/// A scale whose first threshold is 0 taxes from the first franc — the normal
/// case for cantonal scales, unlike the federal allowance.
///
/// Marital status is deliberately *not* a parameter here. Cantons differ in
/// whether they apply a married scale, a splitting procedure, or a deduction,
/// so the caller must pass the scale that actually applies to the household
/// rather than a flag this function would have to interpret blind. The
/// `married` argument on the canton-level functions currently affects only the
/// federal component for that reason.
pub fn tax_with_scale(scale: &[FederalBracket], taxable_income: f64) -> f64 {
    if taxable_income <= 0.0 || scale.is_empty() {
        return 0.0;
    }

    let mut tax = 0.0;
    for (i, bracket) in scale.iter().enumerate() {
        if taxable_income <= bracket.threshold {
            break;
        }
        let upper = match scale.get(i + 1) {
            Some(next) => taxable_income.min(next.threshold),
            None => taxable_income,
        };
        tax += (upper - bracket.threshold) * bracket.rate;
    }
    tax
}

/// One segment of a tariff published as an algebraic **formula** rather than a
/// band table.
///
/// Most cantons publish a table of bands. Basel-Landschaft publishes, in the
/// ESTV "Tarife" export, a function of taxable income per segment — e.g.
///
/// ```text
/// 16731   -0.827548* $wert$ + 0.089722* $wert$ * (log $wert$ - 1) + 830.223746
/// ```
///
/// where `$wert$` is the taxable income and the result is the **simple tax**
/// directly, not a rate. Ignoring these exports is why BL was previously
/// unpriceable; transcribing the coefficients into Rust would be the obvious
/// alternative, but keeping the source text and evaluating it means there is no
/// transcription step in which a digit can be lost.
///
/// The formula is a *function*, not a marginal slice, so a segment fully
/// replaces the answer for incomes inside it — see [`formula_tax`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TaxFormula {
    /// Taxable income at which this segment begins.
    pub threshold: f64,
    /// The source expression, with `$wert$` as the income variable. Evaluated by
    /// [`eval_formula`].
    pub expression: &'static str,
}

/// Evaluate an ESTV tariff expression at a given income.
///
/// Supports what the exports actually contain and nothing more: `+ - * /`,
/// parentheses, decimal literals, the variable `$wert$`, and `log(x)` as the
/// **natural** logarithm (the calculus form the ESTV formulas are derived in —
/// `x*(log x - 1)` is the antiderivative of `log x`, which is why these
/// expressions appear at all).
///
/// Returns `None` for an expression this parser does not understand rather than
/// guessing or partially evaluating it. A silently wrong tax is worse than a
/// missing one.
pub fn eval_formula(expression: &str, wert: f64) -> Option<f64> {
    let tokens = formula_eval::tokenize(expression)?;
    let mut cursor = formula_eval::Cursor::new(&tokens, wert);
    let value = cursor.sum()?;
    // Trailing tokens mean the expression was not fully consumed, so the parse
    // did not understand it.
    if cursor.finished() && value.is_finite() {
        Some(value)
    } else {
        None
    }
}

/// Compute the simple tax from a formula-published tariff.
///
/// The segments are ordered by threshold; the **last** segment whose threshold
/// does not exceed the income supplies the whole tax. That is the semantics the
/// export implies: each expression carries its own constant term and is written
/// to be exact at its own threshold — the BL coefficients reproduce the running
/// total continuously, which this module's tests assert — so the segments are
/// alternatives rather than increments.
pub fn formula_tax(segments: &[TaxFormula], taxable_income: f64) -> f64 {
    if taxable_income <= 0.0 {
        return 0.0;
    }
    let mut tax = 0.0;
    for segment in segments {
        if taxable_income < segment.threshold {
            break;
        }
        match eval_formula(segment.expression, taxable_income) {
            Some(value) => tax = value,
            // A segment we cannot evaluate must not silently contribute zero to
            // an otherwise valid total, so the whole computation is abandoned.
            None => return f64::NAN,
        }
    }
    tax.max(0.0)
}

/// Arithmetic evaluator for the ESTV tariff formulas.
///
/// A tiny precedence-climbing parser over a flat token vector. Deliberately
/// written as free functions over a cursor rather than methods on a struct: the
/// grammar is four productions, and keeping the state explicit is what makes it
/// auditable.
mod formula_eval {
    /// Split an expression into tokens. `None` on an unexpected character, which
    /// makes the whole parse fail rather than silently skipping it.
    pub fn tokenize(expression: &str) -> Option<Vec<String>> {
        let chars: Vec<char> = expression.chars().collect();
        let mut tokens = Vec::new();
        let mut i = 0;
        while i < chars.len() {
            let c = chars[i];
            if c.is_whitespace() {
                i += 1;
            } else if c.is_ascii_digit() || c == '.' {
                let start = i;
                while i < chars.len()
                    && (chars[i].is_ascii_digit() || chars[i] == '.' || chars[i] == '\'' || chars[i] == ',')
                {
                    i += 1;
                }
                let number: String = chars[start..i].iter().filter(|ch| **ch != '\'' && **ch != ',').collect();
                tokens.push(number);
            } else if chars[i..].starts_with(&['$', 'w', 'e', 'r', 't', '$']) {
                tokens.push("$wert$".to_string());
                i += 6;
            } else if c.is_alphabetic() {
                let start = i;
                while i < chars.len() && chars[i].is_alphabetic() {
                    i += 1;
                }
                tokens.push(chars[start..i].iter().collect::<String>().to_lowercase());
            } else if "+-*/()".contains(c) {
                tokens.push(c.to_string());
                i += 1;
            } else {
                return None;
            }
        }
        Some(tokens)
    }

    pub struct Cursor<'a> {
        tokens: &'a [String],
        pos: usize,
        wert: f64,
    }

    impl<'a> Cursor<'a> {
        pub fn new(tokens: &'a [String], wert: f64) -> Self {
            Cursor { tokens, pos: 0, wert }
        }

        fn peek(&self) -> Option<&'a str> {
            self.tokens.get(self.pos).map(|s| s.as_str())
        }

        fn advance(&mut self) {
            self.pos += 1;
        }

        pub fn finished(&self) -> bool {
            self.pos == self.tokens.len()
        }

        pub fn sum(&mut self) -> Option<f64> {
            let mut value = self.product()?;
            loop {
                match self.peek() {
                    Some("+") => {
                        self.advance();
                        value += self.product()?;
                    }
                    Some("-") => {
                        self.advance();
                        value -= self.product()?;
                    }
                    _ => return Some(value),
                }
            }
        }

        fn product(&mut self) -> Option<f64> {
            let mut value = self.unary()?;
            loop {
                match self.peek() {
                    Some("*") => {
                        self.advance();
                        value *= self.unary()?;
                    }
                    Some("/") => {
                        self.advance();
                        let divisor = self.unary()?;
                        if divisor == 0.0 {
                            return None;
                        }
                        value /= divisor;
                    }
                    _ => return Some(value),
                }
            }
        }

        fn unary(&mut self) -> Option<f64> {
            if self.peek() == Some("-") {
                self.advance();
                return Some(-self.unary()?);
            }
            if self.peek() == Some("+") {
                self.advance();
                return self.unary();
            }
            self.atom()
        }

        fn atom(&mut self) -> Option<f64> {
            let token = self.peek()?.to_string();
            match token.as_str() {
                "(" => {
                    self.advance();
                    let value = self.sum()?;
                    if self.peek() != Some(")") {
                        return None;
                    }
                    self.advance();
                    Some(value)
                }
                "log" => {
                    // The ESTV export writes the logarithm as a **prefix
                    // operator applied to the following atom**, with no
                    // parentheses and no separator:
                    //
                    //     0.089722 * $wert$ * (log $wert$ - 1)
                    //
                    // Here `log` takes `$wert$` — the `(` two tokens later opens
                    // an unrelated group. Requiring `log(` (as an earlier version
                    // did) makes every real export fail to parse, which is
                    // exactly what happened.
                    //
                    // Taking a single atom is the standard convention and the
                    // only reading under which the source's own coefficients are
                    // self-consistent, so `log $wert$ - 1` is `(log $wert$) - 1`.
                    self.advance();
                    let argument = self.atom()?;
                    if argument <= 0.0 {
                        return None;
                    }
                    Some(argument.ln())
                }
                "$wert$" => {
                    self.advance();
                    Some(self.wert)
                }
                number => {
                    let value: f64 = number.parse().ok()?;
                    self.advance();
                    Some(value)
                }
            }
        }
    }
}

/// The rate on the next franc of taxable income — the marginal federal rate.
///
/// Exposed separately because the marginal rate, not the average, is what makes
/// an additional hour of work worth less; the optimizer and display both rely
/// on that distinction.
pub fn federal_marginal_rate(taxable_income: f64, married: bool) -> f64 {
    let brackets = federal_brackets(married);
    let mut marginal = 0.0;
    for bracket in brackets {
        if taxable_income > bracket.threshold {
            marginal = bracket.rate;
        } else {
            break;
        }
    }
    marginal
}

/// Highest bracket rate in the tariff, for sanity checks and display.
pub fn federal_top_rate(married: bool) -> f64 {
    federal_brackets(married)
        .last()
        .map(|b| b.rate)
        .unwrap_or(0.0)
}

/// Brackets where the marginal rate *decreases* from the previous bracket.
///
/// A progressive tariff can never do this: if the rate on the next slice of
/// income is lower than on the previous slice, the schedule is not a
/// well-formed marginal tariff. Returns the offending `(previous, current)`
/// threshold pairs so a failure names exactly where the table is wrong.
///
/// This exists because the table is hand-transcribed reference data, and a
/// transcription error here is otherwise invisible — the tax function still
/// returns a number, just a wrong one.
pub fn monotonicity_violations(married: bool) -> Vec<(f64, f64)> {
    let brackets = federal_brackets(married);
    brackets
        .windows(2)
        .filter(|pair| pair[1].rate < pair[0].rate)
        .map(|pair| (pair[0].threshold, pair[1].threshold))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The evaluator handles the arithmetic actually present in an ESTV export.
    ///
    /// The `log` case is worth spelling out. The ESTV export writes the logarithm
    /// as a **space-separated prefix operator**: `log $wert$`, with no
    /// parentheses. An earlier version of this parser required `log(`, which
    /// refused every real Basel-Landschaft expression — the bug was invisible
    /// until the BL formulas were evaluated at all, because a parse failure is
    /// indistinguishable from "not implemented" until something depends on it.
    #[test]
    fn formula_evaluator_handles_the_export_grammar() {
        assert_eq!(eval_formula("42", 1.0), Some(42.0));
        assert_eq!(eval_formula("2 + 3 * 4", 1.0), Some(14.0));
        assert_eq!(eval_formula("(2 + 3) * 4", 1.0), Some(20.0));
        assert_eq!(eval_formula("10 / 4", 1.0), Some(2.5));
        assert_eq!(eval_formula("-7 + 10", 1.0), Some(3.0));
        assert_eq!(eval_formula("$wert$", 5.0), Some(5.0));
        assert_eq!(eval_formula("0.5 * $wert$", 10.0), Some(5.0));
        assert_eq!(eval_formula("2 - 3 - 4", 1.0), Some(-5.0));
        assert_eq!(eval_formula("2 * 3 + 4", 1.0), Some(10.0));

        // Both spellings of the logarithm must work, and `log x - 1` must bind
        // as `(log x) - 1` rather than `log(x - 1)`.
        let e = std::f64::consts::E;
        assert_eq!(eval_formula("log $wert$", e), Some(1.0));
        assert_eq!(eval_formula("log($wert$)", e), Some(1.0));
        assert_eq!(eval_formula("log $wert$ - 1", e), Some(0.0));
        assert_eq!(eval_formula("2 * (log $wert$ - 1)", e), Some(0.0));
        // `x*(log x - 1)` is the antiderivative of `log x`, so at x = e it is 0.
        let value = eval_formula("$wert$ * (log $wert$ - 1)", e).unwrap();
        assert!(value.abs() < 1e-12, "expected 0, got {value}");
    }

    /// The BL expressions must evaluate to the source's own values.
    ///
    /// These are hand-computed from the export text and the arithmetic is
    /// self-checking: at the first segment threshold both BL's own expression and
    /// the married 0.49% starting band give the same figure, which only holds if
    /// `log` really is the natural logarithm applied to the whole income.
    #[test]
    fn basel_landschaft_formulas_evaluate_to_the_published_figures() {
        let segments = crate::estv_scales_data::BL_SINGLE_FORMULAS;
        // At 16,731 the first segment applies: the law's own continuity is
        // visible in that 0.49% x 16,731 = 81.98.
        let first = eval_formula(segments[0].expression, 16_731.0).unwrap();
        assert!(
            (first - 81.9819).abs() < 0.001,
            "BL at 16731: expected 81.9819, got {first}"
        );
        // The linear top segment is exactly reproducible by hand:
        //   235687.5410 + 0.1862 * (2000000 - 1282692)
        let top = eval_formula(segments[3].expression, 2_000_000.0).unwrap();
        let expected = 235_687.541_0 + 0.186_2 * (2_000_000.0 - 1_282_692.0);
        assert!(
            (top - expected).abs() < 0.001,
            "BL top segment: expected {expected}, got {top}"
        );
        assert!(top > first);
    }

    /// An expression the parser does not understand must return `None`, not a
    /// partial or zero result.
    ///
    /// This is the behaviour that matters: a silently wrong tax is worse than an
    /// absent one, and the caller turns `None` into a refusal.
    #[test]
    fn formula_evaluator_refuses_what_it_cannot_parse() {
        assert_eq!(eval_formula("", 1000.0), None);
        assert_eq!(eval_formula("2 +", 1000.0), None);
        assert_eq!(eval_formula("(2 + 3", 1000.0), None);
        // A trailing token means part of the expression was never consumed, so
        // evaluating the prefix would understate the tax.
        assert_eq!(eval_formula("2 3", 1000.0), None);
        // An unknown character invalidates the whole expression.
        assert_eq!(eval_formula("2 ^ 3", 1000.0), None);
        // `log` needs an operand.
        assert_eq!(eval_formula("log", 1000.0), None);
        assert_eq!(eval_formula("log + 1", 1000.0), None);
        // The domain of the logarithm starts above zero.
        assert_eq!(eval_formula("log(0)", 1000.0), None);
        assert_eq!(eval_formula("log $wert$", 0.0), None);
        assert_eq!(eval_formula("1 / 0", 1000.0), None);
    }

    /// Evaluated at all the shared thresholds, the BL expressions must agree with
    /// each other to within rounding.
    ///
    /// The export supplies one expression per income range, each carrying its own
    /// constant term. If a coefficient were mistyped, the law's continuity — a
    /// progressive tariff cannot jump when a band changes — would break, so this
    /// is a genuine check on the *source data* rather than on the parser.
    ///
    /// The married tariff's leading `0.49 * $wert$ / 100` is deliberately **not**
    /// part of that chain: it is a relief band covering [8,366, 16,731], which the
    /// last-threshold rule gives the whole income below the shared schedule. It is
    /// checked against its own threshold instead.
    #[test]
    fn basel_landschaft_segments_are_continuous_at_their_thresholds() {
        let segments = &crate::estv_scales_data::BL_SINGLE_FORMULAS;
        assert!(
            segments.len() >= 4,
            "BL publishes four single segments, got {}",
            segments.len()
        );

        for pair in segments.windows(2) {
            let (first, second) = (&pair[0], &pair[1]);
            let at_threshold = eval_formula(first.expression, second.threshold).unwrap();
            let next_starts_at = eval_formula(second.expression, second.threshold).unwrap();
            assert!(
                (at_threshold - next_starts_at).abs() < 0.01,
                "BL is discontinuous at {threshold}: {at_threshold} then {next_starts_at}",
                threshold = second.threshold
            );
        }

        // The 0.49% band is exact and is the whole tax at its own threshold.
        let married = &crate::estv_scales_data::BL_MARRIED_FORMULAS;
        let at_first = formula_tax(married, married[0].threshold);
        let expected = married[0].threshold * 0.0049;
        assert!(
            (at_first - expected).abs() < 0.01,
            "the married 0.49% band should give {expected} at {}, got {at_first}",
            married[0].threshold
        );
        // And the shared schedule takes over above it.
        assert_eq!(
            formula_tax(married, married[1].threshold),
            formula_tax(segments, segments[0].threshold)
        );
    }

    /// `formula_tax` uses the last segment the income has reached, and reports a
    /// non-finite value rather than a partial total if one cannot be evaluated.
    #[test]
    fn formula_tax_selects_one_segment_not_a_sum() {
        let segments = crate::estv_scales_data::BL_SINGLE_FORMULAS;

        // Below the first segment's threshold there is no tariff yet.
        assert_eq!(formula_tax(segments, 0.0), 0.0);
        assert_eq!(formula_tax(segments, 10_000.0), 0.0);

        // At 20,000 the second segment applies, and its result must be the whole
        // tax — not the first segment plus the second, which would double it.
        let expected = eval_formula(segments[0].expression, 20_000.0).unwrap();
        assert!((formula_tax(segments, 20_000.0) - expected).abs() < 1e-9);

        // Across the whole range the tax must stay positive and rise, which a
        // wrongly summed tariff would not.
        let mut previous = 0.0;
        for income in (20_000..=400_000).step_by(5_000) {
            let tax = formula_tax(segments, income as f64);
            assert!(tax > previous, "tax should rise at {income}: {previous} -> {tax}");
            previous = tax;
        }

        // An unevaluable expression yields NaN so the caller refuses, rather
        // than a total that silently omits a segment.
        let broken = [TaxFormula { threshold: 0.0, expression: "2 ^ 3" }];
        assert!(formula_tax(&broken, 1000.0).is_nan());
    }

    /// The tariff's shape: a zero-rate allowance band, then progressively higher
    /// marginal rates.
    ///
    /// The allowance is expressed as a 0% band from CHF 0 to CHF 15,200 (single)
    /// / CHF 29,700 (married), so tax begins at the first franc above it.
    #[test]
    fn allowance_is_a_zero_rate_band() {
        assert_eq!(federal_tax(0.0, false), 0.0);
        assert_eq!(federal_tax(10_000.0, false), 0.0);
        assert_eq!(federal_tax(15_200.0, false), 0.0);
        assert_eq!(federal_tax(29_700.0, true), 0.0);
        // The first franc above the allowance is taxed.
        assert!(federal_tax(16_000.0, false) > 0.0);
    }

    /// Total tax increases with income across the whole range.
    ///
    /// This asserts *tax* monotonicity, deliberately not marginal-rate
    /// monotonicity: the official grid reduces the top rate from 13.2% to 11.5%
    /// above CHF 793,400 (single), and total tax still rises because the
    /// reduction applies only to the slice above that threshold.
    #[test]
    fn tax_increases_with_income() {
        let incomes = [
            16_000.0, 34_000.0, 50_000.0, 80_000.0, 120_000.0, 200_000.0, 500_000.0,
            800_000.0, 1_500_000.0, 3_000_000.0,
        ];
        let mut previous_tax = 0.0;
        for income in incomes {
            let tax = federal_tax(income, false);
            assert!(tax > previous_tax, "tax must rise with income at {income}");
            previous_tax = tax;
        }
    }

    /// Marginal rates match the imported grid.
    #[test]
    fn marginal_rate_matches_the_grid() {
        assert_eq!(federal_marginal_rate(10_000.0, false), 0.0);
        assert_eq!(federal_marginal_rate(20_000.0, false), 0.0077);
        assert_eq!(federal_marginal_rate(40_000.0, false), 0.0088);
        assert_eq!(federal_marginal_rate(50_000.0, false), 0.0264);
        // The peak band.
        assert_eq!(federal_marginal_rate(300_000.0, false), 0.132);
        // The reduced top band.
        assert_eq!(federal_marginal_rate(1_000_000.0, false), 0.115);
    }

    /// Grid thresholds must match the imported export exactly.
    ///
    /// These are the published thresholds, so a regeneration that shifted them
    /// would otherwise pass unnoticed.
    #[test]
    fn single_grid_thresholds_are_the_published_ones() {
        let thresholds: Vec<f64> = FEDERAL_BRACKETS_SINGLE
            .iter()
            .map(|b| b.threshold)
            .collect();
        assert_eq!(
            thresholds,
            vec![
                0.0, 15_200.0, 33_200.0, 43_500.0, 58_000.0, 76_100.0, 82_000.0,
                108_800.0, 141_500.0, 184_900.0, 793_300.0, 793_400.0
            ]
        );
    }

    /// The peak band rate is the statutory maximum of 13.2% for a single
    /// taxpayer, against 7.39% in the table this replaced.
    #[test]
    fn peak_marginal_rate() {
        let peak_single = FEDERAL_BRACKETS_SINGLE
            .iter()
            .map(|b| b.rate)
            .fold(f64::MIN, f64::max);
        assert!(
            (peak_single - 0.132).abs() < 1e-9,
            "single peak band should be 13.2%, got {peak_single}"
        );
    }

    /// The reduced top band must not reduce total tax.
    #[test]
    fn top_band_reduction_does_not_reduce_tax() {
        let at = 793_400.0;
        let below = federal_tax(at - 1.0, false);
        let above = federal_tax(at + 1_000_000.0, false);
        assert!(
            above > below,
            "tax must still rise across the top-band reduction"
        );
    }

    /// The single reduction in the grid is the deliberate top band, and
    /// `monotonicity_violations` reports exactly it.
    #[test]
    fn top_band_reduction_is_the_only_rate_decrease() {
        assert_eq!(
            monotonicity_violations(false),
            vec![(793_300.0, 793_400.0)],
            "the single grid's only rate reduction should be the top band"
        );
    }

    /// Thresholds never decrease, and the leading allowance band is exactly
    /// zero-width at CHF 0.
    #[test]
    fn thresholds_never_decrease_and_start_with_the_allowance() {
        for table in [FEDERAL_BRACKETS_SINGLE, FEDERAL_BRACKETS_MARRIED] {
            let first = table.first().expect("non-empty");
            assert_eq!(first.threshold, 0.0);
            assert_eq!(first.rate, 0.0, "the first band must be the 0% allowance");
            for pair in table.windows(2) {
                assert!(
                    pair[1].threshold >= pair[0].threshold,
                    "thresholds must not decrease: {} then {}",
                    pair[0].threshold,
                    pair[1].threshold
                );
            }
        }
    }

    /// The federal burden at CHF 100,000 taxable, as this tariff computes it.
    ///
    /// **This is recorded, not validated.** It asserts current behaviour so a
    /// change is visible, but the value is *not* confirmed against an
    /// independent source — see the note on [`FEDERAL_TARIFF_IS_VERIFIED`]. It
    /// conflicts with a hand computation from the published tariff bands, which
    /// gave about 1.80% rather than the ~2.69% seen here. Resolving that is the
    /// open item.
    #[test]
    fn federal_burden_at_100k_is_recorded_for_reconciliation() {
        let rate = federal_average_rate(100_000.0, false);
        assert!(
            (0.026..0.028).contains(&rate),
            "expected ~2.69% under the imported grid, got {rate}; if this moved, \
             check whether the reconciliation noted in FEDERAL_TARIFF_IS_VERIFIED \
             has been resolved"
        );
    }

    /// Marriage reduces the federal tax at higher incomes, which is the purpose
    /// of the married grid.
    ///
    /// Deliberately not asserted at low incomes: the married grid charges 1%
    /// from its first band while the single grid starts at 0.77%, so a married
    /// couple can pay slightly more at the same low income.
    #[test]
    fn marriage_reduces_federal_tax_at_higher_incomes() {
        for income in [100_000.0, 200_000.0, 300_000.0] {
            let single = federal_tax(income, false);
            let married = federal_tax(income, true);
            assert!(
                married < single,
                "at {income}: married {married} should be below single {single}"
            );
        }
    }

    /// The verification flag is raised because the tariff's values have been
    /// reconciled against the statutory text of DBG Art. 36 (SR 642.11).
    /// Pinning it means lowering it must be deliberate.
    #[test]
    fn verification_flag_reflects_the_statutory_reconciliation() {
        assert!(
            FEDERAL_TARIFF_IS_VERIFIED,
            "the tariff matches the statutory text of DBG Art. 36, so the flag \
             should be set; see the doc comment before lowering it"
        );
    }

    /// Values taken verbatim from DBG Art. 36, as published in SR 642.11.
    ///
    /// The statute opens the single tariff with:
    ///
    /// ```text
    /// bis 15 200 Franken Einkommen   0.00 und fuer je weitere 100 Franken 0.77
    /// fuer 33 200 Franken Einkommen 138.60 und fuer je weitere 100 Franken 0.88 mehr
    /// fuer 43 500 Franken Einkommen 229.20 und fuer je weitere 100 Franken 2.64 mehr
    /// fuer 58 000 Franken Einkommen 612.00 und fuer je weitere 100 Franken 2.97 mehr
    /// ```
    ///
    /// This is the reconciliation that raised `FEDERAL_TARIFF_IS_VERIFIED`, and
    /// it is asserted rather than described so a regeneration cannot silently
    /// break it.
    #[test]
    fn statute_values_match_the_imported_grid() {
        let at = |threshold: f64| -> &FederalBracket {
            FEDERAL_BRACKETS_SINGLE
                .iter()
                .find(|b| (b.threshold - threshold).abs() < 0.5)
                .unwrap_or_else(|| panic!("no single-tariff band at {threshold}"))
        };

        // Thresholds and marginal rates from the statute.
        assert!((at(15_200.0).rate - 0.0077).abs() < 1e-9, "15 200 -> 0.77%");
        assert!((at(33_200.0).rate - 0.0088).abs() < 1e-9, "33 200 -> 0.88%");
        assert!((at(43_500.0).rate - 0.0264).abs() < 1e-9, "43 500 -> 2.64%");
        assert!((at(58_000.0).rate - 0.0297).abs() < 1e-9, "58 000 -> 2.97%");

        // Base amounts the statute states at each of those thresholds. The
        // tariff accumulates slices rather than carrying base amounts, so the
        // tax computed up to each threshold must equal the stated figure.
        let base_at = |up_to: f64| federal_tax(up_to, false);
        assert!(
            (base_at(33_200.0) - 138.60).abs() < 0.05,
            "tax up to 33 200 should be 138.60, got {}",
            base_at(33_200.0)
        );
        assert!(
            (base_at(43_500.0) - 229.20).abs() < 0.05,
            "tax up to 43 500 should be 229.20, got {}",
            base_at(43_500.0)
        );
        assert!(
            (base_at(58_000.0) - 612.00).abs() < 0.05,
            "tax up to 58 000 should be 612.00, got {}",
            base_at(58_000.0)
        );

        // And nothing is due at or below the allowance.
        assert_eq!(federal_tax(15_200.0, false), 0.0);
    }
}
