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

//! Whether AI authoring time survives the cost of understanding what it wrote.
//!
//! The naive claim is that AI saves authoring hours, so it saves hours. The claim
//! under test is that the hours reappear later, as the cost of building a mental model
//! that the act of authoring would normally have produced for free.
//!
//! # The model, in one paragraph
//!
//! A fixed set of modules, each with a complexity and an authoring mode. A module
//! starts with a comprehension level that is high when a human wrote it and low when
//! a machine did. Comprehension decays exponentially while the module is untouched,
//! at a rate that depends on the authoring mode — this asymmetry is the hypothesis.
//! Maintenance events arrive as a Poisson process; each costs more the lower the
//! engineer's comprehension, and each resolution raises it. A review gate on creation
//! catches defects with probability proportional to comprehension, and what it misses
//! becomes a further event stream. The baseline is all-human; the AI scenario differs
//! only in authoring mode, so the difference between them is attributable to provenance
//! alone.
//!
//! # What it is not
//!
//! It is **mean-field**: it charges expected value rather than drawing event times, so
//! the phase diagram is a deterministic function and variance is not modelled. The
//! `--paths` mode draws real Poisson arrivals and shows how much that assumption smears
//! the answer.
//!
//! It is also **hypothesis-generating**. A phase diagram says where to look; it does not
//! say what is true. `AI_AUTHORING_AND_COMPREHENSION.md` carries the pre-registration,
//! the provenance of every parameter, and the decision rule that was fixed before any
//! of these numbers existed.

pub mod analysis;
pub mod export;
pub mod model;
pub mod params;
pub mod sweep;

pub use model::{crossover_month, run, run_stochastic, Ledger, Scenario};
pub use params::{Params, Provenance};
pub use sweep::{sensitivity, sweep, verdict, Cell, SensitivityRow, Verdict};
