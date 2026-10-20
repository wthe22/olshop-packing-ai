//! `day.rs`: `state.json` round trip and format, numbering, the duplicate guard, file
//! fingerprints, revert, amend and the atomic save (mirrors tools/analysis/tests/test_picks.py).

use std::fs;
use std::path::{Path, PathBuf};

use packing_engine::day::{
    BatchEntry, BatchWrite, DayError, DayState, FileEntry, Findings, Fingerprint,
    PackingListChoice, PdfEntry, SavedItem, SavedOrder, delete_batch_folder, duplicates_warning,
    fingerprint, now_save_time, save_batch,
};
use packing_engine::groupings::Grouping;
use packing_engine::ids::{BatchNumber, OrderId, PdfNumber, TrackingId};
use packing_engine::names::DOWNLOAD_DIR;
use packing_engine::orders::ReadOrder;
use serde_json::Value;
use testkit::TempDir;

fn batch(number: i64) -> BatchNumber {
    BatchNumber::new(number)
}

fn pdf(number: i64) -> PdfNumber {
    PdfNumber::new(number)
}

fn read_order(id: &str) -> ReadOrder {
    ReadOrder {
        pages: Vec::new(),
        ..testkit::read_order(id, &format!("JY{id}"), Vec::new())
    }
}

fn file_entry(name: &str, xxh3: &str) -> FileEntry {
    FileEntry {
        name: name.to_string(),
        size: 10,
        xxh3: xxh3.to_string(),
        pages: 5,
        legacy_sha256: None,
    }
}

fn pdf_entry(number: i64) -> PdfEntry {
    PdfEntry {
        number: pdf(number),
        code: "A".to_string(),
        name: "Sepatu".to_string(),
        file: format!("{number} A Sepatu ×1.pdf"),
        orders: 1,
        runs: 1,
    }
}

fn saved(order_id: &str, batch_number: i64, pdf_number: i64, run: usize) -> SavedOrder {
    SavedOrder {
        order_id: OrderId::new(order_id),
        tracking_id: Some(TrackingId::new(format!("JY{order_id}"))),
        batch: batch(batch_number),
        pdf: pdf(pdf_number),
        run,
        items: vec![SavedItem {
            name: "Sepatu".to_string(),
            variation: "42".to_string(),
            seller_sku: "SP-0001".to_string(),
            quantity: 2,
        }],
        courier: Some("J&T Express".to_string()),
        total_quantity: 2,
        distinct_items: 1,
        ship_by: Some("2026-10-07T17:00:00".to_string()),
    }
}

/// One saved batch with the fixture's file and packing-list choice.
fn batch_entry(number: i64, pdfs: Vec<PdfEntry>) -> BatchEntry {
    BatchEntry {
        batch: batch(number),
        save_time: "2026-10-07T07:40:12+07:00".to_string(),
        grouping: Grouping::default(),
        files: vec![file_entry("labels_1.pdf", "9f2c")],
        pdfs,
        packing_list: PackingListChoice::default(),
        printed: Vec::new(),
        warnings: Vec::new(),
        errors: Vec::new(),
    }
}

fn batch_write(number: i64, pdfs: Vec<PdfEntry>, saved_orders: Vec<SavedOrder>) -> BatchWrite {
    BatchWrite {
        batch: batch(number),
        save_time: "2026-10-07T07:40:12+07:00".to_string(),
        grouping: Grouping::default(),
        files: vec![file_entry("labels_1.pdf", "9f2c")],
        pdfs,
        packing_list: PackingListChoice::default(),
        warnings: Vec::new(),
        saved: saved_orders,
    }
}

/// A day holding one saved batch, and the state that saved it.
fn day_with_one_batch(dir: &TempDir) -> (PathBuf, DayState) {
    let day = dir.path().join("2026-10-07");
    let mut state = DayState::new("2026-10-07");
    save_batch(
        &day,
        &mut state,
        &batch_write(1, vec![pdf_entry(1)], vec![saved("a", 1, 1, 1)]),
        |_| Ok(Findings::default()),
    )
    .expect("save");
    (day, state)
}

