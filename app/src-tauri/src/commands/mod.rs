//! The window commands, grouped by screen: [`start`] (the data folder, the first namespace,
//! *About*), [`panel`] (the namespaces and their dates), [`day`] (the Day screen, Printed, revert,
//! restore), [`batch`] (new batch, plan, save, amend) and [`search`] (search and reprint). Each
//! command is a thin wrapper over a plain function the tests call.

pub mod batch;
pub mod day;
pub mod panel;
pub mod search;
pub mod start;
