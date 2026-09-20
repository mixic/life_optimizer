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

//! Life Optimizer — domain library.
//!
//! This exposes the tax, consumption, optimization, and simulation modules as a
//! library so the `tests/` integration suite can exercise them directly. The
//! `life-optimizer` binary is a thin CLI wrapper over these modules; it is not
//! duplicated logic.
//!
//! Module map:
//!
//! | Module | Responsibility |
//! |---|---|
//! | [`tax`] | Progressive Swiss tax, social security, deduction lookup |
//! | [`requirements`] | Personal consumption basket, life stages, preferences |
//! | [`optimizer`] | Multi-objective work-percentage utility optimization |
//! | [`monte_carlo`] | Pension projection, conversion-rate scenarios, stress tests |
//! | [`economic_regimes`] | Markov regime-switching return/inflation model |
//! | [`display`] / [`mc_display`] | Terminal presentation |

pub mod tax;
pub mod federal_tax;
pub mod federal_tariff_data;
pub mod estv_scales_data;
pub mod estv_deductions_data;
pub mod deductions;
pub mod cantons;
pub mod canton_steuerfuss_data;
pub mod requirements;
pub mod consumption;
pub mod optimizer;
pub mod display;
pub mod monte_carlo;
pub mod mc_display;
pub mod economic_regimes;

