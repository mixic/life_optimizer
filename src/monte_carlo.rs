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

// Monte Carlo simulation for pension fund returns and retirement quality of life
// Uses log-normal return distribution calibrated to Swiss BVG pension fund data
#![allow(dead_code)]

use rand::SeedableRng;
use rand_distr::{Distribution, Normal};
use crate::economic_regimes::{Regime, TransitionMatrix, simulate_regime_path, simulate_stress_path};

// ─── Conversion rate (Umwandlungssatz) scenarios ─────────────────────────────
//
// The conversion rate is the single largest source of unjustified precision in
// a pension projection: it maps accumulated capital to a lifelong annuity, and
// the rate actually applied by Swiss pension funds is materially below the
// statutory BVG minimum. Reporting one number hides that.

/// Statutory BVG minimum conversion rate (BVG Art. 14), in force since 2014.
pub const STATUTORY_CONVERSION_RATE: f64 = 0.068;
/// Rate typically applied by Swiss Pensionskassen on the mandatory portion.
pub const TYPICAL_FUND_CONVERSION_RATE: f64 = 0.055;
/// Lower bound for the forward projection — expert consensus floor.
pub const MIN_PROJECTED_CONVERSION_RATE: f64 = 0.050;
/// Base year $t_0$ for the linear reduction model.
pub const CONVERSION_RATE_BASE_YEAR: f64 = 2024.0;
/// Annual reduction $\Delta r$ in the linear model (0.036 percentage points).
pub const CONVERSION_RATE_ANNUAL_REDUCTION: f64 = 0.00036;

/// Which conversion rate to apply when turning capital into an annuity.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConversionRateScenario {
    /// Statutory BVG minimum, 6.8%.
    Statutory,
    /// Rate typically applied by a Swiss pension fund, 5.5%.
    FundTypical,
    /// Linear forward projection $r(t) = r_0 - (t - t_0)\Delta r$, floored.
    FutureProjection,
    /// A rate the user supplied for their own pension fund. Always quoted as
    /// given; it is never adjusted for the retirement year, because it is
    /// presented as a known contractual fact rather than a forecast.
    Custom(f64),
}

impl ConversionRateScenario {
    /// The rate to apply at a given retirement year.
    ///
    /// `Statutory` and `FundTypical` are a-priori constants; only
    /// `FutureProjection` varies with the retirement year.
    pub fn rate_for(&self, retirement_year: u32) -> f64 {
        match *self {
            ConversionRateScenario::Statutory => STATUTORY_CONVERSION_RATE,
            ConversionRateScenario::FundTypical => TYPICAL_FUND_CONVERSION_RATE,
            ConversionRateScenario::FutureProjection => project_conversion_rate(retirement_year),
            ConversionRateScenario::Custom(rate) => rate,
        }
    }

    pub fn label(&self) -> String {
        match *self {
            ConversionRateScenario::Statutory => "Statutory BVG (6.8%)".to_string(),
            ConversionRateScenario::FundTypical => "Typical fund (5.5%)".to_string(),
            ConversionRateScenario::FutureProjection => "Future projection".to_string(),
            ConversionRateScenario::Custom(rate) => format!("Your fund ({:.2}%)", rate * 100.0),
        }
    }
}

/// Project the conversion rate for a retirement year using the linear model
/// from `FutureWork.md` §10.3, bounded below by `MIN_PROJECTED_CONVERSION_RATE`.
pub fn project_conversion_rate(retirement_year: u32) -> f64 {
    let years_elapsed = retirement_year as f64 - CONVERSION_RATE_BASE_YEAR;
    let rate = STATUTORY_CONVERSION_RATE - years_elapsed * CONVERSION_RATE_ANNUAL_REDUCTION;
    rate.max(MIN_PROJECTED_CONVERSION_RATE)
}

/// Multiplier applied to the base conversion rate when retirement is deferred
/// past the statutory age, or taken early.
///
/// Deferring means the same capital is annuitized over fewer expected remaining
/// years, which raises the annually payable rate. That effect is independent of
/// which base rate a given fund applies, so it is expressed as a ratio against
/// the statutory-age rate rather than as a second absolute rate table.
pub fn deferred_retirement_factor(retirement_age: u32) -> f64 {
    const AT_65: f64 = 0.068;
    let absolute = match retirement_age {
        ..=62 => 0.050,
        63 => 0.055,
        64 => 0.062,
        65 => AT_65,
        66 => 0.070,
        67 => 0.072,
        68 => 0.074,
        69 => 0.076,
        _ => 0.078, // 70+
    };
    absolute / AT_65
}

/// The effective conversion rate for a retirement age and year under a given
/// scenario: the scenario's base rate, scaled by the deferral factor.
///
/// Single source of truth for the conversion rate, shared by the optimizer's
/// pension estimate and the Monte Carlo projection so the two engines cannot
/// disagree about which Umwandlungssatz is assumed.
pub fn effective_conversion_rate(
    scenario: ConversionRateScenario,
    retirement_age: u32,
    retirement_year: u32,
) -> f64 {
    scenario.rate_for(retirement_year) * deferred_retirement_factor(retirement_age)
}

/// Calendar year a person retiring at `retirement_age` reaches retirement,
/// anchored to the current calendar year and never earlier than the projection
/// model's own base year.
pub fn retirement_year_from(current_age: u32, retirement_age: u32) -> u32 {
    use chrono::Datelike;
    let current_year = chrono::Local::now().year() as f64;
    let base = current_year.max(CONVERSION_RATE_BASE_YEAR) as u32;
    base + retirement_age.saturating_sub(current_age)
}

