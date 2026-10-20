//! Categories commands (08 › *5. Categories*, 07 › *Commands between window and Rust*): load
//! `categories.toml`, check the entries with their condition errors (line/col) and the count each
//! takes in the open batch, convert a condition to and from the boxes tree, and save through
//! `RulesDocument` so comments and the text of unchanged conditions survive.
//!
//! The window keeps the entries (boxes or text); the file is only touched on *Save*. A save
//! rebuilds the open draft's plan from the new categories (08 › *5. Categories*).

use std::collections::BTreeSet;
use std::path::Path;

use packing_engine::groupings::{self, Grouping};
use packing_engine::orders::Order;
use packing_engine::plan::Plan;
use packing_engine::rules::{Category, catch_all_category, load_rules_named, parse_condition};
use packing_engine::rules_edit::{self, RulesDocument, Tree};
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::{AppState, CATEGORIES_FILE, GROUPINGS_DIR};

pub mod sets;

/// One category entry as the file holds it (08 › *5. Categories*): its values and, for an entry
/// the file holds, the index it has there (`origin`). The one catch-all entry of a folder
/// without `categories.toml` has `origin: None`, so a *Save* adds it as a new entry.
#[derive(Serialize)]
pub struct EntryView {
    code: String,
    name: String,
    when: Option<String>,
    origin: Option<usize>,
}

/// The Categories screen's first load: the whole file text (for *Edit file as text*), the entries
/// as written, and the file-level error when the file does not load (08 › *Start*: a broken
/// `categories.toml` opens Categories first with the errors). A folder without the file is not an
/// error: the entries are the one catch-all entry then (08 › *Start* step 3).
#[derive(Serialize)]
pub struct RulesLoad {
    text: String,
    entries: Vec<EntryView>,
    error: Option<String>,
}

/// One entry the screen sends back: its values and, for an entry that was loaded from the file,
/// the index it had there (`origin`) so the save can keep its comment and unchanged condition
/// text. A new entry has `origin: None`.
#[derive(Deserialize)]
pub struct EntryInput {
    code: String,
    name: String,
    when: Option<String>,
    #[serde(default)]
    origin: Option<usize>,
}

/// The live check (08 › *5. Categories*): each entry's condition error (with line/col), a
/// file-level error (a bad or duplicate code, a missing name, a `when` missing on an entry that is
/// not last), and the count each entry takes in the open batch, in order from what is left
/// (`None` when no batch is open or a condition is invalid).
#[derive(Serialize)]
pub struct RulesCheck {
    errors: Vec<Option<String>>,
    general: Option<String>,
    counts: Option<Vec<usize>>,
    /// The file the checked grouping is kept in, relative to the data folder: `categories.toml`
    /// for the grouping `default`, else `categories/<grouping name>.toml` (04 › *Sets*).
    file: String,
}

/// A code is 1-3 letters or digits (04 › *File format*); mirrors `rules.rs`'s private check.
fn valid_code(code: &str) -> bool {
    let chars: Vec<char> = code.chars().collect();
    (1..=3).contains(&chars.len()) && chars.iter().all(|c| c.is_ascii_alphanumeric())
}

// ------------------------------------------------------------------------------- load

fn rules_load_impl(folder: &Path, grouping: &Grouping) -> Result<RulesLoad, String> {
    let file = groupings::file_of(grouping).map_err(|error| error.to_string())?;
    let path = folder.join(&file);
    let text = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        // A set with no file is not an error: its picks are the one catch-all entry
        // (04 › *Sets*, 08 › *Start* step 3, 07 › *Inputs*).
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let category = catch_all_category();
            return Ok(RulesLoad {
                text: String::new(),
                entries: vec![EntryView {
                    code: category.code,
                    name: category.name,
                    when: None,
                    origin: None,
                }],
                error: None,
            });
        }
        Err(error) => return Err(format!("cannot read {}: {error}", path.display())),
    };
    let error = load_rules_named(&text, &file)
        .err()
        .map(|error| error.to_string());
    let entries = match RulesDocument::from_str_named(&text, &file) {
        Ok(document) => document
            .entries()
            .into_iter()
            .enumerate()
            .map(|(origin, entry)| EntryView {
                code: entry.code,
                name: entry.name,
                when: entry.when,
                origin: Some(origin),
            })
            .collect(),
        Err(_) => Vec::new(),
    };
    Ok(RulesLoad {
        text,
        entries,
        error,
    })
}

