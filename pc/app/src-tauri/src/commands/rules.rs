//! Categories commands (08 › *5. Categories*, 07 › *Commands between window and Rust*): load
//! `categories.toml`, check the entries with their condition errors (line/col) and the count each
//! takes in the open batch, convert a condition to and from the boxes tree, and save through
//! `RulesDocument` so comments and the text of unchanged conditions survive.
//!
//! The window keeps the entries (boxes or text); the file is only touched on *Save*. A save
//! rebuilds the open draft's plan from the new categories (08 › *5. Categories*).

use std::collections::BTreeSet;
use std::path::Path;
use std::str::FromStr;

use packing_engine::orders::Order;
use packing_engine::plan::Plan;
use packing_engine::rules::{Category, load_rules, parse_condition};
use packing_engine::rules_edit::{self, RulesDocument, Tree};
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::{AppState, CATEGORIES_FILE};

/// One category entry as the file holds it (08 › *5. Categories*).
#[derive(Serialize)]
pub struct EntryView {
    code: String,
    name: String,
    when: Option<String>,
}

/// The Categories screen's first load: the whole file text (for *Edit file as text*), the entries
/// as written, and the file-level error when the file does not load (08 › *Start*: a broken
/// `categories.toml` opens Categories first with the errors).
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
}

/// A code is 1-3 letters or digits (04 › *File format*); mirrors `rules.rs`'s private check.
fn valid_code(code: &str) -> bool {
    let chars: Vec<char> = code.chars().collect();
    (1..=3).contains(&chars.len()) && chars.iter().all(|c| c.is_ascii_alphanumeric())
}

// ------------------------------------------------------------------------------- load

