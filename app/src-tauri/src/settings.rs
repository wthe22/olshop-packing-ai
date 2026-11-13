//! The files the app keeps (data/README.md): `packing-app.toml` beside the program, the app's own
//! preferences (the data folder and the theme); and, inside the data folder, `parser.toml` (how
//! labels are read) and `config.toml` (the accounts and the sort rules), each written from the
//! bundled copy when it is missing and from then on the owner's.

use std::fs::{self, OpenOptions};
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};

use packing_engine::accounts::Account;
use packing_engine::record::SCHEMA;
use packing_engine::workflows::{
    BUNDLED_CONFIG, BUNDLED_PARSER, CONFIG_FILE, PARSER_FILE, Workflows, load_workflows,
    write_accounts,
};
use serde::{Deserialize, Serialize};

pub(crate) const APP_FILE: &str = "packing-app.toml";

/// The data folder a first start offers: `packing-data` inside the program's folder.
pub(crate) const DEFAULT_DATA_DIR: &str = "packing-data";

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

/// `packing-app.toml` as read; the defaults when the file is missing.
fn read_root(program: &Path) -> Result<Root, String> {
    let path = program.join(APP_FILE);
    let text = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(Root::default()),
        Err(error) => return Err(format!("Cannot read {}: {error}.", path.display())),
    };
    toml::from_str(&text).map_err(|error| format!("{APP_FILE}: {error}"))
}

fn write_root(program: &Path, root: &Root) -> Result<(), String> {
    let text = toml::to_string(root).expect("the preferences are plain TOML");
    let path = program.join(APP_FILE);
    fs::write(&path, text).map_err(|error| format!("Cannot write {}: {error}.", path.display()))
}

/// The data folder `packing-app.toml` names, a relative path taken from the program's folder;
/// none when the file or its `data_dir` is missing.
pub(crate) fn data_dir(program: &Path) -> Result<Option<PathBuf>, String> {
    Ok(read_root(program)?.data_dir.map(|dir| program.join(dir)))
}

/// Write `packing-app.toml` naming `dir`, relative to the program's folder when it is inside it.
/// Returns the folder as the app uses it.
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

/// The theme `packing-app.toml` names; `system` when it names none.
pub(crate) fn theme(program: &Path) -> Result<Theme, String> {
    Ok(read_root(program)?.theme)
}

/// Write the theme into `packing-app.toml`, the data folder kept.
pub(crate) fn set_theme(program: &Path, theme: Theme) -> Result<(), String> {
    let mut root = read_root(program)?;
    root.theme = theme;
    write_root(program, &root)
}

/// The data folder in use, which holds `parser.toml` and `config.toml`.
fn data_folder(program: &Path) -> Result<PathBuf, String> {
    data_dir(program)?.ok_or_else(|| "No data folder is set yet.".to_string())
}

/// Write `text` to `path` unless a file is there already, so a copy that starts at the same time
/// never replaces the other's file.
fn write_if_missing(path: &Path, text: &str) -> Result<(), String> {
    let created = OpenOptions::new().write(true).create_new(true).open(path);
    match created {
        Ok(mut file) => file.write_all(text.as_bytes()),
        Err(error) if error.kind() == ErrorKind::AlreadyExists => Ok(()),
        Err(error) => Err(error),
    }
    .map_err(|error| format!("Cannot write {}: {error}.", path.display()))
}

/// The texts of `parser.toml` and `config.toml`. A missing file is written first, from the
/// bundled copy.
fn texts(program: &Path) -> Result<(String, String), String> {
    let dir = data_folder(program)?;
    let (parser, config) = (dir.join(PARSER_FILE), dir.join(CONFIG_FILE));
    if !parser.is_file() || !config.is_file() {
        fs::create_dir_all(&dir)
            .map_err(|error| format!("Cannot create {}: {error}.", dir.display()))?;
        write_if_missing(&parser, BUNDLED_PARSER)?;
        write_if_missing(&config, BUNDLED_CONFIG)?;
    }
    let read = |path: &Path| {
        fs::read_to_string(path)
            .map_err(|error| format!("Cannot read {}: {error}.", path.display()))
    };
    Ok((read(&parser)?, read(&config)?))
}

/// The rules of the data folder, with the text of `config.toml` (the sort rules and the accounts)
/// they were read from. A file that does not load stops with its name, line and column.
pub(crate) fn workflows(program: &Path) -> Result<(String, Workflows), String> {
    let (parser, config) = texts(program)?;
    let rules = load_workflows(&parser, &config).map_err(|error| error.to_string())?;
    Ok((config, rules))
}

/// The text of `parser.toml`, which the checks of `config.toml` read.
pub(crate) fn parser_text(program: &Path) -> Result<String, String> {
    texts(program).map(|(parser, _)| parser)
}

/// Replace `config.toml` with `text`, through a temporary file renamed into place.
pub(crate) fn write_config(program: &Path, text: &str) -> Result<(), String> {
    replace(&data_folder(program)?.join(CONFIG_FILE), text)
}

/// Replace `parser.toml` with `text`, through a temporary file renamed into place.
pub(crate) fn write_parser(program: &Path, text: &str) -> Result<(), String> {
    replace(&data_folder(program)?.join(PARSER_FILE), text)
}

fn replace(path: &Path, text: &str) -> Result<(), String> {
    let temporary = path.with_extension("toml.partial");
    fs::write(&temporary, text)
        .and_then(|()| fs::rename(&temporary, path))
        .map_err(|error| format!("Cannot write {}: {error}.", path.display()))
}

/// The accounts `config.toml` lists, in the order they were made.
pub(crate) fn accounts(program: &Path) -> Result<Vec<Account>, String> {
    Ok(workflows(program)?.1.accounts)
}

/// Change the accounts of `config.toml` as it is now, so another copy's change is kept; the
/// caller holds the data folder's lock. Nothing is written when `change` refuses. Returns the
/// accounts written.
pub(crate) fn change_accounts(
    program: &Path,
    change: impl FnOnce(&mut Vec<Account>) -> Result<(), String>,
) -> Result<Vec<Account>, String> {
    let (parser, config) = texts(program)?;
    let mut accounts = load_workflows(&parser, &config)
        .map_err(|error| error.to_string())?
        .accounts;
    change(&mut accounts)?;
    let (written, rules) =
        write_accounts(&parser, &config, &accounts).map_err(|error| error.to_string())?;
    write_config(program, &written)?;
    Ok(rules.accounts)
}

/// Change `config.toml` as it is now with `edit`, which gets the two files' texts and returns the
/// new config and the rules it holds; the caller holds the data folder's lock. Nothing is written
/// when `edit` refuses. Returns the rules as they were and as they are now.
pub(crate) fn change_config(
    program: &Path,
    edit: impl FnOnce(&str, &str) -> Result<(String, Workflows), String>,
) -> Result<(Workflows, Workflows), String> {
    let (parser, config) = texts(program)?;
    let before = load_workflows(&parser, &config).map_err(|error| error.to_string())?;
    let (written, after) = edit(&parser, &config)?;
    write_config(program, &written)?;
    Ok((before, after))
}

/// Change `parser.toml` as it is now with `edit`, which gets the two files' texts and returns the
/// new parser and the rules it holds; the caller holds the data folder's lock. Returns the rules.
pub(crate) fn change_parser(
    program: &Path,
    edit: impl FnOnce(&str, &str) -> Result<(String, Workflows), String>,
) -> Result<Workflows, String> {
    let (parser, config) = texts(program)?;
    let (written, rules) = edit(&parser, &config)?;
    write_parser(program, &written)?;
    Ok(rules)
}