/// Uncertainty band around a projected conversion rate, in percentage points.
///
/// `FutureWork.md` §10.5 places stochastic conversion-rate modelling in the
/// long-term bucket and warns it "should not be invented as another set of
/// hand-picked point estimates". This band is therefore deliberately explicit
/// about being an assumption: it represents fund-to-fund and regulatory
/// dispersion around the projected path, not a fitted distribution. The
/// distribution is *centred on the projection*, so the model does not claim to
/// know the direction of the error — only its plausible spread.
pub const CONVERSION_RATE_UNCERTAINTY_BAND: f64 = 0.010;

/// Stochastic conversion-rate outcome for one capital.
///
/// This is the §10.5 long-term item: instead of a single conversion rate, the
/// rate itself is drawn from a distribution centred on the forward projection,
/// and the resulting *pension* distribution is reported. Because $P = \gamma
/// K / 12$ is linear in $\gamma$, uncertainty in the rate translates
/// one-for-one into proportional uncertainty in the pension.
#[derive(Debug, Clone)]
pub struct StochasticConversionRateResult {
    pub capital: f64,
    pub retirement_year: u32,
    pub n_simulations: usize,
    /// Mean conversion rate used to centre the distribution.
    pub mean_rate: f64,
    /// Standard deviation of the rate, in decimal form.
    pub rate_std_dev: f64,
    /// Monthly BVG pension percentiles.
    pub p10_monthly: f64,
    pub p25_monthly: f64,
    pub median_monthly: f64,
    pub p75_monthly: f64,
    pub p90_monthly: f64,
    /// `CVaR`: the mean monthly pension across the worst `alpha` tail. Reported
    /// because §4.4 asks for downside risk explicitly rather than only
    /// percentiles, and the two answer different questions.
    pub cvar_10_monthly: f64,
    /// Probability the monthly pension falls below `floor_monthly`.
    pub prob_below_floor: f64,
    /// The floor used for `prob_below_floor`.
    pub floor_monthly: f64,
}

/// Draw a stochastic conversion rate centred on the projected rate.
///
/// Truncated at [`MIN_PROJECTED_CONVERSION_RATE`] on the low side and the
/// statutory rate on the high side: funds rarely convert above the statutory
/// minimum, and the projection's own floor is treated as a lower bound.
fn sample_conversion_rate(mean_rate: f64, std_dev: f64, rng: &mut impl rand::Rng) -> f64 {
    let normal = Normal::new(mean_rate, std_dev).expect("std dev must be positive and finite");
    let draw: f64 = normal.sample(rng);
    draw.clamp(MIN_PROJECTED_CONVERSION_RATE, STATUTORY_CONVERSION_RATE)
}

/// Simulate the pension distribution arising from conversion-rate uncertainty
/// alone, holding capital fixed.
///
/// `n_simulations` independent draws of the rate, `floor_monthly` the pension
/// level the caller considers unacceptable (used for `prob_below_floor`).
pub fn simulate_conversion_rate_uncertainty(
    capital: f64,
    retirement_year: u32,
    n_simulations: usize,
    floor_monthly: f64,
    seed: Option<u64>,
) -> StochasticConversionRateResult {
    let mean_rate = project_conversion_rate(retirement_year);
    let rate_std_dev = CONVERSION_RATE_UNCERTAINTY_BAND;

    let mut rng: rand::rngs::StdRng = match seed {
        Some(s) => rand::rngs::StdRng::seed_from_u64(s),
        None => rand::rngs::StdRng::from_entropy(),
    };

    let mut monthly: Vec<f64> = (0..n_simulations.max(1))
        .map(|_| {
            let rate = sample_conversion_rate(mean_rate, rate_std_dev, &mut rng);
            capital * rate / 12.0
        })
        .collect();

    monthly.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let n = monthly.len();

    // CVaR over the worst 10%: mean of the tail, which is the quantity a
    // downside-risk report should quote rather than the P10 alone.
    let tail_count = ((n as f64 * 0.10).ceil() as usize).max(1);
    let cvar_10 = monthly[..tail_count].iter().sum::<f64>() / tail_count as f64;

    let below = monthly.iter().filter(|m| **m < floor_monthly).count() as f64;

    StochasticConversionRateResult {
        capital,
        retirement_year,
        n_simulations: n,
        mean_rate,
        rate_std_dev,
        p10_monthly: monthly[n / 10],
        p25_monthly: monthly[n / 4],
        median_monthly: monthly[n / 2],
        p75_monthly: monthly[3 * n / 4],
        p90_monthly: monthly[9 * n / 10],
        cvar_10_monthly: cvar_10,
        prob_below_floor: below / n as f64,
        floor_monthly,
    }
}

/// A named pension fund's conversion rate, from `FutureWork.md` §10.5's
/// "fund-specific profiles" item.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PensionFundProfile {
    /// Short identifier used on the CLI, e.g. `publica`.
    pub id: &'static str,
    /// Display name.
    pub name: &'static str,
    /// The conversion rate applied to the mandatory (BVG) portion.
    pub conversion_rate: f64,
    /// Where the figure comes from, or an explicit statement that it is a
    /// placeholder. Every entry must carry this so no rate is anonymous.
    pub source_note: &'static str,
}

