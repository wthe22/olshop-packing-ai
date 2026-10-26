//! The batch steps of the workflow over the crates that do the work: the chosen files read into a
//! draft (steps 3–10), the draft saved through the journal (11–15), and what follows a save — the
//! Printed tick, revert and the amend's read (16–18). The app and the command line both call them.

mod after;
mod amend;
mod draft;
mod error;
mod files;
mod save;

pub use after::{revert, set_printed};
pub use amend::{Amend, amend};
pub use draft::{Draft, ReadRequest, read_draft};
pub use error::BatchError;
pub use files::{ChosenFile, check_files};
pub use save::{SaveOutcome, SaveProgress, SaveRequest, SaveStage, Saved, save};
