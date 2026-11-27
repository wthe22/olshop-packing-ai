//! The files the app keeps (data/README.md): `packing-app.toml` beside the program, the app's own
//! preferences (the data folder, the theme and archiving on its own); and, inside the data folder, `parser.toml` (how
//! labels are read) and `config.toml` (the accounts and the sort rules), each written from the
//! bundled copy when it is missing and from then on the owner's.

use std::fs::{self, OpenOptions};
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};

use packing_engine::accounts::Account;
use packing_engine::record::SCHEMA;
use packing_engine::schema::{Schema, SchemaFit};
use packing_engine::unknown::{UnknownKey, unknown_toml_keys};
use packing_engine::workflows::{
    BUNDLED_CONFIG, BUNDLED_PARSER, CONFIG_FILE, PARSER_FILE, Workflows, load_workflows,
    of_a_newer_version, unknown_rules_keys, write_accounts,
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

impl Theme {
    fn as_str(self) -> &'static str {
        match self {
            Theme::System => "system",
            Theme::Light => "light",
            Theme::Dark => "dark",
        }
    }
}

/// What this copy reads of `packing-app.toml`; a key it does not know is passed over.
#[derive(Default, Deserialize)]
struct Root {
    data_dir: Option<PathBuf>,
    #[serde(default)]
    theme: Theme,
    archive_after_days: Option<u16>,
}

/// `packing-app.toml`'s text; empty when the file is missing.
fn read_text(program: &Path) -> Result<String, String> {
    let path = program.join(APP_FILE);
    match fs::read_to_string(&path) {
        Ok(text) => Ok(text),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(String::new()),
        Err(error) => Err(format!("Cannot read {}: {error}.", path.display())),
    }
}

/// `packing-app.toml` as read, of a version this copy reads; the defaults when it is missing.
fn read_root(program: &Path) -> Result<Root, String> {
    #[derive(Deserialize)]
    struct Version {
        schema: String,
    }
    let text = read_text(program)?;
    if text.is_empty() {
        return Ok(Root::default());
    }
    let error = |message: String| format!("{APP_FILE}: {message}");
    let found = toml::from_str::<Version>(&text)
        .map_err(|e| error(e.message().to_string()))?
        .schema;
    match Schema::parse(&found).map(|version| SCHEMA.fit(version)) {
        Some(SchemaFit::Reads) => {}
        Some(SchemaFit::Newer) => {
            return Err(error(format!(
                "schema {found} is of a newer format: this copy reads {}. Update this copy.",
                SCHEMA.family()
            )));
        }
        Some(SchemaFit::Older) | None => {
            return Err(error(format!(
                "schema {found} is not one this copy reads ({}).",
                SCHEMA.family()
            )));
        }
    }
    toml::from_str(&text).map_err(|e| error(e.message().to_string()))
}

/// Set `key` in `packing-app.toml`, or remove it with `None`, every other key kept as it is, a
/// newer copy's too; a new file starts with this copy's `schema`.
fn write_key(program: &Path, key: &str, value: Option<toml_edit::Value>) -> Result<(), String> {
    let mut doc: toml_edit::DocumentMut = read_text(program)?
        .parse()
        .map_err(|error| format!("{APP_FILE}: {error}"))?;
    if !doc.contains_key("schema") {
        doc["schema"] = toml_edit::value(SCHEMA.to_string());
    }
    match value {
        Some(value) => doc[key] = toml_edit::Item::Value(value),
        None => {
            doc.remove(key);
        }
    }
    let path = program.join(APP_FILE);
    fs::write(&path, doc.to_string())
        .map_err(|error| format!("Cannot write {}: {error}.", path.display()))
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
    read_root(program)?;
    write_key(
        program,
        "data_dir",
        Some(stored.to_string_lossy().as_ref().into()),
    )?;
    Ok(program.join(stored))
}

/// The theme `packing-app.toml` names; `system` when it names none.
pub(crate) fn theme(program: &Path) -> Result<Theme, String> {
    Ok(read_root(program)?.theme)
}

/// Write the theme into `packing-app.toml`, the data folder kept.
pub(crate) fn set_theme(program: &Path, theme: Theme) -> Result<(), String> {
    read_root(program)?;
    write_key(program, "theme", Some(theme.as_str().into()))
}

/// The number of days after which a date is archived at start; none: off.
pub(crate) fn archive_after_days(program: &Path) -> Result<Option<u16>, String> {
    Ok(read_root(program)?.archive_after_days)
}

/// Write `archive_after_days` into `packing-app.toml`, or remove it with `None`, the rest kept.
pub(crate) fn set_archive_after_days(program: &Path, days: Option<u16>) -> Result<(), String> {
    read_root(program)?;
    write_key(
        program,
        "archive_after_days",
        days.map(|days| i64::from(days).into()),
    )
}

/// The keys of `packing-app.toml`, `parser.toml` and `config.toml` this version does not know, as
/// Settings names them: each is kept and not used. A file of a newer version is expected to hold
/// some and names none; a rules file that does not load names none (its error does).
pub(crate) fn unknown_keys(program: &Path) -> Result<Vec<String>, String> {
    let app = read_text(program)?;
    let mut found: Vec<(&str, UnknownKey)> = Vec::new();
    if !of_a_newer_version(&app) {
        // `schema` is read on its own, before the rest.
        found.extend(
            unknown_toml_keys::<Root>(&app)
                .into_iter()
                .filter(|key| key.path != "schema")
                .map(|key| (APP_FILE, key)),
        );
    }
    if data_dir(program)?.is_some() {
        let (parser, config) = texts(program)?;
        found.extend(unknown_rules_keys(&parser, &config));
    }
    Ok(found
        .into_iter()
        .map(|(file, key)| {
            format!("{file}, {key} is not a key this version knows. It is kept and not used.")
        })
        .collect())
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

/// Write `parser.toml` and `config.toml` from the bundled copies where missing; the data folder
/// opened for writing has both.
pub(crate) fn ensure_files(program: &Path) -> Result<(), String> {
    let dir = data_folder(program)?;
    let (parser, config) = (dir.join(PARSER_FILE), dir.join(CONFIG_FILE));
    if !parser.is_file() || !config.is_file() {
        fs::create_dir_all(&dir)
            .map_err(|error| format!("Cannot create {}: {error}.", dir.display()))?;
        write_if_missing(&parser, BUNDLED_PARSER)?;
        write_if_missing(&config, BUNDLED_CONFIG)?;
    }
    Ok(())
}

/// The texts of `parser.toml` and `config.toml`; a missing file reads as its bundled copy and is
/// not written.
fn texts(program: &Path) -> Result<(String, String), String> {
    let dir = data_folder(program)?;
    let read = |name: &str, bundled: &str| {
        let path = dir.join(name);
        match fs::read_to_string(&path) {
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(bundled.to_string()),
            read => read.map_err(|error| format!("Cannot read {}: {error}.", path.display())),
        }
    };
    Ok((
        read(PARSER_FILE, BUNDLED_PARSER)?,
        read(CONFIG_FILE, BUNDLED_CONFIG)?,
    ))
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
