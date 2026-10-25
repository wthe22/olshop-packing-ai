//! The courier data: the `couriers.toml` table, its loading and the names it holds. The fields a
//! page is read for and the deduction are in the modules beside this file.
//!
//! The courier data lives in the data folder's `couriers.toml` and is the owner's, like
//! `categories.toml`; the bundled file is the default of a fresh data folder and of a folder
//! without the file.

use std::fmt;
use std::path::Path;
use std::sync::OnceLock;

use regex::Regex;

mod deduce;
mod fields;

pub use self::deduce::{
    candidates, deduce_courier, deduce_courier_in, form_names_courier_in, is_unseen_logo,
    logo_courier,
};
pub use self::fields::{find_order_id, find_ship_by, find_tracking_id};

/// The courier data built into the program, written into a new data folder.
pub const COURIERS_TOML: &str = include_str!("../../couriers.toml");

/// The courier file in the data folder.
pub const COURIERS_FILE_NAME: &str = "couriers.toml";

/// Invalid courier file; the message names the place (`couriers.toml`, its line).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CouriersError {
    pub message: String,
}

impl CouriersError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for CouriersError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for CouriersError {}

/// One `[[courier]]` entry, its regexes compiled once.
#[derive(Clone, Debug)]
pub struct Courier {
    pub name: String,
    /// The courier's logo images: XXH3-64 (16 hex) of each one's **encoded bytes**.
    pub logo: Vec<String>,
    /// The courier's logos by **appearance**: XXH3-64 (16 hex) of a 64 × 64 1-bit canvas. The
    /// clue for a **masked** logo, whose encoded bytes name nothing (J&T Cargo, courier.md item 3).
    pub shape: Vec<String>,
    /// Case-insensitive text clues searched in the page text: the logo's stand-in where the logo
    /// cannot be read at all (courier.md item 5).
    pub text: Vec<String>,
    /// The tracking-ID pattern source (as written in `couriers.toml`).
    pub tracking: Option<String>,
    /// `tracking` unanchored (used for the collapsed-text search).
    pub tracking_re: Option<Regex>,
    /// `tracking` anchored: the whole token must match.
    pub tracking_full: Option<Regex>,
}

impl Courier {
    /// One courier for a table of the rule's tests; `tracking` is a regex source.
    pub fn new(
        name: &str,
        logo: &[&str],
        shape: &[&str],
        tracking: Option<&str>,
        text: &[&str],
    ) -> Courier {
        Courier::from_entry(
            name.to_string(),
            logo.iter().map(|value| (*value).to_string()).collect(),
            shape.iter().map(|value| (*value).to_string()).collect(),
            text.iter().map(|value| (*value).to_string()).collect(),
            tracking.map(str::to_string),
        )
    }

    fn from_entry(
        name: String,
        logo: Vec<String>,
        shape: Vec<String>,
        text: Vec<String>,
        tracking: Option<String>,
    ) -> Courier {
        let (tracking_re, tracking_full) = match tracking.as_deref() {
            Some(pattern) => (
                Some(Regex::new(pattern).expect("courier tracking pattern is valid")),
                Some(
                    Regex::new(&format!("^(?:{pattern})$"))
                        .expect("courier tracking pattern is valid"),
                ),
            ),
            None => (None, None),
        };
        Courier {
            name,
            logo,
            shape,
            text,
            tracking,
            tracking_re,
            tracking_full,
        }
    }
}

/// The courier data of `couriers.toml`: the couriers and the hashes answered *Not a courier*.
#[derive(Clone, Debug)]
pub struct CourierTable {
    pub couriers: Vec<Courier>,
    /// Hashes answered **Not a courier** — a buying channel's strip — and so ignored
    /// wherever a logo is looked up. Written back by the logo prompt.
    pub not_courier: Vec<String>,
}

impl CourierTable {
    /// Whether `hash` is a **byte** hash the data already knows: in a courier's `logo` list or in
    /// `not_courier`. `packing-pdf` computes the appearance hash only for a band image whose byte
    /// hash this does not know (courier.md item 3).
    pub fn knows_byte_hash(&self, hash: &str) -> bool {
        self.not_courier.iter().any(|known| known == hash)
            || self
                .couriers
                .iter()
                .any(|courier| courier.logo.iter().any(|known| known == hash))
    }

    /// The courier names in file order: the logo prompt's buttons.
    pub fn names(&self) -> Vec<String> {
        self.couriers
            .iter()
            .map(|courier| courier.name.clone())
            .collect()
    }
}

/// The courier data built into the program, parsed once: the default of a data
/// folder without a `couriers.toml`.
pub fn courier_data() -> &'static CourierTable {
    static DATA: OnceLock<CourierTable> = OnceLock::new();
    DATA.get_or_init(|| load_couriers(COURIERS_TOML).expect("the built-in couriers.toml is valid"))
}

/// The couriers of the built-in data.
pub fn couriers() -> &'static [Courier] {
    &courier_data().couriers
}

/// The courier data of the data folder's file: its text, or the bundled file when `path` does not
/// exist. A broken file is an error naming its line.
pub fn load_couriers_file(path: &Path) -> Result<CourierTable, CouriersError> {
    match std::fs::read_to_string(path) {
        Ok(text) => load_couriers(&text),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(courier_data().clone()),
        Err(error) => Err(CouriersError::new(format!(
            "cannot read {}: {error}",
            path.display()
        ))),
    }
}

const COURIERS_KEYS: &[&str] = &["not_courier", "courier"];
const COURIER_KEYS: &[&str] = &["name", "logo", "shape", "text", "tracking"];

