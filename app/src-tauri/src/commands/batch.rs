//! New batch and Plan/Save commands (08 › *2. New batch*, *3. Plan*, *4. Messages*): check the
//! chosen files, read the labels with progress and a cancel flag, edit the plan and save the
//! batch. The open draft lives in [`AppState`]; its label documents stay open on the PDFium
//! worker until the batch is saved or discarded (07 › *Commands between window and Rust*).
//!
//! The plan view and the plan edits are plain functions, so they are tested on orders built in
//! code (no PDFium needed); the read/save path is tested on `testdata/` and skipped when
//! `vendor/pdfium.dll` is missing.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, mpsc};
use std::thread;

use packing_engine::day::{DayState, PackingListChoice, delete_batch_folder};
use packing_engine::label;
use packing_engine::names;
use packing_engine::orders::{LogoAnswer, Order, ReadOrder, UnseenLogo, display_name};
use packing_engine::plan::{PickResult, Plan};
use packing_engine::rules::{Category, Condition, evaluate, load_rules, parse_condition};
use packing_pdf::batch::{self, Draft, FileCheck};
use packing_pdf::worker::{PdfWorker, Progress};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, State};

use crate::{AmendState, AppState, CATEGORIES_FILE, OpenDraft, commands::day::settings_load};

// ------------------------------------------------------------------------- new batch

/// One chosen label file as the New batch screen shows it (08 › *2. New batch*): page count,
/// size, download time and `used in batch n`.
#[derive(Serialize)]
pub struct FileCheckView {
    path: String,
    name: String,
    size: u64,
    /// `HH:MM` of the file's modified time (its download time).
    time: String,
    pages: usize,
    used_in_batch: Option<i64>,
}

/// The `read-progress` event payload (07 › *Commands between window and Rust*).
#[derive(Clone, Serialize)]
pub struct ReadProgress {
    page: usize,
    pages: usize,
}

/// What the New batch screen needs when reading ends (08 › *Reading*): the counts and warnings,
/// the logo prompt's data and the initial plan view for the Plan screen.
#[derive(Serialize)]
pub struct ReadResult {
    files: usize,
    pages: usize,
    orders: usize,
    /// Orders read before the duplicate guard.
    total_orders: usize,
    already_saved: usize,
    warnings: Vec<String>,
    /// The batch being redone when this read is an amend (07 › *Amend*): the Plan screen is then
    /// titled *Redo batch `<b>`*; `None` for a normal new batch.
    amend: Option<i64>,
    /// The courier names the prompt's buttons show, in `couriers.toml` order (D-LABEL-1).
    couriers: Vec<String>,
    /// The header-band images the courier data does not know (D-LABEL-4): the prompt asks about
    /// each one once, before the Plan screen. Empty when every logo is known.
    logos: Vec<LogoView>,
    plan: PlanView,
}

/// One unseen logo as the logo prompt shows it (08 › *2. New batch* → *The logo prompt*): the
/// image's hash, carried by how many pages and how many orders, and the logo **as printed** —
/// a PNG data URL read back from the PDF, `None` when no page could be read (the screen then
/// shows `hash`).
#[derive(Serialize)]
pub struct LogoView {
    hash: String,
    pages: usize,
    orders: usize,
    picture: Option<String>,
}

/// The owner's answer about one unseen logo (08 › *The logo prompt*); `kind` is the tag, as for
/// [`PlanEdit`]. *Skip all* never reaches Rust: it only stops the screen asking (D-LABEL-4).
#[derive(Deserialize, Debug)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum LogoAnswerInput {
    /// The logo is this courier's: name the orders whose tracking form agrees.
    Courier { name: String },
    /// The image is no courier's logo (a buying channel's strip).
    NotCourier,
    /// Say nothing about it (*I don't know*, and Esc): the orders stay unknown.
    Skip,
}

impl LogoAnswerInput {
    fn to_engine(&self) -> LogoAnswer {
        match self {
            LogoAnswerInput::Courier { name } => LogoAnswer::Courier(name.clone()),
            LogoAnswerInput::NotCourier => LogoAnswer::NotCourier,
            LogoAnswerInput::Skip => LogoAnswer::Skip,
        }
    }
}

/// The Plan screen's data (08 › *3. Plan*).
#[derive(Serialize)]
pub struct PlanView {
    rows: Vec<PlanRow>,
    /// Two saved PDFs of the batch share a name: *Save batch* waits (08 › *3. Plan*).
    has_duplicate_names: bool,
    /// Where *+ Add a pick for this batch* inserts, above the rest row (08 › *3. Plan*).
    add_at: usize,
}

/// One row of the Plan screen (08 › *3. Plan*).
#[derive(Serialize)]
pub struct PlanRow {
    /// The plan entry's index, for the edits; `None` for the generated `? Uncategorised` row.
    index: Option<usize>,
    /// The saved-PDF number this row will get; `None` when it takes no orders.
    number: Option<i64>,
    code: String,
    name: String,
    /// The condition text as written (under the row); `None` when the row has no condition.
    condition: Option<String>,
    orders: usize,
    runs: Vec<RunView>,
    pages: usize,
    file_name: String,
    /// Skipped for this batch only (08 › *3. Plan*: *Use* unticked).
    skipped: bool,
    /// The generated rest row (not a plan entry): no *Use* tick and no moving.
    generated: bool,
    /// The rest row: the last entry when it has no condition, else `? Uncategorised`.
    is_rest: bool,
    /// The `#` of the earlier row with the same file name (*Same name as PDF 6*).
    duplicate_of: Option<i64>,
    can_use: bool,
    can_move_up: bool,
    can_move_down: bool,
    /// Takes 0 orders: shown grey with *0 orders — no PDF* and no number.
    empty: bool,
}