// ------------------------------------------------------------------------------- check

fn rules_check_impl(
    state: &AppState,
    grouping: &Grouping,
    entries: &[EntryInput],
) -> Result<RulesCheck, String> {
    let file = groupings::file_of(grouping).map_err(|error| error.to_string())?;
    let count = entries.len();
    let mut check = EntryCheck::new(count);
    for (index, entry) in entries.iter().enumerate() {
        check.add(index, count, entry);
    }
    let EntryCheck {
        errors,
        general,
        categories,
        conditions_ok,
        ..
    } = check;

    // The counts in the open batch: the same first-match, take-from-what-is-left rule as the plan
    // (08 › *5. Categories*: "In batch 3").
    let counts = if conditions_ok {
        state
            .with_draft(|open| {
                let orders: Vec<Order> = open
                    .draft
                    .orders
                    .iter()
                    .map(|order| order.order.clone())
                    .collect();
                rules_edit::counts(&categories, &orders)
            })
            .ok()
    } else {
        None
    };

    Ok(RulesCheck {
        errors,
        general,
        counts,
        file,
    })
}

/// The running state of the live check: each entry's condition error, the first file-level error,
/// the categories built so far, the codes seen and whether every condition parsed.
struct EntryCheck {
    errors: Vec<Option<String>>,
    general: Option<String>,
    categories: Vec<Category>,
    seen: BTreeSet<String>,
    conditions_ok: bool,
}

impl EntryCheck {
    fn new(entries: usize) -> EntryCheck {
        EntryCheck {
            errors: Vec::with_capacity(entries),
            general: None,
            categories: Vec::with_capacity(entries),
            seen: BTreeSet::new(),
            conditions_ok: true,
        }
    }

    /// Check one entry and add its category.
    fn add(&mut self, index: usize, count: usize, entry: &EntryInput) {
        let code = entry.code.trim();
        let name = entry.name.trim();
        if code.is_empty() {
            self.general
                .get_or_insert(format!("category #{}: missing \"code\"", index + 1));
        } else if !valid_code(code) {
            self.general.get_or_insert(format!(
                "category #{}: \"code\" must be 1-3 letters or digits",
                index + 1
            ));
        } else if !self.seen.insert(code.to_string()) {
            self.general
                .get_or_insert(format!("category \"{code}\": duplicate \"code\""));
        }
        if name.is_empty() {
            self.general
                .get_or_insert(format!("category #{}: missing \"name\"", index + 1));
        }

        match &entry.when {
            None => {
                if index + 1 != count {
                    self.general.get_or_insert(format!(
                        "category #{}: missing \"when\" on a category that is not last",
                        index + 1
                    ));
                }
                self.categories.push(Category {
                    code: code.to_string(),
                    name: name.to_string(),
                    condition: None,
                    when: None,
                });
                self.errors.push(None);
            }
            Some(text) => match parse_condition(text) {
                Ok(condition) => {
                    self.categories.push(Category {
                        code: code.to_string(),
                        name: name.to_string(),
                        condition: Some(condition),
                        when: Some(text.clone()),
                    });
                    self.errors.push(None);
                }
                Err(error) => {
                    self.errors.push(Some(error.to_string()));
                    self.conditions_ok = false;
                }
            },
        }
    }
}

// -------------------------------------------------------------------------------- save