/// The day's `state.json` with `key` removed from its first batch, written to a sibling folder:
/// the file an older program left, before that field existed.
fn state_without(day: &Path, key: &str) -> PathBuf {
    let text = fs::read_to_string(day.join("state.json")).expect("read");
    let mut value: Value = serde_json::from_str(&text).expect("parse");
    value["batches"][0]
        .as_object_mut()
        .expect("batch object")
        .remove(key);
    let older = day.with_file_name(format!(
        "older-{}",
        day.file_name().expect("day name").to_string_lossy()
    ));
    fs::create_dir_all(&older).expect("create older day");
    fs::write(
        older.join("state.json"),
        serde_json::to_string_pretty(&value).expect("serialize"),
    )
    .expect("write older");
    older
}

#[test]
fn a_missing_state_is_an_empty_day_and_a_saved_one_reloads_in_the_07_format() {
    let dir = TempDir::new("state");
    let day = dir.path().join("2026-10-07");

    // No `state.json` yet: the day starts empty and numbered from one.
    let empty = DayState::load(&day).expect("load");
    assert_eq!(empty.day, "2026-10-07");
    assert!(empty.batches.is_empty());
    assert!(empty.saved.is_empty());
    assert_eq!(empty.next_batch_number(), batch(1));
    assert_eq!(empty.next_pdf_number(), pdf(1));

    let mut state = DayState::new("2026-10-07");
    state.batches.push(batch_entry(1, vec![pdf_entry(1)]));
    state.saved.push(saved("580000000000000001", 1, 1, 5));
    state.save(&day).expect("save");

    // The write is whole: the file is there, the temporary one is not, and it loads back equal.
    assert!(day.join("state.json").is_file());
    assert!(!day.join("state.json.tmp").exists());
    assert_eq!(DayState::load(&day).expect("load"), state);

    let text = fs::read_to_string(day.join("state.json")).expect("read");
    let value: Value = serde_json::from_str(&text).expect("parse");
    assert_eq!(value["day"], "2026-10-07");

    let batch = &value["batches"][0];
    assert_eq!(batch["batch"], 1);
    assert_eq!(batch["save_time"], "2026-10-07T07:40:12+07:00");
    // The grouping the batch was packed with is stored with it.
    assert_eq!(batch["set"], "default");
    assert_eq!(batch["files"][0]["name"], "labels_1.pdf");
    assert_eq!(batch["files"][0]["size"], 10);
    assert_eq!(batch["files"][0]["xxh3"], "9f2c");
    assert_eq!(batch["files"][0]["pages"], 5);
    // A saved `state.json` carries `xxh3` only; the legacy field is never written back.
    assert!(batch["files"][0].get("sha256").is_none());
    assert_eq!(batch["pdfs"][0]["number"], 1);
    assert_eq!(batch["pdfs"][0]["code"], "A");
    assert_eq!(batch["pdfs"][0]["name"], "Sepatu");
    assert_eq!(batch["pdfs"][0]["file"], "1 A Sepatu ×1.pdf");
    assert_eq!(batch["pdfs"][0]["orders"], 1);
    assert_eq!(batch["pdfs"][0]["runs"], 1);
    assert_eq!(batch["packing_list"]["scope"], "whole");
    assert_eq!(batch["packing_list"]["layouts"][0], "pick");
    // The warnings, the check's errors and the Printed ticks are stored per batch, each written
    // as an empty list.
    assert_eq!(batch["warnings"].as_array().map(Vec::len), Some(0));
    assert_eq!(batch["errors"].as_array().map(Vec::len), Some(0));
    assert_eq!(batch["printed"].as_array().map(Vec::len), Some(0));

    let entry = &value["saved"][0];
    assert_eq!(entry["order_id"], "580000000000000001");
    assert_eq!(entry["tracking_id"], "JY580000000000000001");
    assert_eq!(entry["batch"], 1);
    assert_eq!(entry["pdf"], 1);
    assert_eq!(entry["run"], 5);
    // The batch carries the time; the order keeps none.
    assert!(entry.get("save_time").is_none());
    // The searched fields are written at save.
    assert_eq!(entry["courier"], "J&T Express");
    assert_eq!(entry["total_quantity"], 2);
    assert_eq!(entry["distinct_items"], 1);
    assert_eq!(entry["ship_by"], "2026-10-07T17:00:00");
    assert_eq!(entry["items"][0]["name"], "Sepatu");
    assert_eq!(entry["items"][0]["variation"], "42");
    assert_eq!(entry["items"][0]["seller_sku"], "SP-0001");
    assert_eq!(entry["items"][0]["quantity"], 2);
}

