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

//! Queueing building blocks: Little's law, the utilisation term, and the VUT
//! approximation that carries the whole argument.
//!
//! # Why the approximation and not an exact result
//!
//! A knowledge workflow is not M/M/c. Its arrivals are not Poisson (work arrives in batches
//! when a deadline approaches), its service times are not exponential (a review is fast or it
//! is a week), and its servers are not interchangeable. The exact models would require
//! specifying distributions that nobody has measured. The VUT approximation needs only the
//! **mean and the squared coefficient of variation** of each, which is a far weaker demand,
//! and its error is bounded in the region that matters.
//!
//! What that costs is stated rather than hidden: the approximation is checked against the
//! exact M/M/c result in `tests/queue.rs`, the measured error is reported, and a reader who
//! needs a number rather than a threshold should distrust the third digit.
//!
//! # The utilisation term is the finding
//!
//! `U(ρ) = ρ^(√(2(c+1)) − 1) / (1 − ρ)` is where the argument lives. It is convex and it
//! diverges at one, so a system run at 0.9 has five times the waiting of one run at 0.8, and
//! the ninth decile of utilisation costs more than all the others together. Lean's target of
//! high utilisation is therefore not a neutral efficiency choice: it is a choice to sit on a
//! convex curve near its asymptote.

/// Little's law: mean work in process equals arrival rate times mean cycle time.
///
/// Stated as a function rather than left implicit because both directions are used: the
/// workflow model uses it to convert a WIP cap into an admissible arrival rate, and the
/// report uses it to check that a simulated queue's numbers are self-consistent.
pub fn wip(arrival_rate: f64, cycle_time: f64) -> f64 {
    arrival_rate.max(0.0) * cycle_time.max(0.0)
}

/// The arrival rate that holds `items_in_process` at `cycle_time`.
///
/// Returns `None` for a non-positive cycle time, where the question has no answer rather than
/// an infinite one.
pub fn arrival_rate_for(items_in_process: f64, cycle_time: f64) -> Option<f64> {
    if cycle_time > 0.0 && items_in_process >= 0.0 {
        Some(items_in_process / cycle_time)
    } else {
        None
    }
}

/// Utilisation: arrival rate divided by capacity, both in items per unit time.
pub fn utilisation(arrival_rate: f64, capacity: f64) -> Option<f64> {
    if capacity <= 0.0 {
        None
    } else {
        Some(arrival_rate.max(0.0) / capacity)
    }
}

/// The variability term `V = (c_a² + c_s²) / 2` of the VUT equation.
///
/// Note what it does *not* contain: any term for "AI". The model never says the machine is
/// variable; it says the machine changes the two coefficients, because generated work arrives
/// in bursts and reviewed work is bimodal. That distinction is what keeps the claim falsifiable.
pub fn variability(arrival_cv: f64, service_cv: f64) -> f64 {
    0.5 * (arrival_cv.max(0.0).powi(2) + service_cv.max(0.0).powi(2))
}

/// The utilisation term `U = ρ^(√(2(c+1)) − 1) / (1 − ρ)` for a pool of `servers`.
///
/// At `servers = 1` the exponent is exactly one and this reduces to `ρ/(1 − ρ)`, which is the
/// Kingman form and is asserted by a test. At `servers > 1` the exponent above one is the
/// standard refinement for pooled servers: a pool degrades more slowly right up to saturation
/// and then falls off a cliff, which is why consolidating reviewers into a shared pool buys
/// resilience as well as efficiency.
///
/// Returns infinity at or above `ρ = 1`: the queue has no steady-state mean there, and
/// returning a large finite number would let a caller mistake instability for slowness.
pub fn utilisation_term(rho: f64, servers: usize) -> f64 {
    if servers == 0 || rho >= 1.0 {
        return f64::INFINITY;
    }
    let rho = rho.max(0.0);
    let exponent = (2.0 * (servers as f64 + 1.0)).sqrt() - 1.0;
    rho.powf(exponent) / (1.0 - rho)
}

