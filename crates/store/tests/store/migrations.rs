//! What the owner's migration would change: views of another schema and batches saved before the
//! receiver's area or the weight was read, found without writing.

use std::fs;

use jiff::civil::date;
use packing_engine::ids::BatchId;
use packing_engine::record::{BatchSaved, RecordBody};
use packing_store::{
    DateFolder, DateMigration, Journal, SESSION_TOML, input_path, migrations, open_date,
};
use testkit::TempDir;

use crate::common::{APP, batch};

/// A batch of `day` saved and its views written; `stored` writes its label PDF into the folder.
fn saved_on(data: &TempDir, day: i8, saved: BatchSaved, stored: bool) -> DateFolder {
    let folder = DateFolder::new(data.path(), date(2026, 10, day));
    if stored {
        let input = &saved.inputs[0];
        fs::create_dir_all(folder.dir()).unwrap();
        let path = input_path(&folder, saved.batch, input.pdf, &input.sha256);
        fs::write(path, b"label PDF").unwrap();
    }
    let mut journal = Journal::open(&folder).unwrap();
    journal
        .commit(vec![RecordBody::BatchSaved(saved)], APP)
        .unwrap();
    open_date(&folder).unwrap();
    folder
}

fn saved_before_the_area(number: u8) -> BatchSaved {
    let mut saved = batch(number, false, Vec::new());
    for order in saved.orders.iter_mut().skip(1) {
        order.area = None;
    }
    saved
}

fn saved_before_the_weight(number: u8) -> BatchSaved {
    let mut saved = batch(number, false, Vec::new());
    saved.orders[0].weight = None;
    saved
}

#[test]
fn the_migration_names_older_views_and_the_batches_whose_labels_it_can_read() {
    let data = TempDir::new("migrations");
    assert!(migrations(data.path()).unwrap().is_empty());

    saved_on(&data, 6, batch(1, false, Vec::new()), true);
    let seventh = saved_on(&data, 7, saved_before_the_area(1), true);
    saved_on(&data, 7, batch(2, false, Vec::new()), true);
    saved_on(&data, 7, saved_before_the_weight(3), true);
    let older = saved_on(&data, 8, batch(1, false, Vec::new()), true);
    let session = fs::read_to_string(older.dir().join(SESSION_TOML)).unwrap();
    fs::write(
        older.dir().join(SESSION_TOML),
        session.replace("schema = \"2.1\"", "schema = \"2.0\""),
    )
    .unwrap();
    saved_on(&data, 9, saved_before_the_area(1), false);
    let damaged = data.path().join("2026-10-10");
    fs::create_dir_all(&damaged).unwrap();
    fs::write(damaged.join("journal.jsonl"), b"not a record\n").unwrap();
    fs::create_dir_all(data.path().join("2026-10-11")).unwrap();
    let journal_before = fs::read(seventh.dir().join("journal.jsonl")).unwrap();

    let orders = saved_before_the_area(1)
        .orders
        .iter()
        .filter(|order| order.area.is_none())
        .count()
        + 1;
    assert_eq!(
        migrations(data.path()).unwrap(),
        [
            DateMigration {
                date: date(2026, 10, 7),
                v0_7: false,
                older_views: None,
                batches: vec![BatchId::new(1), BatchId::new(3)],
                orders,
            },
            DateMigration {
                date: date(2026, 10, 8),
                v0_7: false,
                older_views: Some("2.0".into()),
                batches: Vec::new(),
                orders: 0,
            },
        ],
        "a batch whose label PDF is gone, a damaged journal and a date with none are passed over"
    );
    assert_eq!(
        fs::read(seventh.dir().join("journal.jsonl")).unwrap(),
        journal_before,
        "finding them writes nothing"
    );
}
