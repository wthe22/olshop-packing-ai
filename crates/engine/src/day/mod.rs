//! The day: `state.json`, the duplicate guard, the file fingerprints, revert and amend.
//!
//! One `state.json` per day folder holds the batches, their saved PDFs and every saved order.
//! The file is written whole (to `state.json.tmp`, then replaced) so a crash never leaves half
//! a file. The engine owns the state; deleting a batch folder is [`delete_batch_folder`].

mod amend;
mod error;
mod fingerprint;
mod guard;
mod save;
mod state;

pub use self::amend::Amend;
pub use self::error::DayError;
pub use self::fingerprint::{Fingerprint, fingerprint};
pub use self::guard::{Duplicate, duplicates_warning};
pub use self::save::{BatchWrite, Findings, delete_batch_folder, now_save_time, save_batch};
pub use self::state::{
    BatchEntry, DayState, FileEntry, Layout, PackingListChoice, PdfEntry, SavedItem, SavedOrder,
    Scope, state_path,
};

#[cfg(test)]
mod tests;
