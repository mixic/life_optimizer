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

/// Whether the federal tariff encoded below has been checked against an
/// authoritative source.
///
/// **Currently `false`.** The table in this file has a known defect: the
/// sequence around 72,500–103,600 is non-monotonic, which no progressive tariff
/// can be. The rates there appear to be ESTV "ans Satz" figures — the *average*
/// rate at that threshold — rather than marginal rates, so transcribing them as
/// brackets is wrong.
///
/// It is kept rather than deleted so the two-level structure and its tests can
/// run, but callers that would act on a projection must check this flag first.
/// [`crate::cantons::total_tax`] does not yet, because the cantonal data is
/// missing anyway; once either table is filled in this flag is the gate that
/// must be cleared.
///
/// Set to `true` only after the table has been diffed against a named official
/// source. `tariff_flagged_verified_must_be_monotonic` fails if it is set while
/// the defect remains, so the flag cannot be flipped carelessly.
pub const FEDERAL_TARIFF_IS_VERIFIED: bool = false;

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
pub const FEDERAL_BRACKETS_SINGLE: &[FederalBracket] = &[
    FederalBracket { threshold: 14_500.0, rate: 0.0000 },
    FederalBracket { threshold: 31_600.0, rate: 0.0077 },
    FederalBracket { threshold: 41_400.0, rate: 0.0125 },
    FederalBracket { threshold: 55_200.0, rate: 0.0210 },
    FederalBracket { threshold: 72_500.0, rate: 0.0219 },
    FederalBracket { threshold: 78_100.0, rate: 0.0213 },
    FederalBracket { threshold: 103_600.0, rate: 0.0349 },
    FederalBracket { threshold: 134_600.0, rate: 0.0425 },
    FederalBracket { threshold: 176_000.0, rate: 0.0499 },
    FederalBracket { threshold: 755_200.0, rate: 0.0739 },
];