/// Mean time a work item spends **waiting**, by the VUT approximation.
///
/// This is the quantity lean logic is blind to: it measures labour, and the waiting is not
/// labour. A team can be at 100% labour utilisation with a four-week queue in front of it, and
/// every productivity metric will be satisfied.
pub fn queue_delay(
    rho: f64,
    servers: usize,
    arrival_cv: f64,
    service_cv: f64,
    mean_service: f64,
) -> f64 {
    let effective_process_time = mean_service.max(0.0) / servers.max(1) as f64;
    variability(arrival_cv, service_cv) * utilisation_term(rho, servers) * effective_process_time
}

/// Mean cycle time at one station: time waiting plus the item's own service time.
///
/// The service time is *not* divided by the number of servers: an item is served by one
/// reviewer once, so pooling shortens the wait rather than the service.
pub fn cycle_time(
    rho: f64,
    servers: usize,
    arrival_cv: f64,
    service_cv: f64,
    mean_service: f64,
) -> f64 {
    queue_delay(rho, servers, arrival_cv, service_cv, mean_service) + mean_service.max(0.0)
}

/// M/M/1 waiting time, the exact result the approximation must reproduce at one server.
pub fn mm1_queue_delay(rho: f64, mean_service: f64) -> f64 {
    if rho >= 1.0 {
        f64::INFINITY
    } else {
        rho / (1.0 - rho) * mean_service
    }
}

/// The Erlang C probability that an arrival has to wait, for a pool of `servers` at `rho`.
///
/// Used only as the exact reference the VUT approximation is validated against; nothing in the
/// workflow model depends on it. It computes `P(W > 0)` from the offered load in erlangs, and
/// returns one when the queue is saturated, which is the correct limit rather than a guard.
pub fn erlang_c(rho: f64, servers: usize) -> f64 {
    if servers == 0 || rho >= 1.0 {
        return 1.0;
    }
    let c = servers as f64;
    let offered = c * rho.max(0.0);
    // sum_{k=0}^{c-1} offered^k / k!
    let mut sum = 1.0;
    let mut term = 1.0;
    for k in 1..servers {
        term *= offered / k as f64;
        sum += term;
    }
    let last = term * offered / c; // offered^c / c!
    let tail = last / (1.0 - rho);
    tail / (sum + tail)
}

/// M/M/c waiting time through Erlang C: the exact benchmark for the approximation.
pub fn mmc_queue_delay(rho: f64, servers: usize, mean_service: f64) -> f64 {
    if rho >= 1.0 || servers == 0 {
        return f64::INFINITY;
    }
    let c = servers as f64;
    erlang_c(rho, servers) * mean_service / (c * (1.0 - rho))
}

/// The utilisation at which the VUT waiting time reaches `target_wait`.
///
/// This is the inversion the operating envelope is built from, and it is the answer to "how
/// much of the machine's speed can this team absorb": solve `V·U(ρ)·T = target`, which has no
/// closed form worth writing down, because `U` is a power function over a linear denominator.
/// Bisection on a monotone function is cheaper than the algebra and cannot get the sign wrong.
///
/// Returns `None` when even an empty queue cannot meet the target, or when the target is not
/// reachable below saturation — both are real answers and neither is a number.
pub fn utilisation_for_delay(
    target_wait: f64,
    servers: usize,
    arrival_cv: f64,
    service_cv: f64,
    mean_service: f64,
) -> Option<f64> {
    if target_wait < 0.0 || mean_service <= 0.0 || servers == 0 {
        return None;
    }
    let delay_at = |rho: f64| queue_delay(rho, servers, arrival_cv, service_cv, mean_service);
    if delay_at(0.0) > target_wait {
        return None;
    }
    if delay_at(1.0 - 1e-12) <= target_wait {
        return None;
    }
    let (mut low, mut high) = (0.0_f64, 1.0 - 1e-12);
    for _ in 0..200 {
        let mid = 0.5 * (low + high);
        if delay_at(mid) <= target_wait {
            low = mid;
        } else {
            high = mid;
        }
        if (high - low) < 1e-12 {
            break;
        }
    }
    Some(0.5 * (low + high))
}
