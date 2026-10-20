//! A `categories.toml` as an editable document: comments
//! and the text of the entries that were not changed are kept byte-for-byte.

use std::fmt;
use std::str::FromStr;

use toml_edit::{ArrayOfTables, DocumentMut, Item, Table};

use crate::rules::{
    Category, RULES_FILE_NAME, RulesError, format_condition, load_rules_named, parse_condition,
};

use super::error;

/// The TOML item for a condition text: a literal string (`'…'`) like the file format, so the
/// double quotes inside the condition need no escaping; a basic string only when the text holds
/// a `'`.
fn when_item(text: &str) -> Item {
    if !text.contains('\'')
        && let Ok(value) = format!("'{text}'").parse::<toml_edit::Value>()
    {
        return Item::Value(value);
    }
    toml_edit::value(text)
}

/// One category entry as written in the file (`code`, `name`, `when`; those
/// are the only per-entry keys).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub code: String,
    pub name: String,
    /// The `when` text exactly as in the file; `None` when the entry has no `when`.
    pub when: Option<String>,
}

/// A rules file as an editable document. Comments and
/// the text of the entries that were not changed are kept byte-for-byte; a changed condition is
/// written in the printer's one-line canonical form. `name` is the file's name — `categories.toml`
/// or a set's `categories/<set name>.toml` — so every error names the file it came from.
pub struct RulesDocument {
    doc: DocumentMut,
    name: String,
}

impl FromStr for RulesDocument {
    type Err = RulesError;

    /// Parse the text of the data folder's own `categories.toml` (the set `default`), keeping its
    /// comments and formatting.
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Self::from_str_named(text, RULES_FILE_NAME)
    }
}

impl RulesDocument {
    /// Parse the text of a set's file whose name is `name`, keeping its comments and
    /// formatting.
    pub fn from_str_named(text: &str, name: &str) -> Result<Self, RulesError> {
        let doc = DocumentMut::from_str(text).map_err(|e| error(format!("{name}: {e}")))?;
        let document = Self {
            doc,
            name: name.to_string(),
        };
        document.array()?; // the file must hold `[[category]]` tables
        Ok(document)
    }
}

impl fmt::Display for RulesDocument {
    /// The file text as it would be written (comments and formatting kept).
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.doc)
    }
}

impl RulesDocument {
    /// An empty document for the data folder's own `categories.toml` (the set `default`); *Save*
    /// on the one catch-all entry then writes the file.
    pub fn empty() -> Self {
        Self::empty_named(RULES_FILE_NAME)
    }

    /// An empty document for a set's file whose name is `name`: a data folder
    /// without the file starts from the one catch-all entry and *Save* writes it.
    pub fn empty_named(name: &str) -> Self {
        Self {
            doc: DocumentMut::new(),
            name: name.to_string(),
        }
    }

    fn array(&self) -> Result<&ArrayOfTables, RulesError> {
        match self.doc.get("category") {
            Some(item) => item.as_array_of_tables().ok_or_else(|| {
                error(format!(
                    "{}: \"category\" must be an array of tables",
                    self.name
                ))
            }),
            None => Err(error(format!("{}: no categories", self.name))),
        }
    }

    fn array_mut(&mut self) -> Result<&mut ArrayOfTables, RulesError> {
        let name = self.name.clone();
        // A document without the array yet (an empty one) starts it here, so the first *Save* of
        // a folder without the file writes it.
        if self.doc.get("category").is_none() {
            self.doc["category"] = Item::ArrayOfTables(ArrayOfTables::new());
        }
        match self.doc.get_mut("category") {
            Some(item) => item
                .as_array_of_tables_mut()
                .ok_or_else(|| error(format!("{name}: \"category\" must be an array of tables"))),
            None => Err(error(format!("{name}: no categories"))),
        }
    }

    /// The entries in file order, with their values as written.
    pub fn entries(&self) -> Vec<Entry> {
        self.doc
            .get("category")
            .and_then(Item::as_array_of_tables)
            .map(|array| array.iter().map(entry_of).collect())
            .unwrap_or_default()
    }

    fn table_mut(&mut self, index: usize) -> Result<&mut Table, RulesError> {
        let name = self.name.clone();
        let array = self.array_mut()?;
        let len = array.len();
        array
            .get_mut(index)
            .ok_or_else(|| error(format!("{name}: no category #{} of {len}", index + 1)))
    }

    /// Change an entry's `code` (kept byte-for-byte when it does not change).
    pub fn set_code(&mut self, index: usize, code: &str) -> Result<(), RulesError> {
        self.set_string(index, "code", code)
    }

    /// Change an entry's `name` (kept byte-for-byte when it does not change).
    pub fn set_name(&mut self, index: usize, name: &str) -> Result<(), RulesError> {
        self.set_string(index, "name", name)
    }

    fn set_string(&mut self, index: usize, key: &str, value: &str) -> Result<(), RulesError> {
        let table = self.table_mut(index)?;
        if table.get(key).and_then(Item::as_str) == Some(value) {
            return Ok(());
        }
        table.insert(key, toml_edit::value(value));
        Ok(())
    }

