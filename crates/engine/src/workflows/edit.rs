//! The Sort rules screen's edits written into `packing-workflows.toml` (rules-file/README.md): a
//! rule's groups replaced, a rule added, renamed or deleted. The file is edited through
//! `toml_edit`, so the owner's comments stay with their tables and a filter that was not changed
//! keeps its text exactly; every edit is loaded again before it is returned, so a file that fails
//! a check is never written.

use std::fmt;
use std::str::FromStr;

use toml_edit::{ArrayOfTables, DocumentMut, Item, Table};

use super::{Workflows, WorkflowsError, load_workflows};

/// One label group as the screen sends it back. `origin` is its place in the rule as loaded, so
/// its comments and an unchanged filter's text are kept; none for a group added on the screen.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GroupEdit {
    pub code: String,
    pub name: String,
    pub filter: Option<String>,
    pub origin: Option<usize>,
}

/// What *+ New rule* starts from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NewRule<'a> {
    /// One group `Z Lainnya` with no filter.
    Empty,
    CopyOf(&'a str),
}

/// The file with rule `rule`'s groups replaced by `groups`, in their order.
pub fn write_groups(
    text: &str,
    rule: &str,
    groups: &[GroupEdit],
) -> Result<(String, Workflows), EditError> {
    edit(text, |rules| {
        let table = rule_table(rules, rule)?;
        let old: Vec<Table> = table
            .get("label_groups")
            .and_then(Item::as_array_of_tables)
            .map(|groups| groups.iter().cloned().collect())
            .unwrap_or_default();
        let mut written = ArrayOfTables::new();
        for group in groups {
            let mut table = group
                .origin
                .and_then(|at| old.get(at).cloned())
                .unwrap_or_default();
            set_text(&mut table, "code", &group.code);
            set_text(&mut table, "name", &group.name);
            match &group.filter {
                Some(filter) if table.get("filter").and_then(Item::as_str) != Some(filter) => {
                    table.insert("filter", filter_item(filter));
                }
                Some(_) => {}
                None => {
                    table.remove("filter");
                }
            }
            written.push(table);
        }
        table.insert("label_groups", Item::ArrayOfTables(written));
        Ok(())
    })
}

/// The file with a rule `name` added after the others.
pub fn add_rule(
    text: &str,
    name: &str,
    from: NewRule<'_>,
) -> Result<(String, Workflows), EditError> {
    edit(text, |rules| {
        let mut table = match from {
            NewRule::CopyOf(rule) => {
                let mut copy = rule_table(rules, rule)?.clone();
                copy.decor_mut().clear();
                copy
            }
            NewRule::Empty => {
                let mut group = Table::new();
                group.insert("code", toml_edit::value("Z"));
                group.insert("name", toml_edit::value("Lainnya"));
                let mut groups = ArrayOfTables::new();
                groups.push(group);
                let mut table = Table::new();
                table.insert("label_groups", Item::ArrayOfTables(groups));
                table
            }
        };
        table.insert("name", toml_edit::value(name));
        rules.push(table);
        Ok(())
    })
}

/// The file with rule `rule` named `name`. Saved batches keep the name they recorded.
pub fn rename_rule(text: &str, rule: &str, name: &str) -> Result<(String, Workflows), EditError> {
    edit(text, |rules| {
        set_text(rule_table(rules, rule)?, "name", name);
        Ok(())
    })
}

/// The file without rule `rule`; the last rule cannot go.
pub fn delete_rule(text: &str, rule: &str) -> Result<(String, Workflows), EditError> {
    edit(text, |rules| {
        let at = rules
            .iter()
            .position(|table| table.get("name").and_then(Item::as_str) == Some(rule))
            .ok_or_else(|| EditError::NoRule(rule.to_string()))?;
        if rules.len() == 1 {
            return Err(EditError::LastRule);
        }
        rules.remove(at);
        Ok(())
    })
}

/// Load, change the sort rules, write, and load again: the edit is kept only when the whole file
/// still passes every check. The tables of the sort rules lose their place in the file, so they
/// are written in the order of the list, after the table before them.
fn edit(
    text: &str,
    change: impl FnOnce(&mut ArrayOfTables) -> Result<(), EditError>,
) -> Result<(String, Workflows), EditError> {
    load_workflows(text)?;
    let mut doc = DocumentMut::from_str(text).expect("a file the rules loaded from is TOML");
    let rules = doc
        .get_mut("label_sort_rules")
        .and_then(Item::as_array_of_tables_mut)
        .expect("the rules loaded it as an array of tables");
    change(rules)?;
    for rule in rules.iter_mut() {
        rule.set_position(None);
        if let Some(groups) = rule
            .get_mut("label_groups")
            .and_then(Item::as_array_of_tables_mut)
        {
            for group in groups.iter_mut() {
                group.set_position(None);
            }
        }
    }
    let written = doc.to_string();
    let workflows = load_workflows(&written)?;
    Ok((written, workflows))
}

fn rule_table<'a>(rules: &'a mut ArrayOfTables, rule: &str) -> Result<&'a mut Table, EditError> {
    rules
        .iter_mut()
        .find(|table| table.get("name").and_then(Item::as_str) == Some(rule))
        .ok_or_else(|| EditError::NoRule(rule.to_string()))
}

/// Set a text key only when it changes, so an unchanged value keeps its spelling.
fn set_text(table: &mut Table, key: &str, value: &str) {
    if table.get(key).and_then(Item::as_str) != Some(value) {
        table.insert(key, toml_edit::value(value));
    }
}

/// A filter as a literal string (`'…'`), so its double quotes need no escaping; a basic string
/// when it holds a `'` or a line break.
fn filter_item(text: &str) -> Item {
    if !text.contains(['\'', '\n'])
        && let Ok(value) = format!("'{text}'").parse::<toml_edit::Value>()
    {
        return Item::Value(value);
    }
    toml_edit::value(text)
}

/// Why an edit was not made; the file is as it was.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EditError {
    /// The file, as edited, fails a check.
    Check(WorkflowsError),
    NoRule(String),
    LastRule,
}

impl From<WorkflowsError> for EditError {
    fn from(error: WorkflowsError) -> Self {
        EditError::Check(error)
    }
}

impl fmt::Display for EditError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EditError::Check(error) => error.fmt(f),
            EditError::NoRule(rule) => write!(f, "There is no sort rule {rule:?}."),
            EditError::LastRule => f.write_str("The last sort rule cannot be deleted."),
        }
    }
}

impl std::error::Error for EditError {}
