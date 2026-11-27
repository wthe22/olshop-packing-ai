//! The data folder (data/README.md): the write lock, each date's journal and its fold, the
//! inputs stored by content, the outputs settled by the journal, and the views written from it.
//! The journal is the only truth; every decision is taken from its fold as it is on disk.

mod archive;
mod date;
mod dates;
mod error;
mod files;
mod flatten;
mod fold;
mod index;
mod inputs;
mod journal;
mod legacy;
mod line;
mod lock;
mod migrations;
mod older;
mod outputs;
mod update_log;
mod views;

pub use archive::{
    ArchivePlan, Archived, NotArchivable, PdfFile, archive, is_archived, plan_archive,
};
pub use date::{DateFolder, OpenedDate, open_date, view_date};
pub use dates::{dates, journal_lengths, newest_batch_rule};
pub use error::StoreError;
pub use files::{folder_size, sha256_of};
pub use flatten::{V0_7_BACKUP, flatten_date, is_v0_7};
pub use fold::{
    Damage, DamageReason, Fold, LastWrites, LiveBatch, PassedOver, Reprint, Unknown, UnknownPart,
};
pub use index::{AccountDate, IndexedOrder, ReprintMark, SearchIndex};
pub use inputs::{
    InputProblem, InputState, StoredInput, check_inputs, input_path, live_inputs, store_input,
};
pub use journal::{Journal, Pending};
pub use line::{encode_line, seal};
pub use lock::DataLock;
pub use migrations::{DateMigration, batches_to_read, migrations};
pub use older::{OlderData, OlderVersion, older_data};
pub use outputs::{
    OutputProblem, OutputState, Settled, check_outputs, discard_partial, move_aside, move_forward,
    partial_dir, replaced_dir, settle_partial,
};
pub use update_log::{UPDATE_LOG, UpdateLog, UpdateStep};
pub use views::{
    ChangedLine, DAY_TOML, DAY_TOML_SCHEMA, EditedView, ORDER_CONTENTS_TOML, ORDERS_CSV,
    ORDERS_CSV_SCHEMA, SESSION_TOML, edited_views, views_current, write_views, write_views_again,
};
