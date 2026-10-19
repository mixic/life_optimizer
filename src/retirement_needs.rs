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

//! What retirement actually needs to cost, as an input rather than an assumption.
//!
//! # The inconsistency this module fixes
//!
//! The tool exists because calculators stop at gross-minus-tax and miss what decides the answer.
//! It then adopted, for the one quantity that decides whether a plan is adequate, a rule of thumb:
//! 75% of current net income, described in the README as "a common rule of thumb, not a
//! personalized budget". That is the failure the tool was built to criticise, relocated from the
//! income side to the spending side.
//!
//! The fix is not to replace 75% with a better guess. It is to make the figure an input with a
//! name, report what happens across a range of them, and report **the level at which the plan
//! stops working** — because "coverage 117%" and "you are adequate up to CHF 5,120 a month" are
//! very different statements to a reader deciding whether to trust the number.
//!
//! # Why the break-even figure is the informative one
//!
//! Coverage is `pension / needs`, so it is linear in needs and the break-even is the projected
//! pension itself. That looks like a trivially small result and is the point: it converts an
//! unbounded judgement ("do I need 60% or 90%?") into a comparison the user can actually make
//! ("am I likely to spend more or less than CHF 5,120 a month?"). A coverage percentage answers a
//! question nobody asked; the break-even asks a question the user can answer from their own life.
//!
//! # What this does not do
//!
//! It does not model spending changing with age, health costs, a partner's death, or long-term
//! care. Retirement spending is famously not flat, and this module treats it as flat within each
//! scenario. That is a real limitation and it is why the range is reported rather than a point.

/// The percentages the range is reported over.
///
/// Declared. 60/75/90 brackets the range in which most published rules of thumb sit, and the middle
/// one is the conventional default; the outer ones are what the same plan looks like if the user is
/// disciplined or unlucky about spending.
pub const NEEDS_RANGE: [f64; 3] = [0.60, 0.75, 0.90];

/// How the user's retirement needs are stated.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Basis {
    /// A monthly figure the user supplied. First-class, and takes precedence over any rule of
    /// thumb because it is the only version that is a measurement of their life.
    ExplicitMonthly(f64),
    /// A share of current net income. The declared default is [`Basis::default`], 0.75.
    PercentOfNet(f64),
    /// A monthly figure from the requirements model: the tool's own budget for this household.
    ///
    /// This is what the main optimization path uses, and it is *not* the rule of thumb -- it is a
    /// modelled basket. It is included here so that both paths report a range and a break-even
    /// through one mechanism instead of two that could disagree.
    Modelled(f64),
}

/// The conventional share of net income, named rather than repeated where it is used.
pub const DEFAULT_PERCENT: f64 = 0.75;

impl Default for Basis {
    fn default() -> Self {
        Basis::PercentOfNet(DEFAULT_PERCENT)
    }
}

impl Basis {
    /// The monthly need this basis implies, given the user's net monthly income.
    pub fn monthly(&self, net_monthly_income: f64) -> f64 {
        match *self {
            Basis::ExplicitMonthly(monthly) => monthly.max(0.0),
            Basis::PercentOfNet(share) => net_monthly_income.max(0.0) * share.clamp(0.0, 1.0),
            Basis::Modelled(monthly) => monthly.max(0.0),
        }
    }

    /// Whether the figure came from the user rather than from a rule of thumb.
    pub fn is_explicit(&self) -> bool {
        matches!(self, Basis::ExplicitMonthly(_))
    }

    /// Whether the figure came from the tool's own budget model rather than a rule of thumb.
    pub fn is_modelled(&self) -> bool {
        matches!(self, Basis::Modelled(_))
    }

    /// The share of the reference figure this basis sits at, for labelling the report.
    ///
    /// A percentage basis carries its own share. An explicit figure or a modelled budget *is* the
    /// reference, and the reference sits at the conventional 75% point, so both report 0.75.
    pub fn share(&self) -> f64 {
        match *self {
            Basis::PercentOfNet(share) => share.clamp(0.0, 1.0),
            Basis::ExplicitMonthly(_) | Basis::Modelled(_) => DEFAULT_PERCENT,
        }
    }

