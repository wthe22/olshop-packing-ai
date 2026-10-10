//! The packing app window back end (task 2.8): the data folder, settings, the day overview and
//! the window commands of 07 › *Commands between window and Rust*.
//!
//! The command functions stay thin; the logic lives in plain functions that are unit-tested.
//! The data folder is app state, never a stored setting (07 › *Settings*): at start the app
//! looks for `categories.toml` in its own folder and asks the owner when it is missing
//! (08 › *Start*).

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;

use packing_engine::day::{DayState, Layout, PackingListChoice, PdfEntry, Scope};
use packing_engine::names;
use packing_pdf::worker::PdfWorker;
use serde::{Deserialize, Serialize};
use tauri::State;

/// The built-in `categories.toml` written next to the program on *Use this folder* (08 ›
/// *Start*). A copy of the repository's rules file; it holds no customer data.
const BUILTIN_CATEGORIES: &str = include_str!("../categories.toml");

/// The categories file that marks a folder as a data folder (07 › *Program folder*).
const CATEGORIES_FILE: &str = "categories.toml";
/// The settings file in the data folder (07 › *Settings*).
const SETTINGS_FILE: &str = "settings.json";
/// The day folders inside the data folder (07 › *Day folder*).
const LABELS_DIR: &str = "labels";

// --------------------------------------------------------------------------- app state

/// What the window back end keeps between commands (07 › *Commands between window and Rust*):
/// the data folder and the PDFium worker thread. The data folder is not persisted.
pub struct AppState {
    data_folder: Mutex<Option<PathBuf>>,
    worker: Mutex<Option<PdfWorker>>,
}

impl AppState {
    fn new() -> Self {
        Self {
            data_folder: Mutex::new(None),
            worker: Mutex::new(None),
        }
    }

    /// The data folder in use, or a plain message when none is set yet.
    fn data_folder(&self) -> Result<PathBuf, String> {
        self.data_folder
            .lock()
            .unwrap()
            .clone()
            .ok_or_else(|| "No data folder is set yet.".to_string())
    }

    fn set_data_folder(&self, folder: PathBuf) {
        *self.data_folder.lock().unwrap() = Some(folder);
    }

    /// Spawn the PDFium worker on first use, binding `pdfium.dll` from [`pdfium_dir`]. Returns
    /// whether a worker is available.
    fn ensure_worker(&self) -> bool {
        let mut guard = self.worker.lock().unwrap();
        if guard.is_some() {
            return true;
        }
        match PdfWorker::spawn(pdfium_dir()) {
            Ok(worker) => {
                *guard = Some(worker);
                true
            }
            Err(_) => false,
        }
    }
}

/// The folder the program runs from (07 › *Program folder*).
fn program_folder() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(Path::to_path_buf))
        .unwrap_or_default()
}

/// The folder with `pdfium.dll`: the program's own folder, else (in development builds)
/// `pc/vendor`, so both `npm run tauri dev` and the portable build work (07 › *Program folder*).
fn pdfium_dir() -> PathBuf {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(Path::to_path_buf));
    if let Some(dir) = &exe_dir
        && dir.join("pdfium.dll").is_file()
    {
        return dir.clone();
    }
    #[cfg(debug_assertions)]
    {
        let vendor = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../vendor");
        if vendor.join("pdfium.dll").is_file() {
            return vendor;
        }
    }
    exe_dir.unwrap_or_default()
}

// ------------------------------------------------------------------------------- start

/// What the window needs to open the Start prompt or the Day screen (08 › *Start*).
#[derive(Serialize)]
pub struct StartInfo {
    /// Where the program's exe is.
    exe_dir: String,
    /// Whether `categories.toml` was found next to the program.
    program_categories: bool,
    /// The data folder in use, if one is set.
    data_folder: Option<String>,
    /// Whether the PDFium worker started (its dll was found).
    pdfium_ready: bool,
    /// Today's date on the PC (`YYYY-MM-DD`); the Day screen opens here.
    today: String,
}

fn start_information(state: &AppState) -> StartInfo {
    let exe_dir = program_folder();
    let program_categories = exe_dir.join(CATEGORIES_FILE).is_file();
    if program_categories && state.data_folder.lock().unwrap().is_none() {
        // Found → that folder is the data folder (08 › *Start* step 1).
        state.set_data_folder(exe_dir.clone());
    }
    StartInfo {
        exe_dir: exe_dir.display().to_string(),
        program_categories,
        data_folder: state
            .data_folder
            .lock()
            .unwrap()
            .as_ref()
            .map(|path| path.display().to_string()),
        pdfium_ready: state.ensure_worker(),
        today: today(),
    }
}

