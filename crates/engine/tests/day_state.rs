//! `day.rs`: `state.json` round trip and format, numbering, the duplicate guard, file
//! fingerprints, revert, amend and the atomic save (07 › *Day folder*, *`state.json`*,
//! *Revert and amend*; mirrors tools/analysis/tests/test_picks.py).

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use packing_engine::day::{
    BatchWrite, DayError, DayState, FileEntry, Findings, Layout, PackingListChoice, PdfEntry,
    SavedItem, SavedOrder, Scope, delete_batch_folder, duplicates_warning, fingerprint,
    now_save_time, save_batch,
};
use packing_engine::names::DOWNLOAD_DIR;
use packing_engine::orders::{Order, ReadOrder};
use serde_json::Value;

/// A unique folder under the system temp dir, removed when the test ends.
struct TempDir(PathBuf);

impl TempDir {
    fn new(tag: &str) -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "packing-engine-{tag}-{}-{nanos}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("create temp dir");
        TempDir(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn read_order(id: &str) -> ReadOrder {
    ReadOrder {
        order: Order {
            order_id: id.to_string(),
            tracking_id: format!("JY{id}"),
            courier: "J&T Express".to_string(),
            ship_by: None,
            lines: Vec::new(),
            customer_message: String::new(),
        },
        pages: Vec::new(),
        position: 1,
        has_slip: true,
        qty_total: None,
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
        number,
        code: "A".to_string(),
        name: "Sepatu".to_string(),
        file: format!("{number} A Sepatu ×1.pdf"),
        orders: 1,
        runs: 1,
    }
}

fn saved(order_id: &str, batch: i64, pdf: i64, run: usize) -> SavedOrder {
    SavedOrder {
        order_id: order_id.to_string(),
        tracking_id: format!("JY{order_id}"),
        batch,
        pdf,
        run,
        items: vec![SavedItem {
            name: "Sepatu".to_string(),
            variation: "42".to_string(),
            seller_sku: "SP-0001".to_string(),
            quantity: 2,
        }],
        courier: "J&T Express".to_string(),
        total_quantity: 2,
        distinct_items: 1,
        ship_by: Some("2026-10-07T17:00:00".to_string()),
    }
}

fn batch_write(batch: i64, pdfs: Vec<PdfEntry>, saved_orders: Vec<SavedOrder>) -> BatchWrite {
    BatchWrite {
        batch,
        save_time: "2026-10-07T07:40:12+07:00".to_string(),
        files: vec![file_entry("labels_1.pdf", "9f2c")],
        pdfs,
        packing_list: PackingListChoice::default(),
        warnings: Vec::new(),
        saved: saved_orders,
    }
}

#[test]
fn missing_state_is_an_empty_state_for_the_folder_day() {
    let dir = TempDir::new("missing");
    let day = dir.path().join("2026-10-07");
    let state = DayState::load(&day).expect("load");
    assert_eq!(state.day, "2026-10-07");
    assert!(state.batches.is_empty());
    assert!(state.saved.is_empty());
    assert_eq!(state.next_batch_number(), 1);
    assert_eq!(state.next_pdf_number(), 1);
}

#[test]
fn save_and_load_state_round_trip() {
    let dir = TempDir::new("roundtrip");
    let day = dir.path().join("2026-10-07");

    let mut state = DayState::new("2026-10-07");
    state.batches.push(packing_engine::day::BatchEntry {
        batch: 1,
        save_time: "2026-10-07T07:40:12+07:00".to_string(),
        files: vec![file_entry("labels_1.pdf", "9f2c")],
        pdfs: vec![pdf_entry(1)],
        packing_list: PackingListChoice {
            scope: Scope::Whole,
            layouts: vec![Layout::Pick],
        },
        printed: Vec::new(),
        warnings: Vec::new(),
        errors: Vec::new(),
    });
    state.saved.push(saved("580000000000000001", 1, 1, 5));
    state.save(&day).expect("save");

    assert!(day.join("state.json").is_file());
    assert!(!day.join("state.json.tmp").exists());
    let loaded = DayState::load(&day).expect("load");
    assert_eq!(loaded, state);
}

#[test]
fn state_json_matches_the_07_format() {
    let dir = TempDir::new("format");
    let day = dir.path().join("2026-10-07");

    let mut state = DayState::new("2026-10-07");
    state.batches.push(packing_engine::day::BatchEntry {
        batch: 1,
        save_time: "2026-10-07T07:40:12+07:00".to_string(),
        files: vec![file_entry("labels_1.pdf", "9f2c")],
        pdfs: vec![pdf_entry(1)],
        packing_list: PackingListChoice {
            scope: Scope::Whole,
            layouts: vec![Layout::Pick],
        },
        printed: Vec::new(),
        warnings: Vec::new(),
        errors: Vec::new(),
    });
    state.saved.push(saved("580000000000000001", 1, 1, 5));
    state.save(&day).expect("save");

    let text = fs::read_to_string(day.join("state.json")).expect("read");
    let value: Value = serde_json::from_str(&text).expect("parse");
    assert_eq!(value["day"], "2026-10-07");

    let batch = &value["batches"][0];
    assert_eq!(batch["batch"], 1);
    assert_eq!(batch["save_time"], "2026-10-07T07:40:12+07:00");
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
    // The read warnings are stored per batch (07 › *`state.json`*), defaulted when absent.
    assert_eq!(batch["warnings"].as_array().map(Vec::len), Some(0));
    // The check before saving's errors are stored apart from the warnings, defaulted when absent.
    assert_eq!(batch["errors"].as_array().map(Vec::len), Some(0));
    // The **Printed** ticks are stored per batch (07 › *`state.json`*), defaulted when absent.
    assert_eq!(batch["printed"].as_array().map(Vec::len), Some(0));

    let entry = &value["saved"][0];
    assert_eq!(entry["order_id"], "580000000000000001");
    assert_eq!(entry["tracking_id"], "JY580000000000000001");
    assert_eq!(entry["batch"], 1);
    assert_eq!(entry["pdf"], 1);
    assert_eq!(entry["run"], 5);
    // The 07 example keeps no per-order save_time (the batch carries it).
    assert!(entry.get("save_time").is_none());
    // The searched fields are written at save (07 › *`state.json`*; 2.21).
    assert_eq!(entry["courier"], "J&T Express");
    assert_eq!(entry["total_quantity"], 2);
    assert_eq!(entry["distinct_items"], 1);
    assert_eq!(entry["ship_by"], "2026-10-07T17:00:00");
    assert_eq!(entry["items"][0]["name"], "Sepatu");
    assert_eq!(entry["items"][0]["variation"], "42");
    assert_eq!(entry["items"][0]["seller_sku"], "SP-0001");
    assert_eq!(entry["items"][0]["quantity"], 2);
}

/// An order saved before search existed keeps only its IDs: every searched field is absent in the
/// older `state.json` and loads as its empty value (07 › *`state.json`*; 2.21, Q-UI-3).
#[test]
fn a_saved_order_without_the_searched_fields_loads_with_them_empty() {
    let dir = TempDir::new("legacy-saved");
    let day = dir.path().join("2026-10-07");
    fs::create_dir_all(&day).expect("create day");
    // A day written before 2.21: the saved entry has only the IDs, the batch and the pdf/run.
    let json = r#"{
  "day": "2026-10-07",
  "batches": [],
  "saved": [
    {"order_id": "580000000000000001", "tracking_id": "JY0000001234",
     "batch": 1, "pdf": 1, "run": 5}
  ]
}"#;
    fs::write(day.join("state.json"), json).expect("write legacy state");

    let state = DayState::load(&day).expect("load legacy day");
    let saved = &state.saved[0];
    assert_eq!(saved.order_id, "580000000000000001");
    assert!(saved.items.is_empty());
    assert!(saved.courier.is_empty());
    assert_eq!(saved.total_quantity, 0);
    assert_eq!(saved.distinct_items, 0);
    assert!(saved.ship_by.is_none());

    std::fs::remove_dir_all(dir.path()).ok();
}

#[test]
fn batch_warnings_round_trip_and_default_when_absent() {
    let dir = TempDir::new("warnings");
    let day = dir.path().join("2026-10-07");

    let mut state = DayState::new("2026-10-07");
    let mut write = batch_write(1, vec![pdf_entry(1)], vec![saved("a", 1, 1, 1)]);
    write.warnings = vec!["2 orders were already saved today".to_string()];
    save_batch(&day, &mut state, &write, |_| Ok(Findings::default())).expect("save batch");
    assert_eq!(
        state.batches[0].warnings,
        vec!["2 orders were already saved today".to_string()]
    );

    // An older file without the key loads with no warnings.
    let text = fs::read_to_string(day.join("state.json")).expect("read");
    let mut value: Value = serde_json::from_str(&text).expect("parse");
    value["batches"][0]
        .as_object_mut()
        .expect("batch object")
        .remove("warnings");
    fs::write(
        day.join("state.json"),
        serde_json::to_string_pretty(&value).expect("serialize"),
    )
    .expect("write");
    let loaded = DayState::load(&day).expect("load");
    assert!(loaded.batches[0].warnings.is_empty());

    std::fs::remove_dir_all(dir.path()).ok();
}

#[test]
fn the_check_findings_are_stored_with_the_batch() {
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

    // The check's errors are kept apart; its warnings join the read warnings (07 › *Check
    // before saving*).
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

    // An older file without `errors` loads with none.
    let text = fs::read_to_string(day.join("state.json")).expect("read");
    let mut value: Value = serde_json::from_str(&text).expect("parse");
    value["batches"][0]
        .as_object_mut()
        .expect("batch object")
        .remove("errors");
    fs::write(
        day.join("state.json"),
        serde_json::to_string_pretty(&value).expect("serialize"),
    )
    .expect("write");
    assert!(
        DayState::load(&day).expect("load").batches[0]
            .errors
            .is_empty()
    );

    std::fs::remove_dir_all(dir.path()).ok();
}

#[test]
fn a_printed_tick_survives_a_write_and_reload() {
    let dir = TempDir::new("printed");
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
    .expect("save");

    // Ticking writes the day's `state.json` straight away, as the doc says (07 › *`state.json`*).
    state.set_printed(&day, 1, 2, true).expect("tick");
    let loaded = DayState::load(&day).expect("load");
    assert_eq!(loaded.batches[0].printed, vec![2]);
    assert!(loaded.batches[0].is_printed(2));
    assert!(!loaded.batches[0].is_printed(1));

    // Ticking the same row again leaves one entry.
    state.set_printed(&day, 1, 2, true).expect("tick again");
    assert_eq!(state.batches[0].printed, vec![2]);

    // Unticking removes it, and the file is written whole each time.
    state.set_printed(&day, 1, 2, false).expect("untick");
    assert!(state.batches[0].printed.is_empty());
    assert!(
        DayState::load(&day).expect("load").batches[0]
            .printed
            .is_empty()
    );
}

#[test]
fn a_state_without_the_printed_field_loads_with_no_ticks() {
    let dir = TempDir::new("printed-legacy");
    let day = dir.path().join("2026-10-07");
    fs::create_dir_all(&day).expect("create day");
    // A day written before the tick existed: the batch has no `printed` key.
    let json = r#"{
  "day": "2026-10-07",
  "batches": [
    {
      "batch": 1,
      "save_time": "2026-10-07T07:40:12+07:00",
      "files": [{"name": "labels_1.pdf", "size": 3, "xxh3": "9f2c", "pages": 5}],
      "pdfs": [{"number": 1, "code": "A", "name": "Sepatu", "file": "1 A Sepatu ×1.pdf", "orders": 1, "runs": 1}],
      "packing_list": {"scope": "whole", "layouts": ["pick"]}
    }
  ],
  "saved": []
}"#;
    fs::write(day.join("state.json"), json).expect("write legacy state");