    /// One line describing where the figure came from, for printing beside it.
    pub fn describe(&self, net_monthly_income: f64) -> String {
        match *self {
            Basis::ExplicitMonthly(monthly) => {
                format!("CHF {monthly:.0}/month, supplied by you")
            }
            Basis::PercentOfNet(share) => format!(
                "{:.0}% of your net income (CHF {net_monthly_income:.0}/month) -- \
                 a rule of thumb, not a budget",
                share * 100.0
            ),
            Basis::Modelled(monthly) => {
                format!("CHF {monthly:.0}/month, from this household's modelled budget")
            }
        }
    }

    /// The basis for one of [`NEEDS_RANGE`], so a range can be reported whatever the user chose.
    ///
    /// When the user supplied an explicit figure the range is built around *that*, not around a
    /// percentage of income, because the explicit number is the only one they have actually said.
    pub fn at(&self, share: f64) -> Basis {
        match *self {
            Basis::ExplicitMonthly(monthly) => {
                // Scale the supplied figure by how far the share sits from the conventional 0.75.
                Basis::ExplicitMonthly(monthly * (share / DEFAULT_PERCENT))
            }
            Basis::Modelled(monthly) => Basis::Modelled(monthly * (share / DEFAULT_PERCENT)),
            Basis::PercentOfNet(_) => Basis::PercentOfNet(share),
        }
    }
}

/// Whether a projected pension covers a stated need.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Adequacy {
    /// The projection does not even reach the need.
    Short,
    /// It reaches it, but with less than a tenth to spare.
    Marginal,
    /// It reaches it with a margin.
    Adequate,
}

impl Adequacy {
    /// Classify a coverage ratio.
    pub fn of(coverage: f64) -> Self {
        // Infinite coverage is ADEQUATE, not unknown: it arises when the stated need is zero,
        // and a need of nothing cannot be short. The first version tested `!is_finite()` and
        // classified it as short, which is the wrong way round for the one input where it occurs.
        //
        // `NaN` is classified short, because a computation that failed should not be presented
        // as a pass. Nothing here constructs it; the pessimistic reading is the safe one.
        if coverage.is_nan() || coverage < 1.0 {
            Adequacy::Short
        } else if coverage < 1.10 {
            Adequacy::Marginal
        } else {
            Adequacy::Adequate
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Adequacy::Short => "short",
            Adequacy::Marginal => "marginal",
            Adequacy::Adequate => "adequate",
        }
    }
}

/// One point of the reported range.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NeedsPoint {
    /// The share of the reference figure this point was built from: 0.60, 0.75 and 0.90 for the
    /// range, or the basis's own share for the chosen point.
    ///
    /// Carried rather than recomputed, because recomputing it from the monthly figures is how the
    /// first version of the report labelled the 60% row as 80%: the range is built by scaling
    /// relative to the 75% reference, so a ratio of the two monthly figures is not the share.
    pub share: f64,
    pub basis: Basis,
    /// The monthly need this point represents.
    pub monthly: f64,
    /// Projected real monthly pension divided by that need.
    pub coverage: f64,
    pub verdict: Adequacy,
}

/// The full reading: the basis in use, the range around it, and the break-even.
#[derive(Debug, Clone, PartialEq)]
pub struct NeedsReport {
    pub net_monthly_income: f64,
    /// The need the plan was actually evaluated at.
    pub chosen: NeedsPoint,
    /// 60%, 75% and 90% of whatever the chosen basis is.
    pub range: Vec<NeedsPoint>,
    /// The monthly need at which coverage is exactly one: the most this plan can support.
    pub break_even_monthly: f64,
    /// Projected real monthly pension, the other side of the comparison.
    pub projected_monthly: f64,
}

