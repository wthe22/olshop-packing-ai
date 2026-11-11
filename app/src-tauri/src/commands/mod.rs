//! The window commands, grouped by screen: [`start`] (the data folder, *About*), [`panel`]
//! (the dates), [`day`] (the Day screen, Printed, revert, restore), [`batch`] (new batch, plan,
//! save, amend), [`search`] (search and reprint), [`rules`] (the Sort rules screen) and
//! [`changes`] (what other copies of the app wrote). Each command is a thin wrapper over a plain
//! function the tests call.

pub mod batch;
pub mod changes;
pub mod day;
pub mod panel;
pub mod rules;
pub mod search;
pub mod start;
