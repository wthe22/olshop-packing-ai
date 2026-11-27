//! Archive: a date's PDFs deleted, the live inputs last, everything else kept; the dates refused;
//! an archived date told from its folder; the size of a folder.

use std::fs;
use std::path::Path;

use jiff::civil::{Date, date};
use packing_engine::ids::{BatchId, PdfId};
use packing_engine::record::{BatchReverted, RecordBody};
use packing_store::{
    DateFolder, Journal, NotArchivable, SearchIndex, StoreError, archive, folder_size, input_path,
    is_archived, open_date, plan_archive,
};

use crate::common::*;

const TODAY: Date = date(2026, 11, 27);

/// The input of batch 1 as the record names it.
fn input_name(folder: &DateFolder) -> String {
    let path = input_path(folder, BatchId::new(1), PdfId::new(1), &sha('a'));
    path.file_name().unwrap().to_string_lossy().into_owned()
}

/// Batch 1 saved on 2026-10-07 with its input, its outputs, a reprint, the summaries made on
/// request, the views, an owner's note and an update folder holding a PDF.
fn saved_date(tag: &str) -> (testkit::TempDir, DateFolder) {
    let (data, folder) = date_folder(tag);
    let dir = folder.dir().to_path_buf();
    let saved = batch(1, false, outputs(&dir, 1, "first"));
    Journal::open(&folder)
        .unwrap()
        .commit(vec![RecordBody::BatchSaved(saved)], APP)
        .unwrap();
    for name in [
        input_name(&folder).as_str(),
        "R1 x3.pdf",
        "Day Summary.pdf",
        "1 Summary - Toko Alfa.pdf",
    ] {
        fs::write(dir.join(name), b"%PDF made up").unwrap();
    }
    fs::write(dir.join("notes.txt"), b"the owner's").unwrap();
    fs::create_dir_all(dir.join("update-0.8.0")).unwrap();
    fs::write(dir.join("update-0.8.0").join("1 Summary.pdf"), b"kept").unwrap();
    open_date(&folder).unwrap();
    (data, folder)
}