    /// Set (or clear, with `None`) an entry's condition from its condition text. When the new
    /// condition's canonical form equals the old one the file text is left exactly as it was
    /// ("a category not changed keeps its text exactly as in the file").
    pub fn set_condition(
        &mut self,
        index: usize,
        condition: Option<&str>,
    ) -> Result<(), RulesError> {
        let new_canonical = match condition {
            None => None,
            Some(text) => Some(format_condition(&parse_condition(text)?)),
        };
        let table = self.table_mut(index)?;
        let old_canonical = table
            .get("when")
            .and_then(Item::as_str)
            .and_then(|text| parse_condition(text).ok())
            .map(|cond| format_condition(&cond));
        if new_canonical == old_canonical {
            return Ok(());
        }
        match new_canonical {
            Some(text) => {
                table.insert("when", when_item(&text));
            }
            None => {
                table.remove("when");
            }
        }
        Ok(())
    }

    /// Add an entry at position `index` (*+ Add category* adds one above the rest entry,
    /// i.e. at the last position when the last entry has no condition).
    pub fn add(
        &mut self,
        index: usize,
        code: &str,
        name: &str,
        condition: Option<&str>,
    ) -> Result<(), RulesError> {
        let when = match condition {
            None => None,
            Some(text) => Some(format_condition(&parse_condition(text)?)),
        };
        let mut table = Table::new();
        table.insert("code", toml_edit::value(code));
        table.insert("name", toml_edit::value(name));
        if let Some(text) = when {
            table.insert("when", when_item(&text));
        }

        let name = self.name.clone();
        let array = self.array_mut()?;
        let len = array.len();
        if index > len {
            return Err(error(format!(
                "{name}: cannot add at #{index}: the file has {len} categories"
            )));
        }
        array.insert(index, table);
        self.renumber();
        Ok(())
    }

    pub fn remove(&mut self, index: usize) -> Result<(), RulesError> {
        let name = self.name.clone();
        let array = self.array_mut()?;
        let len = array.len();
        if index >= len {
            return Err(error(format!(
                "{name}: no category #{} of {len}",
                index + 1
            )));
        }
        array.remove(index);
        self.renumber();
        Ok(())
    }

    /// Move the entry at `from` to position `to` (the target position after removal).
    pub fn move_to(&mut self, from: usize, to: usize) -> Result<(), RulesError> {
        let name = self.name.clone();
        let array = self.array_mut()?;
        let len = array.len();
        if from >= len {
            return Err(error(format!("{name}: no category #{} of {len}", from + 1)));
        }
        if to >= len {
            return Err(error(format!(
                "{name}: cannot move to #{} of {len}",
                to + 1
            )));
        }
        let table = array.remove(from);
        array.insert(to, table);
        self.renumber();
        Ok(())
    }

    /// Keep the entries at the given current indices, in that order, and drop the rest (the
    /// Categories screen's add / delete / move, applied in one step). Each index must be a valid
    /// position and appear at most once. The kept `Table`s are moved, not rebuilt, so a comment
    /// or a `when` text attached to an entry survives.
    pub fn reorder(&mut self, keep: &[usize]) -> Result<(), RulesError> {
        let name = self.name.clone();
        let len = self.array()?.len();
        let mut seen = vec![false; len];
        for &index in keep {
            if index >= len || seen[index] {
                return Err(error(format!(
                    "{name}: cannot keep entry #{index} of {len}"
                )));
            }
            seen[index] = true;
        }
        let array = self.array_mut()?;
        let mut tables: Vec<Option<Table>> = (0..len).map(|_| None).collect();
        for index in (0..len).rev() {
            tables[index] = Some(array.remove(index));
        }
        for (position, &index) in keep.iter().enumerate() {
            array.insert(position, tables[index].take().expect("table was removed"));
        }
        self.renumber();
        Ok(())
    }

    /// Validate the document as it would be written (04 validation; errors carry the file's
    /// line/col). Returns the categories, ready for the plan.
    pub fn validate(&self) -> Result<Vec<Category>, RulesError> {
        load_rules_named(&self.to_string(), &self.name)
    }

    /// toml_edit orders array-of-tables entries by their recorded position, so after a reorder the
    /// positions must follow the new array order or the move would be undone when written.
    fn renumber(&mut self) {
        if let Some(item) = self.doc.get_mut("category")
            && let Some(array) = item.as_array_of_tables_mut()
        {
            for (index, table) in array.iter_mut().enumerate() {
                table.set_position(Some(index as isize));
            }
        }
    }
}

fn entry_of(table: &Table) -> Entry {
    Entry {
        code: table
            .get("code")
            .and_then(Item::as_str)
            .unwrap_or_default()
            .to_string(),
        name: table
            .get("name")
            .and_then(Item::as_str)
            .unwrap_or_default()
            .to_string(),
        when: table.get("when").and_then(Item::as_str).map(str::to_string),
    }
}
