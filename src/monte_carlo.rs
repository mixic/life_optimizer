// Monte Carlo simulation for pension fund returns and retirement quality of life
// Uses log-normal return distribution calibrated to Swiss BVG pension fund data
#![allow(dead_code)]

use rand::SeedableRng;
use rand_distr::{Distribution, Normal};
use crate::economic_regimes::{Regime, TransitionMatrix, simulate_regime_path, simulate_stress_path};

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
            let bvg_annual_pension = capital * bvg_conversion;
            let total_annual_pension = ahv_annual + bvg_annual_pension;
            let monthly_pension = total_annual_pension / 12.0;
            let real_monthly = monthly_pension / inflation_deflator;

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
                let monthly_gap = (monthly_needs - monthly_pension).max(0.0);
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
            let coverage = real_monthly / self.monthly_retirement_needs;
            let qol = self.quality_of_life_score(coverage, depleted);
            qol_scores.push(qol);

            if real_monthly >= self.monthly_retirement_needs {
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

            let bvg_annual_pension = capital * bvg_conversion;
            let total_annual_pension = ahv_annual + bvg_annual_pension;
            let monthly_pension = total_annual_pension / 12.0;
            let real_monthly = monthly_pension / cumulative_inflation;

            capitals.push(capital);
            real_pensions.push(real_monthly);

            // ── Drawdown phase ──────────────────────────────────────────
            let mut residual = capital * 0.15;
            let mut depleted = false;
            let mut infl_accum = cumulative_inflation;

            for year in years_working..total_years {
                infl_accum *= 1.0 + inflations[year];
                let needs_nominal = self.monthly_retirement_needs * infl_accum / cumulative_inflation
                    * cumulative_inflation; // needs grow with cumulative inflation from today
                let needs_today_equiv = self.monthly_retirement_needs * (infl_accum);
                let gap = (needs_today_equiv - monthly_pension).max(0.0) * 12.0;
                let _ = needs_nominal;

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

            let coverage = real_monthly / self.monthly_retirement_needs;
            qol_scores.push(self.quality_of_life_score(coverage, depleted));
            if real_monthly >= self.monthly_retirement_needs {
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
            let total_annual_pension = ahv_annual + bvg_annual_pension;
            let monthly_pension = total_annual_pension / 12.0;
            let real_monthly = monthly_pension / cumulative_inflation;

            capitals.push(capital);
            real_pensions.push(real_monthly);

            let mut residual = capital * 0.15;
            let mut depleted = false;
            let mut infl_accum = cumulative_inflation;

            for year in years_working..total_years {
                infl_accum *= 1.0 + inflations[year];
                let needs_today_equiv = self.monthly_retirement_needs * infl_accum;
                let gap = (needs_today_equiv - monthly_pension).max(0.0) * 12.0;

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

            let coverage = real_monthly / self.monthly_retirement_needs;
            qol_scores.push(self.quality_of_life_score(coverage, depleted));
            if real_monthly >= self.monthly_retirement_needs {
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

    /// BVG conversion rate (Umwandlungssatz) - age dependent
    /// Deferred retirement (66-70) typically earns a higher conversion rate
    /// since capital is annuitized over fewer expected remaining years.
    fn bvg_conversion_rate(&self) -> f64 {
        match self.retirement_age {
            ..=62 => 0.050,
            63    => 0.055,
            64    => 0.062,
            65    => 0.068,
            66    => 0.070,
            67    => 0.072,
            68    => 0.074,
            69    => 0.076,
            _     => 0.078, // 70+
        }
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
