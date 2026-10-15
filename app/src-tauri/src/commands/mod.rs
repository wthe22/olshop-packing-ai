//! The window commands, grouped by screen: [`day`] (start, day overview, settings), [`batch`]
//! (new batch, plan, save, revert/amend) and [`rules`] (the Categories editor). Every command is a
//! thin wrapper over a plain function that is unit-tested (07 › *Commands between window and
//! Rust*).

pub mod batch;
pub mod day;
pub mod rules;
