//! The window commands, grouped by screen: [`start`] (the data folder, the owner, *About*),
//! [`day`] (the Day screen, Printed, revert) and [`batch`] (new batch, plan, save, amend). Each
//! command is a thin wrapper over a plain function the tests call.

pub mod batch;
pub mod day;
pub mod start;