impl NeedsReport {
    /// Build the report from a projected pension and the user's stated basis.
    ///
    /// `projected_monthly` is the real (inflation-adjusted) monthly income the plan is expected to
    /// deliver. Nothing here re-runs the projection: the coverage relationship is linear in needs,
    /// so the range and the break-even follow from the one figure. Re-running a 10,000-path Monte
    /// Carlo four times to divide by a different denominator would be theatre.
    pub fn new(projected_monthly: f64, net_monthly_income: f64, basis: Basis) -> Self {
        let point = |basis: Basis, share: f64| {
            let monthly = basis.monthly(net_monthly_income);
            let coverage = if monthly > 0.0 { projected_monthly / monthly } else { f64::INFINITY };
            NeedsPoint { share, basis, monthly, coverage, verdict: Adequacy::of(coverage) }
        };
        let range = NEEDS_RANGE.iter().map(|share| point(basis.at(*share), *share)).collect();
        NeedsReport {
            net_monthly_income,
            chosen: point(basis, basis.share()),
            range,
            break_even_monthly: projected_monthly.max(0.0),
            projected_monthly,
        }
    }

    /// Whether the plan is adequate at the need the user stated.
    pub fn verdict(&self) -> Adequacy {
        self.chosen.verdict
    }

    /// Whether the verdict changes across the reported range.
    ///
    /// A plan that is adequate at 60% and short at 90% has not been shown to be adequate; it has
    /// been shown to be adequate *if the user spends less than the middle of the range*. Saying so
    /// is the whole purpose of reporting the range instead of a point.
    pub fn is_sensitive_to_the_range(&self) -> bool {
        let verdicts: Vec<Adequacy> = self.range.iter().map(|point| point.verdict).collect();
        verdicts.windows(2).any(|pair| pair[0] != pair[1])
    }

    /// One sentence the user can act on.
    pub fn headline(&self) -> String {
        let verdict = self.verdict();
        let break_even = format!("{:.0}", self.break_even_monthly);
        if self.is_sensitive_to_the_range() {
            format!(
                "The verdict depends on what you spend: the plan supports up to CHF {break_even}/month, \
                 which covers the low end of the range and not the high end. {} at your own figure.",
                verdict.label()
            )
        } else {
            match verdict {
                Adequacy::Adequate => format!(
                    "The plan supports up to CHF {break_even}/month and you are at {:.0}% of it.",
                    self.chosen.coverage.min(9.99) * 100.0
                ),
                Adequacy::Marginal => format!(
                    "The plan supports CHF {break_even}/month, within a tenth of what you assume. \
                     A small overspend would break it.",
                ),
                Adequacy::Short => format!(
                    "The plan supports CHF {break_even}/month against a need of CHF {:.0}/month.",
                    self.chosen.monthly
                ),
            }
        }
    }
}

/// The total real monthly income a projection delivers: the BVG pension plus AHV.
///
/// Retiring income is judged on the total, because AHV covers needs exactly as much as the BVG
/// annuity does. The two are kept separate by the Monte Carlo and joined here, in one place, so
/// the adequacy figure and the needs report cannot disagree about what the plan pays.
pub fn projected_monthly(median_real_pension: f64, ahv_monthly: f64) -> f64 {
    median_real_pension.max(0.0) + ahv_monthly.max(0.0)
}

/// Where the basis comes from, for the provenance table the tool prints elsewhere.
pub fn provenance_table() -> Vec<(&'static str, &'static str)> {
    vec![
        (
            "retirement needs",
            "supplied by the user when explicit; otherwise 75% of net income, which is a declared \
             rule of thumb and is reported as one",
        ),
        (
            "needs range",
            "declared: 60/75/90 per cent bracket the published rules of thumb; they are not \
             estimates of your spending",
        ),
        (
            "break-even",
            "derived: coverage is pension over needs, so the break-even need is the projected \
             pension",
        ),
    ]
}
