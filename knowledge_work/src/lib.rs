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

//! Operations-research models of AI-assisted knowledge work, and the operating envelope
//! beyond which its productivity gains become fragility.
//!
//! # The transplantation, stated as an equation
//!
//! Lean production normalises high utilisation and treats buffer as waste. The claim this
//! crate formalises is that AI adoption did not change that norm: when the machine multiplied
//! the rate at which work can be *generated*, the firms on that norm did not bank the slack,
//! they raised arrivals until utilisation was back where lean had left it. The technology
//! changed and the target did not.
//!
//! That is a behavioural assumption, not a measurement, and it is the framework's first named
//! scope condition. Everything below is conditional on it, and [`workflow::Policy`] exists so
//! the alternative can be priced rather than asserted.
//!
//! # Why a queue is the right object
//!
//! Because a knowledge workflow is a service system whose cycle time is a convex function of
//! utilisation, and whose variability term is set by *what fraction of the work arrives
//! unplanned*. Both of those are structural features of a queue rather than a metaphor borrowed
//! from a factory: the review stage has servers, an arrival process, a service-time
//! distribution and a queue, and the same three variables (variability, utilisation, effective
//! process time) govern it. §1 of `OPTIMIZING_FOR_WHAT.md` draws the boundary between what is
//! structural and what is only suggestive, and this crate implements only the structural part.
//!
//! # The loop that couples the two models
//!
//! Review capacity is not a constant. It is the capacity to catch defects, and
//! [`human_capacity`] models how that falls when assistance is solved for rather than chosen.
//! So the loop closes: less capacity catches fewer defects, more defects escape to be
//! discovered late, late discovery costs more, and the review stage carries more load at the
//! same nominal utilisation. A workflow can therefore be stable on the day of adoption and
//! outside its envelope a few years later, without anything about the workflow changing.
//!
//! # What the crate refuses to do
//!
//! It does not calibrate. Every parameter is declared, [`workflow::WorkflowParameters::provenance_table`]
//! says so beside each one, and the results are thresholds and shapes rather than levels.
//! It also does not claim the queue is the whole of knowledge work: a queue says nothing about
//! whether the work was worth doing, and that question lives in the document.

pub mod direction;
pub mod envelope;
pub mod queue;
pub mod sim;
pub mod thesis;
pub mod workflow;
