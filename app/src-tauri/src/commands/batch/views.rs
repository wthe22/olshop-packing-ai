//! What the New batch, Plan and Day screens receive and send back: the serialisable views of a
//! file check, a read, a plan, a logo prompt and a save, the input types, and the one mapping
//! from a checked file to its view.

use std::path::Path;

use packing_engine::groupings::Grouping;
use packing_engine::ids::{BatchNumber, PdfNumber};
use packing_pdf::batch::FileCheck;
use packing_pdf::worker::{SaveProgress, SaveStage};
use serde::{Deserialize, Serialize};

/// One chosen label file as the New batch screen shows it (08 › *2. New batch*): page count,
/// size, download time and `used in batch n`.
#[derive(Serialize)]
pub struct FileCheckView {
    pub(super) path: String,
    pub(super) name: String,
    pub(super) size: u64,
    /// `HH:MM` of the file's modified time (its download time); `None` when it cannot be read.
    pub(super) time: Option<String>,
    pub(super) pages: usize,
    pub(super) used_in_batch: Option<BatchNumber>,
}

/// The `read-progress` event payload (07 › *Commands between window and Rust*).
#[derive(Clone, Serialize)]
pub struct ReadProgress {
    pub(super) page: usize,
    pub(super) pages: usize,
}

/// The `save-progress` event payload (07 › *Check before saving*: *Shown while it runs*). `stage`
/// is *Writing* or *Checking*; the window shows it with a progress bar on the Plan screen.
#[derive(Clone, Serialize)]
pub struct SaveProgressEvent {
    pub(super) stage: SaveStage,
    pub(super) page: usize,
    pub(super) pages: usize,
}

impl SaveProgressEvent {
    pub(super) fn of(progress: SaveProgress) -> Self {
        Self {
            stage: progress.stage,
            page: progress.page,
            pages: progress.pages,
        }
    }
}

/// What the New batch screen needs when reading ends (08 › *Reading*): the counts and warnings,
/// the logo prompt's data and the initial plan view for the Plan screen.
#[derive(Serialize)]
pub struct ReadResult {
    pub(super) files: usize,
    pub(super) pages: usize,
    pub(super) orders: usize,
    /// Orders read before the duplicate guard.
    pub(super) total_orders: usize,
    pub(super) already_saved: usize,
    pub(super) warnings: Vec<String>,
    /// The batch being redone when this read is an amend (07 › *Amend*): the Plan screen is then
    /// titled *Redo batch `<b>`*; `None` for a normal new batch.
    pub(super) amend: Option<BatchNumber>,
    /// The courier names the prompt's buttons show, in `couriers.toml` order (D-LABEL-1).
    pub(super) couriers: Vec<String>,
    /// The header-band images the courier data does not know (D-LABEL-4): the prompt asks about
    /// each one once, before the Plan screen. Empty when every logo is known.
    pub(super) logos: Vec<LogoView>,
    pub(super) plan: PlanView,
}

/// What *Amend* opens with (08 › *1. Day* → *3. Plan*): the batch being redone, its `download/`
/// files listed already selected in the order stored in `state.json`, and the read of those files
/// as an amendment — the Plan screen opens with it at once, the draft open. **Back** on that Plan
/// screen shows the selection page from `files`; only **Save batch `<b>` again** writes, so nothing
/// here touches `state.json` (07 › *Amend*).
#[derive(Serialize)]
pub struct AmendStart {
    /// The batch being redone: titles the Plan screen and the selection page *Redo batch `<b>`*.
    pub(super) batch: BatchNumber,
    /// The batch's `download/` files, already selected — the selection page's starting list
    /// (08 › *2. New batch* → *The amend's selection page*, Q-APP-2).
    pub(super) files: Vec<FileCheckView>,
    /// The read of the batch's own files, shaped as `read_labels` returns for an amend: the Plan
    /// screen shows it at once ([`ReadResult`]).
    pub(super) read: ReadResult,
}

/// One unseen logo as the logo prompt shows it (08 › *2. New batch* → *The logo prompt*): the
/// image's hash, carried by how many pages and how many orders, and the logo **as printed** —
/// a PNG data URL read back from the PDF, `None` when no page could be read (the screen then
/// shows `hash`).
#[derive(Serialize)]
pub struct LogoView {
    pub(super) hash: String,
    pub(super) pages: usize,
    pub(super) orders: usize,
    pub(super) picture: Option<String>,
}

/// The Plan screen's data (08 › *3. Plan*).
#[derive(Serialize)]
pub struct PlanView {
    pub(super) rows: Vec<PlanRow>,
    /// Two saved PDFs of the batch share a name: *Save batch* waits (08 › *3. Plan*).
    pub(super) has_duplicate_names: bool,
    /// Where *+ Add a pick for this batch* inserts, above the rest row (08 › *3. Plan*).
    pub(super) add_at: usize,
    /// The plan is the one catch-all entry, so the picks are unconfigured: the screen says every
    /// order goes into one PDF set (Q-UI-4, 08 › *3. Plan*).
    pub(super) no_picks: bool,
    /// The grouping the plan belongs to (04 › *Sets*): the grouping the batch is packed with, the
    /// one chosen on the New batch screen or, for an amend, the batch's own recorded grouping. The
    /// header names it and **Edit categories** opens it (08 › *3. Plan*).
    pub(super) grouping: Grouping,
}

