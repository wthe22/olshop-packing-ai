//! Start (program.md): the data folder `packing-settings.toml` names, its lock, the rules beside
//! the program and the owner opened; the first owner's name; *About*, the version, and opening a
//! file or folder in Explorer.

use std::path::{Path, PathBuf};
use std::process::Command;

use packing_pdf::PDFIUM_VERSION;
use packing_store::{create_namespace, last_written, namespaces};
use serde::Serialize;
use tauri::State;

use crate::settings::{self, DEFAULT_DATA_DIR};
use crate::{AppState, Data, today};

/// The program's name as the window title and the title bar show it.
pub(crate) const PROGRAM_NAME: &str = "Packing";

/// The shown form of the program's version: `v` + the first two numbers of the workspace
/// `version` (`0.2.0` → `v0.2`). The app inherits the workspace version, so the root `Cargo.toml`
/// is the one place; the window title, the title bar and *About* all take this text.
pub(crate) fn version_text() -> String {
    let mut numbers = crate::APP_VERSION.split('.');
    let major = numbers.next().unwrap_or_default();
    let minor = numbers.next().unwrap_or_default();
    format!("v{major}.{minor}")
}

/// The OS window title and the title bar's brand text.
pub(crate) fn window_title() -> String {
    format!("{PROGRAM_NAME} {}", version_text())
}

/// What the window needs to open the Start prompt, the first owner's name or the Day screen.
#[derive(Serialize)]
pub struct StartInfo {
    /// Where the program is: a first start offers its `data` folder.
    exe_dir: String,
    /// The data folder in use; none until `packing-settings.toml` names one.
    data_folder: Option<String>,
    /// The owner open; none until the first owner is named.
    owner: Option<String>,
    /// Whether the PDFium worker started (its dll was found).
    pdfium_ready: bool,
    /// Today's date on the PC (`YYYY-MM-DD`); the Day screen opens there.
    today: String,
    version: String,
}

/// Open the data folder `packing-settings.toml` names, as at start: the rules checked, the lock
/// taken, the owner whose journal was written last (else the first one) opened.
pub(crate) fn start_information(state: &AppState) -> Result<StartInfo, String> {
    let program = state.program();
    settings::workflows(program)?;
    if !state.has_data()
        && let Some(dir) = settings::data_dir(program)?
    {
        open_data(state, dir)?;
    }
    let (data_folder, owner) = state
        .with_data(|data| Ok((Some(data.dir.display().to_string()), data.owner.clone())))
        .unwrap_or((None, None));
    Ok(StartInfo {
        exe_dir: program.display().to_string(),
        data_folder,
        owner,
        pdfium_ready: state.ensure_worker(),
        today: today().to_string(),
        version: version_text(),
    })
}

/// Take `dir`'s lock and open its owner; another copy holding it stops with the copy named.
fn open_data(state: &AppState, dir: PathBuf) -> Result<(), String> {
    let mut data = Data::new(dir).map_err(|error| error.to_string())?;
    data.owner = match last_written(&data.dir).map_err(|error| error.to_string())? {
        Some(owner) => Some(owner),
        None => namespaces(&data.dir)
            .map_err(|error| error.to_string())?
            .into_iter()
            .next(),
    };
    state.set_data(data);
    Ok(())
}

/// Write `packing-settings.toml` naming `dir` and open it; the folder in use is kept as it is.
pub(crate) fn use_folder(state: &AppState, dir: &Path) -> Result<String, String> {
    let dir = settings::set_data_dir(state.program(), dir)?;
    let in_use = state.with_data(|data| Ok(data.dir == dir)).unwrap_or(false);
    if !in_use {
        open_data(state, dir.clone())?;
    }
    Ok(dir.display().to_string())
}

/// Name an owner: the folder is made at once and opened.
pub(crate) fn new_owner(state: &AppState, name: &str) -> Result<String, String> {
    state.with_data(|data| {
        create_namespace(&data.dir, name).map_err(|error| error.to_string())?;
        data.owner = Some(name.to_string());
        data.day = None;
        Ok(name.to_string())
    })
}

#[tauri::command]
pub fn start_info(state: State<'_, AppState>) -> Result<StartInfo, String> {
    start_information(&state)
}

/// *Use this folder*: the `data` folder inside the program's folder.
#[tauri::command]
pub fn use_program_folder(state: State<'_, AppState>) -> Result<String, String> {
    let dir = state.program().join(DEFAULT_DATA_DIR);
    use_folder(&state, &dir)
}

/// *Choose a folder…* / Settings *Change…*.
#[tauri::command]
pub fn choose_data_folder(state: State<'_, AppState>, path: String) -> Result<String, String> {
    use_folder(&state, Path::new(&path))
}

/// The first owner's name, or **+ New owner**.
#[tauri::command]
pub fn owner_new(state: State<'_, AppState>, name: String) -> Result<String, String> {
    new_owner(&state, &name)
}

/// Settings › *About*.
#[derive(Serialize)]
pub struct About {
    version: String,
    pdfium: String,
    schema: String,
}

#[tauri::command]
pub fn about() -> About {
    About {
        version: version_text(),
        pdfium: PDFIUM_VERSION.to_string(),
        schema: packing_engine::record::SCHEMA.to_string(),
    }
}

/// The Explorer command for `path` (separate so a test can read its arguments back).
fn explorer_command(path: &str) -> Command {
    let mut command = Command::new("explorer");
    command.arg(path);
    command
}

/// Open a file with the PC's default program, or a folder as a folder in Explorer. The path is
/// Explorer's **only** argument: Explorer parses `/select,<path>` itself and does not understand
/// the quotes Rust puts around an argument that holds a space, so it opened *Documents* instead.
fn open_with_explorer(path: &str) -> Result<(), String> {
    explorer_command(path)
        .spawn()
        .map(|_| ())
        .map_err(|error| format!("Cannot open {path}: {error}."))
}

#[tauri::command]
pub fn open_path(path: String) -> Result<(), String> {
    open_with_explorer(&path)
}

#[tauri::command]
pub fn show_in_folder(path: String) -> Result<(), String> {
    open_with_explorer(&path)
}

#[cfg(test)]
mod tests;