/// One run inside a saved PDF, as the expanded row shows it (08 › *3. Plan*):
/// `6-01  Sepatu Standar Samping Motor ×1   96 orders`.
#[derive(Serialize)]
pub struct RunView {
    label: String,
    contents: String,
    orders: usize,
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

/// What *Save batch* wrote, for the green line on the Day screen (08 › *1. Day*).
#[derive(Serialize)]
pub struct SaveResultView {
    batch: i64,
    pdfs: usize,
    packing_lists: usize,
}

// ------------------------------------------------------------------------- the plan view

/// Map the plan and the draft's orders into the Plan screen's rows (08 › *3. Plan*). Every entry
/// becomes a row (skipped and 0-order rows included); the leftovers, if any, become the final
/// `? Uncategorised` row. The numbers, runs and names come from [`Plan::count`] so the screen and
/// the saved PDFs agree.
fn plan_view(plan: &Plan, orders: &[ReadOrder], first_number: i64) -> PlanView {
    let result = plan.count(orders, first_number);
    let entries = plan.entries();
    let index = order_index(orders);

    // The last entry when it has no condition is the rest row and stays last (08 › *3. Plan*).
    let rest_entry = match entries.last() {
        Some(last) if last.category.condition.is_none() => Some(entries.len() - 1),
        _ => None,
    };
    let add_at = rest_entry.unwrap_or(entries.len());

    let mut remaining: Vec<&ReadOrder> = orders.iter().collect();
    let mut picks = result.picks.iter();
    let mut rows: Vec<PlanRow> = Vec::with_capacity(entries.len() + 1);

    for (position, entry) in entries.iter().enumerate() {
        let is_rest = rest_entry == Some(position);
        let base =
            |number, orders: usize, runs, pages, file_name: String, empty, skipped| PlanRow {
                index: Some(position),
                number,
                code: entry.category.code.clone(),
                name: entry.category.name.clone(),
                condition: entry.category.when.clone(),
                orders,
                runs,
                pages,
                file_name,
                skipped,
                generated: false,
                is_rest,
                duplicate_of: None,
                can_use: true,
                can_move_up: position > 0 && !is_rest,
                can_move_down: position + 1 < entries.len()
                    && rest_entry != Some(position + 1)
                    && !is_rest,
                empty,
            };
        if entry.skipped {
            rows.push(base(None, 0, Vec::new(), 0, String::new(), false, true));
            continue;
        }
        let (taken, left) = split(&remaining, entry.category.condition.as_ref());
        remaining = left;
        if taken.is_empty() {
            rows.push(base(None, 0, Vec::new(), 0, String::new(), true, false));
        } else {
            let pick = picks.next().expect("a pick for every non-empty entry");
            rows.push(base(
                Some(pick.number),
                pick.orders.len(),
                run_views(pick, &index),
                pick.pages,
                pick.file_name.clone(),
                false,
                false,
            ));
        }
    }

    // The leftovers when no entry took them become `? Uncategorised` (08 › *3. Plan*).
    if !remaining.is_empty() {
        let pick = picks.next().expect("the rest pick");
        rows.push(PlanRow {
            index: None,
            number: Some(pick.number),
            code: pick.code.clone(),
            name: pick.name.clone(),
            condition: None,
            orders: pick.orders.len(),
            runs: run_views(pick, &index),
            pages: pick.pages,
            file_name: pick.file_name.clone(),
            skipped: false,
            generated: true,
            is_rest: true,
            duplicate_of: None,
            can_use: false,
            can_move_up: false,
            can_move_down: false,
            empty: false,
        });
    }

    // Duplicate saved-PDF names, case-insensitive as Windows is (08 › *3. Plan*).
    let mut seen: Vec<(String, i64)> = Vec::new();
    let mut has_duplicate_names = false;
    for row in &mut rows {
        let Some(number) = row.number else {
            continue;
        };
        let key = row.file_name.to_lowercase();
        if let Some((_, first)) = seen.iter().find(|(name, _)| *name == key) {
            row.duplicate_of = Some(*first);
            has_duplicate_names = true;
        } else {
            seen.push((key, number));
        }
    }

    PlanView {
        rows,
        has_duplicate_names,
        add_at,
    }
}

fn order_index(orders: &[ReadOrder]) -> HashMap<&str, &ReadOrder> {
    orders
        .iter()
        .map(|order| (order.order.order_id.as_str(), order))
        .collect()
}

/// Split the orders into the ones a condition takes and the ones left; `None` takes every order
/// (the rest entry). Mirrors the engine's `count` so the row counts match the saved PDFs.
fn split<'a>(
    orders: &[&'a ReadOrder],
    condition: Option<&Condition>,
) -> (Vec<&'a ReadOrder>, Vec<&'a ReadOrder>) {
    let mut taken = Vec::new();
    let mut left = Vec::new();
    for order in orders {
        let matches = match condition {
            None => true,
            Some(cond) => evaluate(cond, &order.order),
        };
        if matches {
            taken.push(*order);
        } else {
            left.push(*order);
        }
    }
    (taken, left)
}

/// A pick's runs as the expanded row shows them; the label is `<pick number>-<run number>`.
fn run_views(pick: &PickResult, index: &HashMap<&str, &ReadOrder>) -> Vec<RunView> {
    pick.runs
        .iter()
        .map(|run| RunView {
            label: format!("{}-{:02}", pick.number, run.number),
            contents: run
                .orders
                .first()
                .and_then(|id| index.get(id.as_str()))
                .map(|order| run_contents(&order.order))
                .unwrap_or_default(),
            orders: run.orders.len(),
        })
        .collect()
}

/// The identical contents of a run, as text: each line `display name ×quantity`.
fn run_contents(order: &Order) -> String {
    order
        .lines
        .iter()
        .map(|line| {
            format!(
                "{} ×{}",
                display_name(&line.name, &line.variation),
                line.quantity
            )
        })
        .collect::<Vec<_>>()
        .join(", ")
}

// ------------------------------------------------------------------------- the plan edits

/// Apply the window's edits to the open plan in order (08 › *3. Plan*). A bad condition text or a
/// bad file name stops with the engine's message (line/col for a condition).
fn apply_edits(plan: &mut Plan, edits: &[PlanEdit]) -> Result<(), String> {
    for edit in edits {
        match edit {
            PlanEdit::SetSkipped { index, skipped } => plan.set_skipped(*index, *skipped),
            PlanEdit::MoveUp { index } => plan.move_up(*index),
            PlanEdit::MoveDown { index } => plan.move_down(*index),
            PlanEdit::Add {
                at,
                code,
                name,
                condition,
            } => {
                let code = code.trim();
                if code.is_empty() {
                    return Err("the pick code must not be empty".to_string());
                }
                if name.trim().is_empty() {
                    return Err("the pick name must not be empty".to_string());
                }
                let condition = condition.trim();
                let node = parse_condition(condition).map_err(|error| error.to_string())?;
                let category = Category {
                    code: code.to_string(),
                    name: name.trim().to_string(),
                    condition: Some(node),
                    when: Some(condition.to_string()),
                };
                plan.add(*at, category)
            }
            PlanEdit::Rename { index, name } => plan.rename(*index, name),
        }
        .map_err(|error| error.to_string())?;
    }
    Ok(())
}

// ------------------------------------------------------------------------- the logo prompt

/// The screen's read result (08 › *2. New batch*, *The logo prompt*, *3. Plan*): the draft's
/// counts and warnings, the couriers the prompt's buttons show, the unseen logos still to ask
/// about, and the plan view. Used both when a read finishes and after every logo answer, so the
/// screen always shows the current counts and warnings.
fn read_result(open: &OpenDraft) -> ReadResult {
    ReadResult {
        files: open.draft.files.len(),
        pages: open.draft.pages,
        orders: open.draft.orders.len(),
        total_orders: open.draft.total_orders,
        already_saved: open.draft.duplicates.len(),
        warnings: open.draft.warnings.clone(),
        amend: open.amend.as_ref().map(|amend| amend.batch),
        couriers: courier_names(),
        logos: logo_views(&open.draft, &open.logo_pictures),
        plan: plan_view(&open.plan, &open.draft.orders, open.first_number),
    }
}

/// The courier names of `couriers.toml`, in file order: the prompt's buttons (D-LABEL-1, D-LABEL-4).
fn courier_names() -> Vec<String> {
    label::courier_data()
        .couriers
        .iter()
        .map(|courier| courier.name.clone())
        .collect()
}

/// The unseen logos as the prompt shows them, in first-seen order (D-LABEL-4).
fn logo_views(draft: &Draft, pictures: &HashMap<String, Option<String>>) -> Vec<LogoView> {
    draft
        .unseen_logos
        .iter()
        .map(|entry| LogoView {
            hash: entry.hash.clone(),
            pages: entry.pages,
            orders: entry.orders(),
            picture: pictures.get(&entry.hash).cloned().flatten(),
        })
        .collect()
}

/// Read each unseen logo back from the first page carrying it, as a PNG data URL for the prompt
/// (08 › *The logo prompt*). Runs on the PDFium worker, so the label documents stay on their
/// thread. A logo whose page cannot be found or rendered gets `None`: the screen shows its hash.
fn read_pictures(worker: &PdfWorker, draft: &Draft) -> HashMap<String, Option<String>> {
    let mut pictures = HashMap::with_capacity(draft.unseen_logos.len());
    for entry in &draft.unseen_logos {
        pictures.insert(entry.hash.clone(), first_picture(worker, draft, entry));
    }
    pictures
}

/// The logo as printed on the first page of the first order carrying it (the batch's kept orders,
/// so an already-saved order's page is not read).
fn first_picture(worker: &PdfWorker, draft: &Draft, entry: &UnseenLogo) -> Option<String> {
    for order in &draft.orders {
        if !entry.order_ids.iter().any(|id| id == &order.order.order_id) {
            continue;
        }
        for page in &order.pages {
            if let Ok(Some(png)) = worker.logo_picture(page.file, page.page, &entry.hash) {
                return Some(data_url(&png));
            }
        }
    }
    None
}

/// A PNG as a data URL (`data:image/png;base64,…`), for the prompt's `<img>`.
fn data_url(png: &[u8]) -> String {
    format!("data:image/png;base64,{}", base64(png))
}

/// Standard base64 (RFC 4648, with padding). The app has no base64 dependency and the value is
/// one capped PNG of a logo.
fn base64(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let b = [
            chunk[0],
            chunk.get(1).copied().unwrap_or(0),
            chunk.get(2).copied().unwrap_or(0),
        ];
        let n = (u32::from(b[0]) << 16) | (u32::from(b[1]) << 8) | u32::from(b[2]);
        out.push(ALPHABET[(n >> 18) as usize & 63] as char);
        out.push(ALPHABET[(n >> 12) as usize & 63] as char);
        out.push(if chunk.len() > 1 {
            ALPHABET[(n >> 6) as usize & 63] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            ALPHABET[n as usize & 63] as char
        } else {
            '='
        });
    }
    out
}