fn rules_load_impl(folder: &Path) -> Result<RulesLoad, String> {
    let path = folder.join(CATEGORIES_FILE);
    let text = std::fs::read_to_string(&path)
        .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    let error = load_rules(&text).err().map(|error| error.to_string());
    let entries = match RulesDocument::from_str(&text) {
        Ok(document) => document
            .entries()
            .into_iter()
            .map(|entry| EntryView {
                code: entry.code,
                name: entry.name,
                when: entry.when,
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

fn rules_check_impl(state: &AppState, entries: &[EntryInput]) -> Result<RulesCheck, String> {
    let mut errors: Vec<Option<String>> = Vec::with_capacity(entries.len());
    let mut general: Option<String> = None;
    let mut categories: Vec<Category> = Vec::with_capacity(entries.len());
    let mut seen: BTreeSet<String> = BTreeSet::new();
    let count = entries.len();
    let mut conditions_ok = true;

    for (index, entry) in entries.iter().enumerate() {
        let code = entry.code.trim();
        let name = entry.name.trim();
        if code.is_empty() {
            general.get_or_insert(format!("category #{}: missing \"code\"", index + 1));
        } else if !valid_code(code) {
            general.get_or_insert(format!(
                "category #{}: \"code\" must be 1-3 letters or digits",
                index + 1
            ));
        } else if !seen.insert(code.to_string()) {
            general.get_or_insert(format!("category \"{code}\": duplicate \"code\""));
        }
        if name.is_empty() {
            general.get_or_insert(format!("category #{}: missing \"name\"", index + 1));
        }

        match &entry.when {
            None => {
                if index + 1 != count {
                    general.get_or_insert(format!(
                        "category #{}: missing \"when\" on a category that is not last",
                        index + 1
                    ));
                }
                categories.push(Category {
                    code: code.to_string(),
                    name: name.to_string(),
                    condition: None,
                    when: None,
                });
                errors.push(None);
            }
            Some(text) => match parse_condition(text) {
                Ok(condition) => {
                    categories.push(Category {
                        code: code.to_string(),
                        name: name.to_string(),
                        condition: Some(condition),
                        when: Some(text.clone()),
                    });
                    errors.push(None);
                }
                Err(error) => {
                    errors.push(Some(error.to_string()));
                    conditions_ok = false;
                }
            },
        }
    }

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
    })
}

fn rules_check_text_impl(text: &str) -> Result<(), String> {
    load_rules(text)
        .map(|_| ())
        .map_err(|error| error.to_string())
}

// -------------------------------------------------------------------------------- save

/// Save the edited entries through `RulesDocument`: an entry that came from the file keeps its
/// comment and unchanged condition text (its `origin`), a new one is appended, a missing one is
/// dropped, and the order is applied in one `reorder` (08 › *5. Categories*: "comments kept").
fn rules_save_impl(state: &AppState, entries: &[EntryInput]) -> Result<(), String> {
    let folder = state.data_folder()?;
    let path = folder.join(CATEGORIES_FILE);
    let text = std::fs::read_to_string(&path)
        .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    let mut document = RulesDocument::from_str(&text).map_err(|error| error.to_string())?;
    let original = document.entries().len();
    for entry in entries {
        if let Some(origin) = entry.origin
            && origin >= original
        {
            return Err(format!(
                "categories.toml: no category #{} of {original}",
                origin + 1
            ));
        }
    }

    // Change the code, name and condition of the entries that came from the file.
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

    // Append the new entries, remembering every entry's index in the document, then reorder.
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
    document.reorder(&keep).map_err(|error| error.to_string())?;

    let categories = document.validate().map_err(|error| error.to_string())?;
    std::fs::write(&path, document.to_string())
        .map_err(|error| format!("Cannot write to {}: {error}.", path.display()))?;
    rebuild_plan(state, categories);
    Ok(())
}

/// *Edit file as text*: check the whole file, write it, and rebuild the open draft's plan.
fn rules_save_text_impl(state: &AppState, text: &str) -> Result<(), String> {
    let categories = load_rules(text).map_err(|error| error.to_string())?;
    let path = state.data_folder()?.join(CATEGORIES_FILE);
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

/// The categories file: its text, its entries and any file-level error (08 › *5. Categories*).
#[tauri::command]
pub fn rules_load(state: State<'_, AppState>) -> Result<RulesLoad, String> {
    rules_load_impl(&state.data_folder()?)
}

/// Check the entries while the screen is edited: condition errors with line/col, a file-level
/// error, and the live counts in the open batch (08 › *5. Categories*).
#[tauri::command]
pub fn rules_check(
    state: State<'_, AppState>,
    entries: Vec<EntryInput>,
) -> Result<RulesCheck, String> {
    rules_check_impl(&state, &entries)
}

/// Check the whole file text (*Edit file as text*): nothing when it loads, the error otherwise.
#[tauri::command]
pub fn rules_check_text(text: String) -> Result<(), String> {
    rules_check_text_impl(&text)
}

/// *Save* the entries (08 › *5. Categories*): write `categories.toml` and recount the open draft.
#[tauri::command]
pub fn rules_save(state: State<'_, AppState>, entries: Vec<EntryInput>) -> Result<(), String> {
    rules_save_impl(&state, &entries)
}

/// *Edit file as text* → *Save*: write the whole file (checked with `load_rules` first).
#[tauri::command]
pub fn rules_save_text(state: State<'_, AppState>, text: String) -> Result<(), String> {
    rules_save_text_impl(&state, &text)
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
mod tests {
    use super::*;
    use packing_engine::orders::{Line, ReadOrder};
    use packing_engine::plan::Plan;
    use packing_engine::rules_edit::condition_to_tree;
    use packing_pdf::batch::Draft;

    use crate::OpenDraft;

    fn temp_dir(label: &str) -> std::path::PathBuf {
        let dir =
            std::env::temp_dir().join(format!("packing-app-rules-{}-{label}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("temp dir");
        dir
    }

    const FILE: &str = "\
# Picks. Order matters.
[[category]]
code = \"A\"
name = \"One\"
when = 'name contains \"x\"'

# keep me
[[category]]
code = \"B\"
name = \"Two\"
when = '''
name contains \"y\"
and total_quantity >= 2
'''

[[category]]
code = \"Z\"
name = \"Rest\"
";

    fn entry(code: &str, name: &str, when: Option<&str>, origin: Option<usize>) -> EntryInput {
        EntryInput {
            code: code.to_string(),
            name: name.to_string(),
            when: when.map(str::to_string),
            origin,
        }
    }

    fn line(name: &str, quantity: i64) -> Line {
        Line {
            name: name.to_string(),
            variation: String::new(),
            seller_sku: String::new(),
            quantity,
            display_name: name.to_string(),
        }
    }

    fn order(name: &str, quantity: i64) -> Order {
        Order {
            order_id: "580000000000000999".to_string(),
            tracking_id: "JY0000009999".to_string(),
            courier: "JNE".to_string(),
            ship_by: None,
            lines: vec![line(name, quantity)],
            customer_message: String::new(),
        }
    }

    fn read_order(name: &str, quantity: i64) -> ReadOrder {
        ReadOrder {
            order: order(name, quantity),
            pages: vec![packing_engine::orders::PageRef { file: 0, page: 0 }],
            position: 1,
            has_slip: true,
            qty_total: None,
        }
    }

    fn state_with_draft(folder: &Path, orders: Vec<ReadOrder>) -> AppState {
        let state = AppState::new();
        state.set_data_folder(folder.to_path_buf());
        state.set_day("2026-10-07");
        let draft = Draft {
            files: Vec::new(),
            orders,
            duplicates: Vec::new(),
            warnings: Vec::new(),
            pages: 0,
            total_orders: 0,
            slip_pages: 0,
        };
        let plan = Plan::from_categories(&[]);
        state.set_draft(OpenDraft {
            draft,
            plan,
            first_number: 1,
            amend: None,
        });
        state
    }

    #[test]
    fn load_reports_entries_and_the_file_error() {
        let dir = temp_dir("load");
        std::fs::write(dir.join(CATEGORIES_FILE), FILE).expect("write");
        let loaded = rules_load_impl(&dir).expect("load");
        assert!(loaded.error.is_none());
        assert_eq!(loaded.entries.len(), 3);
        assert_eq!(loaded.entries[0].code, "A");
        assert_eq!(
            loaded.entries[1].when.as_deref(),
            Some("name contains \"y\"\nand total_quantity >= 2\n")
        );
        assert!(loaded.entries[2].when.is_none());
        assert!(loaded.text.contains("# Picks. Order matters."));

        // A structurally broken file: the error is reported, entries are empty.
        std::fs::write(dir.join(CATEGORIES_FILE), "[[category]]\ncode =\n").expect("write");
        let broken = rules_load_impl(&dir).expect("load broken");
        assert!(broken.error.is_some());
        assert!(broken.entries.is_empty());

        // Valid TOML but a missing name: the error is reported, the entry is still shown.
        std::fs::write(dir.join(CATEGORIES_FILE), "[[category]]\ncode = \"A\"\n").expect("write");
        let invalid = rules_load_impl(&dir).expect("load invalid");
        assert!(invalid.error.is_some());
        assert_eq!(invalid.entries.len(), 1);
        assert_eq!(invalid.entries[0].code, "A");

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn check_reports_condition_errors_and_counts() {
        let dir = temp_dir("check");
        let state = state_with_draft(
            &dir,
            vec![
                read_order("Sepatu A", 1),
                read_order("Sepatu B", 1),
                read_order("Lain", 1),
            ],
        );
        let entries = vec![
            entry("A", "Sepatu", Some("name contains \"sepatu\""), Some(0)),
            entry("Z", "Rest", None, None),
        ];
        let check = rules_check_impl(&state, &entries).expect("check");
        assert!(check.general.is_none());
        assert_eq!(check.errors, vec![None, None]);
        assert_eq!(check.counts, Some(vec![2, 1]));

        // A bad condition: its own error, and the counts are hidden.
        let bad = vec![
            entry("A", "Sepatu", Some("name contains"), Some(0)),
            entry("Z", "Rest", None, None),
        ];
        let check = rules_check_impl(&state, &bad).expect("check bad");
        assert!(
            check.errors[0]
                .as_deref()
                .unwrap()
                .starts_with("line 1, col ")
        );
        assert!(check.counts.is_none());

        // A duplicate code and a `when` on a non-last entry are file-level errors.
        let wrong = vec![
            entry("A", "One", Some("name contains \"x\""), Some(0)),
            entry("A", "Two", Some("name contains \"y\""), Some(1)),
        ];
        let check = rules_check_impl(&state, &wrong).expect("check wrong");
        assert!(check.general.is_some());

        // No draft open: no counts.
        let no_draft = AppState::new();
        no_draft.set_data_folder(dir.clone());
        let check = rules_check_impl(&no_draft, &entries).expect("check no draft");
        assert!(check.counts.is_none());

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn save_keeps_comments_and_unchanged_texts() {
        let dir = temp_dir("save");
        std::fs::write(dir.join(CATEGORIES_FILE), FILE).expect("write");
        let state = AppState::new();
        state.set_data_folder(dir.clone());

        // Change A's name and move B above A; leave Z (the rest entry) last.
        let entries = vec![
            entry(
                "B",
                "Two",
                Some("name contains \"y\"\nand total_quantity >= 2\n"),
                Some(1),
            ),
            entry("A", "One New", Some("name contains \"x\""), Some(0)),
            entry("Z", "Rest", None, Some(2)),
        ];
        rules_save_impl(&state, &entries).expect("save");

        let out = std::fs::read_to_string(dir.join(CATEGORIES_FILE)).expect("read");
        assert!(
            out.contains("# Picks. Order matters."),
            "file comment kept:\n{out}"
        );
        assert!(out.contains("# keep me"), "entry comment kept:\n{out}");
        assert!(out.contains("One New"), "name changed:\n{out}");
        // B's multi-line condition text is unchanged.
        assert!(
            out.contains("name contains \"y\"\nand total_quantity >= 2"),
            "B when kept:\n{out}"
        );
        let loaded = load_rules(&out).expect("load");
        assert_eq!(
            loaded.iter().map(|c| c.code.as_str()).collect::<Vec<_>>(),
            ["B", "A", "Z"]
        );

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn save_adds_and_deletes_through_the_document() {
        let dir = temp_dir("add");
        std::fs::write(dir.join(CATEGORIES_FILE), FILE).expect("write");
        let state = AppState::new();
        state.set_data_folder(dir.clone());

        // Keep A and Z, add X between them.
        let entries = vec![
            entry("A", "One", Some("name contains \"x\""), Some(0)),
            entry("X", "Extra", Some("courier starts_with \"J&T\""), None),
            entry("Z", "Rest", None, Some(2)),
        ];
        rules_save_impl(&state, &entries).expect("save");
        let out = std::fs::read_to_string(dir.join(CATEGORIES_FILE)).expect("read");
        let loaded = load_rules(&out).expect("load");
        assert_eq!(
            loaded.iter().map(|c| c.code.as_str()).collect::<Vec<_>>(),
            ["A", "X", "Z"]
        );
        // The dropped B is gone.
        assert!(!out.contains("code = \"B\""), "B deleted:\n{out}");

        // An invalid edit is refused and the file is unchanged.
        let before = std::fs::read_to_string(dir.join(CATEGORIES_FILE)).expect("read");
        let bad = vec![entry("A", "One", Some("name contains"), Some(0))];
        assert!(rules_save_impl(&state, &bad).is_err());
        assert_eq!(
            std::fs::read_to_string(dir.join(CATEGORIES_FILE)).expect("read"),
            before
        );

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn condition_boxes_round_trip() {
        let text = "not name contains \"a\" and (name contains \"b\" or total_quantity >= 3)";
        let tree = condition_to_tree(text).expect("boxes");
        let back = rules_edit::tree_to_condition(&tree).expect("text");
        assert_eq!(back, text);
    }

    #[test]
    fn save_text_writes_the_whole_file() {
        let dir = temp_dir("text");
        std::fs::write(dir.join(CATEGORIES_FILE), FILE).expect("write");
        let state = AppState::new();
        state.set_data_folder(dir.clone());

        let text = "# whole file\n[[category]]\ncode = \"A\"\nname = \"One\"\nwhen = 'name contains \"x\"'\n";
        rules_save_text_impl(&state, text).expect("save text");
        assert_eq!(
            std::fs::read_to_string(dir.join(CATEGORIES_FILE)).expect("read"),
            text
        );

        // A broken file is refused.
        assert!(rules_save_text_impl(&state, "[[category]]\ncode = \"A\"\n").is_err());

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn save_rebuilds_the_open_draft_plan() {
        let dir = temp_dir("rebuild");
        std::fs::write(dir.join(CATEGORIES_FILE), FILE).expect("write");
        let state = state_with_draft(&dir, vec![read_order("Sepatu A", 1)]);

        let entries = vec![entry(
            "A",
            "Sepatu",
            Some("name contains \"sepatu\""),
            Some(0),
        )];
        rules_save_impl(&state, &entries).expect("save");
        state
            .with_draft(|open| {
                assert_eq!(open.plan.len(), 1);
                assert_eq!(open.plan.entries()[0].category.code, "A");
            })
            .expect("draft");

        std::fs::remove_dir_all(&dir).ok();
    }
}