    let state = DayState::load(&day).expect("load legacy day");
    assert!(state.batches[0].printed.is_empty());
    assert!(!state.batches[0].is_printed(1));
}

#[test]
fn set_printed_ignores_an_unknown_batch_or_pdf_number() {
    let dir = TempDir::new("printed-unknown");
    let day = dir.path().join("2026-10-07");
    let mut state = DayState::new("2026-10-07");
    save_batch(
        &day,
        &mut state,
        &batch_write(1, vec![pdf_entry(1)], vec![saved("a", 1, 1, 1)]),
        |_| Ok(Findings::default()),
    )
    .expect("save");
    let before = fs::read_to_string(day.join("state.json")).expect("read");

    // An unknown batch and an unknown saved-PDF number change nothing, and write nothing.
    state.set_printed(&day, 9, 1, true).expect("unknown batch");
    state.set_printed(&day, 1, 9, true).expect("unknown pdf");
    assert!(state.batches[0].printed.is_empty());
    let after = fs::read_to_string(day.join("state.json")).expect("read again");
    assert_eq!(before, after, "an unknown row writes nothing");
}

#[test]
fn revert_removes_the_batch_with_its_ticks() {
    let dir = TempDir::new("printed-revert");
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
    state.set_printed(&day, 1, 1, true).expect("tick batch 1");
    state.set_printed(&day, 2, 2, true).expect("tick batch 2");

    // The ticks belong to the batch: a revert removes them with it (07 › *`state.json`*).
    let removed = state.revert_last().expect("a batch");
    assert_eq!(removed.batch, 2);
    assert_eq!(removed.printed, vec![2]);
    assert!(state.batches.iter().all(|batch| batch.batch != 2));
    // The batch left keeps its own tick.
    assert_eq!(state.batches[0].printed, vec![1]);
}

