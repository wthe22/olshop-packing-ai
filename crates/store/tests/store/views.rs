//! The views: their text for a small batch, and rebuilt when missing or stale.

use std::fs;

use packing_engine::ids::BatchId;
use packing_engine::record::{PrintSent, PrintedMarked, RecordBody};
use packing_store::{DAY_TOML, ORDER_CONTENTS_TOML, ORDERS_CSV, open_date, partial_dir};

use crate::common::*;

/// Batch 1 written as a journal with fixed times and its outputs in place, then opened.
fn opened_batch(tag: &str) -> (testkit::TempDir, packing_store::DateFolder) {
    let (data, folder) = date_folder(tag);
    let files = outputs(folder.dir(), 1, "one");
    let mut saved = batch(1, false, files);
    saved.orders[2].shop = None;
    saved.orders[2].area = None;
    let printed = RecordBody::PrintedMarked(PrintedMarked {
        batch: BatchId::new(1),
        group: "A".into(),
        printed: true,
    });
    let sent = RecordBody::PrintSent(PrintSent {
        batch: BatchId::new(1),
        group: "A".into(),
        sent: true,
    });
    write_journal(
        &folder,
        &[
            record(1, started()),
            record(2, RecordBody::BatchSaved(saved)),
            record(3, printed),
            record(4, sent),
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
    assert_eq!(view(&folder, DAY_TOML), include_str!("expected/day.toml"));
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

    let stale = view(&folder, DAY_TOML).replace("journal_seq = 4", "journal_seq = 3");
    fs::write(folder.dir().join(DAY_TOML), stale).unwrap();
    assert!(
        open_date(&folder).unwrap().views_written,
        "a stale day.toml"
    );
    assert!(view(&folder, DAY_TOML).contains("journal_seq = 4"));

    let older = view(&folder, DAY_TOML).replace("schema = \"2.1\"", "schema = \"2.0\"");
    fs::write(folder.dir().join(DAY_TOML), older).unwrap();
    assert!(
        !open_date(&folder).unwrap().views_written,
        "a day.toml of an older version waits for the owner's migration"
    );
    assert!(view(&folder, DAY_TOML).starts_with("schema = \"2.0\""));

    fs::write(folder.dir().join(DAY_TOML), "not toml [").unwrap();
    assert!(
        open_date(&folder).unwrap().views_written,
        "an unreadable day.toml"
    );
    assert!(!partial_dir(folder.dir()).exists());
}

#[test]
fn a_newer_copys_views_are_left_to_it() {
    let (_data, folder) = opened_batch("views-newer");
    let newer = view(&folder, DAY_TOML)
        .replace("schema = \"2.1\"", "schema = \"2.5\"")
        .replace("journal_seq = 4", "journal_seq = 3");
    fs::write(folder.dir().join(DAY_TOML), &newer).unwrap();
    assert!(
        !open_date(&folder).unwrap().views_written,
        "a newer minor's day.toml is not written over, stale or not"
    );
    assert_eq!(view(&folder, DAY_TOML), newer);

    // Without a `day.toml`, `orders.csv` tells its own version.
    fs::remove_file(folder.dir().join(DAY_TOML)).unwrap();
    let csv: String = view(&folder, ORDERS_CSV)
        .lines()
        .map(|line| match line.strip_suffix("schema:2.1") {
            Some(head) => format!("{head}colour,schema:2.5\n"),
            None => format!("{line},\n"),
        })
        .collect();
    fs::write(folder.dir().join(ORDERS_CSV), &csv).unwrap();
    assert!(!open_date(&folder).unwrap().views_written);
    assert_eq!(view(&folder, ORDERS_CSV), csv, "its column is kept");
    fs::remove_file(folder.dir().join(ORDERS_CSV)).unwrap();

    let journal = folder.dir().join("journal.jsonl");
    let mut bytes = fs::read(&journal).unwrap();
    bytes.extend(sealed_line(
        r#"{"seq":5,"time":"2026-10-07T09:00:00+07:00","app":"0.9.0","type":"orders_moved","to":2}"#,
    ));
    fs::write(&journal, bytes).unwrap();
    let opened = open_date(&folder).unwrap();
    assert!(opened.journal.damage().is_none());
    assert!(
        !opened.views_written && !folder.dir().join(DAY_TOML).exists(),
        "a date with a record this copy passed over gets no views from it"
    );
}
