//! The batch steps of the workflow over the crates that do the work: the chosen files read into a
//! draft (steps 3–10), the draft saved through the journal (11–15), and what follows a save — the
//! Printed tick, revert and the amend's read (16–18) — reprint and restore (19–20), the day
//! summary, and the migration the owner starts. The app and the command line both call them.

mod after;
mod amend;
mod draft;
mod error;
mod files;
mod migrate;
mod reprint;
mod restore;
mod reverted;
mod save;
mod summaries;
mod written;

pub use after::{Reverted, revert, set_printed, set_sent};
pub use amend::{Amend, amend};
pub use draft::{Draft, ReadRequest, read_draft};
pub use error::BatchError;
pub use files::{ChosenFile, check_files};
pub use migrate::{DateMigrated, MigrateProgress, MigrateRequest, SkippedBatch, migrate_date};
pub use reprint::{ReprintOutcome, ReprintRequest, reprint};
pub use restore::{RestoreOutcome, RestoreRequest, restore};
pub use reverted::RevertedPdfs;
pub use save::{SaveOutcome, SaveProgress, SaveRequest, SaveStage, Saved, save};
pub use summaries::{
    AccountSummaryOutcome, AccountSummaryRequest, DAY_SUMMARY_PDF, account_summary,
    account_summary_file, day_summary, day_summary_file,
};