/// Known pension-fund conversion rates.
///
/// **These are reference points, not authoritative quotes.** Conversion rates
/// change annually, can differ between the mandatory and super-mandatory
/// portions of the same fund, and are frequently reduced for a fund in
/// underfunding. `FutureWork.md` §7 warns against "a plausible narrative
/// wrapped around unfitted parameters", so every entry states what it
/// represents, and the CLI prints a verification reminder whenever a profile is
/// used.
///
/// Prefer `--conversion-rate` with the figure from your own fund's statement
/// whenever you have it.
pub const PENSION_FUND_PROFILES: &[PensionFundProfile] = &[
    PensionFundProfile {
        id: "publica",
        name: "Publica (federal employee fund)",
        conversion_rate: 0.0515,
        source_note: "Rate applied to the mandatory portion in recent years; verify annually.",
    },
    PensionFundProfile {
        id: "bvk",
        name: "BVK (Zürich cantonal employee fund)",
        conversion_rate: 0.0500,
        source_note: "Approximate rate for the mandatory portion; verify annually.",
    },
    PensionFundProfile {
        id: "statutory",
        name: "Statutory BVG minimum",
        conversion_rate: STATUTORY_CONVERSION_RATE,
        source_note: "BVG Art. 14 legal minimum of 6.8%.",
    },
    PensionFundProfile {
        id: "typical",
        name: "Typical Swiss Pensionskasse",
        conversion_rate: TYPICAL_FUND_CONVERSION_RATE,
        source_note: "Representative market rate, not any specific fund.",
    },
];

/// Look up a fund profile by CLI identifier (case-insensitive).
pub fn find_pension_fund(id: &str) -> Option<&'static PensionFundProfile> {
    let needle = id.trim().to_ascii_lowercase();
    PENSION_FUND_PROFILES.iter().find(|f| f.id == needle)
}

/// Comma-separated list of valid fund identifiers, for error messages.
pub fn pension_fund_ids() -> String {
    PENSION_FUND_PROFILES
        .iter()
        .map(|f| f.id)
        .collect::<Vec<_>>()
        .join(", ")
}

/// Monthly pension outcomes across conversion-rate scenarios for one capital.
#[derive(Debug, Clone)]
pub struct PensionRange {
    pub capital: f64,
    pub retirement_year: u32,
    /// Monthly BVG pension at the statutory 6.8% rate.
    pub statutory: f64,
    /// Monthly BVG pension at the typical 5.5% fund rate.
    pub typical: f64,
    /// Monthly BVG pension at the projected rate for `retirement_year`.
    pub future: f64,
    /// The projected rate used for `future` (exposed so the UI can show it).
    pub future_rate: f64,
    /// Set only when the user supplied an explicit rate for their own fund.
    pub custom: Option<f64>,
    /// The scenario selected by the user to drive the headline projection.
    pub selected: ConversionRateScenario,
    /// The rate actually applied to the headline projection.
    pub selected_rate: f64,
    /// Monthly BVG pension under `selected_rate`.
    pub selected_monthly: f64,
    /// `(minimum, maximum)` across all computed monthly figures.
    pub range: (f64, f64),
    /// Monthly BVG pension difference between the statutory and typical rate.
    pub statutory_minus_typical: f64,
}

impl PensionRange {
    /// Difference between the statutory rate's pension and the given scenario's
    /// pension, in CHF/month. Positive means the statutory rate flatters the outcome.
    pub fn difference_from_statutory(&self, monthly: f64) -> f64 {
        self.statutory - monthly
    }
}

/// Compute the monthly-pension range for a capital under all conversion-rate
/// scenarios, selecting `selected` as the headline figure.
///
/// Monthly pension is $P = \gamma K / 12$ (see `MATHEMATICS.md` §6.2).
pub fn calculate_pension_range(
    capital: f64,
    retirement_year: u32,
    selected: ConversionRateScenario,
) -> PensionRange {
    let monthly = |rate: f64| capital * rate / 12.0;

    let statutory = monthly(STATUTORY_CONVERSION_RATE);
    let typical = monthly(TYPICAL_FUND_CONVERSION_RATE);
    let future_rate = project_conversion_rate(retirement_year);
    let future = monthly(future_rate);
    let custom = match selected {
        ConversionRateScenario::Custom(rate) => Some(monthly(rate)),
        _ => None,
    };

    let selected_rate = selected.rate_for(retirement_year);
    let selected_monthly = monthly(selected_rate);

    let mut values = vec![statutory, typical, future, selected_monthly];
    if let Some(c) = custom {
        values.push(c);
    }
    let minimum = values.iter().cloned().fold(f64::INFINITY, f64::min);
    let maximum = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

    PensionRange {
        capital,
        retirement_year,
        statutory,
        typical,
        future,
        future_rate,
        custom,
        selected,
        selected_rate,
        selected_monthly,
        range: (minimum, maximum),
        statutory_minus_typical: statutory - typical,
    }
}

// ─── Market assumptions (Swiss BVG / LPP historical data) ───────────────────

/// Historical Swiss BVG fund parameters (1985-2024)
/// Conservative asset allocation typical for Swiss Pensionskassen
pub struct MarketAssumptions {
    /// Expected annual real return (above inflation), e.g. 0.032 = 3.2%
    pub expected_real_return: f64,
    /// Annual volatility of returns, e.g. 0.08 = 8%
    pub volatility: f64,
    /// Long-run inflation rate, e.g. 0.015 = 1.5%
    pub inflation: f64,
    /// Correlation between equity and bond returns (negative = diversification)
    pub equity_bond_corr: f64,
}