// ------------------------------------------------------------------------------ reading

/// Read the chosen files into a draft and store it (08 › *2. New batch* → *3. Plan*). Runs on a
/// blocking thread; `on_progress` is called for every page (the window turns it into a
/// `read-progress` event). A stop (no packing slip, every order already saved, a rules error,
/// cancelled) returns its message and writes nothing.
fn read_draft(
    state: &AppState,
    paths: Vec<PathBuf>,
    on_progress: impl Fn(Progress) + Send + 'static,
) -> Result<ReadResult, String> {
    let day_dir = state.day_dir()?;
    let day_state = DayState::load(&day_dir).map_err(|error| error.to_string())?;
    read_into_draft(state, paths, day_state, None, true, on_progress)
}

/// The shared read: `base_state` supplies the duplicate guard and the day's next number; `amend`
/// marks the draft as redoing a batch (07 › *Amend*); `sort_by_name` reads a new batch in
/// file-name order (= download order, 08 › *2. New batch*) and an amend in the given order (its
/// `download/` copy in the order stored in `state.json`).
fn read_into_draft(
    state: &AppState,
    paths: Vec<PathBuf>,
    base_state: DayState,
    amend: Option<AmendState>,
    sort_by_name: bool,
    on_progress: impl Fn(Progress) + Send + 'static,
) -> Result<ReadResult, String> {
    if paths.is_empty() {
        return Err("Choose at least one label file first.".to_string());
    }
    let folder = state.data_folder()?;
    let categories = load_categories(&folder)?;

    let mut files = paths;
    if sort_by_name {
        files.sort_by(|a, b| a.file_name().cmp(&b.file_name()));
    }

    let cancel = Arc::new(AtomicBool::new(false));
    state.set_cancel(cancel.clone());

    let (progress_tx, progress_rx) = mpsc::channel::<Progress>();
    let forwarder = thread::spawn(move || {
        while let Ok(progress) = progress_rx.recv() {
            on_progress(progress);
        }
    });

    let inner = state.with_worker(|worker| {
        batch::check_files(worker, &files, &base_state).and_then(|checks| {
            batch::read(worker, &checks, &base_state, cancel.clone(), progress_tx)
        })
    });

    state.clear_cancel();
    let _ = forwarder.join(); // the sender is dropped when `with_worker` returns

    let draft = inner
        .map_err(|error| error.to_string())?
        .map_err(|error| error.to_string())?;
    // The prompt's pictures are read back while the label documents are still open on the
    // worker (08 › *The logo prompt*); an empty list costs nothing.
    let logo_pictures = state
        .with_worker(|worker| read_pictures(worker, &draft))
        .unwrap_or_default();
    let plan = Plan::from_categories(&categories);
    let first_number = base_state.next_pdf_number();
    let open = OpenDraft {
        draft,
        plan,
        first_number,
        amend,
        logo_pictures,
    };
    let read = read_result(&open);
    state.set_draft(open);
    Ok(read)
}

