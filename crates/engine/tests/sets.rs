//! `sets.rs`: the set list of a data folder, the file of a set, the set-name rules, a set with
//! no file and create/delete (04 › *Sets*; 07 › *Rules file*).

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use packing_engine::rules::catch_all_category;
use packing_engine::sets::{self, CATEGORIES_FILE, DEFAULT_SET, SETS_DIR, SetError};

/// A unique folder under the system temp dir, removed when the test ends.
struct TempDir(PathBuf);

impl TempDir {
    fn new(tag: &str) -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "packing-engine-sets-{tag}-{}-{nanos}",
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

fn names(sets: &[sets::Set]) -> Vec<String> {
    sets.iter().map(|set| set.name.clone()).collect()
}

/// The set list of a data folder (04 › *Sets*): `default` always, then a set per `<name>.toml` in
/// its `categories/` folder. Read with only `categories.toml`, only `categories/*.toml`, with both
/// and with neither; a file that is not `.toml`, and a folder, are not sets.
#[test]
fn the_set_list_is_default_and_the_categories_folder_files() {
    // Only `categories.toml`: the set list is just the set `default`.
    let dir = TempDir::new("list-default");
    fs::write(dir.path().join(CATEGORIES_FILE), SEPATU).expect("write default");
    let list = sets::list(dir.path()).expect("list");
    assert_eq!(names(&list), vec![DEFAULT_SET.to_string()]);
    assert_eq!(list[0].file, CATEGORIES_FILE);

    // Only `categories/*.toml`: `default` plus a set per file, in file-name order.
    let dir = TempDir::new("list-files");
    let sets_dir = dir.path().join(SETS_DIR);
    fs::create_dir_all(&sets_dir).expect("create categories dir");
    fs::write(sets_dir.join("ramadhan.toml"), SEPATU).expect("write ramadhan");
    fs::write(sets_dir.join("b sepatu.toml"), REST).expect("write b sepatu");
    fs::write(sets_dir.join("notes.txt"), "not a set").expect("write notes");
    fs::create_dir_all(sets_dir.join("sub.toml")).expect("a folder named like a set");
    let list = sets::list(dir.path()).expect("list");
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
        "the set default keeps its own file"
    );
    assert_eq!(list[1].file, format!("{SETS_DIR}/b sepatu.toml"));

    // Both.
    fs::write(dir.path().join(CATEGORIES_FILE), SEPATU).expect("write default");
    assert_eq!(names(&sets::list(dir.path()).expect("list")).len(), 3);

    // Neither: `default` alone, the one catch-all set.
    let dir = TempDir::new("list-none");
    let list = sets::list(dir.path()).expect("list");
    assert_eq!(names(&list), vec![DEFAULT_SET.to_string()]);
}

/// A set writes to and loads from its own file, `categories/<set name>.toml`; a broken file gives
/// its own line and column, naming the set's file (04 › *Sets*).
#[test]
fn a_set_reads_and_writes_its_own_file() {
    let dir = TempDir::new("read-write");
    // A new set starts from the one catch-all entry and is written to its own file.
    sets::create(dir.path(), "ramadhan", None).expect("create");
    let file = dir.path().join(SETS_DIR).join("ramadhan.toml");
    assert!(file.is_file(), "a set's file is categories/<set name>.toml");
    assert_eq!(
        sets::load(dir.path(), "ramadhan").expect("load"),
        vec![catch_all_category()]
    );

    // The file's own entries load for that set.
    fs::write(&file, SEPATU).expect("write");
    let categories = sets::load(dir.path(), "ramadhan").expect("load");
    assert_eq!(categories.len(), 1);
    assert_eq!(categories[0].code, "A");

    // A broken file is refused with the set's file, line and column.
    fs::write(
        &file,
        "[[category]]\ncode = \"A\"\nname = \"N\"\nwhen = 'name foo \"x\"'\n",
    )
    .expect("write broken");
    let error = sets::load(dir.path(), "ramadhan").expect_err("a broken set");
    assert!(
        error
            .to_string()
            .contains("categories/ramadhan.toml, line 4, col"),
        "{error}"
    );
}

/// A set with no file — a fresh data folder, or a set named but not saved yet — reads as the one
/// catch-all entry (04 › *Sets*).
#[test]
fn a_set_with_no_file_reads_as_the_one_catch_all_entry() {
    let dir = TempDir::new("no-file");
    assert_eq!(
        sets::load(dir.path(), DEFAULT_SET).expect("load the set default"),
        vec![catch_all_category()]
    );
    assert_eq!(
        sets::load(dir.path(), "ramadhan").expect("load an unsaved set"),
        vec![catch_all_category()]
    );
}

