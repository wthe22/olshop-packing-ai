//! The data folder (data/README.md): the one-writer lock, each date's journal and its fold, the
//! inputs stored by content, the outputs settled by the journal, and the views written from it.
//! The journal is the only truth; every decision is taken from its fold as it is on disk.

mod date;
mod error;
mod files;
mod fold;
mod index;
mod inputs;
mod journal;
mod line;
mod lock;
mod outputs;
mod views;
mod workspaces;

pub use date::{DateFolder, OpenedDate, open_date};
pub use error::StoreError;
pub use files::sha256_of;
pub use fold::{Damage, DamageReason, Fold, LastWrites, LiveBatch, Reprint};
pub use index::{IndexedOrder, SearchIndex};
pub use inputs::{InputProblem, InputState, StoredInput, check_inputs, input_path, store_input};
pub use journal::{Journal, Pending};
pub use line::{encode_line, seal};
pub use lock::DataLock;
pub use outputs::{
    OutputProblem, OutputState, Settled, check_outputs, discard_partial, move_aside, move_forward,
    partial_dir, replaced_dir, settle_partial,
};
pub use views::{ORDER_CONTENTS_TOML, ORDERS_CSV, SESSION_TOML, views_current, write_views};
pub use workspaces::{
    WORKSPACE_LIMIT, WorkspaceNameError, check_workspace_name, create_workspace, dates,
    last_written, workspaces,
};
