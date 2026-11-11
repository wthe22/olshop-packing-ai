//! The dates of the data folder: the list, the journal lengths, the newest batch's rule,
//! data of older versions, and dates that open again as they were.

use std::fs;

use jiff::civil::date;
use packing_engine::ids::BatchId;
use packing_engine::record::RecordBody;
use packing_store::{
    DateFolder, Journal, OlderData, OlderVersion, SESSION_TOML, dates, journal_lengths,
    newest_batch_rule, older_data, open_date,
};
use testkit::{TempDir, testdata};

use crate::common::{APP, batch, outputs};

fn commit(data: &TempDir, day: i8, body: RecordBody) -> Journal {
    let mut journal = Journal::open(&DateFolder::new(data.path(), date(2026, 10, day))).unwrap();
    journal.commit(vec![body], APP).unwrap();
    journal
}

#[test]
fn the_dates_are_the_folders_named_as_a_date() {
    let data = TempDir::new("dates");
    assert!(dates(data.path()).unwrap().is_empty());
    assert!(journal_lengths(data.path()).unwrap().is_empty());
    commit(
        &data,
        8,
        RecordBody::BatchSaved(batch(1, false, Vec::new())),
    );
    for other in [
        "2026-10-06",
        "2026-10-7",
        "2026-13-01",
        "Toko Contoh",
        "labels",
    ] {
        fs::create_dir_all(data.path().join(other)).unwrap();
    }
    fs::write(data.path().join("2026-10-09"), b"a file, not a folder").unwrap();
    assert_eq!(
        dates(data.path()).unwrap(),
        [date(2026, 10, 6), date(2026, 10, 8)]
    );
    let lengths = journal_lengths(data.path()).unwrap();
    assert_eq!(lengths[&date(2026, 10, 6)], 0, "no journal yet");
    assert!(lengths[&date(2026, 10, 8)] > 0);
}

#[test]
fn the_newest_batchs_rule_is_found_on_whichever_date_it_was_saved() {
    let data = TempDir::new("newest-rule");
    assert_eq!(newest_batch_rule(data.path()).unwrap(), None);
    let save = |day: i8, number: u8, rule: &str| {
        let mut saved = batch(number, false, Vec::new());
        saved.sort_rule.name = rule.into();
        commit(&data, day, RecordBody::BatchSaved(saved));
    };
    save(6, 1, "lama");
    save(7, 1, "default");
    save(7, 2, "harian");
    fs::create_dir_all(data.path().join("2026-10-08")).unwrap();
    let damaged = data.path().join("2026-10-09");
    fs::create_dir_all(&damaged).unwrap();
    fs::write(damaged.join("journal.jsonl"), b"not a record\n").unwrap();
    assert_eq!(
        newest_batch_rule(data.path()).unwrap().as_deref(),
        Some("harian"),
        "a later date with no batch, or a damaged journal, is passed over"
    );
}

#[test]
fn data_of_older_versions_is_named_and_left_as_it_is() {
    let data = TempDir::new("older");
    assert!(older_data(data.path()).unwrap().is_empty());
    commit(
        &data,
        7,
        RecordBody::BatchSaved(batch(1, false, Vec::new())),
    );
    fs::create_dir_all(data.path().join("labels").join("2026-10-07")).unwrap();
    fs::write(data.path().join("state.json"), b"{}").unwrap();
    fs::create_dir_all(data.path().join("Toko Alfa")).unwrap();
    fs::write(data.path().join("Toko Alfa").join("settings.toml"), b"").unwrap();
    let beta = data.path().join("Toko Beta").join("2026-10-07");
    fs::create_dir_all(&beta).unwrap();
    fs::write(beta.join("journal.jsonl"), b"").unwrap();
    fs::create_dir_all(data.path().join("Toko Gamma").join("2026-10-07")).unwrap();
    assert_eq!(
        older_data(data.path()).unwrap(),
        [
            OlderData {
                version: OlderVersion::V0_1,
                names: vec!["labels".into(), "state.json".into()],
            },
            OlderData {
                version: OlderVersion::V0_4,
                names: vec!["Toko Alfa".into(), "Toko Beta".into()],
            },
        ],
        "a folder with neither settings nor a journal is no workspace"
    );
    assert_eq!(dates(data.path()).unwrap(), [date(2026, 10, 7)]);

    let v0_4 = testdata("v0.4/data");
    assert_eq!(
        older_data(&v0_4).unwrap(),
        [OlderData {
            version: OlderVersion::V0_4,
            names: vec!["Toko Alfa".into(), "Toko Beta".into()],
        }],
        "the folder the v0.4 program wrote"
    );
    assert!(dates(&v0_4).unwrap().is_empty(), "it is not read");
}

/// Two dates written, opened, then opened again from the journal alone: the same fold and views.
#[test]
fn each_date_opens_again_as_it_was_saved() {
    let data = TempDir::new("reopen");
    for (day, tag) in [(7, "seventh"), (8, "eighth")] {
        let folder = DateFolder::new(data.path(), date(2026, 10, day));
        let files = outputs(&folder.dir().join("output"), 1, tag);
        commit(&data, day, RecordBody::BatchSaved(batch(1, false, files)));
    }
    let eighth = DateFolder::new(data.path(), date(2026, 10, 8));
    let mut journal = Journal::open(&eighth).unwrap();
    let files = outputs(&eighth.dir().join("output"), 2, "second");
    journal
        .commit(vec![RecordBody::BatchSaved(batch(2, false, files))], APP)
        .unwrap();
    assert_eq!(
        dates(data.path()).unwrap(),
        [date(2026, 10, 7), date(2026, 10, 8)]
    );

    for day in [7, 8] {
        let folder = DateFolder::new(data.path(), date(2026, 10, day));
        let first = open_date(&folder).unwrap();
        assert!(first.journal.damage().is_none() && first.outputs.is_empty());
        let session = fs::read_to_string(folder.dir().join(SESSION_TOML)).unwrap();
        assert!(session.starts_with(&format!("schema = \"2.0\"\ndate = \"2026-10-0{day}\"\n")));

        let again = open_date(&folder).unwrap();
        assert_eq!(again.journal.fold(), first.journal.fold(), "day {day}");
        assert!(!again.views_written, "day {day}: the views are current");
        assert_eq!(
            fs::read_to_string(folder.dir().join(SESSION_TOML)).unwrap(),
            session
        );
    }
    let live = |day: i8| {
        let opened = open_date(&DateFolder::new(data.path(), date(2026, 10, day))).unwrap();
        opened
            .journal
            .fold()
            .live
            .keys()
            .copied()
            .collect::<Vec<_>>()
    };
    assert_eq!(live(7), [BatchId::new(1)]);
    assert_eq!(
        live(8),
        [BatchId::new(1), BatchId::new(2)],
        "each date counts its batches from 1"
    );
}
