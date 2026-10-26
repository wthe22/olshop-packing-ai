//! The packing app window back end: the app state and setup. The window commands live in
//! [`commands`]; the batch steps they call are `packing-batch`'s.
//!
//! The data folder is the one `packing-settings.toml` beside the program names, locked while the
//! app runs. The open date (its journal folded) and the open draft are app state, never stored:
//! the journal is the only truth.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use jiff::civil::Date;
use packing_batch::Draft;
use packing_engine::sorting::PlanGroup;
use packing_pdf::worker::PdfWorker;
use packing_store::{
    DataLock, DateFolder, InputProblem, Journal, OutputProblem, StoreError, open_date,
};
use tauri::Manager;

pub mod commands;
mod index;
mod settings;

pub(crate) use index::Index;

/// The app version the journal's records carry.
pub(crate) const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

/// The data folder in use, locked; the workspace open in it and that workspace's open date.
pub(crate) struct Data {
    pub dir: PathBuf,
    _lock: DataLock,
    pub workspace: Option<String>,
    pub day: Option<OpenDay>,
}

impl Data {
    pub(crate) fn new(dir: PathBuf) -> Result<Self, StoreError> {
        let lock = DataLock::acquire(&dir)?;
        Ok(Self {
            dir,
            _lock: lock,
            workspace: None,
            day: None,
        })
    }

    /// The workspace's `date`, opened (folded and checked) unless it is the one open.
    pub(crate) fn open_day(&mut self, date: Date) -> Result<&mut OpenDay, String> {
        let workspace = self
            .workspace
            .clone()
            .ok_or_else(|| "No workspace is open yet.".to_string())?;
        let open = self
            .day
            .as_ref()
            .is_some_and(|day| day.folder.workspace() == workspace && day.folder.date() == date);
        if !open {
            let folder = DateFolder::new(&self.dir, &workspace, date);
            let opened = open_date(&folder).map_err(|error| error.to_string())?;
            self.day = Some(OpenDay {
                folder,
                journal: opened.journal,
                inputs: opened.inputs,
                outputs: opened.outputs,
            });
        }
        Ok(self.day.as_mut().expect("opened above"))
    }
}

/// A date of the open workspace, folded, and what its open-time checks found.
pub(crate) struct OpenDay {
    pub folder: DateFolder,
    pub journal: Journal,
    pub inputs: Vec<InputProblem>,
    pub outputs: Vec<OutputProblem>,
}

/// One group of the open draft's plan as the owner left it.
#[derive(Clone, Debug)]
pub(crate) struct PlanEntry {
    pub group: PlanGroup,
    /// Skipped for this batch only.
    pub skipped: bool,
}

/// A batch read but not saved, on the date it was read for.
pub(crate) struct OpenDraft {
    pub date: Date,
    pub draft: Draft,
    pub entries: Vec<PlanEntry>,
    /// Each unseen logo's hash and the logo as printed (a PNG data URL); none when no page could
    /// be read, and the prompt shows the hash.
    pub logo_pictures: HashMap<String, Option<String>>,
    /// Unseen logos answered *I don't know*: the prompt asks no more about them.
    pub skipped_logos: Vec<String>,
}

impl OpenDraft {
    /// Sort the draft by the groups not skipped, in the owner's order.
    pub(crate) fn apply_plan(&mut self) {
        self.draft.plan = self
            .entries
            .iter()
            .filter(|entry| !entry.skipped)
            .map(|entry| entry.group.clone())
            .collect();
    }
}

/// What the window back end keeps between commands. None of it is stored.
pub struct AppState {
    program: PathBuf,
    data: Mutex<Option<Data>>,
    worker: Mutex<Option<PdfWorker>>,
    draft: Mutex<Option<OpenDraft>>,
    cancel: Mutex<Option<Arc<AtomicBool>>>,
    index: Index,
}

impl AppState {
    pub(crate) fn new(program: PathBuf) -> Self {
        Self {
            program,
            data: Mutex::new(None),
            worker: Mutex::new(None),
            draft: Mutex::new(None),
            cancel: Mutex::new(None),
            index: Index::default(),
        }
    }

    /// The folder the program runs from: `packing-settings.toml` and `packing-workflows.toml`.
    pub(crate) fn program(&self) -> &Path {
        &self.program
    }

    pub(crate) fn has_data(&self) -> bool {
        self.data.lock().unwrap().is_some()
    }

    /// Use `data` from now on; the old folder's lock is released.
    pub(crate) fn set_data(&self, data: Data) {
        *self.data.lock().unwrap() = Some(data);
    }