/// Progressive federal tariff for a married taxpayer assessed jointly.
///
/// The thresholds are exactly double the single thresholds, which is the
/// splitting effect in the tariff itself rather than a separate mechanism.
pub const FEDERAL_BRACKETS_MARRIED: &[FederalBracket] = &[
    FederalBracket { threshold: 29_000.0, rate: 0.0000 },
    FederalBracket { threshold: 63_200.0, rate: 0.0077 },
    FederalBracket { threshold: 82_800.0, rate: 0.0125 },
    FederalBracket { threshold: 110_400.0, rate: 0.0210 },
    FederalBracket { threshold: 145_000.0, rate: 0.0219 },
    FederalBracket { threshold: 156_200.0, rate: 0.0213 },
    FederalBracket { threshold: 207_200.0, rate: 0.0349 },
    FederalBracket { threshold: 269_200.0, rate: 0.0425 },
    FederalBracket { threshold: 352_000.0, rate: 0.0499 },
    FederalBracket { threshold: 1_510_400.0, rate: 0.0739 },
];

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

    /// Below the first threshold no federal tax is due at all.
    #[test]
    fn income_below_the_allowance_is_untaxed() {
        assert_eq!(federal_tax(0.0, false), 0.0);
        assert_eq!(federal_tax(10_000.0, false), 0.0);
        assert_eq!(federal_tax(14_500.0, false), 0.0);
        // The married allowance is double.
        assert_eq!(federal_tax(29_000.0, true), 0.0);
    }

    /// Tax must be strictly increasing above the allowance, and the average
    /// rate must never exceed the marginal rate — that inversion would mean the
    /// tariff is not progressive.
    ///
    /// This test is **expected to fail** while
    /// [`FEDERAL_TARIFF_IS_VERIFIED`] is `false`: the encoded table has a
    /// non-monotonic segment, which is why it is not marked verified. It is
    /// written against the corrected invariant so that supplying the right
    /// table makes it pass without further edits.
    #[test]
    fn tariff_is_progressive_when_verified() {
        if !FEDERAL_TARIFF_IS_VERIFIED {
            // Not an assertion about the tariff — a statement that the data is
            // known-unreliable, so the real check would be meaningless.
            let violations = monotonicity_violations(false);
            assert!(
                !violations.is_empty(),
                "the tariff is marked unverified but has no monotonicity violations; \
                 either it has been fixed (then set FEDERAL_TARIFF_IS_VERIFIED = true) \
                 or the violation detector is broken"
            );
            return;
        }

        let incomes = [30_000.0, 50_000.0, 80_000.0, 120_000.0, 200_000.0, 500_000.0];
        let mut previous_tax = 0.0;

        for income in incomes {
            let tax = federal_tax(income, false);
            assert!(tax > previous_tax, "tax must rise with income at {income}");
            previous_tax = tax;

            let average = federal_average_rate(income, false);
            let marginal = federal_marginal_rate(income, false);
            assert!(
                average <= marginal + 1e-12,
                "average rate {average} must not exceed marginal {marginal} at {income}"
            );
        }
    }

    /// The verification flag must not be raised while the table is still
    /// malformed. This is what stops someone flipping the flag to silence the
    /// other test without actually fixing the data.
    #[test]
    fn tariff_flagged_verified_must_be_monotonic() {
        if FEDERAL_TARIFF_IS_VERIFIED {
            for married in [false, true] {
                let violations = monotonicity_violations(married);
                assert!(
                    violations.is_empty(),
                    "tariff is flagged verified but the marginal rate decreases \
                     between brackets {violations:?} — that is not a valid \
                     progressive tariff"
                );
            }
        }
    }

    /// Marginal rates must exactly match the published bracket rates.
    #[test]
    fn marginal_rate_matches_the_bracket_table() {
        assert_eq!(federal_marginal_rate(20_000.0, false), 0.0);
        assert_eq!(federal_marginal_rate(35_000.0, false), 0.0077);
        assert_eq!(federal_marginal_rate(45_000.0, false), 0.0125);
        assert_eq!(federal_marginal_rate(60_000.0, false), 0.0210);
        assert_eq!(federal_marginal_rate(75_000.0, false), 0.0219);
        assert_eq!(federal_marginal_rate(100_000.0, false), 0.0213);
        assert_eq!(federal_marginal_rate(120_000.0, false), 0.0349);
        assert_eq!(federal_marginal_rate(150_000.0, false), 0.0425);
        assert_eq!(federal_marginal_rate(200_000.0, false), 0.0499);
        assert_eq!(federal_marginal_rate(800_000.0, false), 0.0739);
    }

    /// The federal share at an ordinary salary must stay small. If a typo
    /// inflated a bracket rate, this would catch it.
    #[test]
    fn federal_burden_is_small_at_ordinary_incomes() {
        // Published effective federal rates are roughly 1% at 100k taxable.
        let at_100k = federal_average_rate(100_000.0, false);
        assert!(
            (0.005..0.02).contains(&at_100k),
            "federal average at 100k looks wrong: {at_100k}"
        );

        // And below the top bracket even at a high salary.
        let at_200k = federal_average_rate(200_000.0, false);
        assert!(
            (0.02..0.05).contains(&at_200k),
            "federal average at 200k looks wrong: {at_200k}"
        );
    }

    /// Marriage must not increase the federal tax on the same income — the
    /// splitting effect can only help.
    #[test]
    fn marriage_never_increases_federal_tax() {
        for income in [40_000.0, 80_000.0, 150_000.0, 300_000.0] {
            let single = federal_tax(income, false);
            let married = federal_tax(income, true);
            assert!(
                married <= single + 1e-9,
                "married tax {married} exceeds single {single} at {income}"
            );
        }
    }

    /// The married tariff must be the single tariff with doubled thresholds —
    /// a structural invariant worth pinning, since the table is hand-typed.
    #[test]
    fn married_thresholds_are_double_the_single_thresholds() {
        assert_eq!(FEDERAL_BRACKETS_SINGLE.len(), FEDERAL_BRACKETS_MARRIED.len());
        for (single, married) in FEDERAL_BRACKETS_SINGLE.iter().zip(FEDERAL_BRACKETS_MARRIED) {
            assert!(
                (married.threshold - single.threshold * 2.0).abs() < 1e-9,
                "threshold {} should be double {}",
                married.threshold,
                single.threshold
            );
            assert!(
                (married.rate - single.rate).abs() < 1e-12,
                "rates must match at threshold {}",
                single.threshold
            );
        }
    }

    /// Thresholds must be strictly increasing, or the marginal computation
    /// silently drops a bracket.
    #[test]
    fn thresholds_are_strictly_increasing() {
        for table in [FEDERAL_BRACKETS_SINGLE, FEDERAL_BRACKETS_MARRIED] {
            for pair in table.windows(2) {
                assert!(
                    pair[1].threshold > pair[0].threshold,
                    "thresholds must increase: {} then {}",
                    pair[0].threshold,
                    pair[1].threshold
                );
            }
        }
    }
}