#[test]
fn an_older_state_file_loads_with_the_fields_it_lacks_empty() {
    let dir = TempDir::new("older");
    let (day, state) = day_with_one_batch(&dir);
    assert_eq!(state.batches.len(), 1);

    // A day written before search existed: the saved entry has only the IDs, the batch and the pdf/run.
    let no_search = dir.path().join("2026-10-06");
    fs::create_dir_all(&no_search).expect("create day");
    let json = r#"{
  "day": "2026-10-06",
  "batches": [],
  "saved": [
    {"order_id": "580000000000000001", "tracking_id": "JY0000001234",
     "batch": 1, "pdf": 1, "run": 5}
  ]
}"#;
    fs::write(no_search.join("state.json"), json).expect("write legacy state");
    let older = DayState::load(&no_search).expect("load legacy day");
    let entry = &older.saved[0];
    assert_eq!(entry.order_id.as_str(), "580000000000000001");
    assert!(entry.items.is_empty());
    assert!(entry.courier.is_none());
    assert_eq!(entry.total_quantity, 0);
    assert_eq!(entry.distinct_items, 0);
    assert!(entry.ship_by.is_none());

    // A batch without `printed`: no ticks.
    let no_ticks = state_without(&day, "printed");
    let older = DayState::load(&no_ticks).expect("load older day");
    assert!(older.batches[0].printed.is_empty());
    assert!(!older.batches[0].is_printed(pdf(1)));

    // A day written before the groupings existed: no `set` key, so the default grouping.
    let no_grouping = state_without(&day, "set");
    let older = DayState::load(&no_grouping).expect("load older day");
    assert_eq!(older.batches[0].grouping, Grouping::default());

    // A day written by the older code: the file entry carries `sha256`, no `xxh3`.
    let legacy = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";
    let with_sha = dir.path().join("2026-10-05");
    fs::create_dir_all(&with_sha).expect("create day");
    let json = format!(
        r#"{{
  "day": "2026-10-05",
  "batches": [
    {{
      "batch": 1,
      "save_time": "2026-10-07T07:40:12+07:00",
      "files": [{{"name": "labels_1.pdf", "size": 3, "sha256": "{legacy}", "pages": 5}}],
      "pdfs": [],
      "packing_list": {{"scope": "whole", "layouts": ["pick"]}}
    }}
  ],
  "saved": []
}}"#
    );
    fs::write(with_sha.join("state.json"), json).expect("write legacy state");

    let older = DayState::load(&with_sha).expect("load legacy day");
    // The value is read into the legacy field, not the new one.
    assert_eq!(
        older.batches[0].files[0].legacy_sha256.as_deref(),
        Some(legacy)
    );
    assert!(older.batches[0].files[0].xxh3.is_empty());

    // The legacy hash is never matched, even by a fingerprint carrying the same value.
    assert_eq!(
        older.used_in_batch(&Fingerprint {
            size: 3,
            xxh3: legacy.to_string(),
        }),
        None
    );

    // Saving writes `xxh3` only: the legacy key does not come back.
    older.save(&with_sha).expect("save");
    let text = fs::read_to_string(with_sha.join("state.json")).expect("read");
    assert!(!text.contains("sha256"), "{text}");
    assert!(text.contains("\"xxh3\""), "{text}");
}