fn today() -> String {
    jiff::Zoned::now().strftime("%Y-%m-%d").to_string()
}

/// Write the built-in `categories.toml` into `folder` (08 › *Start*). Fails, e.g. under
/// `Program Files`, with the 08 *cannot write* wording.
fn write_builtin_categories(folder: &Path) -> Result<(), String> {
    let path = folder.join(CATEGORIES_FILE);
    std::fs::write(&path, BUILTIN_CATEGORIES)
        .map_err(|error| format!("Cannot write to {}: {error}.", folder.display()))
}

/// *Use this folder*: write the built-in `categories.toml` next to the program and make that
/// folder the data folder (08 › *Start*). Returns the folder.
fn activate_program_folder(state: &AppState) -> Result<String, String> {
    let folder = program_folder();
    write_builtin_categories(&folder)?;
    state.set_data_folder(folder.clone());
    Ok(folder.display().to_string())
}

/// *Choose a folder…* / Settings *Change…*: use `path` until the app closes; an empty folder
/// gets the built-in `categories.toml` (08 › *Start*, 08 › *6. Settings*). Returns the folder.
fn use_chosen_folder(state: &AppState, path: &str) -> Result<String, String> {
    let folder = PathBuf::from(path);
    if !folder.join(CATEGORIES_FILE).is_file() {
        write_builtin_categories(&folder)?;
    }
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
                })
                .collect(),
            packing_lists: packing_list_files(&batch.packing_list, &batch.pdfs),
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

/// The day folders to switch between, newest first (08 › *Window frame*).
fn days_in(data_folder: &Path) -> Vec<String> {
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

/// The data folder's `settings.json` (07 › *Settings*): the packing-list default. A missing key
/// takes its default, so an older file still loads.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default = "default_scope")]
    pub scope: Scope,
    #[serde(default = "default_layouts")]
    pub layouts: Vec<Layout>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            scope: default_scope(),
            layouts: default_layouts(),
        }
    }
}

fn default_scope() -> Scope {
    Scope::Whole
}

fn default_layouts() -> Vec<Layout> {
    vec![Layout::Pick]
}