fn is_hash(value: &str) -> bool {
    value.len() == 16
        && value
            .chars()
            .all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c))
}

/// A `[[courier]]` entry's string list (`logo`, `shape`, `text`), empty when the key is absent.
/// `place` names what the list belongs to (`courier "SiCepat": `, empty for a top-level list).
fn string_list(
    value: Option<&toml::Value>,
    place: &str,
    key: &str,
) -> Result<Vec<String>, CouriersError> {
    let Some(value) = value else {
        return Ok(Vec::new());
    };
    let toml::Value::Array(entries) = value else {
        return Err(CouriersError::new(format!(
            "{COURIERS_FILE_NAME}: {place}\"{key}\" must be an array of strings"
        )));
    };
    let mut list = Vec::with_capacity(entries.len());
    for entry in entries {
        let toml::Value::String(text) = entry else {
            return Err(CouriersError::new(format!(
                "{COURIERS_FILE_NAME}: {place}\"{key}\" must be an array of strings"
            )));
        };
        list.push(text.clone());
    }
    Ok(list)
}

/// A list of XXH3-64 hashes (16 lowercase hex characters), empty when the key is absent.
fn hash_list(
    value: Option<&toml::Value>,
    place: &str,
    key: &str,
) -> Result<Vec<String>, CouriersError> {
    let list = string_list(value, place, key)?;
    for hash in &list {
        if !is_hash(hash) {
            return Err(CouriersError::new(format!(
                "{COURIERS_FILE_NAME}: {place}\"{key}\" entry \"{hash}\" is not 16 hex characters \
                 (an XXH3-64 hash)"
            )));
        }
    }
    Ok(list)
}

/// One `[[courier]]` entry; errors name the courier by its `name`, else by its position.
fn courier_of(entry: &toml::Value, index: usize) -> Result<Courier, CouriersError> {
    let toml::Value::Table(entry) = entry else {
        return Err(CouriersError::new(format!(
            "{COURIERS_FILE_NAME}: courier #{}: must be a table",
            index + 1
        )));
    };
    let cid = match entry.get("name") {
        Some(toml::Value::String(name)) if !name.is_empty() => format!("\"{name}\""),
        _ => format!("#{}", index + 1),
    };
    for key in entry.keys() {
        if !COURIER_KEYS.contains(&key.as_str()) {
            return Err(CouriersError::new(format!(
                "{COURIERS_FILE_NAME}: courier {cid}: unknown key \"{key}\""
            )));
        }
    }

    let Some(name_value) = entry.get("name") else {
        return Err(CouriersError::new(format!(
            "{COURIERS_FILE_NAME}: courier {cid}: missing \"name\""
        )));
    };
    let toml::Value::String(name) = name_value else {
        return Err(CouriersError::new(format!(
            "{COURIERS_FILE_NAME}: courier {cid}: \"name\" must be a string"
        )));
    };
    if name.is_empty() {
        return Err(CouriersError::new(format!(
            "{COURIERS_FILE_NAME}: courier {cid}: \"name\" must not be empty"
        )));
    }

    let place = format!("courier {cid}: ");
    let logo = hash_list(entry.get("logo"), &place, "logo")?;
    let shape = hash_list(entry.get("shape"), &place, "shape")?;
    let text = string_list(entry.get("text"), &place, "text")?;
    let tracking = match entry.get("tracking") {
        None => None,
        Some(toml::Value::String(pattern)) => Some(pattern.clone()),
        Some(_) => {
            return Err(CouriersError::new(format!(
                "{COURIERS_FILE_NAME}: courier {cid}: \"tracking\" must be a string"
            )));
        }
    };
    if let Some(pattern) = &tracking
        && let Err(error) = Regex::new(pattern)
    {
        return Err(CouriersError::new(format!(
            "{COURIERS_FILE_NAME}: courier {cid}: \"tracking\" is not a valid pattern: {error}"
        )));
    }

    Ok(Courier::from_entry(
        name.clone(),
        logo,
        shape,
        text,
        tracking,
    ))
}

/// Load the couriers from the text of `couriers.toml` (the app reads the file). A TOML error and
/// every field error name the file and, where the TOML parser gives it, the line.
pub fn load_couriers(text: &str) -> Result<CourierTable, CouriersError> {
    let table: toml::Table = toml::from_str(text)
        .map_err(|error| CouriersError::new(format!("{COURIERS_FILE_NAME}: {error}")))?;

    for key in table.keys() {
        if !COURIERS_KEYS.contains(&key.as_str()) {
            return Err(CouriersError::new(format!(
                "{COURIERS_FILE_NAME}: unknown key \"{key}\""
            )));
        }
    }

    let not_courier = match table.get("not_courier") {
        None => Vec::new(),
        Some(value) => hash_list(Some(value), "", "not_courier")?,
    };

    let raw = match table.get("courier") {
        None => {
            return Err(CouriersError::new(format!(
                "{COURIERS_FILE_NAME}: no couriers"
            )));
        }
        Some(toml::Value::Array(entries)) => entries,
        Some(_) => {
            return Err(CouriersError::new(format!(
                "{COURIERS_FILE_NAME}: \"courier\" must be an array of tables"
            )));
        }
    };
    if raw.is_empty() {
        return Err(CouriersError::new(format!(
            "{COURIERS_FILE_NAME}: no couriers"
        )));
    }

    let mut couriers = Vec::with_capacity(raw.len());
    for (index, entry) in raw.iter().enumerate() {
        couriers.push(courier_of(entry, index)?);
    }
    Ok(CourierTable {
        couriers,
        not_courier,
    })
}
