//! Start, Day and Settings commands (08 › *Start*, *1. Day*, *6. Settings*): finding the data
//! folder, the day overview from `state.json`, the settings and *About*.

use std::path::{Path, PathBuf};
use std::process::Command;

use packing_engine::day::{DayState, Layout, PackingListChoice, PdfEntry, Scope};
use packing_engine::names;
use packing_pdf::PDFIUM_VERSION;
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::{
    AppState, BUILTIN_COURIERS, COURIERS_FILE, LABELS_DIR, SETTINGS_FILE, program_folder, today,
};

// ----------------------------------------------------------------------------- version

/// The program's name as the window title and the title bar show it (08 › *Window frame*).
pub(crate) const PROGRAM_NAME: &str = "Packing";

/// The shown form of the program's version (08 › *6. Settings* › *The version*): `v` + the first
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

/// The OS window title and the title bar's brand text (08 › *Window frame*): the program's name
/// and [`version_text`]. [`crate::run`] sets the window title with it at start; the title bar
/// shows the same line, from `StartInfo.version`.
pub(crate) fn window_title() -> String {
    format!("{PROGRAM_NAME} {}", version_text())
}

// ------------------------------------------------------------------------------- start

/// What the window needs to open the Start prompt or the Day screen (08 › *Start*).
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
    /// this (08 › *Window frame*).
    version: String,
}

/// Whether `folder` is a data folder: it holds `couriers.toml` (07 › *Program folder*, 08 ›
/// *Start* step 1). `categories.toml` is not a marker: the app writes it only when the owner
/// saves picks (08 › *5. Categories*).
fn is_data_folder(folder: &Path) -> bool {
    folder.join(COURIERS_FILE).is_file()
}

