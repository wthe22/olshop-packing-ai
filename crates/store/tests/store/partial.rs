//! `output/.partial/` settled by the journal's last record when a date opens: an operation that
//! committed moves forward, one that did not goes back.

use std::fs;

use packing_engine::ids::BatchId;
use packing_engine::record::{BatchReverted, RecordBody};
use packing_store::{
    Journal, OutputState, Settled, move_aside, move_forward, open_date, partial_dir, replaced_dir,
};

use crate::common::*;

const BATCH_ONE_FILES: [&str; 3] = ["1-A Sepatu x2.pdf", "1-Z Lainnya x1.pdf", "1 summary.pdf"];

fn output_names(folder: &packing_store::DateFolder) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(folder.dir().join("output"))
        .unwrap()
        .filter_map(|entry| {
            let entry = entry.unwrap();
            entry
                .file_type()
                .unwrap()
                .is_file()
                .then(|| entry.file_name().to_string_lossy().into_owned())
        })
        .collect();
    names.sort();
    names
}

fn read_output(folder: &packing_store::DateFolder, file: &str) -> String {
    fs::read_to_string(folder.dir().join("output").join(file)).unwrap()
}

/// Batch 1 saved and moved into `output/`.
fn saved_batch_one(tag: &str) -> (testkit::TempDir, packing_store::DateFolder, Journal) {
    let (data, folder) = date_folder(tag);
    let mut journal = Journal::open(&folder).unwrap();
    let files = outputs(&partial_dir(folder.dir()), 1, "one");
    journal
        .commit(vec![RecordBody::BatchSaved(batch(1, false, files))], APP)
        .unwrap();
    move_forward(folder.dir()).unwrap();
    (data, folder, journal)
}

#[test]
fn a_crash_between_the_commit_and_the_moves_is_settled_forward() {
    let (_data, folder) = date_folder("crash-forward");
    let mut journal = Journal::open(&folder).unwrap();
    let files = outputs(&partial_dir(folder.dir()), 1, "one");
    journal
        .commit(vec![RecordBody::BatchSaved(batch(1, false, files))], APP)
        .unwrap();
    // The app stops here: the batch is saved, its files are still in `.partial/`.

    let opened = open_date(&folder).unwrap();
    assert_eq!(opened.settled, Settled::Forward { moved: 3 });
    assert!(!partial_dir(folder.dir()).exists());
    assert_eq!(
        output_names(&folder),
        ["1 summary.pdf", "1-A Sepatu x2.pdf", "1-Z Lainnya x1.pdf"]
    );
    assert!(opened.outputs.is_empty(), "{:?}", opened.outputs);
    assert!(opened.views_written);
}

#[test]
fn a_save_that_was_not_committed_is_settled_back() {
    let (_data, folder, _journal) = saved_batch_one("cancel-back");
    outputs(&partial_dir(folder.dir()), 2, "two");
    // No commit: the save of batch 2 stopped before the append.

    let opened = open_date(&folder).unwrap();
    assert_eq!(opened.settled, Settled::Back { restored: 0 });
    assert!(!partial_dir(folder.dir()).exists());
    assert_eq!(output_names(&folder).len(), 3, "batch 1's files only");
    assert_eq!(opened.journal.fold().next_batch(), BatchId::new(2));
}

#[test]
fn an_amend_that_committed_replaces_the_old_files() {
    let (_data, folder, mut journal) = saved_batch_one("amend-forward");
    move_aside(folder.dir(), &BATCH_ONE_FILES.map(String::from)).unwrap();
    let files = outputs(&partial_dir(folder.dir()), 1, "amended");
    journal
        .commit(vec![RecordBody::BatchSaved(batch(1, true, files))], APP)
        .unwrap();

    let opened = open_date(&folder).unwrap();
    assert_eq!(opened.settled, Settled::Forward { moved: 3 });
    assert_eq!(read_output(&folder, "1-A Sepatu x2.pdf"), "A amended");
    assert!(!replaced_dir(folder.dir()).exists());
    assert!(opened.outputs.is_empty());
}

#[test]
fn an_amend_that_did_not_commit_puts_the_old_files_back() {
    let (_data, folder, _journal) = saved_batch_one("amend-back");
    move_aside(folder.dir(), &BATCH_ONE_FILES.map(String::from)).unwrap();
    outputs(&partial_dir(folder.dir()), 1, "amended");

    let opened = open_date(&folder).unwrap();
    assert_eq!(opened.settled, Settled::Back { restored: 3 });
    assert_eq!(read_output(&folder, "1-A Sepatu x2.pdf"), "A one");
    assert!(opened.outputs.is_empty());
}

#[test]
fn a_revert_settles_by_whether_it_was_committed() {
    for committed in [true, false] {
        let (_data, folder, mut journal) = saved_batch_one(&format!("revert-{committed}"));
        move_aside(folder.dir(), &BATCH_ONE_FILES.map(String::from)).unwrap();
        if committed {
            journal
                .commit(
                    vec![RecordBody::BatchReverted(BatchReverted {
                        batch: BatchId::new(1),
                    })],
                    APP,
                )
                .unwrap();
        }

        let opened = open_date(&folder).unwrap();
        if committed {
            assert_eq!(opened.settled, Settled::Forward { moved: 0 });
            assert!(output_names(&folder).is_empty());
        } else {
            assert_eq!(opened.settled, Settled::Back { restored: 3 });
            assert_eq!(output_names(&folder).len(), 3);
        }
        assert!(!partial_dir(folder.dir()).exists());
    }
}

#[test]
fn a_missing_or_changed_output_is_found_at_open() {
    let (_data, folder, _journal) = saved_batch_one("outputs-check");
    let output = folder.dir().join("output");
    fs::remove_file(output.join("1-A Sepatu x2.pdf")).unwrap();
    fs::write(output.join("1 summary.pdf"), "a longer summary").unwrap();

    let opened = open_date(&folder).unwrap();
    let found: Vec<(&str, OutputState)> = opened
        .outputs
        .iter()
        .map(|problem| (problem.file.as_str(), problem.state))
        .collect();
    assert_eq!(
        found,
        [
            ("1-A Sepatu x2.pdf", OutputState::Missing),
            ("1 summary.pdf", OutputState::Changed)
        ]
    );
}

#[test]
fn a_crash_during_the_moves_is_settled_forward() {
    let (_data, folder) = date_folder("crash-midway");
    let mut journal = Journal::open(&folder).unwrap();
    let files = outputs(&partial_dir(folder.dir()), 1, "one");
    journal
        .commit(vec![RecordBody::BatchSaved(batch(1, false, files))], APP)
        .unwrap();
    let output = folder.dir().join("output");
    fs::rename(
        partial_dir(folder.dir()).join("1-A Sepatu x2.pdf"),
        output.join("1-A Sepatu x2.pdf"),
    )
    .unwrap();

    let opened = open_date(&folder).unwrap();
    assert_eq!(opened.settled, Settled::Forward { moved: 2 });
    assert_eq!(output_names(&folder).len(), 3);
    assert!(opened.outputs.is_empty());
}