/// Save the edited entries through `RulesDocument`: an entry that came from the file keeps its
/// comment and unchanged condition text (its `origin`), a new one is appended (the one catch-all
/// entry of a folder without `categories.toml` included, which writes the file), a missing one is
/// dropped, and the order is applied in one `reorder` (08 › *5. Categories*: "comments kept").
fn rules_save_impl(
    state: &AppState,
    grouping: &Grouping,
    entries: &[EntryInput],
) -> Result<(), String> {
    let folder = state.data_folder()?;
    let file = groupings::file_of(grouping).map_err(|error| error.to_string())?;
    ensure_grouping_dir(&folder, &file)?;
    let path = folder.join(&file);
    let mut document = load_document(&path, &file)?;
    let original = document.entries().len();
    check_origins(&file, original, entries)?;
    edit_existing(&mut document, entries)?;
    place_entries(&mut document, entries)?;

    let categories = document.validate().map_err(|error| error.to_string())?;
    std::fs::write(&path, document.to_string())
        .map_err(|error| format!("Cannot write to {}: {error}.", path.display()))?;
    rebuild_plan(state, categories);
    Ok(())
}

/// A set's file lives in `<data>/categories/`; the folder is created here, so the first save of a
/// new set (04 › *Sets*) can write the file.
fn ensure_grouping_dir(folder: &Path, file: &str) -> Result<(), String> {
    if file == CATEGORIES_FILE {
        return Ok(());
    }
    let dir = folder.join(GROUPINGS_DIR);
    std::fs::create_dir_all(&dir)
        .map_err(|error| format!("Cannot write to {}: {error}.", dir.display()))
}

/// The set's document: its file's entries, or a new one when the file is not there yet.
fn load_document(path: &Path, file: &str) -> Result<RulesDocument, String> {
    match std::fs::read_to_string(path) {
        Ok(text) => RulesDocument::from_str_named(&text, file).map_err(|error| error.to_string()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            Ok(RulesDocument::empty_named(file))
        }
        Err(error) => Err(format!("cannot read {}: {error}", path.display())),
    }
}

/// Every `origin` the screen sent must name an entry the document holds.
fn check_origins(file: &str, original: usize, entries: &[EntryInput]) -> Result<(), String> {
    for entry in entries {
        if let Some(origin) = entry.origin
            && origin >= original
        {
            return Err(format!("{file}: no category #{} of {original}", origin + 1));
        }
    }
    Ok(())
}

/// Change the code, name and condition of the entries that came from the file.
fn edit_existing(document: &mut RulesDocument, entries: &[EntryInput]) -> Result<(), String> {
    for entry in entries {
        if let Some(origin) = entry.origin {
            document
                .set_code(origin, entry.code.trim())
                .map_err(|error| error.to_string())?;
            document
                .set_name(origin, entry.name.trim())
                .map_err(|error| error.to_string())?;
            document
                .set_condition(origin, entry.when.as_deref())
                .map_err(|error| error.to_string())?;
        }
    }
    Ok(())
}

/// Append the new entries, remembering every entry's index in the document, then reorder.
fn place_entries(document: &mut RulesDocument, entries: &[EntryInput]) -> Result<(), String> {
    let mut keep: Vec<usize> = Vec::with_capacity(entries.len());
    for entry in entries {
        match entry.origin {
            Some(origin) => keep.push(origin),
            None => {
                let at = document.entries().len();
                document
                    .add(
                        at,
                        entry.code.trim(),
                        entry.name.trim(),
                        entry.when.as_deref(),
                    )
                    .map_err(|error| error.to_string())?;
                keep.push(at);
            }
        }
    }
    document.reorder(&keep).map_err(|error| error.to_string())
}

/// *Edit file as text*: check the whole file, write it, and rebuild the open draft's plan.
fn rules_save_text_impl(state: &AppState, grouping: &Grouping, text: &str) -> Result<(), String> {
    let folder = state.data_folder()?;
    let file = groupings::file_of(grouping).map_err(|error| error.to_string())?;
    let categories = load_rules_named(text, &file).map_err(|error| error.to_string())?;
    ensure_grouping_dir(&folder, &file)?;
    let path = folder.join(&file);
    std::fs::write(&path, text)
        .map_err(|error| format!("Cannot write to {}: {error}.", path.display()))?;
    rebuild_plan(state, categories);
    Ok(())
}

