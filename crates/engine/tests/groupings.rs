//! `groupings.rs`: the grouping list of a data folder, the file of a grouping, the grouping-name
//! rules, a grouping with no file and create/delete (04 › *Sets*; 07 › *Rules file*).

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use packing_engine::groupings::{
    self, CATEGORIES_FILE, DEFAULT_GROUPING, GROUPINGS_DIR, Grouping, GroupingEntry, GroupingError,
};
use packing_engine::rules::catch_all_category;

/// A unique folder under the system temp dir, removed when the test ends.
struct TempDir(PathBuf);

impl TempDir {
    fn new(tag: &str) -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "packing-engine-groupings-{tag}-{}-{nanos}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("create temp dir");
        TempDir(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

const SEPATU: &str =
    "[[category]]\ncode = \"A\"\nname = \"Sepatu\"\nwhen = 'name contains \"sepatu\"'\n";
const REST: &str = "[[category]]\ncode = \"Z\"\nname = \"Lainnya\"\n";

fn grouping(name: &str) -> Grouping {
    Grouping::new(name)
}

fn names(groupings: &[GroupingEntry]) -> Vec<String> {
    groupings
        .iter()
        .map(|entry| entry.name.to_string())
        .collect()
}

/// The grouping list of a data folder (04 › *Sets*): `default` always, then a grouping per
/// `<name>.toml` in its `categories/` folder. Read with only `categories.toml`, only
/// `categories/*.toml`, with both and with neither; a file that is not `.toml`, and a folder, are
/// not groupings.
#[test]
fn the_grouping_list_is_default_and_the_categories_folder_files() {
    // Only `categories.toml`: the grouping list is just the grouping `default`.
    let dir = TempDir::new("list-default");
    fs::write(dir.path().join(CATEGORIES_FILE), SEPATU).expect("write default");
    let list = groupings::list(dir.path()).expect("list");
    assert_eq!(names(&list), vec![DEFAULT_GROUPING.to_string()]);
    assert_eq!(list[0].file, CATEGORIES_FILE);

    // Only `categories/*.toml`: `default` plus a grouping per file, in file-name order.
    let dir = TempDir::new("list-files");
    let groupings_dir = dir.path().join(GROUPINGS_DIR);
    fs::create_dir_all(&groupings_dir).expect("create categories dir");
    fs::write(groupings_dir.join("ramadhan.toml"), SEPATU).expect("write ramadhan");
    fs::write(groupings_dir.join("b sepatu.toml"), REST).expect("write b sepatu");
    fs::write(groupings_dir.join("notes.txt"), "not a grouping").expect("write notes");
    fs::create_dir_all(groupings_dir.join("sub.toml")).expect("a folder named like a grouping");
    let list = groupings::list(dir.path()).expect("list");
    assert_eq!(
        names(&list),
        vec![
            "default".to_string(),
            "b sepatu".to_string(),
            "ramadhan".to_string()
        ]
    );
    assert_eq!(
        list[0].file, CATEGORIES_FILE,
        "the grouping default keeps its own file"
    );
    assert_eq!(list[1].file, format!("{GROUPINGS_DIR}/b sepatu.toml"));

    // Both.
    fs::write(dir.path().join(CATEGORIES_FILE), SEPATU).expect("write default");
    assert_eq!(names(&groupings::list(dir.path()).expect("list")).len(), 3);

    // Neither: `default` alone, the one catch-all grouping.
    let dir = TempDir::new("list-none");
    let list = groupings::list(dir.path()).expect("list");
    assert_eq!(names(&list), vec![DEFAULT_GROUPING.to_string()]);
}

/// A grouping writes to and loads from its own file, `categories/<grouping name>.toml`; a broken
/// file gives its own line and column, naming the grouping's file (04 › *Sets*).
#[test]
fn a_grouping_reads_and_writes_its_own_file() {
    let dir = TempDir::new("read-write");
    // A new grouping starts from the one catch-all entry and is written to its own file.
    groupings::create(dir.path(), &grouping("ramadhan"), None).expect("create");
    let file = dir.path().join(GROUPINGS_DIR).join("ramadhan.toml");
    assert!(
        file.is_file(),
        "a grouping's file is categories/<grouping name>.toml"
    );
    assert_eq!(
        groupings::load(dir.path(), &grouping("ramadhan")).expect("load"),
        vec![catch_all_category()]
    );

    // The file's own entries load for that grouping.
    fs::write(&file, SEPATU).expect("write");
    let categories = groupings::load(dir.path(), &grouping("ramadhan")).expect("load");
    assert_eq!(categories.len(), 1);
    assert_eq!(categories[0].code, "A");

    // A broken file is refused with the grouping's file, line and column.
    fs::write(
        &file,
        "[[category]]\ncode = \"A\"\nname = \"N\"\nwhen = 'name foo \"x\"'\n",
    )
    .expect("write broken");
    let error = groupings::load(dir.path(), &grouping("ramadhan")).expect_err("a broken grouping");
    assert!(
        error
            .to_string()
            .contains("categories/ramadhan.toml, line 4, col"),
        "{error}"
    );
}

/// A grouping with no file — a fresh data folder, or a grouping named but not saved yet — reads
/// as the one catch-all entry (04 › *Sets*).
#[test]
fn a_grouping_with_no_file_reads_as_the_one_catch_all_entry() {
    let dir = TempDir::new("no-file");
    assert_eq!(
        groupings::load(dir.path(), &Grouping::default()).expect("load the grouping default"),
        vec![catch_all_category()]
    );
    assert_eq!(
        groupings::load(dir.path(), &grouping("ramadhan")).expect("load an unsaved grouping"),
        vec![catch_all_category()]
    );
}

/// The data folder's own `categories.toml` is never moved, renamed or rewritten by listing the
/// groupings or by another grouping's save (04 › *Sets*).
#[test]
fn the_data_folders_own_categories_file_is_never_moved_or_rewritten() {
    let dir = TempDir::new("default-untouched");
    let own = dir.path().join(CATEGORIES_FILE);
    fs::write(&own, SEPATU).expect("write");

    let _ = groupings::list(dir.path()).expect("list");
    let _ = groupings::load(dir.path(), &Grouping::default()).expect("load the grouping default");
    groupings::create(dir.path(), &grouping("ramadhan"), None).expect("create another grouping");
    let _ = groupings::load(dir.path(), &grouping("ramadhan")).expect("load another grouping");

    assert!(own.is_file(), "categories.toml is not moved or renamed");
    assert_eq!(
        fs::read_to_string(&own).expect("read"),
        SEPATU,
        "categories.toml is not rewritten"
    );
    assert_eq!(
        groupings::file_of(&Grouping::default()).expect("file"),
        CATEGORIES_FILE
    );
}

/// A file `categories/default.toml` is refused with both files named (04 › *Sets*).
#[test]
fn a_categories_default_file_is_refused_with_both_files_named() {
    let dir = TempDir::new("default-file");
    let groupings_dir = dir.path().join(GROUPINGS_DIR);
    fs::create_dir_all(&groupings_dir).expect("create categories dir");
    fs::write(groupings_dir.join("default.toml"), REST).expect("write");

    let error = groupings::list(dir.path()).expect_err("refused");
    assert_eq!(
        error,
        GroupingError::DefaultFile {
            default: CATEGORIES_FILE.to_string(),
            file: format!("{GROUPINGS_DIR}/default.toml"),
        }
    );
    let message = error.to_string();
    assert!(message.contains(CATEGORIES_FILE), "{message}");
    assert!(
        message.contains(&format!("{GROUPINGS_DIR}/default.toml")),
        "{message}"
    );
}

/// The grouping-name rules (04 › *Sets*): the file name is the grouping name, characters Windows
/// forbids become `-`, an empty name is refused, and the name `default` is the data folder's own
/// `categories.toml` — not created, not deleted, not renamed.
#[test]
fn the_grouping_name_rules_refuse_an_empty_name_and_default() {
    assert_eq!(
        groupings::file_of(&grouping("ramadhan")).expect("file"),
        format!("{GROUPINGS_DIR}/ramadhan.toml")
    );
    assert_eq!(
        groupings::file_of(&Grouping::default()).expect("file"),
        CATEGORIES_FILE
    );
    assert_eq!(
        groupings::file_of(&grouping("a/b:c")).expect("file"),
        format!("{GROUPINGS_DIR}/a-b-c.toml")
    );
    assert_eq!(
        groupings::file_of(&grouping("   ")),
        Err(GroupingError::EmptyName)
    );

    let dir = TempDir::new("refuse");
    assert_eq!(
        groupings::create(dir.path(), &Grouping::default(), None),
        Err(GroupingError::ReservedDefault)
    );
    assert_eq!(
        groupings::delete(dir.path(), &Grouping::default()),
        Err(GroupingError::ReservedDefault)
    );
    assert!(
        !dir.path().join(CATEGORIES_FILE).exists(),
        "nothing is written for the grouping default"
    );
    assert_eq!(
        groupings::create(dir.path(), &grouping("  "), None),
        Err(GroupingError::EmptyName)
    );
}

/// **Delete set** removes the grouping's file (04 › *Sets*): its name leaves the grouping list and
/// its categories fall back to the one catch-all entry; a grouping with no file is already gone.
#[test]
fn deleting_a_grouping_removes_its_file_and_its_name_leaves_the_list() {
    let dir = TempDir::new("delete");
    groupings::create(dir.path(), &grouping("ramadhan"), None).expect("create");
    assert_eq!(
        names(&groupings::list(dir.path()).expect("list")),
        vec!["default".to_string(), "ramadhan".to_string()]
    );

    groupings::delete(dir.path(), &grouping("ramadhan")).expect("delete");
    assert!(
        !dir.path()
            .join(GROUPINGS_DIR)
            .join("ramadhan.toml")
            .exists()
    );
    assert_eq!(
        names(&groupings::list(dir.path()).expect("list")),
        vec![DEFAULT_GROUPING.to_string()]
    );
    assert_eq!(
        groupings::load(dir.path(), &grouping("ramadhan")).expect("load"),
        vec![catch_all_category()]
    );
    groupings::delete(dir.path(), &grouping("gone"))
        .expect("a grouping with no file is already gone");
}

/// A new grouping is empty (the one catch-all entry) or a copy of another grouping's file
/// (04 › *Sets*); a file already there is never overwritten.
#[test]
fn a_new_grouping_is_empty_or_a_copy_of_another_grouping() {
    let dir = TempDir::new("copy");
    fs::write(dir.path().join(CATEGORIES_FILE), SEPATU).expect("write default");
    groupings::create(
        dir.path(),
        &grouping("ramadhan"),
        Some(&Grouping::default()),
    )
    .expect("copy the grouping default");
    assert_eq!(
        fs::read_to_string(dir.path().join(GROUPINGS_DIR).join("ramadhan.toml")).expect("read"),
        SEPATU,
        "the copy is the other grouping's file, byte for byte"
    );

    // Copying a grouping with no file starts from the one catch-all entry.
    groupings::create(dir.path(), &grouping("kosong"), Some(&grouping("gone")))
        .expect("copy a grouping with no file");
    assert_eq!(
        groupings::load(dir.path(), &grouping("kosong")).expect("load"),
        vec![catch_all_category()]
    );

    // A file already there is never overwritten.
    let error = groupings::create(dir.path(), &grouping("ramadhan"), None).expect_err("exists");
    assert!(matches!(error, GroupingError::Exists { .. }), "{error:?}");
    assert_eq!(
        fs::read_to_string(dir.path().join(GROUPINGS_DIR).join("ramadhan.toml")).expect("read"),
        SEPATU
    );
}
