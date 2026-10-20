//! The window commands, grouped by screen: [`day`] (start, day overview, settings), [`batch`]
//! (new batch, plan, save, revert/amend) and [`rules`] (the Categories editor). Every command is a
//! thin wrapper over a plain function that is unit-tested.

pub mod batch;
pub mod day;
pub mod rules;
pub mod search;