// ----------------------------------------------------------------------- revert and amend

/// Revert the last batch (07 › *Revert*): delete its folder **first**, so a file open in a PDF
/// viewer stops with the 08 *file in use* message and changes nothing, then remove the batch and
/// its orders from `state.json`; the numbers are reused.
fn revert_last_impl(state: &AppState, day: &str) -> Result<(), String> {
    state.set_day(day);
    let day_dir = state.day_dir()?;
    let mut day_state = DayState::load(&day_dir).map_err(|error| error.to_string())?;
    let Some(last) = day_state.batches.iter().map(|batch| batch.batch).max() else {
        return Err("No batch to undo on this day.".to_string());
    };
    let folder = day_dir.join(names::batch_folder(last));
    // Name the open file (07 › *Revert*: "the app says which file to close") and delete nothing
    // when one is found, so the day is left exactly as it was.
    if let Some(file) = locked_file(&folder) {
        return Err(file_in_use(&file));
    }
    delete_batch_folder(&day_dir, last).map_err(|error| error.to_string())?;
    day_state.revert_last();
    day_state.save(&day_dir).map_err(|error| error.to_string())
}

/// The 08 *Messages* file-in-use wording, naming the file to close.
fn file_in_use(path: &Path) -> String {
    format!(
        "{} is open in another program. Close it (e.g. the PDF viewer) and try again.",
        path.display()
    )
}

/// The first file inside `folder` (and its subfolders) that another program holds open. A viewer
/// keeps a PDF with read-only sharing, so opening it for writing fails with a sharing violation
/// (`ERROR_SHARING_VIOLATION`, 32); that is the file the owner must close before Revert.
fn locked_file(folder: &Path) -> Option<PathBuf> {
    let mut stack = vec![folder.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if let Err(error) = std::fs::OpenOptions::new().write(true).open(&path)
                && error.raw_os_error() == Some(32)
            {
                return Some(path);
            }
        }
    }
    None
}

/// Amend the last batch (07 › *Amend*): read its `download/` copy again, in the order stored in
/// `state.json`, with the batch removed from the day state so the duplicate guard ignores its own
/// orders and the numbers are reused. Returns the plan for the *Redo batch `<b>`* screen.
fn start_amend_impl(state: &AppState, day: &str) -> Result<ReadResult, String> {
    state.set_day(day);
    let day_dir = state.day_dir()?;
    let day_state = DayState::load(&day_dir).map_err(|error| error.to_string())?;
    let amend = day_state
        .amend(&day_dir)
        .ok_or_else(|| "No batch to redo on this day.".to_string())?;
    let base_state = amend.state.clone();
    let amend_state = AmendState {
        batch: amend.batch,
        state: amend.state,
    };
    read_into_draft(
        state,
        amend.files,
        base_state,
        Some(amend_state),
        false,
        |_| {},
    )
}

/// Answer the logo prompt about one unseen logo (08 › *2. New batch* → *The logo prompt*;
/// D-LABEL-4): the order's courier is filled where the logo's courier and the tracking-ID form
/// agree, the draft's warnings and the plan view are recounted, and the refreshed result goes
/// back to the screen. An unknown hash changes nothing. *Skip all* never comes here: it only
/// stops the screen asking.
fn answer_logo_impl(
    state: &AppState,
    hash: &str,
    answer: &LogoAnswerInput,
) -> Result<ReadResult, String> {
    state.with_draft(|open| {
        open.draft.answer_logo(hash, &answer.to_engine());
        // The answered logo is spent; its picture is no longer needed.
        open.logo_pictures.remove(hash);
        read_result(open)
    })
}

/// Load `<data>/categories.toml` (08 › *Messages*: a rules error stops the batch with its
/// line/col).
fn load_categories(folder: &Path) -> Result<Vec<Category>, String> {
    let path = folder.join(CATEGORIES_FILE);
    let text = std::fs::read_to_string(&path)
        .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    load_rules(&text)
        .map_err(|error| format!("The categories have an error: {error}. Fix it in Categories."))
}

// ------------------------------------------------------------------------------- saving

/// Apply the plan and save the batch (07 › *What happens in one batch* step 6), then forget the
/// draft. A failed save keeps the draft so *Save batch* can be pressed again.
fn save_batch_impl(
    state: &AppState,
    packing: Option<PackingListChoice>,
) -> Result<SaveResultView, String> {
    let folder = state.data_folder()?;
    let day = state.day();
    let day_dir = state.day_dir()?;
    let packing = match packing {
        Some(choice) => choice,
        None => settings_load(&folder)?.packing_choice(),
    };
    let open = state
        .take_draft()
        .ok_or_else(|| "No batch is open.".to_string())?;
    // An amend saves into the state without its batch, so the numbers are reused and the batch is
    // replaced; a new batch reads the day's state from disk (07 › *Amend*).
    let mut day_state = match &open.amend {
        Some(amend) => amend.state.clone(),
        None => DayState::load(&day_dir).map_err(|error| error.to_string())?,
    };
    let plan = open.plan.count(&open.draft.orders, open.first_number);

    let result = state.with_worker(|worker| {
        batch::save(
            worker,
            &open.draft,
            &plan,
            &packing,
            &day,
            &day_dir,
            &mut day_state,
        )
    });

    match result {
        Ok(Ok(save)) => Ok(SaveResultView {
            batch: save.batch,
            pdfs: save.pdf_files.len(),
            packing_lists: save.packing_lists.len(),
        }),
        Ok(Err(error)) => {
            state.restore_draft(open);
            Err(error.to_string())
        }
        Err(message) => {
            state.restore_draft(open);
            Err(message)
        }
    }
}

// ----------------------------------------------------------------------------- helpers

fn file_check_view(check: FileCheck) -> FileCheckView {
    FileCheckView {
        path: check.path.display().to_string(),
        name: check.name,
        size: check.size,
        time: modified_time(&check.path),
        pages: check.pages,
        used_in_batch: check.used_in_batch,
    }
}