#[test]
fn the_batch_records_the_read_warnings_and_the_check_findings() {
    let dir = TempDir::new("findings");
    let day = dir.path().join("2026-10-07");
    let mut state = DayState::new("2026-10-07");
    let mut write = batch_write(1, vec![pdf_entry(1)], vec![saved("a", 1, 1, 1)]);
    write.warnings = vec!["read warning".to_string()];
    save_batch(&day, &mut state, &write, |_| {
        Ok(Findings {
            errors: vec!["a page is missing".to_string()],
            warnings: vec!["the page order differs".to_string()],
        })
    })
    .expect("save batch");

    // The check's errors are kept apart; its warnings join the read warnings.
    assert_eq!(
        state.batches[0].errors,
        vec!["a page is missing".to_string()]
    );
    assert_eq!(
        state.batches[0].warnings,
        vec![
            "read warning".to_string(),
            "the page order differs".to_string()
        ]
    );

    // An older file without one of the keys loads with it empty.
    let no_errors = state_without(&day, "errors");
    let older = DayState::load(&no_errors).expect("load");
    assert!(older.batches[0].errors.is_empty());
    assert_eq!(older.batches[0].warnings.len(), 2);

    let no_warnings = state_without(&day, "warnings");
    let older = DayState::load(&no_warnings).expect("load");
    assert!(older.batches[0].warnings.is_empty());
    assert_eq!(older.batches[0].errors.len(), 1);
}

#[test]
fn a_printed_tick_is_saved_at_once_and_removed_with_its_batch() {
    let dir = TempDir::new("printed-revert");
    let day = dir.path().join("2026-10-07");
    let mut state = DayState::new("2026-10-07");
    save_batch(
        &day,
        &mut state,
        &batch_write(
            1,
            vec![pdf_entry(1), pdf_entry(2)],
            vec![saved("a", 1, 1, 1)],
        ),
        |_| Ok(Findings::default()),
    )
    .expect("save 1");
    save_batch(
        &day,
        &mut state,
        &batch_write(2, vec![pdf_entry(3)], vec![saved("b", 2, 3, 1)]),
        |_| Ok(Findings::default()),
    )
    .expect("save 2");

    // Ticking writes the day's `state.json` straight away.
    state
        .set_printed(&day, batch(1), pdf(2), true)
        .expect("tick");
    let loaded = DayState::load(&day).expect("load");
    assert_eq!(loaded.batches[0].printed, vec![pdf(2)]);
    assert!(loaded.batches[0].is_printed(pdf(2)));
    assert!(!loaded.batches[0].is_printed(pdf(1)));

    // Ticking the same row again leaves one entry; unticking removes it, in the file too.
    state
        .set_printed(&day, batch(1), pdf(2), true)
        .expect("tick again");
    assert_eq!(state.batches[0].printed, vec![pdf(2)]);
    state
        .set_printed(&day, batch(1), pdf(2), false)
        .expect("untick");
    assert!(state.batches[0].printed.is_empty());
    assert!(
        DayState::load(&day).expect("load").batches[0]
            .printed
            .is_empty()
    );

    // An unknown batch and an unknown saved-PDF number change nothing, and write nothing.
    let before = fs::read_to_string(day.join("state.json")).expect("read");
    state
        .set_printed(&day, batch(9), pdf(1), true)
        .expect("unknown batch");
    state
        .set_printed(&day, batch(1), pdf(9), true)
        .expect("unknown pdf");
    assert!(state.batches[0].printed.is_empty());
    let after = fs::read_to_string(day.join("state.json")).expect("read again");
    assert_eq!(before, after, "an unknown row writes nothing");

    // The ticks belong to the batch: a revert removes them with it, and the batch left keeps its own.
    state
        .set_printed(&day, batch(1), pdf(1), true)
        .expect("tick batch 1");
    state
        .set_printed(&day, batch(2), pdf(3), true)
        .expect("tick batch 2");
    let removed = state.revert_last().expect("a batch");
    assert_eq!(removed.batch, batch(2));
    assert_eq!(removed.printed, vec![pdf(3)]);
    assert!(state.batches.iter().all(|entry| entry.batch != batch(2)));
    assert_eq!(state.batches[0].printed, vec![pdf(1)]);
}

#[test]
fn a_second_batch_of_the_day_continues_the_numbering() {
    let dir = TempDir::new("numbers");
    let day = dir.path().join("2026-10-07");
    let mut state = DayState::new("2026-10-07");

    save_batch(
        &day,
        &mut state,
        &batch_write(
            1,
            vec![pdf_entry(1), pdf_entry(2)],
            vec![saved("a", 1, 1, 1)],
        ),
        |_| Ok(Findings::default()),
    )
    .expect("save batch 1");

    assert_eq!(state.next_batch_number(), batch(2));
    assert_eq!(state.next_pdf_number(), pdf(3));

    save_batch(
        &day,
        &mut state,
        &batch_write(2, vec![pdf_entry(3)], vec![saved("b", 2, 4, 2)]),
        |_| Ok(Findings::default()),
    )
    .expect("save batch 2");
    assert_eq!(state.next_batch_number(), batch(3));
    assert_eq!(state.next_pdf_number(), pdf(5));
}