impl MarketAssumptions {
    /// Conservative: typical Swiss Pensionskasse (60% bonds / 40% equity)
    pub fn conservative() -> Self {
        Self {
            expected_real_return: 0.020,
            volatility: 0.060,
            inflation: 0.015,
            equity_bond_corr: -0.20,
        }
    }

    /// Base case: balanced portfolio, BVG minimum + surplus
    pub fn base_case() -> Self {
        Self {
            expected_real_return: 0.032,
            volatility: 0.085,
            inflation: 0.015,
            equity_bond_corr: -0.15,
        }
    }

    /// Optimistic: well-managed fund, more equity exposure
    pub fn optimistic() -> Self {
        Self {
            expected_real_return: 0.045,
            volatility: 0.110,
            inflation: 0.015,
            equity_bond_corr: -0.10,
        }
    }
}

// ─── Single simulation path ───────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct SimulationPath {
    pub final_capital: f64,
    pub annual_pension: f64,
    pub monthly_pension: f64,
    pub real_monthly_pension: f64, // inflation-adjusted to today's CHF
    pub depleted: bool,            // true if capital runs out before life_expectancy
    pub depletion_age: Option<u32>,
}

// ─── Aggregated results across all paths ─────────────────────────────────────

#[derive(Debug, Clone)]
pub struct MonteCarloResult {
    pub n_simulations: usize,
    pub work_percentage: f64,
    pub annual_contribution: f64,
    pub years_contributing: u32,

    // Capital at retirement
    pub median_capital: f64,
    pub p10_capital: f64,    // 10th percentile (bad scenario)
    pub p25_capital: f64,
    pub p75_capital: f64,
    pub p90_capital: f64,    // 90th percentile (good scenario)

    // Monthly pension (nominal, at retirement)
    pub median_monthly_pension: f64,
    pub p10_monthly_pension: f64,
    pub p90_monthly_pension: f64,

    // Monthly pension in today's CHF (real)
    pub median_real_pension: f64,
    pub p10_real_pension: f64,
    pub p90_real_pension: f64,

    // Sustainability until life_expectancy
    pub prob_adequate: f64,        // probability pension covers needs
    pub prob_depletion: f64,       // probability capital depletes before life_expectancy
    pub median_depletion_age: Option<u32>,

    // Quality of life scores (0-10)
    pub median_qol_score: f64,
    pub p10_qol_score: f64,
    pub p90_qol_score: f64,

    // AHV (Pillar 1) - deterministic
    pub ahv_monthly: f64,

    // Total income in retirement
    pub median_total_monthly: f64,
    pub p10_total_monthly: f64,
    pub p90_total_monthly: f64,
}

// ─── Regime-switching (recession/inflation-aware) results ────────────────────

#[derive(Debug, Clone)]
pub struct RegimeSwitchingResult {
    pub n_simulations: usize,
    pub median_capital: f64,
    pub p10_capital: f64,
    pub p90_capital: f64,
    pub median_real_pension: f64,
    pub p10_real_pension: f64,
    pub p90_real_pension: f64,
    pub median_qol_score: f64,
    pub p10_qol_score: f64,
    pub p90_qol_score: f64,
    pub prob_adequate: f64,
    pub prob_depletion: f64,
    pub prob_recession_at_retirement: f64,
    pub avg_recession_years_during_career: f64,
    pub ahv_monthly: f64,
}

// ─── Main simulator ───────────────────────────────────────────────────────────

pub struct PensionSimulator {
    pub current_age: u32,
    pub retirement_age: u32,
    pub life_expectancy: u32,
    pub current_salary: f64,
    pub work_percentage: f64,
    pub married: bool,
    pub existing_bvg_capital: f64,  // already accumulated (if known)
    pub pillar3a_annual: f64,       // annual Pillar 3a contribution
    pub monthly_retirement_needs: f64,
    pub n_simulations: usize,
    pub seed: Option<u64>,
    /// Which conversion rate (Umwandlungssatz) to apply at retirement.
    /// Defaults to `Statutory` so existing behaviour is unchanged unless the
    /// caller opts into a different rate.
    pub conversion_scenario: ConversionRateScenario,
}

impl PensionSimulator {
    pub fn new(
        current_age: u32,
        retirement_age: u32,
        life_expectancy: u32,
        current_salary: f64,
        work_percentage: f64,
        married: bool,
        monthly_retirement_needs: f64,
    ) -> Self {
        Self {
            current_age,
            retirement_age,
            life_expectancy,
            current_salary,
            work_percentage,
            married,
            existing_bvg_capital: 0.0,
            pillar3a_annual: 0.0,
            monthly_retirement_needs,
            n_simulations: 10_000,
            seed: Some(42), // reproducible by default
            conversion_scenario: ConversionRateScenario::Statutory,
        }
    }

