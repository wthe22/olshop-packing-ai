//! Start, Day and Settings commands: finding the data
//! folder, the day overview from `state.json`, the settings and *About*.

use std::path::{Path, PathBuf};
use std::process::Command;

use packing_engine::day::{DayState, Layout, PackingListChoice, PdfEntry, Scope};
use packing_engine::groupings::Grouping;
use packing_engine::ids::{BatchNumber, PdfNumber};
use packing_engine::names;
use packing_pdf::PDFIUM_VERSION;
use serde::Serialize;
use tauri::State;

use crate::{AppState, BUILTIN_COURIERS, COURIERS_FILE, LABELS_DIR, program_folder, today};

mod settings;

pub(crate) use self::settings::{
    SettingsView, remember_last_grouping, settings_load, settings_save,
};

/// The program's name as the window title and the title bar show it.
pub(crate) const PROGRAM_NAME: &str = "Packing";

/// The shown form of the program's version: `v` + the first
/// two numbers of the workspace `version` (`0.1.0` → `v0.1`). `CARGO_PKG_VERSION` carries the
/// workspace version, because the app inherits it (`version.workspace = true` in
/// `app/src-tauri/Cargo.toml`), so the root `Cargo.toml` is the one place. The window title, the
/// title bar and *About* all take their text from this one function, so the three cannot differ.
pub(crate) fn version_text() -> String {
    let version = env!("CARGO_PKG_VERSION");
    let mut numbers = version.split('.');
    let major = numbers.next().unwrap_or_default();
    let minor = numbers.next().unwrap_or_default();
    format!("v{major}.{minor}")
}

/// The OS window title and the title bar's brand text: the program's name
/// and [`version_text`]. [`crate::run`] sets the window title with it at start; the title bar
/// shows the same line, from `StartInfo.version`.
pub(crate) fn window_title() -> String {
    format!("{PROGRAM_NAME} {}", version_text())
}

/// What the window needs to open the Start prompt or the Day screen.
#[derive(Serialize)]
pub struct StartInfo {
    /// Where the program's exe is.
    exe_dir: String,
    /// Whether `couriers.toml` was found next to the program.
    program_couriers: bool,
    /// The data folder in use, if one is set.
    data_folder: Option<String>,
    /// Whether the PDFium worker started (its dll was found).
    pdfium_ready: bool,
    /// Today's date on the PC (`YYYY-MM-DD`); the Day screen opens here.
    today: String,
    /// The shown version ([`version_text`]); the title bar's brand text is the program's name and
    /// this.
    version: String,
}

/// Whether `folder` is a data folder: it holds `couriers.toml`. `categories.toml` is not a marker: the app writes it only when the owner
/// saves picks.
fn is_data_folder(folder: &Path) -> bool {
    folder.join(COURIERS_FILE).is_file()
}

fn start_information(state: &AppState) -> StartInfo {
    let exe_dir = program_folder();
    let program_couriers = is_data_folder(&exe_dir);
    if program_couriers && state.data_folder().is_err() {
        // Found → that folder is the data folder.
        state.set_data_folder(exe_dir.clone());
    }
    let day = today();
    state.set_day(&day);
    StartInfo {
        exe_dir: exe_dir.display().to_string(),
        program_couriers,
        data_folder: state
            .data_folder()
            .ok()
            .map(|path| path.display().to_string()),
        pdfium_ready: state.ensure_worker(),
        today: day,
        version: version_text(),
    }
}

/// Put the built-in `couriers.toml` into `folder` when it has none.
/// An existing file — the owner's, or one the logo prompt wrote to — is never overwritten.
fn ensure_builtin_couriers(folder: &Path) -> Result<(), String> {
    let path = folder.join(COURIERS_FILE);
    if path.is_file() {
        return Ok(());
    }
    std::fs::write(&path, BUILTIN_COURIERS)
        .map_err(|error| format!("Cannot write to {}: {error}.", folder.display()))
}

/// *Use this folder*: write the built-in `couriers.toml` next to the program and make that
/// folder the data folder. No picks are written: a folder without
/// `categories.toml` uses the one catch-all entry. Returns the folder.
fn activate_program_folder(state: &AppState) -> Result<String, String> {
    let folder = program_folder();
    ensure_builtin_couriers(&folder)?;
    state.set_data_folder(folder.clone());
    Ok(folder.display().to_string())
}

/// *Choose a folder…* / Settings *Change…*: use `path` until the app closes; a folder without a
/// `couriers.toml` gets the built-in one, and its own `categories.toml`, if any, is kept as it
/// is. Returns the folder.
fn use_chosen_folder(state: &AppState, path: &str) -> Result<String, String> {
    let folder = PathBuf::from(path);
    ensure_builtin_couriers(&folder)?;
    state.set_data_folder(folder.clone());
    Ok(folder.display().to_string())
}

