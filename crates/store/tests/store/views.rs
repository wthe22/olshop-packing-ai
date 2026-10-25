//! The views: their text for a small batch, and rebuilt when missing or stale.

use std::fs;

use packing_engine::ids::BatchId;
use packing_engine::record::{PrintedMarked, RecordBody};
use packing_store::{ORDER_CONTENTS_TOML, ORDERS_CSV, SESSION_TOML, open_date, partial_dir};

use crate::common::*;

/// Batch 1 written as a journal with fixed times and its outputs in place, then opened.
fn opened_batch(tag: &str) -> (testkit::TempDir, packing_store::DateFolder) {
    let (data, folder) = date_folder(tag);
    let files = outputs(&folder.dir().join("output"), 1, "one");
    let printed = RecordBody::PrintedMarked(PrintedMarked {
        batch: BatchId::new(1),
        group: "A".into(),
        printed: true,
    });
    write_journal(
        &folder,
        &[
            record(1, started()),
            record(2, RecordBody::BatchSaved(batch(1, false, files))),
            record(3, printed),
        ],
        b"",
    );
    assert!(open_date(&folder).unwrap().views_written);
    (data, folder)
}

fn view(folder: &packing_store::DateFolder, name: &str) -> String {
    fs::read_to_string(folder.dir().join(name)).unwrap()
}

#[test]
fn the_views_of_a_small_batch() {
    let (_data, folder) = opened_batch("views-text");
    assert_eq!(
        view(&folder, ORDERS_CSV),
        include_str!("expected/orders.csv")
    );
    assert_eq!(
        view(&folder, ORDER_CONTENTS_TOML),
        include_str!("expected/order-contents.toml")
    );
    assert_eq!(
        view(&folder, SESSION_TOML),
        include_str!("expected/session.toml")
    );
}

#[test]
fn a_missing_or_stale_view_is_written_again_and_a_current_one_is_left() {
    let (_data, folder) = opened_batch("views-rebuild");
    assert!(
        !open_date(&folder).unwrap().views_written,
        "current views are left"
    );

    fs::remove_file(folder.dir().join(ORDERS_CSV)).unwrap();
    assert!(open_date(&folder).unwrap().views_written, "a missing view");
    assert!(folder.dir().join(ORDERS_CSV).is_file());

    let stale = view(&folder, SESSION_TOML).replace("journal_seq = 3", "journal_seq = 2");
    fs::write(folder.dir().join(SESSION_TOML), stale).unwrap();
    assert!(
        open_date(&folder).unwrap().views_written,
        "a stale session.toml"
    );
    assert!(view(&folder, SESSION_TOML).contains("journal_seq = 3"));

    fs::write(folder.dir().join(SESSION_TOML), "not toml [").unwrap();
    assert!(
        open_date(&folder).unwrap().views_written,
        "an unreadable session.toml"
    );
    assert!(!partial_dir(folder.dir()).exists());
}