/// Rebuild the open draft's plan from the new categories, so the Plan screen recounts with them
/// (08 › *Window frame*: an open draft is kept and recounted). No draft open: nothing to do.
fn rebuild_plan(state: &AppState, categories: Vec<Category>) {
    let _ = state.with_draft(|open| {
        open.plan = Plan::from_categories(&categories);
    });
}

// ---------------------------------------------------------------------------- commands

/// The set's file: its text, its entries and any file-level error (08 › *5. Categories*; 04 ›
/// *Sets*).
#[tauri::command]
pub fn rules_load(state: State<'_, AppState>, grouping: Grouping) -> Result<RulesLoad, String> {
    rules_load_impl(&state.data_folder()?, &grouping)
}

/// Check the entries while the screen is edited: condition errors with line/col, a file-level
/// error, and the live counts in the open batch (08 › *5. Categories*). `set` names the file the
/// entries belong to.
#[tauri::command]
pub fn rules_check(
    state: State<'_, AppState>,
    grouping: Grouping,
    entries: Vec<EntryInput>,
) -> Result<RulesCheck, String> {
    rules_check_impl(&state, &grouping, &entries)
}

/// Check the whole file text (*Edit file as text*): nothing when it loads, the error otherwise.
/// The check is the same for every grouping; `grouping` names the file the error comes from
/// (04 › *Sets*).
#[tauri::command]
pub fn rules_check_text(grouping: Grouping, text: String) -> Result<(), String> {
    let file = groupings::file_of(&grouping).map_err(|error| error.to_string())?;
    load_rules_named(&text, &file)
        .map(|_| ())
        .map_err(|error| error.to_string())
}

/// *Save* the entries (08 › *5. Categories*): write the set's file and recount the open draft.
#[tauri::command]
pub fn rules_save(
    state: State<'_, AppState>,
    grouping: Grouping,
    entries: Vec<EntryInput>,
) -> Result<(), String> {
    rules_save_impl(&state, &grouping, &entries)
}

/// *Edit file as text* → *Save*: write the whole of the set's file (checked first).
#[tauri::command]
pub fn rules_save_text(
    state: State<'_, AppState>,
    grouping: Grouping,
    text: String,
) -> Result<(), String> {
    rules_save_text_impl(&state, &grouping, &text)
}

/// **+ New set** (08 › *5. Categories*; 04 › *Sets*): write `categories/<name>.toml`, empty or a
/// copy of `copy_from`'s file. The name `default` is the data folder's own `categories.toml` and
/// cannot be created, and an existing file is never overwritten.
#[tauri::command]
pub fn new_grouping(
    state: State<'_, AppState>,
    name: Grouping,
    copy_from: Option<Grouping>,
) -> Result<(), String> {
    let folder = state.data_folder()?;
    groupings::create(&folder, &name, copy_from.as_ref()).map_err(|error| error.to_string())
}

/// **Delete set** (08 › *5. Categories*; 04 › *Sets*): delete that grouping's file, whose
/// categories then fall back to the one catch-all entry. The grouping `default` cannot be
/// deleted.
#[tauri::command]
pub fn delete_grouping(state: State<'_, AppState>, name: Grouping) -> Result<(), String> {
    let folder = state.data_folder()?;
    groupings::delete(&folder, &name).map_err(|error| error.to_string())
}

/// The groupings of the data folder (07 › *Commands*: `groupings_list`): the grouping names, the
/// file of each and the grouping used last.
#[tauri::command]
pub fn groupings_list(state: State<'_, AppState>) -> Result<sets::GroupingsList, String> {
    sets::groupings_list_impl(&state.data_folder()?)
}

/// A condition text → boxes tree, for the boxes editor (07 › *Commands*: the engine's parser).
#[tauri::command]
pub fn condition_to_tree(text: String) -> Result<Tree, String> {
    rules_edit::condition_to_tree(&text).map_err(|error| error.to_string())
}

/// A boxes tree → the printer's canonical condition text (07 › *Commands*: the engine's printer).
#[tauri::command]
pub fn tree_to_condition(tree: Tree) -> Result<String, String> {
    rules_edit::tree_to_condition(&tree).map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests;