#[test]
fn a_fingerprint_is_the_size_and_xxh3_of_the_bytes() {
    let dir = TempDir::new("fingerprint");
    // The second case is larger than the 64 KB read buffer, so the file is hashed in several
    // chunks; its digest is compared with the one-shot hash of the same bytes.
    let big: Vec<u8> = (0..200_000u32)
        .map(|i| i.wrapping_mul(31).wrapping_add(7) as u8)
        .collect();
    let cases: Vec<(Vec<u8>, Option<&str>)> = vec![
        // The known XXH3-64 (seed 0) of "abc".
        (b"abc".to_vec(), Some("78af5f94892f3950")),
        (big, None),
    ];

    for (bytes, known) in cases {
        let path = dir.path().join("labels.pdf");
        fs::write(&path, &bytes).expect("write");
        let print = fingerprint(&path).expect("fingerprint");
        assert_eq!(print.size, bytes.len() as u64);
        assert_eq!(
            print.xxh3,
            format!("{:016x}", xxhash_rust::xxh3::xxh3_64(&bytes))
        );
        // 16 lowercase hex characters.
        assert_eq!(print.xxh3.len(), 16);
        assert_eq!(print.xxh3, print.xxh3.to_lowercase());
        if let Some(known) = known {
            assert_eq!(print.xxh3, known);
        }
    }

    assert!(matches!(
        fingerprint(&dir.path().join("missing")),
        Err(DayError::Read { .. })
    ));
}

#[test]
fn a_file_used_in_a_batch_is_recognised_by_its_fingerprint_and_a_changed_byte_is_not() {
    let dir = TempDir::new("usedin");
    let (day, mut state) = day_with_one_batch(&dir);

    // The fixture batch stores a short hash by hand: the size and the hash must both match.
    assert_eq!(
        state.used_in_batch(&Fingerprint {
            size: 10,
            xxh3: "9f2c".to_string(),
        }),
        Some(batch(1))
    );
    assert_eq!(
        state.used_in_batch(&Fingerprint {
            size: 11,
            xxh3: "9f2c".to_string(),
        }),
        None
    );

    // A made-up label file of 500 bytes, recorded in a second batch with its real fingerprint.
    let path = day.join(DOWNLOAD_DIR).join("labels_1.pdf");
    fs::create_dir_all(path.parent().expect("download")).expect("download");
    let bytes: Vec<u8> = (0..500u32).map(|i| (i % 251) as u8).collect();
    fs::write(&path, &bytes).expect("write");
    let print = fingerprint(&path).expect("fingerprint");
    state.batches.push(BatchEntry {
        files: vec![FileEntry {
            name: "labels_1.pdf".to_string(),
            size: print.size,
            xxh3: print.xxh3.clone(),
            pages: 5,
            legacy_sha256: None,
        }],
        ..batch_entry(2, vec![pdf_entry(2)])
    });
    // The same file, again: recognised as used in the batch that holds it.
    assert_eq!(state.used_in_batch(&print), Some(batch(2)));

    // One changed byte gives a different hash, so the file is not recognised (same size).
    let mut changed = bytes.clone();
    changed[0] ^= 0x01;
    fs::write(&path, &changed).expect("write changed");
    let changed_print = fingerprint(&path).expect("fingerprint");
    assert_eq!(changed_print.size, print.size);
    assert_ne!(changed_print.xxh3, print.xxh3);
    assert_eq!(state.used_in_batch(&changed_print), None);
}

