//! End-to-end test of `packing-cli prepare/revert/amend` on the made-up testdata: the batch
//! folder, its saved PDFs, packing list, summary, download copy and `state.json`, the
//! duplicate guard, revert, amend and the no-slip stop (07 › *What happens in one batch*).
//!
//! Skipped with a message when `pdfium.dll` has not been downloaded (pc/tools/get-pdfium.sh).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

use packing_engine::day::DayState;
use packing_engine::names;
use serde_json::Value;

const DAY: &str = "2026-10-07";

/// The pinned `pdfium.dll` directory, or `None` when it has not been downloaded.
fn vendor_dir() -> Option<PathBuf> {
    let vendor = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../vendor");
    vendor.join("pdfium.dll").exists().then_some(vendor)
}

fn testdata(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../testdata")
        .join(name)
}

/// A unique folder under the system temp dir, removed when the test ends.
struct TempDir(PathBuf);

impl TempDir {
    fn new(tag: &str) -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("packing-cli-{tag}-{}-{nanos}", std::process::id()));
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

/// Run the built binary; `argv` is everything after the binary path.
fn run(argv: &[&str], vendor: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_packing-cli"))
        .args(argv)
        .arg("--pdfium")
        .arg(vendor)
        .output()
        .expect("run packing-cli")
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

/// The expected saved-PDF file names from `expected-picks.json` (07 › *Plan*).
fn expected_file_names() -> Vec<String> {
    let text = fs::read_to_string(testdata("expected-picks.json")).expect("expected-picks");
    let picks: Vec<Value> = serde_json::from_str(&text).expect("parse expected-picks");
    picks
        .iter()
        .map(|pick| {
            names::suggested_saved_pdf_name(
                pick["number"].as_i64().unwrap(),
                pick["code"].as_str().unwrap(),
                pick["name"].as_str().unwrap(),
                pick["orders"].as_u64().unwrap() as usize,
            )
        })
        .collect()
}

/// The saved PDFs of a batch folder (everything but the packing list).
fn pdf_files(batch: &Path) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = fs::read_dir(batch)
        .expect("read batch folder")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension().is_some_and(|e| e == "pdf")
                && !path
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .starts_with("packing-list")
        })
        .collect();
    files.sort();
    files
}

/// The total pages of the given PDFs, using a page-counting closure (`pdfium_render` is not a
/// direct dependency of this crate, so the type never has to be named here).
fn total_pages(count: &dyn Fn(&Path) -> usize, files: &[PathBuf]) -> usize {
    files.iter().map(|file| count(file)).sum()
}

fn assert_batch_one(data: &Path, count: &dyn Fn(&Path) -> usize) -> Vec<String> {
    let batch = data.join("labels").join(DAY).join("batch 1");
    assert!(batch.is_dir(), "batch 1 folder must exist");

    let names: Vec<String> = pdf_files(&batch)
        .iter()
        .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
        .collect();
    let mut expected = expected_file_names();
    expected.sort();
    let mut got = names.clone();
    got.sort();
    assert_eq!(got, expected, "saved-PDF names match expected-picks.json");

    let pages = total_pages(count, &pdf_files(&batch));
    assert_eq!(pages, 24, "the saved PDFs hold every label page");

    assert!(batch.join("packing-list.pdf").is_file(), "packing list");
    assert!(batch.join("summary.txt").is_file(), "summary");
    assert!(
        batch.join("download").join("labels-slip.pdf").is_file(),
        "download copy"
    );

    let state = DayState::load(&data.join("labels").join(DAY)).expect("state");
    assert_eq!(state.batches.len(), 1);
    assert_eq!(state.saved.len(), 23);
    assert_eq!(state.batches[0].batch, 1);
    assert_eq!(state.batches[0].pdfs.len(), expected.len());

    names
}

