//! Made-up batches and a temporary data folder for the store's tests.

use std::fs;
use std::path::Path;

use jiff::civil::date;
use packing_engine::record::{OutputFile, Record, RecordBody, SCHEMA, Started};
use packing_store::{DateFolder, encode_line, sha256_of};
use testkit::TempDir;

pub use testkit::{batch, sha, time};

pub const WORKSPACE: &str = "Toko Contoh";
pub const APP: &str = "0.2.0";

/// A fresh data folder and the date 2026-10-07 of the workspace `Toko Contoh` in it.
pub fn date_folder(tag: &str) -> (TempDir, DateFolder) {
    let data = TempDir::new(tag);
    let folder = DateFolder::new(data.path(), WORKSPACE, date(2026, 10, 7));
    (data, folder)
}

/// Write `bytes` as `output/<file>` (or into `dir` when given) and describe it as the record does.
pub fn output_file(dir: &Path, file: &str, group: Option<&str>, bytes: &[u8]) -> OutputFile {
    fs::create_dir_all(dir).unwrap();
    let path = dir.join(file);
    fs::write(&path, bytes).unwrap();
    OutputFile {
        file: file.to_string(),
        group: group.map(str::to_string),
        pages: 1,
        size: bytes.len() as u64,
        sha256: sha256_of(&path).unwrap(),
    }
}

/// The outputs of a batch written into `dir`: group A, group Z and the summary.
pub fn outputs(dir: &Path, number: u8, tag: &str) -> Vec<OutputFile> {
    vec![
        output_file(
            dir,
            &format!("{number}-A Sepatu x2.pdf"),
            Some("A"),
            format!("A {tag}").as_bytes(),
        ),
        output_file(
            dir,
            &format!("{number}-Z Lainnya x1.pdf"),
            Some("Z"),
            format!("Z {tag}").as_bytes(),
        ),
        output_file(
            dir,
            &format!("{number} summary.pdf"),
            None,
            format!("S {tag}").as_bytes(),
        ),
    ]
}

/// A record as a line would hold it, at a fixed time.
pub fn record(seq: u32, body: RecordBody) -> Record {
    Record {
        seq,
        time: time("2026-10-07T09:12:00+07:00"),
        app: APP.to_string(),
        body,
    }
}

pub fn started() -> RecordBody {
    RecordBody::Started(Started {
        schema: SCHEMA.to_string(),
        workspace: WORKSPACE.to_string(),
        date: date(2026, 10, 7),
    })
}

/// Write `records` as the date's journal, each line sealed, with `tail` after them.
pub fn write_journal(folder: &DateFolder, records: &[Record], tail: &[u8]) {
    fs::create_dir_all(folder.dir()).unwrap();
    let mut bytes: Vec<u8> = records.iter().flat_map(encode_line).collect();
    bytes.extend_from_slice(tail);
    fs::write(folder.dir().join("journal.jsonl"), bytes).unwrap();
}

pub fn journal_text(folder: &DateFolder) -> String {
    fs::read_to_string(folder.dir().join("journal.jsonl")).unwrap()
}