    /// Run Monte Carlo simulation under given market assumptions
    pub fn run(&self, assumptions: &MarketAssumptions) -> MonteCarloResult {
        let working_income = self.current_salary * self.work_percentage;
        let years_working = (self.retirement_age - self.current_age) as f64;
        let years_retired = (self.life_expectancy - self.retirement_age) as f64;

        let annual_3a_contribution = self.pillar3a_annual;

        // Nominal mean return = real return + inflation (log-normal)
        let mu_nominal = assumptions.expected_real_return + assumptions.inflation;
        // Log-normal parameters
        let sigma = assumptions.volatility;
        let mu_ln = mu_nominal - 0.5 * sigma * sigma;

        let mut rng = if let Some(s) = self.seed {
            rand::rngs::StdRng::seed_from_u64(s)
        } else {
            rand::rngs::StdRng::from_entropy()
        };

        let normal = Normal::new(mu_ln, sigma).unwrap();

        // AHV (Pillar 1) - deterministic
        let ahv_max = if self.married { 44_100.0f64 } else { 29_400.0f64 };
        let ahv_annual = (working_income * 0.30).min(ahv_max);
        let ahv_monthly = ahv_annual / 12.0;

        // BVG conversion rate (age-dependent, higher for deferred retirement)
        let bvg_conversion = self.bvg_conversion_rate();

        // Inflation factor: deflate pension back to today's CHF
        let inflation_deflator = (1.0 + assumptions.inflation)
            .powf(years_working);

        let mut capitals: Vec<f64> = Vec::with_capacity(self.n_simulations);
        let mut monthly_pensions: Vec<f64> = Vec::with_capacity(self.n_simulations);
        let mut real_pensions: Vec<f64> = Vec::with_capacity(self.n_simulations);
        let mut qol_scores: Vec<f64> = Vec::with_capacity(self.n_simulations);
        let mut depletion_ages: Vec<Option<u32>> = Vec::with_capacity(self.n_simulations);
        let mut adequate_count = 0usize;
        let mut depleted_count = 0usize;

        for _ in 0..self.n_simulations {
            // ── Accumulation phase ──────────────────────────────────────────
            let mut capital = self.existing_bvg_capital;

            for year in 0..years_working as usize {
                let age_this_year = self.current_age + year as u32;
                let bvg_rate = Self::bvg_contribution_rate_at_age(age_this_year);
                let annual_bvg_contribution = working_income * bvg_rate;

                // Annual return drawn from log-normal
                let r: f64 = normal.sample(&mut rng).exp();
                capital = capital * r + annual_bvg_contribution + annual_3a_contribution;
            }

            capitals.push(capital);

            // ── Convert to pension ──────────────────────────────────────────
            // The pension arrays hold the BVG annuity only. AHV is deterministic
            // and is added back explicitly wherever total income is needed, so
            // that this path and `run_regime_switching` agree on what the
            // figures mean. Previously AHV was baked in here and then added
            // again for `median_total_monthly`, which double-counted it.
            let bvg_annual_pension = capital * bvg_conversion;
            let monthly_pension = bvg_annual_pension / 12.0;
            let real_monthly = monthly_pension / inflation_deflator;
            let total_monthly_pension = monthly_pension + ahv_monthly;

            monthly_pensions.push(monthly_pension);
            real_pensions.push(real_monthly);

            // ── Drawdown phase: check sustainability ────────────────────────
            // Model residual capital (capital not converted via annuity)
            // In Switzerland BVG is mostly annuity, but model remaining cushion
            let mut residual = capital * 0.15; // ~15% lump sum option
            let mut depleted = false;
            let mut depletion_age = None;

            for year_in_ret in 0..years_retired as usize {
                let monthly_needs = self.monthly_retirement_needs
                    * (1.0 + assumptions.inflation).powf(years_working + year_in_ret as f64);
                // The gap is measured against total income: AHV genuinely
                // reduces what the residual capital must cover.
                let monthly_gap = (monthly_needs - total_monthly_pension).max(0.0);
                let annual_gap = monthly_gap * 12.0;

                // Draw down residual capital to cover gap
                if annual_gap > 0.0 {
                    residual -= annual_gap;
                    if residual < 0.0 && !depleted {
                        depleted = true;
                        depletion_age = Some(self.retirement_age + year_in_ret as u32);
                    }
                }
            }

            if depleted {
                depleted_count += 1;
            }
            depletion_ages.push(depletion_age);

            // ── Quality of life score ────────────────────────────────────────
            // Adequacy is judged on total retirement income: AHV counts toward
            // covering needs just as much as the BVG annuity does.
            let total_real_monthly = real_monthly + ahv_monthly;
            let coverage = total_real_monthly / self.monthly_retirement_needs;
            let qol = self.quality_of_life_score(coverage, depleted);
            qol_scores.push(qol);

            if total_real_monthly >= self.monthly_retirement_needs {
                adequate_count += 1;
            }
        }

        // ── Aggregate statistics ─────────────────────────────────────────────
        capitals.sort_by(|a, b| a.partial_cmp(b).unwrap());
        monthly_pensions.sort_by(|a, b| a.partial_cmp(b).unwrap());
        real_pensions.sort_by(|a, b| a.partial_cmp(b).unwrap());
        qol_scores.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let n = self.n_simulations;

        // Compute median depletion age
        let depletion_ages_flat: Vec<u32> = depletion_ages
            .iter()
            .filter_map(|x| *x)
            .collect();
        let median_depletion_age = if !depletion_ages_flat.is_empty() {
            let mut sorted = depletion_ages_flat.clone();
            sorted.sort();
            Some(sorted[sorted.len() / 2])
        } else {
            None
        };

        MonteCarloResult {
            n_simulations: n,
            work_percentage: self.work_percentage,
            annual_contribution: working_income * Self::bvg_contribution_rate_at_age(self.retirement_age.saturating_sub(1)) + annual_3a_contribution,
            years_contributing: self.retirement_age - self.current_age,

            median_capital:  capitals[n / 2],
            p10_capital:     capitals[n / 10],
            p25_capital:     capitals[n / 4],
            p75_capital:     capitals[3 * n / 4],
            p90_capital:     capitals[9 * n / 10],

            median_monthly_pension: monthly_pensions[n / 2],
            p10_monthly_pension:    monthly_pensions[n / 10],
            p90_monthly_pension:    monthly_pensions[9 * n / 10],

            median_real_pension: real_pensions[n / 2],
            p10_real_pension:    real_pensions[n / 10],
            p90_real_pension:    real_pensions[9 * n / 10],

            prob_adequate:  adequate_count as f64 / n as f64,
            prob_depletion: depleted_count as f64 / n as f64,
            median_depletion_age,

            median_qol_score: qol_scores[n / 2],
            p10_qol_score:    qol_scores[n / 10],
            p90_qol_score:    qol_scores[9 * n / 10],

            ahv_monthly,

            median_total_monthly: ahv_monthly + monthly_pensions[n / 2],
            p10_total_monthly:    ahv_monthly + monthly_pensions[n / 10],
            p90_total_monthly:    ahv_monthly + monthly_pensions[9 * n / 10],
        }
    }