fn names(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(dir)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

fn refusal(folder: &DateFolder, today: Date) -> Option<NotArchivable> {
    match plan_archive(folder, today) {
        Ok(_) => None,
        Err(StoreError::NotArchivable(reason)) => Some(reason),
        Err(error) => panic!("{error}"),
    }
}

#[test]
fn archive_deletes_every_pdf_of_the_date_and_keeps_the_rest() {
    let (_data, folder) = saved_date("archive");
    let dir = folder.dir();
    let input = input_name(&folder);
    let plan = plan_archive(&folder, TODAY).unwrap();
    let planned: Vec<&str> = plan.pdfs.iter().map(|pdf| pdf.name.as_str()).collect();
    assert_eq!(planned.len(), 7);
    assert_eq!(planned.last(), Some(&input.as_str()), "the input goes last");
    assert_eq!(
        plan.bytes(),
        plan.pdfs.iter().map(|pdf| pdf.bytes).sum::<u64>()
    );

    let done = archive(plan);
    assert!(done.archived);
    assert!(done.left.is_empty());
    assert_eq!(done.deleted.len(), 7);
    assert_eq!(
        names(dir),
        [
            "day.toml",
            "journal.jsonl",
            "notes.txt",
            "order-contents.toml",
            "orders.csv",
            "update-0.8.0"
        ]
    );
    assert!(dir.join("update-0.8.0").join("1 Summary.pdf").is_file());

    let opened = open_date(&folder).unwrap();
    assert!(opened.archived);
    assert!(
        opened.inputs.is_empty() && opened.outputs.is_empty(),
        "nothing is missing"
    );
    assert_eq!(refusal(&folder, TODAY), Some(NotArchivable::Archived));
}

#[cfg(windows)]
#[test]
fn a_pdf_windows_refuses_to_delete_is_left_and_the_date_is_not_archived() {
    use std::os::windows::fs::OpenOptionsExt;

    let (_data, folder) = saved_date("archive-refused");
    let input = folder.dir().join(input_name(&folder));
    let held = fs::OpenOptions::new()
        .read(true)
        .share_mode(0)
        .open(&input)
        .unwrap();
    let done = archive(plan_archive(&folder, TODAY).unwrap());
    assert_eq!(done.left, [input_name(&folder)]);
    assert_eq!(done.deleted.len(), 6, "the rest went on");
    assert!(!done.archived);
    assert_eq!(refusal(&folder, TODAY), None, "it can be archived again");

    drop(held);
    let again = archive(plan_archive(&folder, TODAY).unwrap());
    assert_eq!(again.deleted, [input_name(&folder)]);
    assert!(again.archived);
}

#[test]
fn today_a_partial_date_a_date_without_a_batch_and_a_damaged_one_are_not_archived() {
    let (_data, folder) = saved_date("archive-refusals");
    assert_eq!(refusal(&folder, folder.date()), Some(NotArchivable::Today));

    let partial = folder.dir().join(".partial");
    fs::create_dir_all(&partial).unwrap();
    assert_eq!(refusal(&folder, TODAY), Some(NotArchivable::Partial));
    fs::remove_dir_all(&partial).unwrap();
    assert_eq!(refusal(&folder, TODAY), None);

    let reverted = BatchReverted {
        batch: BatchId::new(1),
    };
    Journal::open(&folder)
        .unwrap()
        .commit(vec![RecordBody::BatchReverted(reverted)], APP)
        .unwrap();
    assert_eq!(refusal(&folder, TODAY), Some(NotArchivable::NoBatch));

    let empty = DateFolder::new(folder.dir().parent().unwrap(), date(2026, 10, 8));
    fs::create_dir_all(empty.dir()).unwrap();
    assert_eq!(refusal(&empty, TODAY), Some(NotArchivable::NoBatch));

    write_journal(&empty, &[record(1, started())], b"");
    assert_eq!(refusal(&empty, TODAY), Some(NotArchivable::Damaged));
}

#[test]
fn a_date_is_archived_when_none_of_its_live_inputs_is_there() {
    let data = testkit::TempDir::new("archived-table");
    let dir = data.path();
    fs::write(dir.join("1 Input 01 a.pdf"), b"").unwrap();
    let inputs = |names: &[&str]| {
        names
            .iter()
            .map(|name| name.to_string())
            .collect::<Vec<_>>()
    };
    for (live, archived) in [
        (inputs(&[]), false),
        (inputs(&["1 Input 01 a.pdf"]), false),
        (inputs(&["1 Input 01 a.pdf", "2 Input 01 b.pdf"]), false),
        (inputs(&["2 Input 01 b.pdf"]), true),
        (inputs(&["2 Input 01 b.pdf", "3 Input 01 c.pdf"]), true),
    ] {
        assert_eq!(is_archived(dir, &live), archived, "{live:?}");
    }
}

#[test]
fn the_index_tells_the_archived_dates_from_their_folders() {
    let (data, folder) = saved_date("archive-index");
    let index = SearchIndex::build(data.path()).unwrap();
    assert!(index.archived_dates(data.path()).is_empty());
    archive(plan_archive(&folder, TODAY).unwrap());
    assert_eq!(
        index
            .archived_dates(data.path())
            .into_iter()
            .collect::<Vec<_>>(),
        [folder.date()]
    );
}

#[test]
fn a_folders_size_is_every_file_in_it_and_its_subfolders() {
    let data = testkit::TempDir::new("folder-size");
    let dir = data.path();
    assert_eq!(folder_size(&dir.join("not there")).unwrap(), 0);
    fs::write(dir.join("a.pdf"), [0u8; 100]).unwrap();
    fs::create_dir_all(dir.join("sub").join("deeper")).unwrap();
    fs::write(dir.join("sub").join("b.toml"), [0u8; 20]).unwrap();
    fs::write(dir.join("sub").join("deeper").join("c"), [0u8; 3]).unwrap();
    assert_eq!(folder_size(dir).unwrap(), 123);
}
