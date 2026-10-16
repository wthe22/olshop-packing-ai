//! The courier data as an editable document: the logo prompt's answers are written into
//! `couriers.toml`, keeping the file's comments and order (D-LABEL-4, Q-LABEL-7 default;
//! 07 › *Inputs*).
//!
//! Like `RulesDocument` for `categories.toml`, the file is edited through `toml_edit`, so a
//! comment and the entries the prompt did not change survive a write. The app writes the hash the
//! owner taught into the file in the data folder; the engine reads it again on the next batch.

use std::fmt;
use std::str::FromStr;

use toml_edit::{Array, ArrayOfTables, DocumentMut, Item, Table};

use crate::label::{COURIERS_FILE_NAME, CouriersError};

/// A small `CouriersError` (the field is public, the constructor in `label` is not).
fn error(message: impl Into<String>) -> CouriersError {
    CouriersError {
        message: message.into(),
    }
}

/// `couriers.toml` as an editable document. The hashes the owner taught are the only changes the
/// app makes; everything else in the file stays byte-for-byte.
#[derive(Debug)]
pub struct CouriersDocument {
    doc: DocumentMut,
}

impl FromStr for CouriersDocument {
    type Err = CouriersError;

    /// Parse the text of `couriers.toml`, keeping its comments and formatting.
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let doc =
            DocumentMut::from_str(text).map_err(|e| error(format!("{COURIERS_FILE_NAME}: {e}")))?;
        let document = Self { doc };
        document.array()?; // the file must hold `[[courier]]` tables
        Ok(document)
    }
}

impl fmt::Display for CouriersDocument {
    /// The file text as it would be written (comments and formatting kept).
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.doc)
    }
}

impl CouriersDocument {
    fn array(&self) -> Result<&ArrayOfTables, CouriersError> {
        match self.doc.get("courier") {
            Some(item) => item.as_array_of_tables().ok_or_else(|| {
                error(format!(
                    "{COURIERS_FILE_NAME}: \"courier\" must be an array of tables"
                ))
            }),
            None => Err(error(format!("{COURIERS_FILE_NAME}: no couriers"))),
        }
    }

    fn courier_table_mut(&mut self, name: &str) -> Result<&mut Table, CouriersError> {
        let array = match self.doc.get_mut("courier") {
            Some(item) => item.as_array_of_tables_mut().ok_or_else(|| {
                error(format!(
                    "{COURIERS_FILE_NAME}: \"courier\" must be an array of tables"
                ))
            })?,
            None => return Err(error(format!("{COURIERS_FILE_NAME}: no couriers"))),
        };
        for table in array.iter_mut() {
            if table.get("name").and_then(Item::as_str) == Some(name) {
                return Ok(table);
            }
        }
        Err(error(format!(
            "{COURIERS_FILE_NAME}: no courier named \"{name}\""
        )))
    }

    /// Remember a logo the owner named: its hash is appended to that courier's `logo` list
    /// (Q-LABEL-7 default). An already-remembered hash changes nothing.
    pub fn add_courier_logo(&mut self, name: &str, hash: &str) -> Result<(), CouriersError> {
        let table = self.courier_table_mut(name)?;
        push_hash(table, "logo", hash)
    }

    /// Remember an image answered **Not a courier** (a buying channel's strip, Q-LABEL-5): its
    /// hash is appended to the top-level `not_courier` list.
    pub fn add_not_courier(&mut self, hash: &str) -> Result<(), CouriersError> {
        // `not_courier` is a plain key of the root table; `toml_edit` writes the root table's
        // values before every sub-table, so a freshly inserted key still lands above the first
        // `[[courier]]` (which TOML requires for it to belong to the root at all).
        let root = self.doc.as_table_mut();
        push_hash(root, "not_courier", hash)
    }
}

/// Append `hash` to `table`'s list `key`, creating the list when the key is absent; a value
/// already in the list is left alone. `key` and its list keep their comment and formatting.
fn push_hash(table: &mut Table, key: &str, hash: &str) -> Result<(), CouriersError> {
    match table.get_mut(key) {
        Some(item) => {
            let array = item.as_array_mut().ok_or_else(|| {
                error(format!(
                    "{COURIERS_FILE_NAME}: \"{key}\" must be an array of strings"
                ))
            })?;
            if array.iter().any(|value| value.as_str() == Some(hash)) {
                return Ok(());
            }
            array.push(hash);
        }
        None => {
            let mut array = Array::new();
            array.push(hash);
            table.insert(key, Item::Value(array.into()));
        }
    }
    Ok(())
}