fn start_information(state: &AppState) -> StartInfo {
    let exe_dir = program_folder();
    let program_couriers = is_data_folder(&exe_dir);
    if program_couriers && state.data_folder().is_err() {
        // Found → that folder is the data folder (08 › *Start* step 1).
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

/// Put the built-in `couriers.toml` into `folder` when it has none (08 › *Start*, *6. Settings*).
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
/// folder the data folder (08 › *Start*). No picks are written: a folder without
/// `categories.toml` uses the one catch-all entry. Returns the folder.
fn activate_program_folder(state: &AppState) -> Result<String, String> {
    let folder = program_folder();
    ensure_builtin_couriers(&folder)?;
    state.set_data_folder(folder.clone());
    Ok(folder.display().to_string())
}

/// *Choose a folder…* / Settings *Change…*: use `path` until the app closes; a folder without a
/// `couriers.toml` gets the built-in one, and its own `categories.toml`, if any, is kept as it
/// is (08 › *Start*, 08 › *6. Settings*). Returns the folder.
fn use_chosen_folder(state: &AppState, path: &str) -> Result<String, String> {
    let folder = PathBuf::from(path);
    ensure_builtin_couriers(&folder)?;
    state.set_data_folder(folder.clone());
    Ok(folder.display().to_string())
}

// -------------------------------------------------------------------------------- day

/// The Day screen's view of one saved PDF (08 › *1. Day*).
#[derive(Serialize)]
pub struct PdfView {
    number: i64,
    code: String,
    name: String,
    file: String,
    orders: usize,
    runs: usize,
    /// The row's **Printed** tick, from the batch's `printed` list (07 › *`state.json`*;
    /// 08 › *1. Day*): a note only.
    printed: bool,
}

/// The Day screen's view of one batch (08 › *1. Day*).
#[derive(Serialize)]
pub struct BatchView {
    batch: i64,
    save_time: String,
    /// `HH:MM` of `save_time`.
    time: String,
    files: usize,
    orders: usize,
    pdfs: Vec<PdfView>,
    /// The batch's packing-list files, for *Open* (usually one).
    packing_lists: Vec<String>,
    /// The read warnings of the batch, for *Warnings ▸* (08 › *1. Day*).
    warnings: Vec<String>,
    /// The check before saving's errors, for *Saved with `<n>` errors ▸* (08 › *1. Day*); empty
    /// unless the batch was saved anyway (07 › *Check before saving*).
    errors: Vec<String>,
    /// The batch folder, for *Open folder*.
    folder: String,
}

/// The day's totals (08 › *1. Day* header).
#[derive(Serialize)]
pub struct Totals {
    batches: usize,
    orders: usize,
    pdfs: usize,
}

/// The Day screen's data: the day's batches, newest first (08 › *1. Day*).
#[derive(Serialize)]
pub struct DayOverview {
    day: String,
    batches: Vec<BatchView>,
    totals: Totals,
}

/// Read `<data>/labels/<day>/state.json` (07 › *Day folder*). A missing day folder is an empty
/// overview (08 › *1. Day*: "Nothing saved on this day yet.").
fn day_overview_at(data_folder: &Path, day: &str) -> Result<DayOverview, String> {
    let day_dir = data_folder.join(LABELS_DIR).join(day);
    let state = DayState::load(&day_dir).map_err(|error| error.to_string())?;
    Ok(overview_of(&state, &day_dir))
}

/// Write one saved-PDF row's **Printed** tick into the day's `state.json` (07 › *`state.json`*;
/// 08 › *1. Day*); a batch or saved-PDF number the day does not hold changes nothing.
fn set_printed_at(
    data_folder: &Path,
    day: &str,
    batch: i64,
    pdf: i64,
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
    batches.reverse(); // newest on top (08 › *1. Day*)
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

/// The day folders to switch between, newest first (08 › *Window frame*). Search's *All days*
/// reads the same list (2.21).
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

/// The packing-list files a batch wrote, by its scope and layouts (07 › *Day folder*). The
/// layout is part of the name only when several layouts were written.
fn packing_list_files(choice: &PackingListChoice, pdfs: &[PdfEntry]) -> Vec<String> {
    let several = choice.layouts.len() > 1;
    let name = |number: Option<i64>, layout: Layout| {
        names::packing_list_file_name(number, several.then(|| layout_name(layout)))
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

fn layout_name(layout: Layout) -> &'static str {
    match layout {
        Layout::Full => "full",
        Layout::Summary => "summary",
        Layout::Pick => "pick",
    }
}

// --------------------------------------------------------------------------- settings

/// The data folder's `settings.json` (07 › *Settings*): the packing-list default and the picks
/// set used last. A missing key takes its default, so an older file still loads.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default = "default_scope")]
    pub scope: Scope,
    #[serde(default = "default_layouts")]
    pub layouts: Vec<Layout>,
    /// The picks set used last (07 › *Settings*; 04 › *Sets*): the [New batch](../ui/new-batch.md)
    /// screen preselects it, and each batch records its own. Absent in an older file, which loads
    /// as `default`.
    #[serde(default = "default_set")]
    pub set: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            scope: default_scope(),
            layouts: default_layouts(),
            set: default_set(),
        }
    }
}

impl Settings {
    /// The packing-list choice a batch starts with (08 › *3. Plan*: defaults from Settings).
    pub(crate) fn packing_choice(&self) -> PackingListChoice {
        PackingListChoice {
            scope: self.scope,
            layouts: self.layouts.clone(),
        }
    }
}

fn default_scope() -> Scope {
    Scope::Whole
}

fn default_layouts() -> Vec<Layout> {
    vec![Layout::Pick]
}

/// The picks set an older `settings.json` (or one without the key) loads as (04 › *Sets*).
fn default_set() -> String {
    packing_engine::sets::DEFAULT_SET.to_string()
}

pub(crate) fn settings_load(data_folder: &Path) -> Result<Settings, String> {
    let path = data_folder.join(SETTINGS_FILE);
    if !path.is_file() {
        return Ok(Settings::default());
    }
    let text = std::fs::read_to_string(&path)
        .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    serde_json::from_str(&text).map_err(|error| format!("cannot read {}: {error}", path.display()))
}

fn settings_save(data_folder: &Path, settings: &Settings) -> Result<(), String> {
    let path = data_folder.join(SETTINGS_FILE);
    let mut text = serde_json::to_string_pretty(settings).map_err(|error| error.to_string())?;
    text.push('\n');
    std::fs::write(&path, text)
        .map_err(|error| format!("Cannot write to {}: {error}.", path.display()))
}

/// Remember the picks set used last in the data folder's `settings.json` (07 › *Settings*; 04 ›
/// *Sets*), so the next **New batch** preselects it. Best-effort: a read is never stopped by a
/// settings problem, and a save of the same set writes nothing.
pub(crate) fn remember_last_set(data_folder: &Path, set: &str) {
    let Ok(mut settings) = settings_load(data_folder) else {
        return;
    };
    if settings.set == set {
        return;
    }
    settings.set = set.to_string();
    let _ = settings_save(data_folder, &settings);
}

// ----------------------------------------------------------------------------- about

/// Settings › *About* (08 › *6. Settings*; D14: no log file, so the version and the PDFium
/// version only).
#[derive(Serialize)]
pub struct About {
    version: String,
    pdfium: String,
}

// ----------------------------------------------------------------------------- shell

/// The Explorer command for `path` (separate so a test can read its arguments back).
fn explorer_command(path: &str) -> Command {
    let mut command = Command::new("explorer");
    command.arg(path);
    command
}

/// Open a file with the PC's default program, or a folder as a folder in Explorer (08 › *1. Day*:
/// *Open* and *Open folder*; 08 › *6. Settings*).
///
/// The path is Explorer's **only** argument. Explorer parses `/select,<path>` itself and does not
/// understand the quotes Rust puts around an argument that holds a space, so it opened *Documents*
/// (or *Home*) instead of the folder (review round 1: the batch folder always has a space in its
/// path, e.g. `…\labels\2026-10-11\batch 1`). A bare path works for both: a folder opens as a
/// folder, a file opens in its default program.
fn open_with_explorer(path: &str) -> Result<(), String> {
    explorer_command(path)
        .spawn()
        .map(|_| ())
        .map_err(|error| format!("Cannot open {path}: {error}."))
}

// -------------------------------------------------------------------------- commands

/// Where the program is, whether `categories.toml` was found next to it, the data folder in
/// use and today's date (08 › *Start*).
#[tauri::command]
pub fn start_info(state: State<'_, AppState>) -> StartInfo {
    start_information(&state)
}

/// *Use this folder* (08 › *Start*): write the built-in `categories.toml` next to the program.
#[tauri::command]
pub fn use_program_folder(state: State<'_, AppState>) -> Result<String, String> {
    activate_program_folder(&state)
}

/// *Choose a folder…* / Settings *Change…*: use the folder until the app closes (08 › *Start*).
#[tauri::command]
pub fn choose_data_folder(state: State<'_, AppState>, path: String) -> Result<String, String> {
    use_chosen_folder(&state, &path)
}

/// The day's batches, saved PDFs, warnings and totals, from `state.json` (08 › *1. Day*).
#[tauri::command]
pub fn day_overview(state: State<'_, AppState>, day: String) -> Result<DayOverview, String> {
    state.set_day(&day); // a new batch saves into the day being shown
    day_overview_at(&state.data_folder()?, &day)
}

/// The **Printed** tick of one saved-PDF row of a batch (08 › *1. Day*; 07 › *`state.json`*):
/// writes the day's `state.json`; it changes nothing else.
#[tauri::command]
pub fn set_printed(
    state: State<'_, AppState>,
    day: String,
    batch: i64,
    pdf: i64,
    printed: bool,
) -> Result<(), String> {
    state.set_day(&day);
    set_printed_at(&state.data_folder()?, &day, batch, pdf, printed)
}

/// The day folders to switch between (08 › *Window frame*).
#[tauri::command]
pub fn days(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    Ok(days_in(&state.data_folder()?))
}

/// The settings (08 › *6. Settings*).
#[tauri::command]
pub fn settings_get(state: State<'_, AppState>) -> Result<Settings, String> {
    settings_load(&state.data_folder()?)
}

/// Save the settings (08 › *6. Settings*).
#[tauri::command]
pub fn settings_set(state: State<'_, AppState>, settings: Settings) -> Result<(), String> {
    settings_save(&state.data_folder()?, &settings)
}

/// Settings › *About*: the app version and the PDFium version (08 › *6. Settings*). The version
/// is the shown form from [`version_text`], the one place.
#[tauri::command]
pub fn about() -> About {
    About {
        version: version_text(),
        pdfium: PDFIUM_VERSION.to_string(),
    }
}

/// Open a file in the default viewer (08 › *1. Day*: *Open*).
#[tauri::command]
pub fn open_path(path: String) -> Result<(), String> {
    open_with_explorer(&path)
}

/// Open a folder in Explorer (08 › *1. Day*: *Open folder*; 08 › *6. Settings*).
#[tauri::command]
pub fn show_in_folder(path: String) -> Result<(), String> {
    open_with_explorer(&path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use packing_engine::day::{BatchEntry, FileEntry, PackingListChoice, PdfEntry};

    fn temp_dir(label: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("packing-app-test-{}-{label}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("temp dir");
        dir
    }

    fn file_entry(name: &str) -> FileEntry {
        FileEntry {
            name: name.to_string(),
            size: 1,
            xxh3: "00".to_string(),
            pages: 1,
            legacy_sha256: None,
        }
    }

    fn batch_entry(
        batch: i64,
        time: &str,
        pdf: PdfEntry,
        warnings: Vec<String>,
        errors: Vec<String>,
    ) -> BatchEntry {
        BatchEntry {
            batch,
            save_time: time.to_string(),
            set: "default".to_string(),
            files: vec![file_entry("a.pdf")],
            pdfs: vec![pdf],
            packing_list: PackingListChoice::default(),
            printed: Vec::new(),
            warnings,
            errors,
        }
    }

    #[test]
    fn settings_default_and_round_trip() {
        let dir = temp_dir("settings");
        assert_eq!(settings_load(&dir).expect("default"), Settings::default());

        let custom = Settings {
            scope: Scope::PerPdf,
            layouts: vec![Layout::Full, Layout::Summary],
            set: "ramadhan".to_string(),
        };
        settings_save(&dir, &custom).expect("save");
        assert_eq!(settings_load(&dir).expect("load"), custom);
        assert_eq!(custom.packing_choice().scope, Scope::PerPdf);

        // A missing key takes its default: an older file without the set loads as `default`
        // (07 › *Settings*; 04 › *Sets*).
        std::fs::write(dir.join(SETTINGS_FILE), "{\n  \"scope\": \"none\"\n}\n").expect("write");
        let partial = settings_load(&dir).expect("partial");
        assert_eq!(partial.scope, Scope::None);
        assert_eq!(partial.layouts, vec![Layout::Pick]);
        assert_eq!(partial.set, packing_engine::sets::DEFAULT_SET);

        std::fs::remove_dir_all(&dir).ok();
    }

    /// The picks set used last is remembered in `settings.json` (07 › *Settings*; 04 › *Sets*), so
    /// the next New batch preselects it; a read with the same set writes nothing, and a folder
    /// whose settings cannot be read is left alone.
    #[test]
    fn the_last_used_set_is_remembered() {
        let dir = temp_dir("settings-set");
        // No file yet: the default, and remembering a set writes it.
        assert_eq!(settings_load(&dir).expect("default").set, "default");
        remember_last_set(&dir, "ramadhan");
        assert_eq!(settings_load(&dir).expect("load").set, "ramadhan");

        // The packing-list keys are kept (the file is written whole).
        remember_last_set(&dir, "default");
        let settings = settings_load(&dir).expect("load again");
        assert_eq!(settings.set, "default");
        assert_eq!(settings.scope, Scope::Whole);
        assert_eq!(settings.layouts, vec![Layout::Pick]);

        // A broken settings file is not touched.
        std::fs::write(dir.join(SETTINGS_FILE), "not json").expect("write broken");
        remember_last_set(&dir, "ramadhan");
        assert_eq!(
            std::fs::read_to_string(dir.join(SETTINGS_FILE)).expect("read"),
            "not json"
        );

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn overview_reads_state_newest_first_totals_and_warnings() {
        let dir = temp_dir("overview");
        let day_dir = dir.join(LABELS_DIR).join("2026-10-07");
        let mut state = DayState::new("2026-10-07");
        state.batches.push(batch_entry(
            1,
            "2026-10-07T07:40:12+07:00",
            PdfEntry {
                number: 1,
                code: "A".to_string(),
                name: "Sepatu".to_string(),
                file: "1 A Sepatu ×172.pdf".to_string(),
                orders: 172,
                runs: 4,
            },
            vec!["no tracking ID for: 580000000000000004".to_string()],
            Vec::new(),
        ));
        state.batches.push(batch_entry(
            2,
            "2026-10-07T09:12:00+07:00",
            PdfEntry {
                number: 2,
                code: "Z".to_string(),
                name: "Lainnya".to_string(),
                file: "2 Z Lainnya ×10.pdf".to_string(),
                orders: 10,
                runs: 2,
            },
            Vec::new(),
            vec!["PDF 2 is missing a page".to_string()],
        ));
        state.save(&day_dir).expect("save state");

        let overview = day_overview_at(&dir, "2026-10-07").expect("overview");
        assert_eq!(overview.day, "2026-10-07");
        assert_eq!(overview.batches.len(), 2);
        assert_eq!(overview.batches[0].batch, 2); // newest on top
        assert_eq!(overview.batches[0].time, "09:12");
        assert_eq!(overview.batches[0].orders, 10);
        assert!(overview.batches[0].warnings.is_empty());
        // The check before saving's errors reach the Day screen's view.
        assert_eq!(
            overview.batches[0].errors,
            vec!["PDF 2 is missing a page".to_string()]
        );
        assert_eq!(overview.batches[0].packing_lists, vec!["packing-list.pdf"]);
        assert!(overview.batches[0].folder.ends_with("batch 2"));
        assert_eq!(
            overview.batches[1].warnings,
            vec!["no tracking ID for: 580000000000000004".to_string()]
        );
        assert!(overview.batches[1].errors.is_empty(), "this batch had none");
        assert_eq!(overview.totals.batches, 2);
        assert_eq!(overview.totals.orders, 182);
        assert_eq!(overview.totals.pdfs, 2);

        // A missing day folder is an empty overview.
        let empty = day_overview_at(&dir, "2026-10-08").expect("empty");
        assert!(empty.batches.is_empty());
        assert_eq!(empty.totals.orders, 0);

        std::fs::remove_dir_all(&dir).ok();
    }

    /// The Day screen's **Printed** tick: the write is read back by a second `day_overview` (the
    /// tick survives closing and reopening), and an unknown batch or saved-PDF number changes
    /// nothing (08 › *1. Day*; 07 › *`state.json`*).
    #[test]
    fn set_printed_writes_the_tick_and_the_overview_reports_it() {
        let dir = temp_dir("printed");
        let day_dir = dir.join(LABELS_DIR).join("2026-10-07");
        let mut state = DayState::new("2026-10-07");
        state.batches.push(batch_entry(
            1,
            "2026-10-07T07:40:12+07:00",
            PdfEntry {
                number: 1,
                code: "A".to_string(),
                name: "Sepatu".to_string(),
                file: "1 A Sepatu ×172.pdf".to_string(),
                orders: 172,
                runs: 4,
            },
            Vec::new(),
            Vec::new(),
        ));
        state.save(&day_dir).expect("save state");

        // Before: the row is unticked.
        let before = day_overview_at(&dir, "2026-10-07").expect("overview");
        assert!(!before.batches[0].pdfs[0].printed);

        // Ticking writes the file; a fresh `day_overview` reports the tick.
        set_printed_at(&dir, "2026-10-07", 1, 1, true).expect("tick");
        let after = day_overview_at(&dir, "2026-10-07").expect("overview again");
        assert!(after.batches[0].pdfs[0].printed);

        // Unticking is read back too.
        set_printed_at(&dir, "2026-10-07", 1, 1, false).expect("untick");
        let after = day_overview_at(&dir, "2026-10-07").expect("overview again");
        assert!(!after.batches[0].pdfs[0].printed);

        // An unknown batch or saved-PDF number changes nothing.
        set_printed_at(&dir, "2026-10-07", 9, 1, true).expect("unknown batch");
        set_printed_at(&dir, "2026-10-07", 1, 9, true).expect("unknown pdf");
        let after = day_overview_at(&dir, "2026-10-07").expect("overview again");
        assert!(!after.batches[0].pdfs[0].printed);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn days_lists_folders_newest_first() {
        let dir = temp_dir("days");
        let labels = dir.join(LABELS_DIR);
        std::fs::create_dir_all(labels.join("2026-10-06")).expect("day 1");
        std::fs::create_dir_all(labels.join("2026-10-07")).expect("day 2");
        std::fs::write(labels.join("stray.txt"), "x").expect("stray file");

        assert_eq!(
            days_in(&dir),
            vec!["2026-10-07".to_string(), "2026-10-06".to_string()]
        );
        // No labels folder yet: no days.
        assert!(days_in(&temp_dir("days-empty")).is_empty());

        std::fs::remove_dir_all(&dir).ok();
    }

    /// A first start writes `couriers.toml` and no `categories.toml`; the folder is a data folder
    /// from then on, and the owner's own `couriers.toml` (or one the logo prompt wrote to) is
    /// never overwritten (08 › *Start*, 08 › *6. Settings*).
    #[test]
    fn a_first_start_writes_couriers_only_and_marks_the_folder_with_it() {
        let dir = temp_dir("first-start");
        assert!(
            !is_data_folder(&dir),
            "an empty folder is not a data folder"
        );

        // The picks file is not a marker: a folder with only `categories.toml` still asks
        // (08 › *Start* step 1).
        std::fs::write(
            dir.join(crate::CATEGORIES_FILE),
            "[[category]]\ncode = \"Z\"\nname = \"Rest\"\n",
        )
        .expect("write");
        assert!(
            !is_data_folder(&dir),
            "categories.toml is not the marker of a data folder"
        );
        std::fs::remove_file(dir.join(crate::CATEGORIES_FILE)).expect("remove");

        ensure_builtin_couriers(&dir).expect("couriers");
        assert!(is_data_folder(&dir), "couriers.toml marks the folder");
        assert_eq!(
            std::fs::read_to_string(dir.join(COURIERS_FILE)).expect("read"),
            BUILTIN_COURIERS
        );
        assert!(
            !dir.join(crate::CATEGORIES_FILE).exists(),
            "a first start writes no categories.toml"
        );

        let mine = "# mine\n[[courier]]\nname = \"X\"\nlogo = [\"0123456789abcdef\"]\n";
        std::fs::write(dir.join(COURIERS_FILE), mine).expect("write");
        ensure_builtin_couriers(&dir).expect("keep");
        assert_eq!(
            std::fs::read_to_string(dir.join(COURIERS_FILE)).expect("read"),
            mine
        );

        std::fs::remove_dir_all(&dir).ok();
    }

    /// *Choose a folder…* / Settings *Change…*: a folder without a `couriers.toml` gets the
    /// built-in one and no picks; a folder with its own files is used as it is (08 › *Start*,
    /// 08 › *6. Settings*).
    #[test]
    fn choosing_a_folder_writes_only_a_missing_couriers() {
        let state = AppState::new();
        let mine = "# mine\n[[courier]]\nname = \"X\"\nlogo = [\"0123456789abcdef\"]\n";

        let empty = temp_dir("chosen-empty");
        use_chosen_folder(&state, empty.to_str().expect("utf8")).expect("choose");
        assert!(empty.join(COURIERS_FILE).is_file());
        assert!(
            !empty.join(crate::CATEGORIES_FILE).exists(),
            "no picks are written"
        );

        let own = temp_dir("chosen-own");
        let rules = "[[category]]\ncode = \"Z\"\nname = \"Rest\"\n";
        std::fs::write(own.join(crate::CATEGORIES_FILE), rules).expect("write");
        std::fs::write(own.join(COURIERS_FILE), mine).expect("write");
        use_chosen_folder(&state, own.to_str().expect("utf8")).expect("choose");
        assert_eq!(
            std::fs::read_to_string(own.join(crate::CATEGORIES_FILE)).expect("read"),
            rules,
            "the folder's own picks are kept"
        );
        assert_eq!(
            std::fs::read_to_string(own.join(COURIERS_FILE)).expect("read"),
            mine
        );

        std::fs::remove_dir_all(&empty).ok();
        std::fs::remove_dir_all(&own).ok();
    }

    #[test]
    fn about_reports_the_app_and_pdfium_versions() {
        let about = about();
        assert_eq!(about.version, version_text());
        assert_eq!(about.pdfium, PDFIUM_VERSION);
    }

    /// The shown version is `v` + the first two numbers of the workspace `version`, the one place
    /// (08 › *6. Settings* › *The version*). The expected text is derived from the same
    /// `CARGO_PKG_VERSION` here, so a hard-coded `v0.1` in the app would pass today and fail as
    /// soon as the workspace `version` changes — which is exactly the property the owner asked
    /// for: changing the number in `Cargo.toml` moves every spot.
    #[test]
    fn version_text_is_v_and_the_first_two_numbers_of_the_workspace_version() {
        let expected = {
            let mut numbers = env!("CARGO_PKG_VERSION").split('.');
            let major = numbers.next().expect("a major number");
            let minor = numbers.next().expect("a minor number");
            format!("v{major}.{minor}")
        };
        assert_eq!(version_text(), expected);
        // Only the first two numbers are shown (`0.1.0` → `v0.1`), never the third.
        assert_eq!(version_text().matches('.').count(), 1);
    }

    /// The window title and *About* both take the version from [`version_text`], the one place, so
    /// they cannot differ (08 › *Window frame*, 08 › *6. Settings*). A literal in either spot
    /// passes today but fails as soon as the workspace `version` changes.
    #[test]
    fn the_window_title_and_about_take_the_version_from_that_one_function() {
        let shown = version_text();
        assert_eq!(window_title(), format!("{PROGRAM_NAME} {shown}"));
        assert_eq!(about().version, shown);
    }

    /// `app/src-tauri/tauri.conf.json` carries no second copy of the version (08 › *6. Settings* ›
    /// *The version*): no `version` key — Tauri then takes it from `app/src-tauri/Cargo.toml`,
    /// which the app inherits from the workspace — and the window `title` holds no number, because
    /// [`crate::run`] sets the real title at start.
    #[test]
    fn tauri_conf_has_no_version_key_and_no_number_in_the_window_title() {
        let conf: serde_json::Value = serde_json::from_str(include_str!("../../tauri.conf.json"))
            .expect("tauri.conf.json is JSON");
        assert!(
            conf.get("version").is_none(),
            "tauri.conf.json must not carry a `version` key"
        );
        let title = conf["app"]["windows"][0]["title"]
            .as_str()
            .expect("the window title is a string");
        assert!(
            !title.chars().any(|character| character.is_ascii_digit()),
            "the window title must carry no version number: {title}"
        );
    }

    /// The shell calls hand Explorer the path and nothing else. `/select,<path>` is not parsed by
    /// Explorer when the path holds a space — it opened *Documents* (or *Home*) instead of the
    /// folder — and the batch folder always has a space in its path (review round 1).
    #[test]
    fn explorer_gets_the_path_as_its_only_argument() {
        let path = r"C:\packing data\labels\2026-10-11\batch 1";
        let command = explorer_command(path);
        assert_eq!(command.get_program().to_string_lossy(), "explorer");
        let arguments: Vec<String> = command
            .get_args()
            .map(|argument| argument.to_string_lossy().into_owned())
            .collect();
        assert_eq!(arguments, vec![path.to_string()]);
    }
}