#[test]
fn prepare_revert_amend_round_trip() {
    let Some(vendor) = vendor_dir() else {
        eprintln!("skipped: run pc/tools/get-pdfium.sh");
        return;
    };
    if !Path::new(r"C:\Windows\Fonts\arial.ttf").exists() {
        eprintln!("skipped: Arial font not installed");
        return;
    }
    let Ok(pdfium) = packing_pdf::bind(&vendor) else {
        eprintln!("skipped: pdfium.dll could not be bound");
        return;
    };
    let cancel = std::sync::atomic::AtomicBool::new(false);
    let count = |file: &Path| {
        packing_pdf::read::read_all(&pdfium, &[file.to_path_buf()], &cancel, &mut |_, _| {})
            .expect("read saved PDF")
            .len()
    };

    let data = TempDir::new("roundtrip");
    fs::copy(
        testdata("categories.toml"),
        data.path().join("categories.toml"),
    )
    .expect("copy categories");
    let labels = testdata("labels-slip.pdf");
    let labels = labels.to_str().expect("label path");

    // 1. First prepare writes batch 1.
    let output = run(
        &[
            "prepare",
            "--labels",
            labels,
            "--data",
            data.path().to_str().unwrap(),
            "--day",
            DAY,
        ],
        &vendor,
    );
    assert!(
        output.status.success(),
        "prepare must succeed: {}",
        stderr(&output)
    );
    let first = assert_batch_one(data.path(), &count);

    // 2. A second prepare of the same file stops: every order is already saved.
    let output = run(
        &[
            "prepare",
            "--labels",
            labels,
            "--data",
            data.path().to_str().unwrap(),
            "--day",
            DAY,
        ],
        &vendor,
    );
    assert!(!output.status.success(), "second prepare must stop");
    assert!(
        stderr(&output).contains("already saved today"),
        "got: {}",
        stderr(&output)
    );
    assert!(
        !data
            .path()
            .join("labels")
            .join(DAY)
            .join("batch 2")
            .exists(),
        "nothing new is written"
    );
    assert_eq!(
        DayState::load(&data.path().join("labels").join(DAY))
            .expect("state")
            .batches
            .len(),
        1
    );

    // 3. Revert removes batch 1 and its state entries.
    let output = run(
        &[
            "revert",
            "--data",
            data.path().to_str().unwrap(),
            "--day",
            DAY,
        ],
        &vendor,
    );
    assert!(output.status.success(), "revert: {}", stderr(&output));
    assert!(
        !data
            .path()
            .join("labels")
            .join(DAY)
            .join("batch 1")
            .exists()
    );
    let state = DayState::load(&data.path().join("labels").join(DAY)).expect("state");
    assert!(state.batches.is_empty());
    assert!(state.saved.is_empty());

    // 4. Prepare again gives the same files and page counts.
    let output = run(
        &[
            "prepare",
            "--labels",
            labels,
            "--data",
            data.path().to_str().unwrap(),
            "--day",
            DAY,
        ],
        &vendor,
    );
    assert!(
        output.status.success(),
        "prepare again: {}",
        stderr(&output)
    );
    let again = assert_batch_one(data.path(), &count);
    assert_eq!(again, first, "the same file names come back");

    // 5. Amend re-saves batch 1 from its download copy.
    let output = run(
        &[
            "amend",
            "--data",
            data.path().to_str().unwrap(),
            "--day",
            DAY,
        ],
        &vendor,
    );
    assert!(output.status.success(), "amend: {}", stderr(&output));
    let amended = assert_batch_one(data.path(), &count);
    assert_eq!(amended, first);

    // 6. Plain labels stop with the no-slip message and write nothing.
    let plain = testdata("labels-plain.pdf");
    let plain = plain.to_str().expect("plain path");
    let output = run(
        &[
            "prepare",
            "--labels",
            plain,
            "--data",
            data.path().to_str().unwrap(),
            "--day",
            DAY,
        ],
        &vendor,
    );
    assert!(!output.status.success(), "plain labels must stop");
    assert!(
        stderr(&output).contains("no packing slip"),
        "got: {}",
        stderr(&output)
    );
    assert!(
        !data
            .path()
            .join("labels")
            .join(DAY)
            .join("batch 2")
            .exists(),
        "plain labels write nothing"
    );
}
