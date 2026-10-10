//! The window commands, grouped by screen: [`day`] (start, day overview, settings) and [`batch`]
//! (new batch, plan, save). Every command is a thin wrapper over a plain function that is
//! unit-tested (07 › *Commands between window and Rust*).

pub mod batch;
pub mod day;