/// `HH:MM` of a file's modified time, or empty when it cannot be read.
fn modified_time(path: &Path) -> String {
    let Ok(metadata) = std::fs::metadata(path) else {
        return String::new();
    };
    let Ok(modified) = metadata.modified() else {
        return String::new();
    };
    let Ok(timestamp) = jiff::Timestamp::try_from(modified) else {
        return String::new();
    };
    timestamp
        .to_zoned(jiff::tz::TimeZone::system())
        .strftime("%H:%M")
        .to_string()
}

// ---------------------------------------------------------------------------- commands

/// *Add files…* / a drop: the page count, size, time and `used in batch n` of each chosen file
/// (08 › *2. New batch*). Needs only the page count, so it is quick.
#[tauri::command]
pub fn check_files(
    state: State<'_, AppState>,
    paths: Vec<String>,
) -> Result<Vec<FileCheckView>, String> {
    let paths: Vec<PathBuf> = paths.into_iter().map(PathBuf::from).collect();
    let day_dir = state.day_dir()?;
    let day_state = DayState::load(&day_dir).map_err(|error| error.to_string())?;
    let inner = state.with_worker(|worker| batch::check_files(worker, &paths, &day_state))?;
    let checks = inner.map_err(|error| error.to_string())?;
    Ok(checks.into_iter().map(file_check_view).collect())
}

/// *Read labels* (08 › *Reading*): read on a blocking thread, emitting `read-progress {page,
/// pages}`; returns the draft summary and the plan, or the stop error text.
#[tauri::command]
pub async fn read_labels(app: AppHandle, paths: Vec<String>) -> Result<ReadResult, String> {
    let paths: Vec<PathBuf> = paths.into_iter().map(PathBuf::from).collect();
    tauri::async_runtime::spawn_blocking(move || {
        let progress_app = app.clone();
        let state = app.state::<AppState>();
        read_draft(&state, paths, move |progress| {
            let _ = progress_app.emit(
                "read-progress",
                ReadProgress {
                    page: progress.page,
                    pages: progress.pages,
                },
            );
        })
    })
    .await
    .map_err(|error| format!("the reading task stopped: {error}"))?
}

/// *Cancel* (08 › *Reading*): stop the running read; nothing is written.
#[tauri::command]
pub fn cancel_read(state: State<'_, AppState>) {
    state.request_cancel();
}

/// Answer the logo prompt about one unseen logo (08 › *The logo prompt*; D-LABEL-4): applies the
/// answer to the open draft and returns the refreshed read result, so the screen shows the new
/// counts, warnings and remaining logos. *Skip all* is the screen's alone and calls nothing.
#[tauri::command]
pub fn answer_logo(
    state: State<'_, AppState>,
    hash: String,
    answer: LogoAnswerInput,
) -> Result<ReadResult, String> {
    answer_logo_impl(&state, &hash, &answer)
}

/// The live check of the *Add a pick* condition text (08 › *3. Plan*; the boxes editor is 2.12):
/// the engine's parser message with line/col, or nothing when it is valid.
#[tauri::command]
pub fn check_condition(condition: String) -> Result<(), String> {
    parse_condition(&condition)
        .map(|_| ())
        .map_err(|error| error.to_string())
}

/// Apply the plan edits and recount (08 › *3. Plan*: every change recounts all rows at once).
#[tauri::command]
pub fn plan(state: State<'_, AppState>, edits: Vec<PlanEdit>) -> Result<PlanView, String> {
    state.with_draft(|open| {
        apply_edits(&mut open.plan, &edits)?;
        let first_number = open.first_number;
        Ok(plan_view(&open.plan, &open.draft.orders, first_number))
    })?
}

/// *Save batch* (08 › *3. Plan* → *1. Day*): write the batch and forget the draft.
#[tauri::command]
pub fn save_batch(
    state: State<'_, AppState>,
    packing: Option<PackingListChoice>,
) -> Result<SaveResultView, String> {
    save_batch_impl(&state, packing)
}

/// *Cancel* on the Plan screen (08 › *3. Plan*): discard the open batch; nothing was saved. For an
/// amend this leaves the original batch untouched (07 › *Amend*).
#[tauri::command]
pub fn discard_draft(state: State<'_, AppState>) {
    state.clear_draft();
}

/// *Revert* the last batch (08 › *1. Day*): delete its folder and forget it. A file open in a PDF
/// viewer stops with the 08 *file in use* message and changes nothing.
#[tauri::command]
pub fn revert_last(state: State<'_, AppState>, day: String) -> Result<(), String> {
    revert_last_impl(&state, &day)
}

/// *Amend* the last batch (08 › *1. Day*): read its `download/` copy into a draft and open the
/// *Redo batch `<b>`* Plan screen. Runs on a blocking thread; no progress is shown.
#[tauri::command]
pub async fn start_amend(app: AppHandle, day: String) -> Result<ReadResult, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        start_amend_impl(&state, &day)
    })
    .await
    .map_err(|error| format!("the amend task stopped: {error}"))?
}

#[cfg(test)]
mod tests {
    use super::*;
    use packing_engine::orders::{Line, Warning};
    use packing_engine::plan::REST_CODE;

    fn line(name: &str, variation: &str, quantity: i64) -> Line {
        Line {
            name: name.to_string(),
            variation: variation.to_string(),
            seller_sku: String::new(),
            quantity,
            display_name: display_name(name, variation),
        }
    }

    fn read_order(id: &str, tracking: &str, lines: Vec<Line>) -> ReadOrder {
        ReadOrder {
            order: Order {
                order_id: id.to_string(),
                tracking_id: tracking.to_string(),
                courier: "J&T Express".to_string(),
                ship_by: None,
                lines,
                customer_message: String::new(),
            },
            pages: vec![packing_engine::orders::PageRef { file: 0, page: 0 }],
            position: 1,
            has_slip: true,
            qty_total: None,
        }
    }

    fn category(code: &str, name: &str, when: Option<&str>) -> Category {
        Category {
            code: code.to_string(),
            name: name.to_string(),
            condition: when.map(|text| parse_condition(text).expect("condition")),
            when: when.map(str::to_string),
        }
    }

    /// Three sepatu, two spion, one leftover; two categories with conditions so the leftovers
    /// become the generated rest row.
    fn orders() -> Vec<ReadOrder> {
        vec![
            read_order("o1", "t1", vec![line("Sepatu A", "", 1)]),
            read_order("o2", "t2", vec![line("Sepatu B", "", 1)]),
            read_order("o3", "t3", vec![line("Sepatu C", "", 1)]),
            read_order("o4", "t4", vec![line("Spion X", "", 1)]),
            read_order("o5", "t5", vec![line("Spion Y", "", 1)]),
            read_order("o6", "t6", vec![line("Lain", "", 1)]),
        ]
    }