/// The Day screen's view of one saved PDF.
#[derive(Serialize)]
pub struct PdfView {
    number: PdfNumber,
    code: String,
    name: String,
    file: String,
    orders: usize,
    runs: usize,
    /// The row's **Printed** tick, from the batch's `printed` list: a note only.
    printed: bool,
}

/// The Day screen's view of one batch.
#[derive(Serialize)]
pub struct BatchView {
    batch: BatchNumber,
    save_time: String,
    /// `HH:MM` of `save_time`.
    time: String,
    /// The grouping the batch was packed with, from its `state.json` entry. **Amend** names it in its confirmation and reads that grouping again.
    grouping: Grouping,
    files: usize,
    orders: usize,
    pdfs: Vec<PdfView>,
    /// The batch's packing-list files, for *Open* (usually one).
    packing_lists: Vec<String>,
    /// The read warnings of the batch, for *Warnings ▸*.
    warnings: Vec<String>,
    /// The check before saving's errors, for *Saved with `<n>` errors ▸*; empty
    /// unless the batch was saved anyway.
    errors: Vec<String>,
    /// The batch folder, for *Open folder*.
    folder: String,
}

/// The day's totals.
#[derive(Serialize)]
pub struct Totals {
    batches: usize,
    orders: usize,
    pdfs: usize,
}

/// The Day screen's data: the day's batches, newest first.
#[derive(Serialize)]
pub struct DayOverview {
    day: String,
    batches: Vec<BatchView>,
    totals: Totals,
}

/// Read `<data>/labels/<day>/state.json`. A missing day folder is an empty
/// overview.
fn day_overview_at(data_folder: &Path, day: &str) -> Result<DayOverview, String> {
    let day_dir = data_folder.join(LABELS_DIR).join(day);
    let state = DayState::load(&day_dir).map_err(|error| error.to_string())?;
    Ok(overview_of(&state, &day_dir))
}

/// Write one saved-PDF row's **Printed** tick into the day's `state.json`; a batch or saved-PDF number the day does not hold changes nothing.
fn set_printed_time(
    data_folder: &Path,
    day: &str,
    batch: BatchNumber,
    pdf: PdfNumber,
    printed: bool,
) -> Result<(), String> {
    let day_dir = data_folder.join(LABELS_DIR).join(day);
    let mut state = DayState::load(&day_dir).map_err(|error| error.to_string())?;
    state
        .set_printed(&day_dir, batch, pdf, printed)
        .map_err(|error| error.to_string())
}

fn overview_of(state: &DayState, day_dir: &Path) -> DayOverview {
    let mut batches: Vec<BatchView> = state
        .batches
        .iter()
        .map(|batch| BatchView {
            batch: batch.batch,
            save_time: batch.save_time.clone(),
            time: clock(&batch.save_time),
            grouping: batch.grouping.clone(),
            files: batch.files.len(),
            orders: batch.pdfs.iter().map(|pdf| pdf.orders).sum(),
            pdfs: batch
                .pdfs
                .iter()
                .map(|pdf| PdfView {
                    number: pdf.number,
                    code: pdf.code.clone(),
                    name: pdf.name.clone(),
                    file: pdf.file.clone(),
                    orders: pdf.orders,
                    runs: pdf.runs,
                    printed: batch.is_printed(pdf.number),
                })
                .collect(),
            packing_lists: packing_list_files(&batch.packing_list, &batch.pdfs),
            warnings: batch.warnings.clone(),
            errors: batch.errors.clone(),
            folder: day_dir
                .join(names::batch_folder(batch.batch))
                .display()
                .to_string(),
        })
        .collect();
    batches.reverse(); // newest on top
    let totals = Totals {
        batches: batches.len(),
        orders: batches.iter().map(|batch| batch.orders).sum(),
        pdfs: batches.iter().map(|batch| batch.pdfs.len()).sum(),
    };
    DayOverview {
        day: state.day.clone(),
        batches,
        totals,
    }
}

/// The day folders to switch between, newest first. Search's *All days*
/// reads the same list.
pub(crate) fn days_in(data_folder: &Path) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(data_folder.join(LABELS_DIR)) else {
        return Vec::new();
    };
    let mut days: Vec<String> = entries
        .flatten()
        .filter(|entry| entry.file_type().map(|kind| kind.is_dir()).unwrap_or(false))
        .filter_map(|entry| entry.file_name().into_string().ok())
        .collect();
    days.sort();
    days.reverse();
    days
}