#[test]
fn duplicate_guard_drops_saved_orders_with_batch_pdf_and_time() {
    let dir = TempDir::new("duplicate");
    let day = dir.path().join("2026-10-07");
    let mut state = DayState::new("2026-10-07");
    save_batch(
        &day,
        &mut state,
        &batch_write(
            1,
            vec![pdf_entry(1)],
            vec![saved("580000000000000001", 1, 1, 5)],
        ),
        |_| Ok(Findings::default()),
    )
    .expect("save");

    let orders = vec![
        read_order("580000000000000001"), // already saved
        read_order("580000000000000002"), // new
    ];
    let (kept, duplicates) = state.drop_saved(&orders);
    assert_eq!(kept.len(), 1);
    assert_eq!(kept[0].order.order_id.as_str(), "580000000000000002");
    assert_eq!(duplicates.len(), 1);
    let dup = &duplicates[0];
    assert_eq!(dup.order_id.as_str(), "580000000000000001");
    assert_eq!(
        dup.tracking_id.as_ref().map(TrackingId::as_str),
        Some("JY580000000000000001")
    );
    assert_eq!(dup.batch, batch(1));
    assert_eq!(dup.pdf, pdf(1));
    assert_eq!(dup.time, "2026-10-07T07:40:12+07:00");
    assert_eq!(
        duplicates_warning(&duplicates),
        "1 orders were already saved today and are left out: JY580000000000000001 (batch 1, PDF 1, 2026-10-07T07:40:12+07:00)"
    );
}

#[test]
fn a_reverted_batch_forgets_its_folder_and_its_numbers_are_reused() {
    let dir = TempDir::new("revert");
    let day = dir.path().join("2026-10-07");
    let mut state = DayState::new("2026-10-07");
    save_batch(
        &day,
        &mut state,
        &batch_write(1, vec![pdf_entry(1)], vec![saved("a", 1, 1, 1)]),
        |_| Ok(Findings::default()),
    )
    .expect("save 1");
    save_batch(
        &day,
        &mut state,
        &batch_write(2, vec![pdf_entry(2)], vec![saved("b", 2, 2, 1)]),
        |_| Ok(Findings::default()),
    )
    .expect("save 2");

    let removed = state.revert_last().expect("a batch");
    assert_eq!(removed.batch, batch(2));
    assert_eq!(state.batches.len(), 1);
    assert!(state.saved.iter().all(|order| order.batch != batch(2)));
    assert_eq!(state.next_batch_number(), batch(2));
    assert_eq!(state.next_pdf_number(), pdf(2));

    // With no batch left, revert is a no-op.
    state.revert_last();
    assert!(state.revert_last().is_none());

    // The folder goes; a missing one is not an error.
    assert!(day.join("batch 1").is_dir());
    delete_batch_folder(&day, batch(1)).expect("delete");
    assert!(!day.join("batch 1").exists());
    delete_batch_folder(&day, batch(9)).expect("missing is fine");
}

#[cfg(windows)]
#[test]
fn a_file_open_in_another_program_reports_file_in_use() {
    use std::fs::OpenOptions;
    use std::os::windows::fs::OpenOptionsExt;

    let dir = TempDir::new("inuse");
    let (day, _state) = day_with_one_batch(&dir);
    let file = day.join("batch 1").join("1 A Sepatu ×1.pdf");
    fs::write(&file, b"x").expect("write");

    // Open with no sharing, as a PDF viewer locks a file for writing.
    let _lock = OpenOptions::new()
        .read(true)
        .share_mode(0)
        .open(&file)
        .expect("lock");
    let error = delete_batch_folder(&day, batch(1)).expect_err("locked folder cannot be deleted");
    assert!(matches!(error, DayError::FileInUse { .. }), "{error:?}");
}

