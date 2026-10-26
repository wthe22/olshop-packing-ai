//! The two files beside the program (data/README.md): `packing-settings.toml`, which names the data
//! folder and the theme, and `packing-workflows.toml`, the owner's rules, written from the bundled
//! copy when it is missing and from then on the owner's.

use std::fs;
use std::path::{Path, PathBuf};

use packing_engine::record::SCHEMA;
use packing_engine::workflows::{BUNDLED_WORKFLOWS, WORKFLOWS_FILE, Workflows, load_workflows};
use serde::{Deserialize, Serialize};

pub(crate) const SETTINGS_FILE: &str = "packing-settings.toml";

/// The data folder a first start offers: `data` inside the program's folder.
pub(crate) const DEFAULT_DATA_DIR: &str = "data";

/// The window's colours: Windows' own choice, or always light or dark.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    #[default]
    System,
    Light,
    Dark,
}

#[derive(Serialize, Deserialize)]
struct Root {
    schema: String,
    data_dir: Option<PathBuf>,
    #[serde(default)]
    theme: Theme,
}

impl Default for Root {
    fn default() -> Self {
        Root {
            schema: SCHEMA.to_string(),
            data_dir: None,
            theme: Theme::default(),
        }
    }
}

/// `packing-settings.toml` as read; the defaults when the file is missing.
fn read_root(program: &Path) -> Result<Root, String> {
    let path = program.join(SETTINGS_FILE);
    let text = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Root::default()),
        Err(error) => return Err(format!("Cannot read {}: {error}.", path.display())),
    };
    toml::from_str(&text).map_err(|error| format!("{SETTINGS_FILE}: {error}"))
}

fn write_root(program: &Path, root: &Root) -> Result<(), String> {
    let text = toml::to_string(root).expect("the settings are plain TOML");
    let path = program.join(SETTINGS_FILE);
    fs::write(&path, text).map_err(|error| format!("Cannot write {}: {error}.", path.display()))
}

/// The data folder `packing-settings.toml` names, a relative path taken from the program's
/// folder; none when the file or its `data_dir` is missing.
pub(crate) fn data_dir(program: &Path) -> Result<Option<PathBuf>, String> {
    Ok(read_root(program)?.data_dir.map(|dir| program.join(dir)))
}

/// Write `packing-settings.toml` naming `dir`, relative to the program's folder when it is inside
/// it. Returns the folder as the app uses it.
pub(crate) fn set_data_dir(program: &Path, dir: &Path) -> Result<PathBuf, String> {
    let stored = dir
        .strip_prefix(program)
        .map_or_else(|_| dir.to_path_buf(), Path::to_path_buf);
    let mut root = read_root(program)?;
    root.schema = SCHEMA.to_string();
    root.data_dir = Some(stored.clone());
    write_root(program, &root)?;
    Ok(program.join(stored))
}

/// The theme `packing-settings.toml` names; `system` when it names none.
pub(crate) fn theme(program: &Path) -> Result<Theme, String> {
    Ok(read_root(program)?.theme)
}

/// Write the theme into `packing-settings.toml`, the data folder kept.
pub(crate) fn set_theme(program: &Path, theme: Theme) -> Result<(), String> {
    let mut root = read_root(program)?;
    root.theme = theme;
    write_root(program, &root)
}

/// The rules beside the program, as text and as read; the bundled copy is written there first
/// when there is none. A file that does not load stops with its line and column.
pub(crate) fn workflows(program: &Path) -> Result<(String, Workflows), String> {
    let path = program.join(WORKFLOWS_FILE);
    if !path.is_file() {
        write_workflows(program, BUNDLED_WORKFLOWS)?;
    }
    let text = fs::read_to_string(&path)
        .map_err(|error| format!("Cannot read {}: {error}.", path.display()))?;
    let rules = load_workflows(&text).map_err(|error| error.to_string())?;
    Ok((text, rules))
}

/// Replace `packing-workflows.toml` with `text`, through a temporary file renamed into place.
pub(crate) fn write_workflows(program: &Path, text: &str) -> Result<(), String> {
    let path = program.join(WORKFLOWS_FILE);
    let temporary = path.with_extension("toml.partial");
    fs::write(&temporary, text)
        .and_then(|()| fs::rename(&temporary, &path))
        .map_err(|error| format!("Cannot write {}: {error}.", path.display()))
}
