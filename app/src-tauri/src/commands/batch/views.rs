//! What the New batch, Plan and save screens are sent, and the edits they send back. The
//! TypeScript interfaces in `app/src/lib/api/batch.ts` mirror these by hand.

use packing_batch::{ChosenFile, SaveProgress, SaveStage};
use packing_engine::ids::BatchId;
use serde::{Deserialize, Serialize};

use super::repeats::RepeatsView;

/// One chosen file as the file list shows it.
#[derive(Serialize)]
pub struct FileCheckView {
    pub(super) path: String,
    pub(super) name: String,
    pub(super) size: u64,
    /// `HH:MM` of the file's modify time.
    pub(super) time: String,
    pub(super) pages: u16,
    /// *used in batch n*: a live batch of the date read the same bytes.
    pub(super) used_in_batch: Option<BatchId>,
}

pub(super) fn file_view(file: &ChosenFile) -> FileCheckView {
    FileCheckView {
        path: file.path.display().to_string(),
        name: file.filename.clone(),
        size: file.size,
        time: file.modify_time.zoned().strftime("%H:%M").to_string(),
        pages: file.page_count,
        used_in_batch: file.used_in,
    }
}

#[derive(Clone, Serialize)]
pub struct ReadProgress {
    pub(super) page: usize,
    pub(super) pages: usize,
}

/// The `save-progress` event: the stage and how far it has got.
#[derive(Clone, Serialize)]
pub struct SaveProgressEvent {
    stage: StageView,
    page: usize,
    pages: usize,
}

#[derive(Clone, Copy, Serialize)]
enum StageView {
    CopyingInputs,
    Writing,
    Verifying,
    Saving,
}

impl SaveProgressEvent {
    pub(super) fn of(progress: SaveProgress) -> Self {
        let stage = match progress.stage {
            SaveStage::CopyingInputs => StageView::CopyingInputs,
            SaveStage::Writing => StageView::Writing,
            SaveStage::Verifying => StageView::Verifying,
            SaveStage::Saving => StageView::Saving,
        };
        Self {
            stage,
            page: progress.done,
            pages: progress.total,
        }
    }
}

/// What the New batch screen needs when reading ends, and again after every logo answer.
#[derive(Serialize)]
pub struct ReadResult {
    pub(super) files: usize,
    pub(super) pages: usize,
    /// The orders of the batch, one copy each.
    pub(super) orders: usize,
    /// The orders read, those saved earlier included.
    pub(super) total_orders: usize,
    pub(super) already_saved: usize,
    pub(super) warnings: Vec<String>,
    /// The batch an amend redoes: the Plan screen is titled *Redo batch `<b>`*.
    pub(super) amend: Option<BatchId>,
    /// The courier names the logo prompt's buttons show, in file order.
    pub(super) couriers: Vec<String>,
    /// The header images the rules do not know and the owner has not passed over.
    pub(super) logos: Vec<LogoView>,
    /// The orders that appear more than once; none when every order has one copy.
    pub(super) repeats: Option<RepeatsView>,
    pub(super) plan: PlanView,
}

/// What *Amend* opens with: the batch's own inputs as the file list and their read.
#[derive(Serialize)]
pub struct AmendStart {
    pub(super) batch: BatchId,
    pub(super) files: Vec<FileCheckView>,
    pub(super) read: ReadResult,
}

/// One unseen logo as the prompt shows it; `picture` is a PNG data URL, none when no page could
/// be read (the screen then shows the hash).
#[derive(Serialize)]
pub struct LogoView {
    pub(super) hash: String,
    pub(super) pages: usize,
    pub(super) orders: usize,
    pub(super) picture: Option<String>,
}

/// The Plan screen's data.
#[derive(Serialize)]
pub struct PlanView {
    pub(super) rows: Vec<PlanRow>,
    /// Two outputs of the batch share a name: *Save batch* waits.
    pub(super) has_duplicate_names: bool,
    /// Where *+ Add a group for this batch* inserts, above the rest row.
    pub(super) add_at: usize,
    /// The sort rule is one group that takes every order: one group PDF.
    pub(super) no_categories: bool,
    /// The sort rule's name.
    pub(super) grouping: String,
}

/// One row of the Plan screen: a group of the plan, its last group taking the rest, or the
/// `? Uncategorised` group the plan's filters left orders for.
#[derive(Serialize)]
pub struct PlanRow {
    #[serde(flatten)]
    pub(super) kind: PlanRowKind,
    pub(super) code: String,
    pub(super) name: String,
    /// The filter as written; none on a group that takes every order left.
    pub(super) condition: Option<String>,
    pub(super) orders: usize,
    /// The order contents in printing order.
    pub(super) runs: Vec<RunView>,
    pub(super) pages: usize,
    /// The group PDF's name; empty when the group takes no order.
    pub(super) file_name: String,
    pub(super) skipped: bool,
    /// The code of the earlier row whose file has the same name.
    pub(super) duplicate_of: Option<String>,
    pub(super) can_move_up: bool,
    pub(super) can_move_down: bool,
}

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PlanRowKind {
    /// A group of the plan, edited through its index.
    Entry { index: usize },
    /// The plan's last group, with no filter: it takes every order left and never moves.
    Rest { index: usize },
    /// `? Uncategorised`: not a group of the plan, so it has no edits.
    Generated,
}

/// One order content of a group: its items and how many orders have it.
#[derive(Serialize)]
pub struct RunView {
    pub(super) contents: String,
    pub(super) orders: usize,
}

/// One plan change from the window, applied in order.
#[derive(Deserialize, Debug)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PlanEdit {
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
    /// A one-off group at `at`, above the rest row.
    Add {
        at: usize,
        code: String,
        name: String,
        condition: String,
    },
    /// The group PDF's whole name.
    Rename {
        index: usize,
        name: String,
    },
}

/// What *Save batch* did: saved, or stopped by the verifier's findings.
#[derive(Serialize)]
pub struct SaveResultView {
    pub(super) batch: BatchId,
    pub(super) pdfs: usize,
    pub(super) saved: bool,
    pub(super) errors: Vec<String>,
    pub(super) warnings: Vec<String>,
    /// What failed after the commit: the batch is saved, and the next open settles it.
    pub(super) after_save: Vec<String>,
}