    /// Run under all three market scenarios and return all three
    pub fn run_all_scenarios(&self) -> (MonteCarloResult, MonteCarloResult, MonteCarloResult) {
        let conservative = self.run(&MarketAssumptions::conservative());
        let base = self.run(&MarketAssumptions::base_case());
        let optimistic = self.run(&MarketAssumptions::optimistic());
        (conservative, base, optimistic)
    }

    /// Run a regime-switching (Markov chain) simulation that models realistic
    /// clustering of booms, recessions, and stagflation shocks over the full
    /// working + retirement horizon, rather than a single static return distribution.
    /// This is the most realistic scenario for long horizons (e.g. working to 70).
    pub fn run_regime_switching(&self) -> RegimeSwitchingResult {
        let working_income = self.current_salary * self.work_percentage;
        let years_working = (self.retirement_age - self.current_age) as usize;
        let years_retired = (self.life_expectancy - self.retirement_age) as usize;
        let total_years = years_working + years_retired;

        let ahv_max = if self.married { 44_100.0f64 } else { 29_400.0f64 };
        let ahv_annual = (working_income * 0.30).min(ahv_max);
        let ahv_monthly = ahv_annual / 12.0;
        let bvg_conversion = self.bvg_conversion_rate();

        let transitions = TransitionMatrix::calibrated();
        let mut rng = if let Some(s) = self.seed {
            rand::rngs::StdRng::seed_from_u64(s)
        } else {
            rand::rngs::StdRng::from_entropy()
        };

        let mut capitals: Vec<f64> = Vec::with_capacity(self.n_simulations);
        let mut real_pensions: Vec<f64> = Vec::with_capacity(self.n_simulations);
        let mut qol_scores: Vec<f64> = Vec::with_capacity(self.n_simulations);
        let mut regime_at_retirement: Vec<Regime> = Vec::with_capacity(self.n_simulations);
        let mut depleted_count = 0usize;
        let mut adequate_count = 0usize;
        let mut recession_years_experienced: Vec<usize> = Vec::with_capacity(self.n_simulations);

        for _ in 0..self.n_simulations {
            let (returns, inflations, regimes) =
                simulate_regime_path(total_years, None, &transitions, &mut rng);

            // ── Accumulation phase ──────────────────────────────────────
            let mut capital = self.existing_bvg_capital;
            let mut cumulative_inflation = 1.0f64;
            let mut n_recession_years = 0usize;

            for year in 0..years_working {
                let age_this_year = self.current_age + year as u32;
                let bvg_rate = Self::bvg_contribution_rate_at_age(age_this_year);
                let annual_contribution = working_income * bvg_rate + self.pillar3a_annual;

                capital = capital * (1.0 + returns[year]) + annual_contribution;
                cumulative_inflation *= 1.0 + inflations[year];

                if matches!(regimes[year], Regime::Recession | Regime::Stagflation) {
                    n_recession_years += 1;
                }
            }
            recession_years_experienced.push(n_recession_years);

            let regime_near_retirement = regimes.get(years_working.saturating_sub(1))
                .copied()
                .unwrap_or(Regime::Normal);
            regime_at_retirement.push(regime_near_retirement);

            // BVG annuity only, matching `run`. AHV is added back explicitly
            // wherever total income is required.
            let bvg_annual_pension = capital * bvg_conversion;
            let monthly_pension = bvg_annual_pension / 12.0;
            let real_monthly = monthly_pension / cumulative_inflation;
            let total_monthly_pension = monthly_pension + ahv_monthly;

            capitals.push(capital);
            real_pensions.push(real_monthly);

            // ── Drawdown phase ──────────────────────────────────────────
            let mut residual = capital * 0.15;
            let mut depleted = false;
            let mut infl_accum = cumulative_inflation;

            for year in years_working..total_years {
                infl_accum *= 1.0 + inflations[year];
                let needs_today_equiv = self.monthly_retirement_needs * (infl_accum);
                let gap = (needs_today_equiv - total_monthly_pension).max(0.0) * 12.0;

                if gap > 0.0 {
                    // residual also earns/loses returns during drawdown
                    residual = residual * (1.0 + returns[year]) - gap;
                    if residual < 0.0 && !depleted {
                        depleted = true;
                    }
                } else {
                    residual = residual * (1.0 + returns[year]);
                }
            }

            if depleted {
                depleted_count += 1;
            }

            // Adequacy is judged on total retirement income: AHV counts toward
            // covering needs just as much as the BVG annuity does.
            let total_real_monthly = real_monthly + ahv_monthly;
            let coverage = total_real_monthly / self.monthly_retirement_needs;
            qol_scores.push(self.quality_of_life_score(coverage, depleted));
            if total_real_monthly >= self.monthly_retirement_needs {
                adequate_count += 1;
            }
        }

        capitals.sort_by(|a, b| a.partial_cmp(b).unwrap());
        real_pensions.sort_by(|a, b| a.partial_cmp(b).unwrap());
        qol_scores.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let n = self.n_simulations;

        let recession_frac_recession = regime_at_retirement.iter()
            .filter(|r| matches!(r, Regime::Recession | Regime::Stagflation))
            .count() as f64 / n as f64;

        let avg_recession_years = recession_years_experienced.iter().sum::<usize>() as f64 / n as f64;

        RegimeSwitchingResult {
            n_simulations: n,
            median_capital: capitals[n / 2],
            p10_capital: capitals[n / 10],
            p90_capital: capitals[9 * n / 10],
            median_real_pension: real_pensions[n / 2],
            p10_real_pension: real_pensions[n / 10],
            p90_real_pension: real_pensions[9 * n / 10],
            median_qol_score: qol_scores[n / 2],
            p10_qol_score: qol_scores[n / 10],
            p90_qol_score: qol_scores[9 * n / 10],
            prob_adequate: adequate_count as f64 / n as f64,
            prob_depletion: depleted_count as f64 / n as f64,
            prob_recession_at_retirement: recession_frac_recession,
            avg_recession_years_during_career: avg_recession_years,
            ahv_monthly,
        }
    }