#[test]
fn an_amended_batch_is_written_without_ticks() {
    let dir = TempDir::new("printed-amend");
    let day = dir.path().join("2026-10-07");
    let mut state = DayState::new("2026-10-07");
    save_batch(
        &day,
        &mut state,
        &batch_write(1, vec![pdf_entry(1)], vec![saved("a", 1, 1, 1)]),
        |_| Ok(Findings::default()),
    )
    .expect("save");
    state.set_printed(&day, 1, 1, true).expect("tick");
    assert_eq!(state.batches[0].printed, vec![1]);

    // An amend writes the batch afresh, so its new entry starts with no ticks
    // (07 › *`state.json`*).
    let amend = state.amend(&day).expect("amend");
    let mut amended = amend.state;
    save_batch(
        &day,
        &mut amended,
        &batch_write(1, vec![pdf_entry(1)], vec![saved("a", 1, 1, 1)]),
        |_| Ok(Findings::default()),
    )
    .expect("save again");
    assert_eq!(amended.batches.len(), 1);
    assert!(amended.batches[0].printed.is_empty());
    assert!(
        DayState::load(&day).expect("load").batches[0]
            .printed
            .is_empty()
    );
}

#[test]
fn next_numbers_continue_through_the_day() {
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

    assert_eq!(state.next_batch_number(), 2);
    assert_eq!(state.next_pdf_number(), 3);

    save_batch(
        &day,
        &mut state,
        &batch_write(2, vec![pdf_entry(3)], vec![saved("b", 2, 4, 2)]),
        |_| Ok(Findings::default()),
    )
    .expect("save batch 2");
    assert_eq!(state.next_batch_number(), 3);
    assert_eq!(state.next_pdf_number(), 5);
}

