//! `day.rs`: `state.json` round trip and format, numbering, the duplicate guard, file
//! fingerprints, revert, amend and the atomic save (07 › *Day folder*, *`state.json`*,
//! *Revert and amend*; mirrors script/tests/test_picks.py).

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use packing_engine::day::{
    BatchWrite, DayError, DayState, FileEntry, Layout, PackingListChoice, PdfEntry, SavedOrder,
    Scope, delete_batch_folder, duplicates_warning, fingerprint, now_save_time, save_batch,
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

fn file_entry(name: &str, sha: &str) -> FileEntry {
    FileEntry {
        name: name.to_string(),
        size: 10,
        sha256: sha.to_string(),
        pages: 5,
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
    }
}

fn batch_write(batch: i64, pdfs: Vec<PdfEntry>, saved_orders: Vec<SavedOrder>) -> BatchWrite {
    BatchWrite {
        batch,
        save_time: "2026-10-07T07:40:12+07:00".to_string(),
        files: vec![file_entry("labels_1.pdf", "9f2c")],
        pdfs,
        packing_list: PackingListChoice::default(),
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
    assert_eq!(batch["files"][0]["sha256"], "9f2c");
    assert_eq!(batch["files"][0]["pages"], 5);
    assert_eq!(batch["pdfs"][0]["number"], 1);
    assert_eq!(batch["pdfs"][0]["code"], "A");
    assert_eq!(batch["pdfs"][0]["name"], "Sepatu");
    assert_eq!(batch["pdfs"][0]["file"], "1 A Sepatu ×1.pdf");
    assert_eq!(batch["pdfs"][0]["orders"], 1);
    assert_eq!(batch["pdfs"][0]["runs"], 1);
    assert_eq!(batch["packing_list"]["scope"], "whole");
    assert_eq!(batch["packing_list"]["layouts"][0], "pick");

    let entry = &value["saved"][0];
    assert_eq!(entry["order_id"], "580000000000000001");
    assert_eq!(entry["tracking_id"], "JY580000000000000001");
    assert_eq!(entry["batch"], 1);
    assert_eq!(entry["pdf"], 1);
    assert_eq!(entry["run"], 5);
    // The 07 example keeps no per-order save_time (the batch carries it).
    assert!(entry.get("save_time").is_none());
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
        |_| Ok(()),
    )
    .expect("save batch 1");

    assert_eq!(state.next_batch_number(), 2);
    assert_eq!(state.next_pdf_number(), 3);

    save_batch(
        &day,
        &mut state,
        &batch_write(2, vec![pdf_entry(3)], vec![saved("b", 2, 4, 2)]),
        |_| Ok(()),
    )
    .expect("save batch 2");
    assert_eq!(state.next_batch_number(), 3);
    assert_eq!(state.next_pdf_number(), 5);
}

#[test]
fn fingerprint_is_the_size_and_sha256_of_the_bytes() {
    let dir = TempDir::new("fingerprint");
    let path = dir.path().join("labels.pdf");
    fs::write(&path, b"abc").expect("write");

    let print = fingerprint(&path).expect("fingerprint");
    assert_eq!(print.size, 3);
    // sha256("abc") — the streaming read must give the standard digest.
    assert_eq!(
        print.sha256,
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
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
        |_| Ok(()),
    )
    .expect("save");
    // The batch stores only the short sha in the fixture; make the fingerprint match by hand.
    let stored = packing_engine::day::Fingerprint {
        size: 10,
        sha256: "9f2c".to_string(),
    };
    assert_eq!(state.used_in_batch(&stored), Some(1));
    assert_eq!(state.used_in_batch(&print), None);
    assert_eq!(
        state.used_in_batch(&packing_engine::day::Fingerprint {
            size: 11,
            sha256: "9f2c".to_string(),
        }),
        None
    );
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
        |_| Ok(()),
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
        |_| Ok(()),
    )
    .expect("save 1");
    save_batch(
        &day,
        &mut state,
        &batch_write(2, vec![pdf_entry(2)], vec![saved("b", 2, 2, 1)]),
        |_| Ok(()),
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
        |_| Ok(()),
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
        |_| Ok(()),
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
        |_| Ok(()),
    )
    .expect("save 1");
    save_batch(
        &day,
        &mut state,
        &batch_write(2, vec![pdf_entry(2)], vec![saved("b", 2, 2, 1)]),
        |_| Ok(()),
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
            Ok(())
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
            Ok(())
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
            Ok(())
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