/// `HH:MM` of an ISO save time (`2026-10-07T07:40:12+07:00` → `07:40`).
fn clock(save_time: &str) -> String {
    save_time.chars().skip(11).take(5).collect()
}

/// The packing-list files a batch wrote, by its scope and layouts. The
/// layout is part of the name only when several layouts were written.
fn packing_list_files(choice: &PackingListChoice, pdfs: &[PdfEntry]) -> Vec<String> {
    let several = choice.layouts.len() > 1;
    let name = |number: Option<PdfNumber>, layout: Layout| {
        names::packing_list_file_name(number, several.then(|| layout.as_str()))
    };
    match choice.scope {
        Scope::None => Vec::new(),
        Scope::Whole => choice
            .layouts
            .iter()
            .map(|layout| name(None, *layout))
            .collect(),
        Scope::PerPdf => pdfs
            .iter()
            .flat_map(|pdf| {
                choice
                    .layouts
                    .iter()
                    .map(move |layout| name(Some(pdf.number), *layout))
            })
            .collect(),
    }
}

/// Settings › *About*.
#[derive(Serialize)]
pub struct About {
    version: String,
    pdfium: String,
}

/// The Explorer command for `path` (separate so a test can read its arguments back).
fn explorer_command(path: &str) -> Command {
    let mut command = Command::new("explorer");
    command.arg(path);
    command
}

/// Open a file with the PC's default program, or a folder as a folder in Explorer.
///
/// The path is Explorer's **only** argument. Explorer parses `/select,<path>` itself and does not
/// understand the quotes Rust puts around an argument that holds a space, so it opened *Documents*
/// (or *Home*) instead of the folder (the batch folder always has a space in its
/// path, e.g. `…\labels\2026-10-11\batch 1`). A bare path works for both: a folder opens as a
/// folder, a file opens in its default program.
fn open_with_explorer(path: &str) -> Result<(), String> {
    explorer_command(path)
        .spawn()
        .map(|_| ())
        .map_err(|error| format!("Cannot open {path}: {error}."))
}

/// Where the program is, whether `categories.toml` was found next to it, the data folder in
/// use and today's date.
#[tauri::command]
pub fn start_info(state: State<'_, AppState>) -> StartInfo {
    start_information(&state)
}

/// *Use this folder*: write the built-in `categories.toml` next to the program.
#[tauri::command]
pub fn use_program_folder(state: State<'_, AppState>) -> Result<String, String> {
    activate_program_folder(&state)
}

/// *Choose a folder…* / Settings *Change…*: use the folder until the app closes.
#[tauri::command]
pub fn choose_data_folder(state: State<'_, AppState>, path: String) -> Result<String, String> {
    use_chosen_folder(&state, &path)
}

/// The day's batches, saved PDFs, warnings and totals, from `state.json`.
#[tauri::command]
pub fn day_overview(state: State<'_, AppState>, day: String) -> Result<DayOverview, String> {
    state.set_day(&day); // a new batch saves into the day being shown
    day_overview_at(&state.data_folder()?, &day)
}

/// The **Printed** tick of one saved-PDF row of a batch:
/// writes the day's `state.json`; it changes nothing else.
#[tauri::command]
pub fn set_printed(
    state: State<'_, AppState>,
    day: String,
    batch: BatchNumber,
    pdf: PdfNumber,
    printed: bool,
) -> Result<(), String> {
    state.set_day(&day);
    set_printed_time(&state.data_folder()?, &day, batch, pdf, printed)
}

/// The day folders to switch between.
#[tauri::command]
pub fn days(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    Ok(days_in(&state.data_folder()?))
}

/// The settings.
#[tauri::command]
pub fn settings_get(state: State<'_, AppState>) -> Result<SettingsView, String> {
    Ok(SettingsView::of(settings_load(&state.data_folder()?)?))
}

/// Save the settings.
#[tauri::command]
pub fn settings_set(state: State<'_, AppState>, settings: SettingsView) -> Result<(), String> {
    settings_save(&state.data_folder()?, &settings.into_settings())
}

/// Settings › *About*: the app version and the PDFium version. The version
/// is the shown form from [`version_text`], the one place.
#[tauri::command]
pub fn about() -> About {
    About {
        version: version_text(),
        pdfium: PDFIUM_VERSION.to_string(),
    }
}

/// Open a file in the default viewer.
#[tauri::command]
pub fn open_path(path: String) -> Result<(), String> {
    open_with_explorer(&path)
}

/// Open a folder in Explorer.
#[tauri::command]
pub fn show_in_folder(path: String) -> Result<(), String> {
    open_with_explorer(&path)
}

#[cfg(test)]
mod tests;