    /// Stress test: force a recession/stagflation shock to hit right around
    /// retirement (sequence-of-returns risk). This is the single biggest risk
    /// factor for people retiring with a lump-sum-style pension conversion.
    pub fn run_retirement_shock_stress_test(&self) -> RegimeSwitchingResult {
        let working_income = self.current_salary * self.work_percentage;
        let years_working = (self.retirement_age - self.current_age) as usize;
        let years_retired = (self.life_expectancy - self.retirement_age) as usize;
        let total_years = years_working + years_retired;

        let ahv_max = if self.married { 44_100.0f64 } else { 29_400.0f64 };
        let ahv_annual = (working_income * 0.30).min(ahv_max);
        let ahv_monthly = ahv_annual / 12.0;
        let bvg_conversion = self.bvg_conversion_rate();

        let transitions = TransitionMatrix::calibrated();
        let mut rng = if let Some(s) = self.seed {
            rand::rngs::StdRng::seed_from_u64(s)
        } else {
            rand::rngs::StdRng::from_entropy()
        };

        // Shock window: 2 years before retirement through 1 year after (classic
        // sequence-of-returns danger zone), forced into Recession or Stagflation
        let shock_start = years_working.saturating_sub(2);
        let shock_duration = 3usize;

        let mut capitals: Vec<f64> = Vec::with_capacity(self.n_simulations);
        let mut real_pensions: Vec<f64> = Vec::with_capacity(self.n_simulations);
        let mut qol_scores: Vec<f64> = Vec::with_capacity(self.n_simulations);
        let mut depleted_count = 0usize;
        let mut adequate_count = 0usize;

        for i in 0..self.n_simulations {
            // Alternate shock type between simulations: recession vs stagflation
            let shock_regime = if i % 2 == 0 { Regime::Recession } else { Regime::Stagflation };

            let (returns, inflations, _regimes) = simulate_stress_path(
                total_years, shock_start, shock_regime, shock_duration, &transitions, &mut rng,
            );

            let mut capital = self.existing_bvg_capital;
            let mut cumulative_inflation = 1.0f64;

            for year in 0..years_working {
                let age_this_year = self.current_age + year as u32;
                let bvg_rate = Self::bvg_contribution_rate_at_age(age_this_year);
                let annual_contribution = working_income * bvg_rate + self.pillar3a_annual;
                capital = capital * (1.0 + returns[year]) + annual_contribution;
                cumulative_inflation *= 1.0 + inflations[year];
            }

            let bvg_annual_pension = capital * bvg_conversion;
            let monthly_pension = bvg_annual_pension / 12.0;
            let real_monthly = monthly_pension / cumulative_inflation;
            let total_monthly_pension = monthly_pension + ahv_monthly;

            capitals.push(capital);
            real_pensions.push(real_monthly);

            let mut residual = capital * 0.15;
            let mut depleted = false;
            let mut infl_accum = cumulative_inflation;

            for year in years_working..total_years {
                infl_accum *= 1.0 + inflations[year];
                let needs_today_equiv = self.monthly_retirement_needs * infl_accum;
                let gap = (needs_today_equiv - total_monthly_pension).max(0.0) * 12.0;

                if gap > 0.0 {
                    residual = residual * (1.0 + returns[year]) - gap;
                    if residual < 0.0 && !depleted {
                        depleted = true;
                    }
                } else {
                    residual = residual * (1.0 + returns[year]);
                }
            }

            if depleted {
                depleted_count += 1;
            }

            // Adequacy is judged on total retirement income: AHV counts toward
            // covering needs just as much as the BVG annuity does.
            let total_real_monthly = real_monthly + ahv_monthly;
            let coverage = total_real_monthly / self.monthly_retirement_needs;
            qol_scores.push(self.quality_of_life_score(coverage, depleted));
            if total_real_monthly >= self.monthly_retirement_needs {
                adequate_count += 1;
            }
        }

        capitals.sort_by(|a, b| a.partial_cmp(b).unwrap());
        real_pensions.sort_by(|a, b| a.partial_cmp(b).unwrap());
        qol_scores.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let n = self.n_simulations;

        RegimeSwitchingResult {
            n_simulations: n,
            median_capital: capitals[n / 2],
            p10_capital: capitals[n / 10],
            p90_capital: capitals[9 * n / 10],
            median_real_pension: real_pensions[n / 2],
            p10_real_pension: real_pensions[n / 10],
            p90_real_pension: real_pensions[9 * n / 10],
            median_qol_score: qol_scores[n / 2],
            p10_qol_score: qol_scores[n / 10],
            p90_qol_score: qol_scores[9 * n / 10],
            prob_adequate: adequate_count as f64 / n as f64,
            prob_depletion: depleted_count as f64 / n as f64,
            prob_recession_at_retirement: 1.0, // forced by construction
            avg_recession_years_during_career: shock_duration as f64,
            ahv_monthly,
        }
    }

