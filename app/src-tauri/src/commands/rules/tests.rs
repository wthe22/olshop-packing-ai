use super::*;
use packing_engine::ids::PdfNumber;
use packing_engine::orders::ReadOrder;
use packing_engine::plan::Plan;
use packing_engine::rules::load_rules;

use super::sets::groupings_list_impl;
use packing_engine::rules_edit::condition_to_tree;
use packing_pdf::batch::Draft;
use testkit::order;

use crate::OpenDraft;
use testkit::temp_dir;

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

/// One read order of that line in an open draft, on the made-up order and tracking IDs.
fn read_order(name: &str, quantity: i64) -> ReadOrder {
    ReadOrder {
        order: order(name, quantity, Some("JNE")),
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
        unseen_logos: Vec::new(),
        read_warnings: Vec::new(),
    };
    let plan = Plan::from_categories(&[]);
    state.set_draft(OpenDraft {
        draft,
        plan,
        first_number: PdfNumber::new(1),
        couriers: packing_engine::label::courier_data().clone(),
        grouping: Grouping::default(),
        amend: None,
        logo_pictures: Default::default(),
    });
    state
}

#[test]
fn loading_the_picks_reports_its_entries_or_its_error() {
    let dir = temp_dir("load");
    std::fs::write(dir.join(CATEGORIES_FILE), FILE).expect("write");
    let loaded = rules_load_impl(&dir, &Grouping::default()).expect("load");
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
    let broken = rules_load_impl(&dir, &Grouping::default()).expect("load broken");
    assert!(broken.error.is_some());
    assert!(broken.entries.is_empty());

    // Valid TOML but a missing name: the error is reported, the entry is still shown.
    std::fs::write(dir.join(CATEGORIES_FILE), "[[category]]\ncode = \"A\"\n").expect("write");
    let invalid = rules_load_impl(&dir, &Grouping::default()).expect("load invalid");
    assert!(invalid.error.is_some());
    assert_eq!(invalid.entries.len(), 1);
    assert_eq!(invalid.entries[0].code, "A");

    // A grouping name that cannot be a file name is refused by every command, before any read.
    let bad = temp_dir("bad-name");
    let state = AppState::new();
    state.set_data_folder(bad.clone());
    assert!(rules_load_impl(&bad, &Grouping::new("  ")).is_err());
    assert!(rules_check_impl(&state, &Grouping::new("  "), &[]).is_err());
    assert!(rules_save_impl(&state, &Grouping::new("  "), &[]).is_err());
    // A name with a forbidden character is turned into `-` for the file it names.
    let check = rules_check_impl(&state, &Grouping::new("a/b"), &[]).expect("check");
    assert_eq!(check.file, "categories/a-b.toml");

    for dir in [dir, bad] {
        std::fs::remove_dir_all(&dir).ok();
    }
}