#[test]
fn fingerprint_is_the_size_and_xxh3_of_the_bytes() {
    let dir = TempDir::new("fingerprint");
    let path = dir.path().join("labels.pdf");
    fs::write(&path, b"abc").expect("write");

    let print = fingerprint(&path).expect("fingerprint");
    assert_eq!(print.size, 3);
    // The known XXH3-64 (seed 0) of "abc"; the streaming read must give it ...
    assert_eq!(print.xxh3, "78af5f94892f3950");
    // ... and the one-shot XXH3-64 of the same bytes, as 16 lowercase hex characters.
    assert_eq!(
        print.xxh3,
        format!("{:016x}", xxhash_rust::xxh3::xxh3_64(b"abc"))
    );

    assert!(matches!(
        fingerprint(&dir.path().join("missing")),
        Err(DayError::Read { .. })
    ));
}

#[test]
fn used_in_batch_finds_the_file_by_fingerprint() {
    let dir = TempDir::new("usedin");
    let day = dir.path().join("2026-10-07");
    let path = dir.path().join("labels.pdf");
    fs::write(&path, b"abc").expect("write");
    let print = fingerprint(&path).expect("fingerprint");

    let mut state = DayState::new("2026-10-07");
    save_batch(
        &day,
        &mut state,
        &batch_write(1, vec![pdf_entry(1)], vec![saved("a", 1, 1, 1)]),
        |_| Ok(Findings::default()),
    )
    .expect("save");
    // The batch stores only the short hash in the fixture; make the fingerprint match by hand.
    let stored = packing_engine::day::Fingerprint {
        size: 10,
        xxh3: "9f2c".to_string(),
    };
    assert_eq!(state.used_in_batch(&stored), Some(1));
    assert_eq!(state.used_in_batch(&print), None);
    assert_eq!(
        state.used_in_batch(&packing_engine::day::Fingerprint {
            size: 11,
            xxh3: "9f2c".to_string(),
        }),
        None
    );
}