    // ── Helpers ──────────────────────────────────────────────────────────────

    /// Age-dependent BVG contribution rate (Altersgutschriften) for a given age
    fn bvg_contribution_rate_at_age(age: u32) -> f64 {
        match age {
            0..=24  => 0.00,
            25..=34 => 0.07,
            35..=44 => 0.10,
            45..=54 => 0.15,
            _       => 0.18, // 55+
        }
    }

    /// Age-dependent BVG contribution rate at current age (kept for compatibility)
    fn bvg_contribution_rate(&self) -> f64 {
        Self::bvg_contribution_rate_at_age(self.current_age)
    }

    /// BVG conversion rate (Umwandlungssatz) applied at retirement.
    ///
    /// Delegates to the shared [`effective_conversion_rate`] so the optimizer's
    /// pension estimate and this projection cannot drift apart.
    fn bvg_conversion_rate(&self) -> f64 {
        effective_conversion_rate(
            self.conversion_scenario,
            self.retirement_age,
            self.retirement_year(),
        )
    }

    /// Calendar year of retirement, used by the forward conversion-rate
    /// projection.
    fn retirement_year(&self) -> u32 {
        retirement_year_from(self.current_age, self.retirement_age)
    }

    /// Monthly-pension range across all conversion-rate scenarios, for this
    /// simulator's median projected capital. This is what makes the retirement
    /// uncertainty visible instead of reporting a single, falsely precise number.
    pub fn pension_range_for_capital(&self, capital: f64) -> PensionRange {
        calculate_pension_range(capital, self.retirement_year(), self.conversion_scenario)
    }

    /// Pension distribution from conversion-rate uncertainty alone, holding
    /// capital fixed at the supplied value.
    pub fn conversion_rate_uncertainty(
        &self,
        capital: f64,
        floor_monthly: f64,
    ) -> StochasticConversionRateResult {
        simulate_conversion_rate_uncertainty(
            capital,
            self.retirement_year(),
            self.n_simulations,
            floor_monthly,
            self.seed,
        )
    }

    /// The conversion rate this simulator will apply, for display purposes.
    pub fn effective_conversion_rate(&self) -> f64 {
        self.bvg_conversion_rate()
    }

    /// Quality of life score 0-10 during retirement
    fn quality_of_life_score(&self, coverage: f64, depleted: bool) -> f64 {
        if depleted {
            return 1.0; // Very poor - ran out of money
        }
        match coverage {
            c if c >= 1.50 => 10.0, // 150%+ needs: excellent
            c if c >= 1.25 => 9.0,
            c if c >= 1.10 => 8.0,
            c if c >= 1.00 => 7.0,  // exactly meets needs: good
            c if c >= 0.90 => 5.5,  // 90%: slightly below
            c if c >= 0.75 => 4.0,  // 75%: noticeably reduced lifestyle
            c if c >= 0.60 => 2.5,  // 60%: significant hardship
            _              => 1.0,  // below 60%: very poor
        }
    }
}

// ─── Compare work percentages ─────────────────────────────────────────────────

#[derive(Debug)]
pub struct WorkPctComparison {
    pub work_pct: f64,
    pub conservative: MonteCarloResult,
    pub base: MonteCarloResult,
    pub optimistic: MonteCarloResult,
}

pub fn compare_work_percentages(
    current_age: u32,
    retirement_age: u32,
    life_expectancy: u32,
    full_salary: f64,
    married: bool,
    monthly_retirement_needs: f64,
    pillar3a_annual: f64,
    candidates: &[f64],
) -> Vec<WorkPctComparison> {
    candidates.iter().map(|&pct| {
        let mut sim = PensionSimulator::new(
            current_age,
            retirement_age,
            life_expectancy,
            full_salary,
            pct,
            married,
            monthly_retirement_needs,
        );
        sim.pillar3a_annual = pillar3a_annual;
        sim.n_simulations = 10_000;

        let (con, base, opt) = sim.run_all_scenarios();
        WorkPctComparison { work_pct: pct, conservative: con, base, optimistic: opt }
    }).collect()
}

