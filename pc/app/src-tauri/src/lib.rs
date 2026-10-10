//! The packing app window back end (tasks 2.8-2.10): app state and setup. The window commands
//! live in [`commands::day`] (start, day, settings) and [`commands::batch`] (new batch, plan,
//! save); the logic is in plain functions that are unit-tested.
//!
//! The command functions stay thin. The data folder is app state, never a stored setting
//! (07 › *Settings*): at start the app looks for `categories.toml` in its own folder and asks
//! the owner when it is missing (08 › *Start*). The open draft (07 › *Words*) is app state too;
//! its orders stay on the PDFium worker thread that opened the label documents.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use packing_engine::plan::Plan;
use packing_pdf::batch::Draft;
use packing_pdf::worker::PdfWorker;

pub mod commands;

/// The built-in `categories.toml` written next to the program on *Use this folder* (08 ›
/// *Start*). A copy of the repository's rules file; it holds no customer data.
pub(crate) const BUILTIN_CATEGORIES: &str = include_str!("../categories.toml");

/// The categories file that marks a folder as a data folder (07 › *Program folder*).
pub(crate) const CATEGORIES_FILE: &str = "categories.toml";
/// The settings file in the data folder (07 › *Settings*).
pub(crate) const SETTINGS_FILE: &str = "settings.json";
/// The day folders inside the data folder (07 › *Day folder*).
pub(crate) const LABELS_DIR: &str = "labels";

// --------------------------------------------------------------------------- app state

/// A batch that has been read but not saved (07 › *Words*): the orders, the plan being edited and
/// the day's first saved-PDF number. It lives in [`AppState`] until it is saved or discarded.
pub(crate) struct OpenDraft {
    pub draft: Draft,
    pub plan: Plan,
    /// The day's next saved-PDF number at the time of reading (08 › *3. Plan*: the numbers `#`).
    pub first_number: i64,
}

/// What the window back end keeps between commands (07 › *Commands between window and Rust*): the
/// data folder, the day being shown, the PDFium worker thread, the open draft and the read cancel
/// flag. None of it is persisted.
pub struct AppState {
    data_folder: Mutex<Option<PathBuf>>,
    day: Mutex<String>,
    worker: Mutex<Option<PdfWorker>>,
    draft: Mutex<Option<OpenDraft>>,
    cancel: Mutex<Option<Arc<AtomicBool>>>,
}

impl AppState {
    pub(crate) fn new() -> Self {
        Self {
            data_folder: Mutex::new(None),
            day: Mutex::new(String::new()),
            worker: Mutex::new(None),
            draft: Mutex::new(None),
            cancel: Mutex::new(None),
        }
    }

    /// The data folder in use, or a plain message when none is set yet.
    pub(crate) fn data_folder(&self) -> Result<PathBuf, String> {
        self.data_folder
            .lock()
            .unwrap()
            .clone()
            .ok_or_else(|| "No data folder is set yet.".to_string())
    }

    pub(crate) fn set_data_folder(&self, folder: PathBuf) {
        *self.data_folder.lock().unwrap() = Some(folder);
    }

    /// The day being shown (07 › *Day folder*); the batch commands save into it.
    pub(crate) fn day(&self) -> String {
        self.day.lock().unwrap().clone()
    }

    pub(crate) fn set_day(&self, day: &str) {
        *self.day.lock().unwrap() = day.to_string();
    }

    /// The day folder `<data>/labels/<day>/` (07 › *Day folder*).
    pub(crate) fn day_dir(&self) -> Result<PathBuf, String> {
        Ok(self.data_folder()?.join(LABELS_DIR).join(self.day()))
    }

    /// Spawn the PDFium worker on first use, binding `pdfium.dll` from [`pdfium_dir`]. Returns
    /// whether a worker is available.
    pub(crate) fn ensure_worker(&self) -> bool {
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

    /// Run `f` with the PDFium worker while holding its lock (every PDFium call must be on the
    /// worker thread; 07 › *Commands between window and Rust*).
    pub(crate) fn with_worker<R>(&self, f: impl FnOnce(&PdfWorker) -> R) -> Result<R, String> {
        let guard = self.worker.lock().unwrap();
        let worker = guard
            .as_ref()
            .ok_or_else(|| "The PDF library is not available.".to_string())?;
        Ok(f(worker))
    }

    /// Store a freshly read draft (replacing any earlier one).
    pub(crate) fn set_draft(&self, open: OpenDraft) {
        *self.draft.lock().unwrap() = Some(open);
    }

    /// Take the open draft out (a save or a discard); `None` when no batch is open.
    pub(crate) fn take_draft(&self) -> Option<OpenDraft> {
        self.draft.lock().unwrap().take()
    }

    /// Put a draft back (a save that failed and can be retried).
    pub(crate) fn restore_draft(&self, open: OpenDraft) {
        *self.draft.lock().unwrap() = Some(open);
    }

    /// Run `f` with the open draft, or return the "no batch is open" message.
    pub(crate) fn with_draft<R>(&self, f: impl FnOnce(&mut OpenDraft) -> R) -> Result<R, String> {
        let mut guard = self.draft.lock().unwrap();
        let open = guard
            .as_mut()
            .ok_or_else(|| "No batch is open.".to_string())?;
        Ok(f(open))
    }

    pub(crate) fn clear_draft(&self) {
        *self.draft.lock().unwrap() = None;
    }

    /// Remember the cancel flag of the running read (08 › *Reading*: *Cancel*).
    pub(crate) fn set_cancel(&self, flag: Arc<AtomicBool>) {
        *self.cancel.lock().unwrap() = Some(flag);
    }

    pub(crate) fn clear_cancel(&self) {
        *self.cancel.lock().unwrap() = None;
    }

    /// Stop the running read (08 › *Reading*: *Cancel*); nothing is written.
    pub(crate) fn request_cancel(&self) {
        if let Some(flag) = self.cancel.lock().unwrap().as_ref() {
            flag.store(true, Ordering::Relaxed);
        }
    }
}

/// The folder the program runs from (07 › *Program folder*).
pub(crate) fn program_folder() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(Path::to_path_buf))
        .unwrap_or_default()
}

/// The folder with `pdfium.dll`: the program's own folder, else (in development builds)
/// `pc/vendor`, so both `npm run tauri dev` and the portable build work (07 › *Program folder*).
pub(crate) fn pdfium_dir() -> PathBuf {
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

/// Today's date on the PC (`YYYY-MM-DD`); the Day screen opens here (07 › *Day folder*).
pub(crate) fn today() -> String {
    jiff::Zoned::now().strftime("%Y-%m-%d").to_string()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            commands::day::start_info,
            commands::day::use_program_folder,
            commands::day::choose_data_folder,
            commands::day::day_overview,
            commands::day::days,
            commands::day::settings_get,
            commands::day::settings_set,
            commands::day::about,
            commands::day::open_path,
            commands::day::show_in_folder,
            commands::batch::check_files,
            commands::batch::read_labels,
            commands::batch::cancel_read,
            commands::batch::check_condition,
            commands::batch::plan,
            commands::batch::save_batch,
            commands::batch::discard_draft,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;
    use packing_engine::rules::load_rules;

    #[test]
    fn builtin_categories_are_a_valid_rules_file() {
        assert!(load_rules(BUILTIN_CATEGORIES).is_ok());
    }
}