    fn sepatu_plan() -> Plan {
        Plan::from_categories(&[
            category("A", "Sepatu", Some("name contains \"sepatu\"")),
            category("B", "Spion", Some("name contains \"spion\"")),
        ])
    }

    #[test]
    fn plan_view_numbers_rows_and_adds_the_rest_row() {
        let view = plan_view(&sepatu_plan(), &orders(), 4);
        assert_eq!(view.rows.len(), 3);
        assert_eq!(view.add_at, 2); // after the two entries (no rest entry)

        let a = &view.rows[0];
        assert_eq!(a.number, Some(4));
        assert_eq!(a.code, "A");
        assert_eq!(a.orders, 3);
        assert_eq!(a.runs.len(), 3); // three different contents, one order each
        assert_eq!(a.runs[0].label, "4-01");
        assert_eq!(a.runs[0].orders, 1);
        assert_eq!(a.runs[0].contents, "Sepatu A ×1");
        assert_eq!(a.runs[2].label, "4-03");
        assert_eq!(a.file_name, "4 A Sepatu ×3.pdf");
        assert!(!a.empty && !a.is_rest && a.can_use);
        assert!(!a.can_move_up && a.can_move_down); // first row

        let b = &view.rows[1];
        assert_eq!(b.number, Some(5));
        assert_eq!(b.orders, 2);
        assert_eq!(b.condition.as_deref(), Some("name contains \"spion\""));

        let rest = &view.rows[2];
        assert!(rest.generated && rest.is_rest && !rest.can_use);
        assert_eq!(rest.index, None);
        assert_eq!(rest.number, Some(6));
        assert_eq!(rest.code, REST_CODE);
        assert_eq!(rest.name, "Uncategorised");
        assert_eq!(rest.orders, 1);
        assert!(!rest.can_move_up && !rest.can_move_down);

        assert!(!view.has_duplicate_names);
    }

    #[test]
    fn plan_view_shows_zero_order_and_skipped_rows() {
        let mut plan = sepatu_plan();
        // Untick the sepatu row: its three orders fall to the rest row.
        apply_edits(
            &mut plan,
            &[PlanEdit::SetSkipped {
                index: 0,
                skipped: true,
            }],
        )
        .expect("skip");
        let view = plan_view(&plan, &orders(), 1);

        assert!(view.rows[0].skipped);
        assert_eq!(view.rows[0].orders, 0);
        assert_eq!(view.rows[0].number, None);
        // Spion takes 2; the rest row takes the 3 sepatu + 1 lain.
        assert_eq!(view.rows[1].orders, 2);
        assert_eq!(view.rows[2].orders, 4);
        assert_eq!(view.rows[2].number, Some(2)); // the skipped row used no number

        // A condition that matches nothing shows a 0-order row with no number.
        let plan = Plan::from_categories(&[
            category("A", "Nope", Some("name contains \"zzz\"")),
            category("Z", "Lainnya", None),
        ]);
        let view = plan_view(&plan, &orders(), 1);
        assert!(view.rows[0].empty);
        assert_eq!(view.rows[0].number, None);
        assert_eq!(view.rows[1].number, Some(1)); // the rest entry catches everything
        assert!(view.rows[1].is_rest);
        assert_eq!(view.add_at, 1); // before the rest entry
    }

    #[test]
    fn plan_view_flags_duplicate_file_names() {
        let mut plan = sepatu_plan();
        apply_edits(
            &mut plan,
            &[
                PlanEdit::Rename {
                    index: 0,
                    name: "same.pdf".to_string(),
                },
                PlanEdit::Rename {
                    index: 1,
                    name: "Same.pdf".to_string(),
                },
            ],
        )
        .expect("rename");
        let view = plan_view(&plan, &orders(), 1);
        assert!(view.has_duplicate_names);
        assert_eq!(view.rows[0].duplicate_of, None);
        assert_eq!(view.rows[1].duplicate_of, Some(1)); // *Same name as PDF 1*
    }

    #[test]
    fn apply_edits_moves_adds_and_reports_condition_errors() {
        let mut plan = sepatu_plan();
        apply_edits(&mut plan, &[PlanEdit::MoveDown { index: 0 }]).expect("move");
        assert_eq!(plan.entries()[0].category.code, "B");
        apply_edits(&mut plan, &[PlanEdit::MoveUp { index: 1 }]).expect("move back");
        assert_eq!(plan.entries()[0].category.code, "A");

        apply_edits(
            &mut plan,
            &[PlanEdit::Add {
                at: 2,
                code: "X".to_string(),
                name: "One off".to_string(),
                condition: "total_quantity >= 2".to_string(),
            }],
        )
        .expect("add");
        assert_eq!(plan.len(), 3);
        assert_eq!(plan.entries()[2].category.code, "X");
        assert_eq!(
            plan.entries()[2].category.when.as_deref(),
            Some("total_quantity >= 2")
        );

        let error = apply_edits(
            &mut plan,
            &[PlanEdit::Add {
                at: 3,
                code: "Y".to_string(),
                name: "Bad".to_string(),
                condition: "name contains".to_string(),
            }],
        )
        .expect_err("bad condition");
        assert!(error.contains("line 1, col"), "{error}");

        let error = apply_edits(
            &mut plan,
            &[PlanEdit::Rename {
                index: 0,
                name: "   ".to_string(),
            }],
        )
        .expect_err("empty name");
        assert!(error.contains("empty"), "{error}");
    }

    #[test]
    fn check_condition_reports_the_engine_message() {
        assert!(check_condition("name contains \"x\"".to_string()).is_ok());
        let error = check_condition("name contains".to_string()).expect_err("bad");
        assert!(error.starts_with("line 1, col "), "{error}");
    }

    // ------------------------------------------------------ the logo prompt (2.16, D-LABEL-4)