/// A folder without `categories.toml` is not an error: the Categories screen loads the one
/// catch-all entry, which takes every order of the open batch and writes the file on *Save*.
#[test]
fn a_folder_without_the_file_starts_from_the_one_catch_all_entry() {
    let dir = temp_dir("load-no-file");
    let loaded = rules_load_impl(&dir, &Grouping::default()).expect("load");
    assert!(loaded.error.is_none());
    assert_eq!(loaded.text, "");
    assert_eq!(loaded.entries.len(), 1);
    assert_eq!(loaded.entries[0].code, "Z");
    assert_eq!(loaded.entries[0].name, "Lainnya");
    assert!(loaded.entries[0].when.is_none());
    assert_eq!(loaded.entries[0].origin, None, "not an entry of a file");

    // It takes every order of the open batch.
    let state = state_with_draft(&dir, vec![read_order("Sepatu A", 1), read_order("Lain", 1)]);
    let entries = vec![entry("Z", "Lainnya", None, None)];
    let check = rules_check_impl(&state, &Grouping::default(), &entries).expect("check");
    assert!(check.general.is_none());
    assert_eq!(check.counts, Some(vec![2]));

    // *Save* writes the file the folder did not have; it is the file's entry from then on.
    rules_save_impl(&state, &Grouping::default(), &entries).expect("save");
    let written = std::fs::read_to_string(dir.join(CATEGORIES_FILE)).expect("written");
    let categories = load_rules(&written).expect("the written file loads");
    assert_eq!(categories.len(), 1);
    assert_eq!(categories[0].code, "Z");
    assert!(categories[0].condition.is_none());
    assert_eq!(
        rules_load_impl(&dir, &Grouping::default())
            .expect("load again")
            .entries[0]
            .origin,
        Some(0)
    );

    // Adding a category above it and pressing *Save* writes both entries.
    let added = vec![
        entry("A", "Sepatu", Some("name contains \"sepatu\""), None),
        entry("Z", "Lainnya", None, None),
    ];
    rules_save_impl(&state, &Grouping::default(), &added).expect("save added");
    let text = std::fs::read_to_string(dir.join(CATEGORIES_FILE)).expect("read");
    assert_eq!(
        load_rules(&text)
            .expect("load")
            .iter()
            .map(|category| category.code.as_str())
            .collect::<Vec<_>>(),
        ["A", "Z"]
    );

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn the_check_counts_the_open_batch_and_reports_its_errors() {
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
    let check = rules_check_impl(&state, &Grouping::default(), &entries).expect("check");
    assert!(check.general.is_none());
    assert_eq!(check.errors, vec![None, None]);
    assert_eq!(check.counts, Some(vec![2, 1]));

    // A bad condition: its own error, and the counts are hidden.
    let bad = vec![
        entry("A", "Sepatu", Some("name contains"), Some(0)),
        entry("Z", "Rest", None, None),
    ];
    let check = rules_check_impl(&state, &Grouping::default(), &bad).expect("check bad");
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
    let check = rules_check_impl(&state, &Grouping::default(), &wrong).expect("check wrong");
    assert!(check.general.is_some());

    // No draft open: no counts.
    let no_draft = AppState::new();
    no_draft.set_data_folder(dir.clone());
    let check =
        rules_check_impl(&no_draft, &Grouping::default(), &entries).expect("check no draft");
    assert!(check.counts.is_none());
    std::fs::remove_dir_all(&dir).ok();

    // The screen opens the grouping it is asked for: opened from the Plan screen's *Edit
    // categories* it reads the batch's own set, so its entries and the counts *In batch b* are
    // that set's, not the data folder's own `categories.toml` (codes A, B, Z).
    let grouped = temp_dir("open-grouping");
    std::fs::write(grouped.join(CATEGORIES_FILE), FILE).expect("write");
    std::fs::create_dir_all(grouped.join(GROUPINGS_DIR)).expect("groupings folder");
    std::fs::write(
        grouped.join(GROUPINGS_DIR).join("ramadhan.toml"),
        "# the batch's set\n[[category]]\ncode = \"S\"\nname = \"Sepatu\"\nwhen = 'name contains \"sepatu\"'\n\n[[category]]\ncode = \"Z\"\nname = \"Rest\"\n",
    )
    .expect("write the set");
    let state = state_with_draft(
        &grouped,
        vec![read_order("Sepatu A", 1), read_order("Lain", 1)],
    );

    let loaded = rules_load_impl(&grouped, &Grouping::new("ramadhan")).expect("load the set");
    assert!(loaded.error.is_none());
    let codes: Vec<&str> = loaded.entries.iter().map(|e| e.code.as_str()).collect();
    assert_eq!(codes, ["S", "Z"]);

    let entries = vec![
        entry("S", "Sepatu", Some("name contains \"sepatu\""), Some(0)),
        entry("Z", "Rest", None, Some(1)),
    ];
    let check = rules_check_impl(&state, &Grouping::new("ramadhan"), &entries).expect("check");
    assert!(check.general.is_none());
    assert_eq!(check.file, "categories/ramadhan.toml");
    assert_eq!(
        check.counts,
        Some(vec![1, 1]),
        "the counts are the batch's set's entries applied to the open draft"
    );

    std::fs::remove_dir_all(&grouped).ok();
}

#[test]
fn saving_keeps_the_comments_and_the_order_and_writes_the_edits() {
    let dir = temp_dir("save");
    std::fs::write(dir.join(CATEGORIES_FILE), FILE).expect("write");
    let state = state_with_draft(&dir, vec![read_order("Sepatu A", 1)]);

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
    rules_save_impl(&state, &Grouping::default(), &entries).expect("save");

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

    // The open draft's plan is rebuilt from the saved entries.
    state
        .with_draft(|open| {
            assert_eq!(open.plan.len(), 3);
            assert_eq!(open.plan.entries()[0].category.code, "B");
        })
        .expect("draft");

    // A dropped entry goes and an added one takes its place.
    let entries = vec![
        entry("A", "One New", Some("name contains \"x\""), Some(0)),
        entry("X", "Extra", Some("courier starts_with \"J&T\""), None),
        entry("Z", "Rest", None, Some(2)),
    ];
    rules_save_impl(&state, &Grouping::default(), &entries).expect("save");
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
    assert!(rules_save_impl(&state, &Grouping::default(), &bad).is_err());
    assert_eq!(
        std::fs::read_to_string(dir.join(CATEGORIES_FILE)).expect("read"),
        before
    );

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn saving_writes_its_own_groupings_file_and_the_whole_text_as_given() {
    // A save writes the chosen grouping's file — `categories.toml` for `default`, else
    // `categories/<grouping name>.toml` — and never touches the data folder's own file.
    let dir = temp_dir("save-grouping");
    std::fs::write(dir.join(CATEGORIES_FILE), FILE).expect("write");
    let before = std::fs::read_to_string(dir.join(CATEGORIES_FILE)).expect("read");
    let state = AppState::new();
    state.set_data_folder(dir.clone());

    // The grouping has no file yet: the screen starts from the one catch-all entry, and *Save*
    // writes it to `categories/ramadhan.toml`.
    let loaded = rules_load_impl(&dir, &Grouping::new("ramadhan")).expect("load");
    assert_eq!(loaded.text, "");
    assert_eq!(loaded.entries.len(), 1);
    assert_eq!(loaded.entries[0].code, "Z");

    let entries = vec![
        entry("A", "Sepatu", Some("name contains \"sepatu\""), None),
        entry("Z", "Lainnya", None, None),
    ];
    rules_save_impl(&state, &Grouping::new("ramadhan"), &entries).expect("save");
    let written =
        std::fs::read_to_string(dir.join(GROUPINGS_DIR).join("ramadhan.toml")).expect("written");
    assert_eq!(
        load_rules(&written)
            .expect("load")
            .iter()
            .map(|category| category.code.as_str())
            .collect::<Vec<_>>(),
        ["A", "Z"]
    );
    assert_eq!(
        std::fs::read_to_string(dir.join(CATEGORIES_FILE)).expect("read"),
        before,
        "the data folder's own categories.toml is untouched"
    );

    // The written file is the grouping's entries from then on, and the check names that file.
    let again = rules_load_impl(&dir, &Grouping::new("ramadhan")).expect("load again");
    assert_eq!(again.entries[0].origin, Some(0));
    let check = rules_check_impl(&state, &Grouping::new("ramadhan"), &entries).expect("check");
    assert_eq!(check.file, "categories/ramadhan.toml");
    std::fs::remove_dir_all(&dir).ok();

    // The text editor saves the whole file as given; a broken file is refused.
    let text_dir = temp_dir("text");
    std::fs::write(text_dir.join(CATEGORIES_FILE), FILE).expect("write");
    let state = AppState::new();
    state.set_data_folder(text_dir.clone());
    let text =
        "# whole file\n[[category]]\ncode = \"A\"\nname = \"One\"\nwhen = 'name contains \"x\"'\n";
    rules_save_text_impl(&state, &Grouping::default(), text).expect("save text");
    assert_eq!(
        std::fs::read_to_string(text_dir.join(CATEGORIES_FILE)).expect("read"),
        text
    );
    assert!(
        rules_save_text_impl(&state, &Grouping::default(), "[[category]]\ncode = \"A\"\n").is_err()
    );
    std::fs::remove_dir_all(&text_dir).ok();
}

#[test]
fn condition_boxes_round_trip() {
    let text = "not name contains \"a\" and (name contains \"b\" or total_quantity >= 3)";
    let tree = condition_to_tree(text).expect("boxes");
    let back = rules_edit::tree_to_condition(&tree).expect("text");
    assert_eq!(back, text);
}

/// `groupings_list` names the groupings of the data folder with the file of each and the grouping
/// used last, which the next New batch preselects; an older `settings.json` without the key loads
/// as `default`.
#[test]
fn groupings_list_reports_the_groupings_their_files_and_the_grouping_used_last() {
    let dir = temp_dir("groupings-list");
    std::fs::write(dir.join(CATEGORIES_FILE), FILE).expect("write");
    groupings::create(&dir, &Grouping::new("ramadhan"), None).expect("new set");

    // No settings file: the grouping used last is `default`.
    let list = groupings_list_impl(&dir).expect("list");
    assert_eq!(
        list.groupings
            .iter()
            .map(|entry| entry.name.as_str())
            .collect::<Vec<_>>(),
        ["default", "ramadhan"]
    );
    assert_eq!(list.groupings[0].file, CATEGORIES_FILE);
    assert_eq!(list.groupings[1].file, "categories/ramadhan.toml");
    assert_eq!(list.last_grouping, Grouping::default());

    // `settings.json` remembers the grouping used last ...
    std::fs::write(
        dir.join(crate::SETTINGS_FILE),
        "{\n  \"scope\": \"whole\",\n  \"layouts\": [\"pick\"],\n  \"set\": \"ramadhan\"\n}\n",
    )
    .expect("write settings");
    assert_eq!(
        groupings_list_impl(&dir).expect("list").last_grouping,
        Grouping::new("ramadhan")
    );

    // ... and an older file without it loads as `default`.
    std::fs::write(
        dir.join(crate::SETTINGS_FILE),
        "{\n  \"scope\": \"whole\"\n}\n",
    )
    .expect("write older settings");
    assert_eq!(
        groupings_list_impl(&dir).expect("list").last_grouping,
        Grouping::default()
    );
}