/// One row of the Plan screen (08 › *3. Plan*): a plan entry, the plan's rest entry, or the
/// generated `? Uncategorised` row. Which one it is lives in [`PlanRowKind`]; the flags the kind
/// or another field already tells (whether *Use* is offered, whether the row takes no orders)
/// are derived in the screen.
#[derive(Serialize)]
pub struct PlanRow {
    #[serde(flatten)]
    pub(super) kind: PlanRowKind,
    /// The saved-PDF number this row will get; `None` when it takes no orders.
    pub(super) number: Option<PdfNumber>,
    pub(super) code: String,
    pub(super) name: String,
    /// The condition text as written (under the row); `None` when the row has no condition.
    pub(super) condition: Option<String>,
    pub(super) orders: usize,
    pub(super) runs: Vec<RunView>,
    pub(super) pages: usize,
    pub(super) file_name: String,
    /// Skipped for this batch only (08 › *3. Plan*: *Use* unticked).
    pub(super) skipped: bool,
    /// The `#` of the earlier row with the same file name (*Same name as PDF 6*).
    pub(super) duplicate_of: Option<PdfNumber>,
    pub(super) can_move_up: bool,
    pub(super) can_move_down: bool,
}

/// What a Plan row is (08 › *3. Plan*), serialised under the `kind` tag. The window edits only the
/// rows that are plan entries, so only those carry the [`index`](PlanRowKind::Entry) the edits name.
#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PlanRowKind {
    /// A plan entry, edited through its index (an entry with a condition, or one added for the batch).
    Entry { index: usize },
    /// The plan's own rest entry: its last entry, which has no condition and takes the leftover
    /// orders. An entry too, so its index edits it, but it never moves.
    Rest { index: usize },
    /// The generated `? Uncategorised` row, when no entry took the leftovers: not a plan entry, so
    /// no *Use* tick and no edits.
    Generated,
}

/// One run inside a saved PDF, as the expanded row shows it (08 › *3. Plan*):
/// `6-01  Sepatu Standar Samping Motor ×1   96 orders`.
#[derive(Serialize)]
pub struct RunView {
    pub(super) label: String,
    pub(super) contents: String,
    pub(super) orders: usize,
}

/// One plan change from the window (08 › *3. Plan*). Applied to the open plan in order.
#[derive(Deserialize, Debug)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PlanEdit {
    /// Tick/untick *Use*: skip the pick for this batch only.
    SetSkipped {
        index: usize,
        skipped: bool,
    },
    MoveUp {
        index: usize,
    },
    MoveDown {
        index: usize,
    },
    /// *+ Add a pick for this batch*: a one-off pick at `at` (above the rest row).
    Add {
        at: usize,
        code: String,
        name: String,
        condition: String,
    },
    /// The whole saved-PDF name (07 › *Plan*: number included).
    Rename {
        index: usize,
        name: String,
    },
}

/// What *Save batch* wrote, for the green line on the Day screen (08 › *1. Day*), and the check
/// before saving's findings when it stopped or the owner chose *Save anyway* (07 › *Check
/// before saving*).
#[derive(Serialize)]
pub struct SaveResultView {
    pub(super) batch: BatchNumber,
    pub(super) pdfs: usize,
    pub(super) packing_lists: usize,
    /// Whether the batch was written; `false` when the check found something and the dialog's
    /// *Save anyway* was not chosen. The Plan screen then keeps the draft and shows the findings.
    pub(super) saved: bool,
    /// The check's errors, as plain text (the dialog's first list, kept apart from the warnings).
    pub(super) errors: Vec<String>,
    /// The check's warnings (the dialog's second list).
    pub(super) warnings: Vec<String>,
}

pub(super) fn file_check_view(check: FileCheck) -> FileCheckView {
    FileCheckView {
        path: check.path.display().to_string(),
        name: check.name,
        size: check.size,
        time: modified_time(&check.path),
        pages: check.pages,
        used_in_batch: check.used_in_batch,
    }
}

/// `HH:MM` of a file's modified time, or `None` when it cannot be read.
pub(super) fn modified_time(path: &Path) -> Option<String> {
    let metadata = std::fs::metadata(path).ok()?;
    let modified = metadata.modified().ok()?;
    let timestamp = jiff::Timestamp::try_from(modified).ok()?;
    Some(
        timestamp
            .to_zoned(jiff::tz::TimeZone::system())
            .strftime("%H:%M")
            .to_string(),
    )
}