/// The data folder's own `categories.toml` is never moved, renamed or rewritten by listing the
/// sets or by another set's save (04 › *Sets*).
#[test]
fn the_data_folders_own_categories_file_is_never_moved_or_rewritten() {
    let dir = TempDir::new("default-untouched");
    let own = dir.path().join(CATEGORIES_FILE);
    fs::write(&own, SEPATU).expect("write");

    let _ = sets::list(dir.path()).expect("list");
    let _ = sets::load(dir.path(), DEFAULT_SET).expect("load the set default");
    sets::create(dir.path(), "ramadhan", None).expect("create another set");
    let _ = sets::load(dir.path(), "ramadhan").expect("load another set");

    assert!(own.is_file(), "categories.toml is not moved or renamed");
    assert_eq!(
        fs::read_to_string(&own).expect("read"),
        SEPATU,
        "categories.toml is not rewritten"
    );
    assert_eq!(sets::file_of(DEFAULT_SET).expect("file"), CATEGORIES_FILE);
}

/// A file `categories/default.toml` is refused with both files named (04 › *Sets*).
#[test]
fn a_categories_default_file_is_refused_with_both_files_named() {
    let dir = TempDir::new("default-file");
    let sets_dir = dir.path().join(SETS_DIR);
    fs::create_dir_all(&sets_dir).expect("create categories dir");
    fs::write(sets_dir.join("default.toml"), REST).expect("write");

    let error = sets::list(dir.path()).expect_err("refused");
    assert_eq!(
        error,
        SetError::DefaultFile {
            default: CATEGORIES_FILE.to_string(),
            file: format!("{SETS_DIR}/default.toml"),
        }
    );
    let message = error.to_string();
    assert!(message.contains(CATEGORIES_FILE), "{message}");
    assert!(
        message.contains(&format!("{SETS_DIR}/default.toml")),
        "{message}"
    );
}

/// The set-name rules (04 › *Sets*): the file name is the set name, characters Windows forbids
/// become `-`, an empty name is refused, and the name `default` is the data folder's own
/// `categories.toml` — not created, not deleted, not renamed.
#[test]
fn the_set_name_rules_refuse_an_empty_name_and_default() {
    assert_eq!(
        sets::file_of("ramadhan").expect("file"),
        format!("{SETS_DIR}/ramadhan.toml")
    );
    assert_eq!(sets::file_of(DEFAULT_SET).expect("file"), CATEGORIES_FILE);
    assert_eq!(
        sets::file_of("a/b:c").expect("file"),
        format!("{SETS_DIR}/a-b-c.toml")
    );
    assert_eq!(sets::file_of("   "), Err(SetError::EmptyName));

    let dir = TempDir::new("refuse");
    assert_eq!(
        sets::create(dir.path(), DEFAULT_SET, None),
        Err(SetError::ReservedDefault)
    );
    assert_eq!(
        sets::delete(dir.path(), DEFAULT_SET),
        Err(SetError::ReservedDefault)
    );
    assert!(
        !dir.path().join(CATEGORIES_FILE).exists(),
        "nothing is written for the set default"
    );
    assert_eq!(
        sets::create(dir.path(), "  ", None),
        Err(SetError::EmptyName)
    );
}

/// **Delete set** removes the set's file (04 › *Sets*): its name leaves the set list and its picks
/// fall back to the one catch-all entry; a set with no file is already gone.
#[test]
fn deleting_a_set_removes_its_file_and_its_name_leaves_the_list() {
    let dir = TempDir::new("delete");
    sets::create(dir.path(), "ramadhan", None).expect("create");
    assert_eq!(
        names(&sets::list(dir.path()).expect("list")),
        vec!["default".to_string(), "ramadhan".to_string()]
    );

    sets::delete(dir.path(), "ramadhan").expect("delete");
    assert!(!dir.path().join(SETS_DIR).join("ramadhan.toml").exists());
    assert_eq!(
        names(&sets::list(dir.path()).expect("list")),
        vec![DEFAULT_SET.to_string()]
    );
    assert_eq!(
        sets::load(dir.path(), "ramadhan").expect("load"),
        vec![catch_all_category()]
    );
    sets::delete(dir.path(), "gone").expect("a set with no file is already gone");
}

/// A new set is empty (the one catch-all entry) or a copy of another set's file (04 › *Sets*); a
/// file already there is never overwritten.
#[test]
fn a_new_set_is_empty_or_a_copy_of_another_set() {
    let dir = TempDir::new("copy");
    fs::write(dir.path().join(CATEGORIES_FILE), SEPATU).expect("write default");
    sets::create(dir.path(), "ramadhan", Some(DEFAULT_SET)).expect("copy the set default");
    assert_eq!(
        fs::read_to_string(dir.path().join(SETS_DIR).join("ramadhan.toml")).expect("read"),
        SEPATU,
        "the copy is the other set's file, byte for byte"
    );

    // Copying a set with no file starts from the one catch-all entry.
    sets::create(dir.path(), "kosong", Some("gone")).expect("copy a set with no file");
    assert_eq!(
        sets::load(dir.path(), "kosong").expect("load"),
        vec![catch_all_category()]
    );

    // A file already there is never overwritten.
    let error = sets::create(dir.path(), "ramadhan", None).expect_err("exists");
    assert!(matches!(error, SetError::Exists { .. }), "{error:?}");
    assert_eq!(
        fs::read_to_string(dir.path().join(SETS_DIR).join("ramadhan.toml")).expect("read"),
        SEPATU
    );
}
