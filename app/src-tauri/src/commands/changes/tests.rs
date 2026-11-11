//! Two copies of the app on one data folder: what one writes, the other's look finds and reads
//! again; a write waits for the other copy's and decides on the journal as it is on disk; a date
//! viewed while the other copy writes is opened again after; a date the other copy made is listed.

use std::thread;
use std::time::Duration;

use jiff::civil::Date;
use packing_engine::ids::BatchId;
use packing_engine::record::{OutputFile, PrintedMarked, RecordBody};
use packing_store::DataLock;
use testkit::{TempDir, batch, sha};

use super::*;
use crate::commands::day::{mark_printed, overview};
use crate::commands::start::{start_information, use_folder};

const DAY: &str = "2026-10-07";

/// Two copies of one program folder, the second started while the first runs, each showing
/// `DAY` and having looked once.
fn two_copies(tag: &str) -> (TempDir, AppState, AppState) {
    let program = TempDir::new(tag);
    let first = AppState::new(program.path().to_path_buf());
    start_information(&first).unwrap();
    use_folder(&first, &program.path().join("data")).unwrap();
    let second = AppState::new(program.path().to_path_buf());
    start_information(&second).expect("the second copy opens the folder the first uses");
    for state in [&first, &second] {
        state.index().wait();
        overview(state, DAY).unwrap();
        changes(state).unwrap();
    }
    (program, first, second)
}

fn save_batch(state: &AppState) {
    let date: Date = DAY.parse().unwrap();
    state
        .write_day(date, |open| {
            let pdf = OutputFile {
                file: "1-A Sepatu x1.pdf".into(),
                group: Some("A".into()),
                pages: 1,
                size: 100,
                sha256: sha('b'),
            };
            let saved = RecordBody::BatchSaved(batch(1, false, vec![pdf]));
            open.journal
                .commit(vec![saved], "0.3.3")
                .map(drop)
                .map_err(|error| error.to_string())
        })
        .unwrap();
}

fn printed(state: &AppState) -> Result<(), String> {
    let mark = PrintedMarked {
        batch: BatchId::new(1),
        group: "A".into(),
        printed: true,
    };
    mark_printed(state, DAY, mark)
}

fn batches(state: &AppState) -> usize {
    let shown = serde_json::to_value(overview(state, DAY).unwrap()).unwrap();
    shown["batches"].as_array().unwrap().len()
}

#[test]
fn what_one_copy_writes_the_other_finds_and_writes_after() {
    let (_program, first, second) = two_copies("app-changes-batch");
    assert_eq!(changes(&second).unwrap(), DataChanges::default());

    save_batch(&first);
    let found = DataChanges {
        day: true,
        panel: true,
    };
    assert_eq!(changes(&second).unwrap(), found);
    assert_eq!(batches(&second), 1);
    assert_eq!(
        changes(&second).unwrap(),
        DataChanges::default(),
        "read again, the date is current"
    );

    printed(&second).expect("the second copy's write follows the first's batch");
    assert!(changes(&first).unwrap().day, "and the first finds it");
}

#[test]
fn a_write_waits_for_the_other_copy_and_a_date_viewed_meanwhile_opens_again() {
    let (program, first, second) = two_copies("app-changes-held");
    save_batch(&first);
    let held = DataLock::acquire(&program.path().join("data")).unwrap();
    assert_eq!(batches(&second), 1, "viewed while the other copy writes");
    assert!(changes(&second).unwrap().day, "viewed only, so read again");

    let release = thread::spawn(move || {
        thread::sleep(Duration::from_millis(300));
        drop(held);
    });
    printed(&second).expect("the write waits for the lock");
    release.join().unwrap();
    batches(&second);
    assert!(
        !changes(&second).unwrap().day,
        "opened again once the write ended"
    );
}

#[test]
fn a_date_another_copy_made_is_listed() {
    let (program, _first, second) = two_copies("app-changes-dates");
    std::fs::create_dir_all(program.path().join("data").join("2026-10-06")).unwrap();
    assert_eq!(
        changes(&second).unwrap(),
        DataChanges {
            day: false,
            panel: true,
        }
    );
    assert_eq!(changes(&second).unwrap(), DataChanges::default());
}