    /// A state holding a draft of two orders the courier data could not name — one `JY…` (J&T
    /// Express alone), one `00…` (shared with SiCepat) — both carrying one unseen logo, with a
    /// plan whose first pick takes the courier `J&T Express`.
    fn state_with_logo_draft(folder: &Path) -> AppState {
        let unknown = |id: &str, tracking: &str, name: &str| {
            let mut order = read_order(id, tracking, vec![line(name, "", 1)]);
            order.order.courier = String::new();
            order
        };
        let mut draft = Draft {
            files: Vec::new(),
            orders: vec![
                unknown("580000000000000001", "JY0000001234", "Sepatu A"),
                unknown("580000000000000002", "000000000202", "Spion X"),
            ],
            duplicates: Vec::new(),
            warnings: Vec::new(),
            pages: 2,
            total_orders: 2,
            slip_pages: 2,
            unseen_logos: vec![UnseenLogo {
                hash: "ffff".to_string(),
                appearance: None,
                pages: 2,
                order_ids: vec![
                    "580000000000000001".to_string(),
                    "580000000000000002".to_string(),
                ],
            }],
            read_warnings: Vec::new(),
        };
        draft.read_warnings = vec![Warning::unknown_courier(&draft.orders).expect("both unknown")];
        draft.warnings = draft
            .read_warnings
            .iter()
            .map(|warning| warning.to_string())
            .collect();

        let state = AppState::new();
        state.set_data_folder(folder.to_path_buf());
        state.set_day("2026-10-07");
        state.set_draft(OpenDraft {
            draft,
            plan: Plan::from_categories(&[category(
                "A",
                "J&T",
                Some("courier starts_with \"J&T\""),
            )]),
            first_number: 1,
            amend: None,
            logo_pictures: HashMap::from([(
                "ffff".to_string(),
                Some("data:image/png;base64,Zm9v".to_string()),
            )]),
        });
        state
    }

    #[test]
    fn base64_encodes_with_padding() {
        assert_eq!(base64(b""), "");
        assert_eq!(base64(b"f"), "Zg==");
        assert_eq!(base64(b"fo"), "Zm8=");
        assert_eq!(base64(b"foo"), "Zm9v");
        assert_eq!(base64(b"foob"), "Zm9vYg==");
        assert_eq!(base64(b"fooba"), "Zm9vYmE=");
        assert_eq!(base64(b"foobar"), "Zm9vYmFy");
        assert_eq!(data_url(b"foo"), "data:image/png;base64,Zm9v");
    }

