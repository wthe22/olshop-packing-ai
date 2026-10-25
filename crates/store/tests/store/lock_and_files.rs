//! The data folder's lock, and the inputs stored by content and checked at open.

use std::fs;

use packing_engine::record::RecordBody;
use packing_store::{
    DataLock, InputState, Journal, StoreError, check_inputs, input_path, open_date, sha256_of,
    store_input,
};
use testkit::TempDir;

use crate::common::*;

#[test]
fn a_second_copy_finds_the_folder_held_and_by_whom() {
    let data = TempDir::new("lock");
    let lock = DataLock::acquire(data.path()).unwrap();
    let holder = fs::read_to_string(data.path().join(".lock-holder")).unwrap();
    assert!(
        holder.contains(&format!("process {}", std::process::id())),
        "{holder}"
    );

    match DataLock::acquire(data.path()) {
        Err(StoreError::LockHeld { holder: named }) => assert_eq!(named, holder.trim()),
        other => panic!("expected the lock to be held, got {:?}", other.err()),
    }
    drop(lock);
    assert!(
        DataLock::acquire(data.path()).is_ok(),
        "free again once dropped"
    );
}

#[test]
fn an_input_is_stored_once_by_its_content() {
    let (data, folder) = date_folder("inputs-store");
    let source = data.path().join("labels_1.pdf");
    fs::write(&source, b"%PDF-1.4 made up").unwrap();

    let stored = store_input(folder.dir(), &source).unwrap();
    assert_eq!(stored.sha256, sha256_of(&source).unwrap());
    assert_eq!(stored.size, 16);
    let path = input_path(folder.dir(), &stored.sha256);
    assert_eq!(fs::read(&path).unwrap(), b"%PDF-1.4 made up");
    assert_eq!(
        store_input(folder.dir(), &source).unwrap(),
        stored,
        "stored again: the same"
    );
    let names: Vec<_> = fs::read_dir(path.parent().unwrap()).unwrap().collect();
    assert_eq!(names.len(), 1, "no .partial left");
}

#[test]
fn an_input_missing_or_changed_is_named_at_open() {
    let (data, folder) = date_folder("inputs-check");
    let source = data.path().join("labels_1.pdf");
    fs::write(&source, b"%PDF-1.4 made up").unwrap();
    let stored = store_input(folder.dir(), &source).unwrap();
    let mut saved = batch(1, false, Vec::new());
    saved.inputs[0].sha256 = stored.sha256.clone();
    let mut journal = Journal::open(&folder).unwrap();
    journal
        .commit(vec![RecordBody::BatchSaved(saved)], APP)
        .unwrap();
    assert!(
        check_inputs(folder.dir(), journal.fold())
            .unwrap()
            .is_empty()
    );

    let path = input_path(folder.dir(), &stored.sha256);
    fs::write(&path, b"%PDF-1.4 changed").unwrap();
    let opened = open_date(&folder).unwrap();
    assert_eq!(opened.inputs.len(), 1);
    assert_eq!(opened.inputs[0].state, InputState::Changed);
    assert_eq!(opened.inputs[0].filename, "labels_1.pdf");

    fs::remove_file(&path).unwrap();
    let opened = open_date(&folder).unwrap();
    assert_eq!(opened.inputs[0].state, InputState::Missing);
}

#[test]
fn a_date_without_a_journal_is_empty_and_makes_no_folder() {
    let (_data, folder) = date_folder("empty-date");
    let opened = open_date(&folder).unwrap();
    assert_eq!(opened.journal.fold().last_seq, 0);
    assert!(!opened.views_written);
    assert!(!folder.dir().exists());
}