    /// Run `f` with the data folder, or say that none is set.
    pub(crate) fn with_data<R>(
        &self,
        f: impl FnOnce(&mut Data) -> Result<R, String>,
    ) -> Result<R, String> {
        let mut guard = self.data.lock().unwrap();
        let data = guard
            .as_mut()
            .ok_or_else(|| "No data folder is set yet.".to_string())?;
        f(data)
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

    /// Run `f` with the PDFium worker while holding its lock.
    pub(crate) fn with_worker<R>(&self, f: impl FnOnce(&PdfWorker) -> R) -> Result<R, String> {
        let guard = self.worker.lock().unwrap();
        let worker = guard
            .as_ref()
            .ok_or_else(|| "The PDF library is not available.".to_string())?;
        Ok(f(worker))
    }

    /// Keep a freshly read draft, replacing any earlier one.
    pub(crate) fn set_draft(&self, open: OpenDraft) {
        *self.draft.lock().unwrap() = Some(open);
    }

    /// Run `f` with the open draft, or return the "no batch is open" message.
    pub(crate) fn with_draft<R>(
        &self,
        f: impl FnOnce(&mut OpenDraft) -> Result<R, String>,
    ) -> Result<R, String> {
        let mut guard = self.draft.lock().unwrap();
        let open = guard
            .as_mut()
            .ok_or_else(|| "No batch is open.".to_string())?;
        f(open)
    }

    pub(crate) fn clear_draft(&self) {
        *self.draft.lock().unwrap() = None;
    }

    /// Remember the cancel flag of the running read.
    pub(crate) fn set_cancel(&self, flag: Arc<AtomicBool>) {
        *self.cancel.lock().unwrap() = Some(flag);
    }

    pub(crate) fn clear_cancel(&self) {
        *self.cancel.lock().unwrap() = None;
    }

    /// The search index of the open workspace.
    pub(crate) fn index(&self) -> &Index {
        &self.index
    }

    /// Stop the running read; nothing is written.
    pub(crate) fn request_cancel(&self) {
        if let Some(flag) = self.cancel.lock().unwrap().as_ref() {
            flag.store(true, Ordering::Relaxed);
        }
    }
}

/// The folder the program runs from.
pub(crate) fn program_folder() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(Path::to_path_buf))
        .unwrap_or_default()
}

/// The folder with `pdfium.dll`: the program's own folder, else (in development builds)
/// `vendor`, so both `npm run tauri dev` and the portable build work.
pub(crate) fn pdfium_dir() -> PathBuf {
    let exe_dir = program_folder();
    if exe_dir.join("pdfium.dll").is_file() {
        return exe_dir;
    }
    #[cfg(debug_assertions)]
    {
        let vendor = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../vendor");
        if vendor.join("pdfium.dll").is_file() {
            return vendor;
        }
    }
    exe_dir
}

/// Today's date on the PC; the Day screen opens there.
pub(crate) fn today() -> Date {
    jiff::Zoned::now().date()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::new(program_folder()))
        .setup(|app| {
            // The OS window title carries the version from the one place
            // ([`commands::start::window_title`]). It is set here, on the Rust side: setting it
            // from the window would need the `core:window:allow-set-title` permission, which no
            // screen needs. The window `title` in `tauri.conf.json` is the bare name.
            if let Some(window) = app.get_webview_window("main") {
                window.set_title(&commands::start::window_title())?;
                // The title bar takes the stored theme before the window draws; a settings file
                // that does not read is reported by `start_info`.
                let theme = settings::theme(&program_folder()).unwrap_or_default();
                window.set_theme(commands::start::window_theme(theme))?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::start::start_info,
            commands::start::use_program_folder,
            commands::start::choose_data_folder,
            commands::start::workspace_new,
            commands::start::about,
            commands::start::set_theme,
            commands::start::open_path,
            commands::start::show_in_folder,
            commands::day::day_overview,
            commands::day::set_printed,
            commands::day::revert_last,
            commands::day::restore_output,
            commands::day::day_summary,
            commands::panel::workspaces_list,
            commands::panel::workspace_open,
            commands::search::search_orders,
            commands::search::reprint,
            commands::search::reprint_orders_of,
            commands::batch::check_files,
            commands::batch::sort_rules,
            commands::batch::read_labels,
            commands::batch::cancel_read,
            commands::batch::answer_logo,
            commands::batch::check_condition,
            commands::rules::rules_list,
            commands::rules::rules_load,
            commands::rules::rules_check,
            commands::rules::rules_save,
            commands::rules::rule_new,
            commands::rules::rule_rename,
            commands::rules::rule_delete,
            commands::rules::rules_check_text,
            commands::rules::rules_save_text,
            commands::rules::filter_to_tree,
            commands::rules::tree_to_filter,
            commands::batch::resolve_repeats,
            commands::batch::plan,
            commands::batch::save_batch,
            commands::batch::discard_draft,
            commands::batch::start_amend,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