#[test]
fn the_same_file_is_recognised_twice_and_a_changed_byte_is_not() {
    let dir = TempDir::new("twice");
    let day = dir.path().join("2026-10-07");

    // A made-up label file: 500 bytes, so it is not one of the fixtures.
    let path = day.join("download").join("labels_1.pdf");
    fs::create_dir_all(path.parent().expect("download")).expect("download");
    let bytes: Vec<u8> = (0..500u32).map(|i| (i % 251) as u8).collect();
    fs::write(&path, &bytes).expect("write");
    let print = fingerprint(&path).expect("fingerprint");

    let mut state = DayState::new("2026-10-07");
    state.batches.push(packing_engine::day::BatchEntry {
        batch: 1,
        save_time: "2026-10-07T07:40:12+07:00".to_string(),
        files: vec![FileEntry {
            name: "labels_1.pdf".to_string(),
            size: print.size,
            xxh3: print.xxh3.clone(),
            pages: 5,
            legacy_sha256: None,
        }],
        pdfs: vec![pdf_entry(1)],
        packing_list: PackingListChoice::default(),
        printed: Vec::new(),
        warnings: Vec::new(),
        errors: Vec::new(),
    });
    // The same file, again: recognised as used in batch 1.
    assert_eq!(state.used_in_batch(&print), Some(1));

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
fn the_streamed_hash_equals_xxh3_64_of_the_whole_made_up_file() {
    let dir = TempDir::new("stream");
    let path = dir.path().join("big.pdf");
    // Larger than the 64 KB read buffer, made up, so the file is hashed in several chunks.
    let bytes: Vec<u8> = (0..200_000u32)
        .map(|i| i.wrapping_mul(31).wrapping_add(7) as u8)
        .collect();
    fs::write(&path, &bytes).expect("write");

    let print = fingerprint(&path).expect("fingerprint");
    assert_eq!(print.size, bytes.len() as u64);
    // The one-shot XXH3-64 of the same bytes: the streaming read must give the same digest.
    assert_eq!(
        print.xxh3,
        format!("{:016x}", xxhash_rust::xxh3::xxh3_64(&bytes))
    );
    assert_eq!(print.xxh3.len(), 16);
    assert_eq!(print.xxh3, print.xxh3.to_lowercase());
}

#[test]
fn a_day_with_sha256_entries_loads_and_the_legacy_hash_is_never_matched() {
    let dir = TempDir::new("legacy");
    let day = dir.path().join("2026-10-07");
    fs::create_dir_all(&day).expect("create day");
    // A day written by the older code: the file entry carries `sha256`, no `xxh3`.
    let legacy = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";
    let json = format!(
        r#"{{
  "day": "2026-10-07",
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
    fs::write(day.join("state.json"), json).expect("write legacy state");

    let state = DayState::load(&day).expect("load legacy day");
    assert_eq!(state.batches.len(), 1);
    // The value is read into the legacy field, not the new one.
    assert_eq!(
        state.batches[0].files[0].legacy_sha256.as_deref(),
        Some(legacy)
    );
    assert!(state.batches[0].files[0].xxh3.is_empty());

    // The legacy hash is never matched, even by a fingerprint carrying the same value.
    let same_value = packing_engine::day::Fingerprint {
        size: 3,
        xxh3: legacy.to_string(),
    };
    assert_eq!(state.used_in_batch(&same_value), None);

    // Saving writes `xxh3` only: the legacy key does not come back.
    state.save(&day).expect("save");
    let text = fs::read_to_string(day.join("state.json")).expect("read");
    assert!(!text.contains("sha256"), "{text}");
    assert!(text.contains("\"xxh3\""), "{text}");
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
    assert_eq!(kept[0].order.order_id, "580000000000000002");
    assert_eq!(duplicates.len(), 1);
    let dup = &duplicates[0];
    assert_eq!(dup.order_id, "580000000000000001");
    assert_eq!(dup.tracking_id, "JY580000000000000001");
    assert_eq!(dup.batch, 1);
    assert_eq!(dup.pdf, 1);
    assert_eq!(dup.time, "2026-10-07T07:40:12+07:00");
    assert_eq!(
        duplicates_warning(&duplicates),
        "1 orders were already saved today and are left out: JY580000000000000001 (batch 1, PDF 1, 2026-10-07T07:40:12+07:00)"
    );
}

#[test]
fn revert_forgets_the_last_batch_and_reuses_its_numbers() {
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
    assert_eq!(removed.batch, 2);
    assert_eq!(state.batches.len(), 1);
    assert!(state.saved.iter().all(|order| order.batch != 2));
    assert_eq!(state.next_batch_number(), 2);
    assert_eq!(state.next_pdf_number(), 2);

    // No batch: revert is a no-op.
    state.revert_last();
    assert!(state.revert_last().is_none());
}

#[test]
fn delete_batch_folder_removes_the_folder_and_ignores_a_missing_one() {
    let dir = TempDir::new("delete");
    let day = dir.path().join("2026-10-07");
    let mut state = DayState::new("2026-10-07");
    save_batch(
        &day,
        &mut state,
        &batch_write(1, vec![pdf_entry(1)], vec![saved("a", 1, 1, 1)]),
        |_| Ok(Findings::default()),
    )
    .expect("save");

    assert!(day.join("batch 1").is_dir());
    delete_batch_folder(&day, 1).expect("delete");
    assert!(!day.join("batch 1").exists());
    delete_batch_folder(&day, 9).expect("missing is fine");
}

#[cfg(windows)]
#[test]
fn a_file_open_in_another_program_reports_file_in_use() {
    use std::fs::OpenOptions;
    use std::os::windows::fs::OpenOptionsExt;

    let dir = TempDir::new("inuse");
    let day = dir.path().join("2026-10-07");
    let mut state = DayState::new("2026-10-07");
    save_batch(
        &day,
        &mut state,
        &batch_write(1, vec![pdf_entry(1)], vec![saved("a", 1, 1, 1)]),
        |_| Ok(Findings::default()),
    )
    .expect("save");
    let file = day.join("batch 1").join("1 A Sepatu ×1.pdf");
    fs::write(&file, b"x").expect("write");

    // Open with no sharing, as a PDF viewer locks a file for writing.
    let _lock = OpenOptions::new()
        .read(true)
        .share_mode(0)
        .open(&file)
        .expect("lock");
    let error = delete_batch_folder(&day, 1).expect_err("locked folder cannot be deleted");
    assert!(matches!(error, DayError::FileInUse { .. }), "{error:?}");
}

#[test]
fn amend_gives_the_download_files_and_the_state_without_the_batch() {
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
    save_batch(
        &day,
        &mut state,
        &batch_write(2, vec![pdf_entry(2)], vec![saved("b", 2, 2, 1)]),
        |_| Ok(Findings::default()),
    )
    .expect("save 2");

    let amend = state.amend(&day).expect("amend");
    assert_eq!(amend.batch, 2);
    assert_eq!(
        amend.files,
        vec![day.join("batch 2").join(DOWNLOAD_DIR).join("labels_1.pdf")]
    );
    assert_eq!(amend.state.batches.len(), 1);
    assert_eq!(amend.state.next_batch_number(), 2);
    assert_eq!(amend.state.next_pdf_number(), 2);
    // The original state is untouched.
    assert_eq!(state.batches.len(), 2);
}

#[test]
fn save_writes_the_folder_and_state_and_leaves_no_partial() {
    let dir = TempDir::new("saveok");
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

    assert_eq!(final_dir, day.join("batch 1"));
    assert!(final_dir.join("1 A Sepatu ×1.pdf").is_file());
    assert!(!day.join("batch 1.partial").exists());
    assert!(day.join("state.json").is_file());
    assert_eq!(state.batches.len(), 1);
    assert_eq!(DayState::load(&day).expect("load"), state);
}

#[test]
fn save_replaces_an_amended_batch_folder() {
    let dir = TempDir::new("amend-save");
    let day = dir.path().join("2026-10-07");
    let mut state = DayState::new("2026-10-07");
    save_batch(
        &day,
        &mut state,
        &batch_write(1, vec![pdf_entry(1)], vec![saved("a", 1, 1, 1)]),
        |temp| {
            fs::write(temp.join("old.txt"), b"old").expect("old");
            Ok(Findings::default())
        },
    )
    .expect("save");
    // Save the same batch number again (amend): the old folder is replaced.
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
}

#[test]
fn save_rolls_back_when_the_writer_fails() {
    let dir = TempDir::new("rollback");
    let day = dir.path().join("2026-10-07");

    let mut state = DayState::new("2026-10-07");
    let error = save_batch(
        &day,
        &mut state,
        &batch_write(1, vec![pdf_entry(1)], vec![saved("a", 1, 1, 1)]),
        |temp| {
            // Write something, then fail: the temporary folder must be deleted.
            fs::write(temp.join("half.pdf"), b"%PDF").expect("half");
            Err(DayError::Write {
                folder: temp.to_path_buf(),
                message: "disk full".to_string(),
            })
        },
    )
    .expect_err("writer failed");

    assert!(matches!(error, DayError::Write { .. }));
    assert!(!day.join("batch 1").exists());
    assert!(!day.join("batch 1.partial").exists());
    assert!(!day.join("state.json").exists());
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