    #[test]
    fn answer_logo_names_the_form_agreeing_orders_and_recounts_the_plan() {
        let dir = temp_dir("logo-answer");
        let state = state_with_logo_draft(&dir);

        // Before the answer: the prompt's logo, the courier buttons, a pick with no orders.
        let before = state.with_draft(|open| read_result(open)).expect("draft");
        assert_eq!(before.logos.len(), 1);
        assert_eq!(before.logos[0].hash, "ffff");
        assert_eq!(before.logos[0].pages, 2);
        assert_eq!(before.logos[0].orders, 2);
        assert_eq!(
            before.logos[0].picture.as_deref(),
            Some("data:image/png;base64,Zm9v")
        );
        assert_eq!(
            before.couriers,
            vec!["J&T Express", "SiCepat", "J&T Cargo", "IDX", "SPX"]
        );
        assert_eq!(before.plan.rows[0].orders, 0);
        assert_eq!(before.plan.rows[0].number, None);
        assert_eq!(before.plan.rows[1].orders, 2); // the rest row takes both
        assert!(before.warnings[0].contains("Courier unknown for 2 orders"));

        // The JY form names J&T Express; the shared 00 form does not.
        let after = answer_logo_impl(
            &state,
            "ffff",
            &LogoAnswerInput::Courier {
                name: "J&T Express".to_string(),
            },
        )
        .expect("answer");
        assert!(after.logos.is_empty(), "the answered logo is spent");
        assert_eq!(after.plan.rows[0].orders, 1); // the courier pick takes the JY order
        assert_eq!(after.plan.rows[0].number, Some(1));
        assert_eq!(after.plan.rows[1].orders, 1);
        assert_eq!(after.plan.rows[1].number, Some(2));
        assert_eq!(after.orders, before.orders); // no order left or joined the batch
        assert!(after.warnings[0].contains("Courier unknown for 1 order: 000000000202"));

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn skip_and_not_a_courier_leave_the_orders_unknown() {
        for answer in [LogoAnswerInput::Skip, LogoAnswerInput::NotCourier] {
            let dir = temp_dir("logo-skip");
            let state = state_with_logo_draft(&dir);
            let result = answer_logo_impl(&state, "ffff", &answer).expect("answer");

            assert!(result.logos.is_empty(), "the answered logo is spent");
            assert_eq!(result.plan.rows[0].orders, 0);
            assert!(result.warnings[0].contains("Courier unknown for 2 orders"));
            // Nothing was applied: the orders still have no courier.
            state
                .with_draft(|open| {
                    assert_eq!(open.draft.orders[0].order.courier, "");
                    assert_eq!(open.draft.orders[1].order.courier, "");
                })
                .expect("draft");

            std::fs::remove_dir_all(&dir).ok();
        }
    }

    #[test]
    fn an_unknown_hash_changes_nothing() {
        let dir = temp_dir("logo-unknown");
        let state = state_with_logo_draft(&dir);
        let result = answer_logo_impl(
            &state,
            "0123456789abcdef",
            &LogoAnswerInput::Courier {
                name: "SiCepat".to_string(),
            },
        )
        .expect("answer");
        assert_eq!(result.logos.len(), 1, "the logo is still asked about");
        assert_eq!(result.plan.rows[0].orders, 0);
        std::fs::remove_dir_all(&dir).ok();
    }

    // ------------------------------------------------------------------ PDF-backed (testdata)

    /// The pinned `pdfium.dll` directory, or `None` when it has not been downloaded.
    fn vendor_dir() -> Option<PathBuf> {
        let vendor = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../vendor");
        vendor.join("pdfium.dll").exists().then_some(vendor)
    }

    /// `testdata/` at the repository root.
    fn testdata(name: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../testdata")
            .join(name)
    }

    fn temp_dir(label: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("packing-app-batch-{}-{label}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("temp dir");
        dir
    }

    fn app_state(folder: &Path) -> AppState {
        let state = AppState::new();
        state.set_data_folder(folder.to_path_buf());
        state.set_day("2026-10-07");
        assert!(state.ensure_worker(), "run tools/get-pdfium.sh");
        state
    }

    #[test]
    fn read_and_save_testdata_labels_slip() {
        let Some(_vendor) = vendor_dir() else {
            eprintln!("skipped: run tools/get-pdfium.sh");
            return;
        };
        let dir = temp_dir("slip");
        std::fs::copy(testdata("categories.toml"), dir.join(CATEGORIES_FILE)).expect("copy rules");
        let state = app_state(&dir);

        let read = read_draft(&state, vec![testdata("labels-slip.pdf")], |_| {}).expect("read");
        assert_eq!(read.orders, 23);
        assert_eq!(read.total_orders, 23);
        assert_eq!(read.already_saved, 0);
        assert_eq!(read.pages, 24);
        let numbered: Vec<(i64, usize)> = read
            .plan
            .rows
            .iter()
            .filter_map(|row| row.number.map(|number| (number, row.orders)))
            .collect();
        assert_eq!(numbered, vec![(1, 7), (2, 8), (3, 4), (4, 2), (5, 2)]);

        let save = save_batch_impl(&state, None).expect("save");
        assert_eq!(save.batch, 1);
        assert_eq!(save.pdfs, 5);
        assert_eq!(save.packing_lists, 1);
        assert!(dir.join("labels/2026-10-07/state.json").is_file());

        // A second read of the same file: every order was already saved today.
        let again = read_draft(&state, vec![testdata("labels-slip.pdf")], |_| {})
            .err()
            .expect("already saved");
        assert!(again.contains("already saved today"), "{again}");

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn read_plain_labels_stops_with_no_slip() {
        let Some(_vendor) = vendor_dir() else {
            eprintln!("skipped: run tools/get-pdfium.sh");
            return;
        };
        let dir = temp_dir("plain");
        std::fs::copy(testdata("categories.toml"), dir.join(CATEGORIES_FILE)).expect("copy rules");
        let state = app_state(&dir);

        let error = read_draft(&state, vec![testdata("labels-plain.pdf")], |_| {})
            .err()
            .expect("no slip");
        assert!(error.contains("no packing slip"), "{error}");

        std::fs::remove_dir_all(&dir).ok();
    }

    // ------------------------------------------------------------------ revert / amend (2.11)

    #[test]
    fn revert_last_deletes_the_folder_and_the_state() {
        let Some(_vendor) = vendor_dir() else {
            eprintln!("skipped: run tools/get-pdfium.sh");
            return;
        };
        let dir = temp_dir("revert");
        std::fs::copy(testdata("categories.toml"), dir.join(CATEGORIES_FILE)).expect("copy rules");
        let state = app_state(&dir);

        read_draft(&state, vec![testdata("labels-slip.pdf")], |_| {}).expect("read");
        let save = save_batch_impl(&state, None).expect("save");
        assert_eq!(save.batch, 1);
        let folder = dir.join("labels/2026-10-07/batch 1");
        assert!(folder.is_dir());

        revert_last_impl(&state, "2026-10-07").expect("revert");
        assert!(!folder.exists(), "the batch folder is gone");
        let day_state = DayState::load(&dir.join("labels/2026-10-07")).expect("state");
        assert!(day_state.batches.is_empty());
        assert!(day_state.saved.is_empty());

        // The numbers are reused: the next batch is batch 1 again.
        read_draft(&state, vec![testdata("labels-slip.pdf")], |_| {}).expect("read again");
        let save = save_batch_impl(&state, None).expect("save again");
        assert_eq!(save.batch, 1);

        // No batch to undo on an untouched day.
        assert!(revert_last_impl(&state, "2026-10-08").is_err());

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn amend_reads_the_download_copy_and_replaces_the_batch() {
        let Some(_vendor) = vendor_dir() else {
            eprintln!("skipped: run tools/get-pdfium.sh");
            return;
        };
        let dir = temp_dir("amend");
        std::fs::copy(testdata("categories.toml"), dir.join(CATEGORIES_FILE)).expect("copy rules");
        let state = app_state(&dir);

        read_draft(&state, vec![testdata("labels-slip.pdf")], |_| {}).expect("read");
        assert_eq!(save_batch_impl(&state, None).expect("save").batch, 1);
        let folder = dir.join("labels/2026-10-07/batch 1");
        assert!(folder.join("download").is_dir());

        // Amend reads the batch's own download copy; its orders are not seen as already saved.
        let read = start_amend_impl(&state, "2026-10-07").expect("amend");
        assert_eq!(read.amend, Some(1));
        assert_eq!(read.orders, 23);
        assert_eq!(read.already_saved, 0);

        // Saving again replaces batch 1 (same number, one batch in the state).
        assert_eq!(save_batch_impl(&state, None).expect("save again").batch, 1);
        let day_state = DayState::load(&dir.join("labels/2026-10-07")).expect("state");
        assert_eq!(day_state.batches.len(), 1);
        assert_eq!(day_state.batches[0].batch, 1);
        assert!(folder.is_dir());

        // Cancel leaves the old batch untouched.
        start_amend_impl(&state, "2026-10-07").expect("amend again");
        state.clear_draft();
        assert!(folder.is_dir());
        assert_eq!(
            DayState::load(&dir.join("labels/2026-10-07"))
                .expect("state")
                .batches
                .len(),
            1
        );

        std::fs::remove_dir_all(&dir).ok();
    }

    // ------------------------------------------------- the logo prompt on real (testdata) pages

    #[test]
    fn the_testdata_labels_show_no_logo_prompt_and_list_the_couriers() {
        let Some(_vendor) = vendor_dir() else {
            eprintln!("skipped: run tools/get-pdfium.sh");
            return;
        };
        let dir = temp_dir("logo-prompt");
        std::fs::copy(testdata("categories.toml"), dir.join(CATEGORIES_FILE)).expect("copy rules");
        let state = app_state(&dir);

        // The testdata labels carry no images, so every logo is unknown-and-absent: the prompt
        // never opens, and the buttons still carry `couriers.toml`'s names in file order.
        let read = read_draft(&state, vec![testdata("labels-slip.pdf")], |_| {}).expect("read");
        assert!(read.logos.is_empty(), "the fpdf2 fixtures have no images");
        assert_eq!(
            read.couriers,
            vec!["J&T Express", "SiCepat", "J&T Cargo", "IDX", "SPX"]
        );

        // An amend read goes through the same path: it too ends with no prompt and the couriers.
        assert_eq!(save_batch_impl(&state, None).expect("save").batch, 1);
        let amend = start_amend_impl(&state, "2026-10-07").expect("amend");
        assert_eq!(amend.amend, Some(1));
        assert!(amend.logos.is_empty());
        assert_eq!(amend.couriers, read.couriers);

        // Answering a hash no logo carries changes nothing and still returns a full result.
        let again =
            answer_logo_impl(&state, "ffffffffffffffff", &LogoAnswerInput::Skip).expect("answer");
        assert_eq!(again.orders, amend.orders);
        assert!(again.logos.is_empty());

        std::fs::remove_dir_all(&dir).ok();
    }
}