fn settings_load(data_folder: &Path) -> Result<Settings, String> {
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

// ----------------------------------------------------------------------------- shell

/// Open a file in the PC's default viewer (08 › *1. Day*: *Open*).
fn open_in_viewer(path: &str) -> Result<(), String> {
    Command::new("explorer")
        .arg(path)
        .spawn()
        .map(|_| ())
        .map_err(|error| format!("Cannot open {path}: {error}."))
}

/// Show a file (or folder) in Explorer with it selected (08 › *1. Day*: *Open folder*). The
/// `/select,` and the path are one argument, as Explorer expects.
fn reveal_in_folder(path: &str) -> Result<(), String> {
    Command::new("explorer")
        .arg(format!("/select,{path}"))
        .spawn()
        .map(|_| ())
        .map_err(|error| format!("Cannot open {path}: {error}."))
}

// -------------------------------------------------------------------------- commands

/// Where the program is, whether `categories.toml` was found next to it, the data folder in
/// use and today's date (08 › *Start*).
#[tauri::command]
fn start_info(state: State<'_, AppState>) -> StartInfo {
    start_information(&state)
}

/// *Use this folder* (08 › *Start*): write the built-in `categories.toml` next to the program.
#[tauri::command]
fn use_program_folder(state: State<'_, AppState>) -> Result<String, String> {
    activate_program_folder(&state)
}

/// *Choose a folder…* / Settings *Change…*: use the folder until the app closes (08 › *Start*).
#[tauri::command]
fn choose_data_folder(state: State<'_, AppState>, path: String) -> Result<String, String> {
    use_chosen_folder(&state, &path)
}

/// The day's batches, saved PDFs and totals, from `state.json` (08 › *1. Day*).
#[tauri::command]
fn day_overview(state: State<'_, AppState>, day: String) -> Result<DayOverview, String> {
    day_overview_at(&state.data_folder()?, &day)
}

/// The day folders to switch between (08 › *Window frame*).
#[tauri::command]
fn days(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    Ok(days_in(&state.data_folder()?))
}

/// The settings (08 › *6. Settings*).
#[tauri::command]
fn settings_get(state: State<'_, AppState>) -> Result<Settings, String> {
    settings_load(&state.data_folder()?)
}

/// Save the settings (08 › *6. Settings*).
#[tauri::command]
fn settings_set(state: State<'_, AppState>, settings: Settings) -> Result<(), String> {
    settings_save(&state.data_folder()?, &settings)
}

/// Open a file in the default viewer.
#[tauri::command]
fn open_path(path: String) -> Result<(), String> {
    open_in_viewer(&path)
}

/// Show a file in Explorer with it selected.
#[tauri::command]
fn show_in_folder(path: String) -> Result<(), String> {
    reveal_in_folder(&path)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            start_info,
            use_program_folder,
            choose_data_folder,
            day_overview,
            days,
            settings_get,
            settings_set,
            open_path,
            show_in_folder,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;
    use packing_engine::day::{BatchEntry, FileEntry, PackingListChoice, PdfEntry};
    use packing_engine::rules::load_rules;

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
            sha256: "00".to_string(),
            pages: 1,
        }
    }

    #[test]
    fn builtin_categories_are_a_valid_rules_file() {
        assert!(load_rules(BUILTIN_CATEGORIES).is_ok());
    }

    #[test]
    fn settings_default_and_round_trip() {
        let dir = temp_dir("settings");
        assert_eq!(settings_load(&dir).expect("default"), Settings::default());

        let custom = Settings {
            scope: Scope::PerPdf,
            layouts: vec![Layout::Full, Layout::Summary],
        };
        settings_save(&dir, &custom).expect("save");
        assert_eq!(settings_load(&dir).expect("load"), custom);

        // A missing key takes its default.
        std::fs::write(dir.join(SETTINGS_FILE), "{\n  \"scope\": \"none\"\n}\n").expect("write");
        let partial = settings_load(&dir).expect("partial");
        assert_eq!(partial.scope, Scope::None);
        assert_eq!(partial.layouts, vec![Layout::Pick]);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn overview_reads_state_newest_first_and_totals() {
        let dir = temp_dir("overview");
        let day_dir = dir.join(LABELS_DIR).join("2026-10-07");
        let mut state = DayState::new("2026-10-07");
        state.batches.push(BatchEntry {
            batch: 1,
            save_time: "2026-10-07T07:40:12+07:00".to_string(),
            files: vec![file_entry("a.pdf")],
            pdfs: vec![PdfEntry {
                number: 1,
                code: "A".to_string(),
                name: "Sepatu".to_string(),
                file: "1 A Sepatu ×172.pdf".to_string(),
                orders: 172,
                runs: 4,
            }],
            packing_list: PackingListChoice::default(),
        });
        state.batches.push(BatchEntry {
            batch: 2,
            save_time: "2026-10-07T09:12:00+07:00".to_string(),
            files: vec![],
            pdfs: vec![PdfEntry {
                number: 2,
                code: "Z".to_string(),
                name: "Lainnya".to_string(),
                file: "2 Z Lainnya ×10.pdf".to_string(),
                orders: 10,
                runs: 2,
            }],
            packing_list: PackingListChoice::default(),
        });
        state.save(&day_dir).expect("save state");

        let overview = day_overview_at(&dir, "2026-10-07").expect("overview");
        assert_eq!(overview.day, "2026-10-07");
        assert_eq!(overview.batches.len(), 2);
        assert_eq!(overview.batches[0].batch, 2); // newest on top
        assert_eq!(overview.batches[0].time, "09:12");
        assert_eq!(overview.batches[0].orders, 10);
        assert_eq!(overview.batches[0].packing_lists, vec!["packing-list.pdf"]);
        assert!(overview.batches[0].folder.ends_with("batch 2"));
        assert_eq!(overview.totals.batches, 2);
        assert_eq!(overview.totals.orders, 182);
        assert_eq!(overview.totals.pdfs, 2);

        // A missing day folder is an empty overview.
        let empty = day_overview_at(&dir, "2026-10-08").expect("empty");
        assert!(empty.batches.is_empty());
        assert_eq!(empty.totals.orders, 0);

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

    #[test]
    fn program_folder_is_a_data_folder_once_the_builtin_file_is_written() {
        let dir = temp_dir("program");
        assert!(!dir.join(CATEGORIES_FILE).is_file());
        write_builtin_categories(&dir).expect("write");
        assert!(dir.join(CATEGORIES_FILE).is_file());
        std::fs::remove_dir_all(&dir).ok();
    }
}