#[test]
fn an_amend_reads_the_batchs_files_and_its_own_grouping() {
    let dir = TempDir::new("amend");
    let day = dir.path().join("2026-10-07");
    let mut state = DayState::new("2026-10-07");
    save_batch(
        &day,
        &mut state,
        &batch_write(1, vec![pdf_entry(1)], vec![saved("a", 1, 1, 1)]),
        |_| Ok(Findings::default()),
    )
    .expect("save 1");
    let mut write = batch_write(2, vec![pdf_entry(2)], vec![saved("b", 2, 2, 1)]);
    write.grouping = Grouping::new("ramadhan");
    save_batch(&day, &mut state, &write, |_| Ok(Findings::default())).expect("save 2");
    // The grouping a batch was packed with is written at save and read back.
    assert_eq!(
        DayState::load(&day).expect("load").batches[1].grouping,
        Grouping::new("ramadhan")
    );
    state
        .set_printed(&day, batch(2), pdf(2), true)
        .expect("tick");

    // The amend carries that batch's own grouping and its download folder, and holds the batch's
    // numbers free for the save.
    let amend = state.amend(&day).expect("amend");
    assert_eq!(amend.batch, batch(2));
    assert_eq!(amend.grouping, Grouping::new("ramadhan"));
    assert_eq!(
        amend.files,
        vec![day.join("batch 2").join(DOWNLOAD_DIR).join("labels_1.pdf")]
    );
    assert_eq!(amend.state.batches.len(), 1);
    assert_eq!(amend.state.next_batch_number(), batch(2));
    assert_eq!(amend.state.next_pdf_number(), pdf(2));
    // The original state is untouched.
    assert_eq!(state.batches.len(), 2);
    assert_eq!(state.batches[1].printed, vec![pdf(2)]);

    // An amend writes the batch afresh, so its new entry starts with no ticks.
    let mut amended = amend.state;
    save_batch(&day, &mut amended, &write, |_| Ok(Findings::default())).expect("save again");
    assert_eq!(amended.batches.len(), 2);
    assert!(amended.batches[1].printed.is_empty());
    assert!(
        DayState::load(&day).expect("load").batches[1]
            .printed
            .is_empty()
    );
}

#[test]
fn saving_a_batch_writes_the_folder_and_state_and_rolls_back_on_a_failure() {
    let dir = TempDir::new("save");
    let day = dir.path().join("2026-10-07");

    let mut state = DayState::new("2026-10-07");
    let final_dir = save_batch(
        &day,
        &mut state,
        &batch_write(1, vec![pdf_entry(1)], vec![saved("a", 1, 1, 1)]),
        |temp| {
            fs::create_dir_all(temp.join(DOWNLOAD_DIR)).expect("download");
            fs::write(temp.join("1 A Sepatu ×1.pdf"), b"%PDF").expect("pdf");
            Ok(Findings::default())
        },
    )
    .expect("save");

    // The writer filled a temporary folder, which is moved into place whole.
    assert_eq!(final_dir, day.join("batch 1"));
    assert!(final_dir.join("1 A Sepatu ×1.pdf").is_file());
    assert!(!day.join("batch 1.partial").exists());
    assert!(day.join("state.json").is_file());
    assert_eq!(state.batches.len(), 1);
    assert_eq!(DayState::load(&day).expect("load"), state);

    // Saving the same batch number again (amend) replaces the old folder.
    fs::write(day.join("batch 1").join("old.txt"), b"old").expect("old");
    save_batch(
        &day,
        &mut state,
        &batch_write(1, vec![pdf_entry(1)], vec![saved("a", 1, 1, 1)]),
        |temp| {
            fs::write(temp.join("new.txt"), b"new").expect("new");
            Ok(Findings::default())
        },
    )
    .expect("save again");
    assert!(day.join("batch 1").join("new.txt").is_file());
    assert!(!day.join("batch 1").join("old.txt").exists());

    // A writer that fails after writing leaves nothing behind: no folder, no partial, no state.
    let failing = dir.path().join("2026-10-04");
    let mut state = DayState::new("2026-10-04");
    let error = save_batch(
        &failing,
        &mut state,
        &batch_write(1, vec![pdf_entry(1)], vec![saved("a", 1, 1, 1)]),
        |temp| {
            fs::write(temp.join("half.pdf"), b"%PDF").expect("half");
            Err(DayError::Write {
                folder: temp.to_path_buf(),
                message: "disk full".to_string(),
            })
        },
    )
    .expect_err("writer failed");

    assert!(matches!(error, DayError::Write { .. }));
    assert!(!failing.join("batch 1").exists());
    assert!(!failing.join("batch 1.partial").exists());
    assert!(!failing.join("state.json").exists());
    assert!(state.batches.is_empty());
    assert!(state.saved.is_empty());
}

#[test]
fn now_save_time_is_iso_with_an_offset() {
    let now = now_save_time();
    assert_eq!(now.len(), 25, "{now}");
    assert_eq!(&now[10..11], "T");
    assert_eq!(&now[19..20], "+", "the test host is UTC+07:00 ({now})");
}
