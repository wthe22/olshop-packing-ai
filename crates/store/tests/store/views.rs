//! The views: their text for a small batch, rebuilt when missing or stale, and named when changed
//! by hand.

use std::fs;

use packing_engine::ids::BatchId;
use packing_engine::record::{PrintSent, PrintedMarked, RecordBody};
use packing_store::{
    ChangedLine, DAY_TOML, EditedView, ORDER_CONTENTS_TOML, ORDERS_CSV, edited_views, open_date,
    partial_dir,
};

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

/// The views of the date changed by hand, against its journal.
fn edited(folder: &packing_store::DateFolder) -> Vec<EditedView> {
    let opened = open_date(folder).unwrap();
    edited_views(folder.dir(), Some(opened.journal.fold()))
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

#[test]
fn a_view_changed_by_hand_is_named_with_its_changed_lines() {
    let (_data, folder) = opened_batch("views-edited");
    assert_eq!(edited(&folder), []);
    for file in [ORDERS_CSV, ORDER_CONTENTS_TOML, DAY_TOML] {
        let (_data, folder) = opened_batch(&format!("views-edited-{file}"));
        let written = view(&folder, file);
        let line = written.lines().nth(1).unwrap().to_string();
        let changed = written.replacen(&line, &format!("{line} "), 1);
        fs::write(folder.dir().join(file), &changed).unwrap();
        assert!(
            !open_date(&folder).unwrap().views_written,
            "{file} is not written over"
        );
        // A row of orders.csv gives its changed cell: the last, the schema column's.
        let change = match file {
            ORDERS_CSV => ChangedLine {
                number: 2,
                column: Some("schema:2.1".into()),
                written: String::new(),
                found: " ".into(),
            },
            _ => ChangedLine {
                number: 2,
                column: None,
                written: line.clone(),
                found: format!("{line} "),
            },
        };
        assert_eq!(
            edited(&folder),
            [EditedView {
                file,
                lines: vec![change],
                more: 0,
            }],
            "{file}"
        );
        assert_eq!(view(&folder, file), changed);
    }
}

#[test]
fn a_day_toml_without_hashes_is_not_checked() {
    let (_data, folder) = opened_batch("views-unsealed");
    let day = view(&folder, DAY_TOML);
    let unsealed: String = day
        .lines()
        .filter(|line| !line.contains("sha256"))
        .map(|line| format!("{line}\n"))
        .collect();
    fs::write(folder.dir().join(DAY_TOML), unsealed).unwrap();
    fs::write(folder.dir().join(ORDERS_CSV), "changed\n").unwrap();
    assert_eq!(edited(&folder), []);
}

#[test]
fn a_view_without_a_journal_to_compare_is_named_alone() {
    let (_data, folder) = opened_batch("views-no-fold");
    fs::write(folder.dir().join(ORDERS_CSV), "changed\n").unwrap();
    let edited = edited_views(folder.dir(), None);
    assert_eq!(
        edited,
        [EditedView {
            file: ORDERS_CSV,
            lines: Vec::new(),
            more: 0,
        }]
    );
}
