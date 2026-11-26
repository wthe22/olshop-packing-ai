//! A date folder of v0.7 made flat: its old folders kept in `v0.7 backup/`, each file copied up under
//! its v0.8 name, and a stopped conversion finished by the next.

use std::fs;

use jiff::civil::date;
use packing_engine::ids::{BatchId, PdfId};
use packing_engine::record::RecordBody;
use packing_store::{
    DateFolder, Journal, V0_7_BACKUP, flatten_date, input_path, is_v0_7, migrations, sha256_of,
};
use testkit::TempDir;

use crate::common::{APP, batch, outputs};

#[test]
fn a_v0_7_date_folder_is_made_flat_and_its_old_files_kept() {
    let data = TempDir::new("flatten");
    let folder = DateFolder::new(data.path(), date(2026, 10, 7));
    let dir = folder.dir().to_path_buf();
    let (input_dir, output_dir) = (dir.join("input"), dir.join("output"));
    fs::create_dir_all(&input_dir).unwrap();
    fs::write(input_dir.join("labels_1.pdf"), b"label PDF").unwrap();
    fs::write(input_dir.join("other.pdf"), b"a download no batch read").unwrap();
    let label = sha256_of(&input_dir.join("labels_1.pdf")).unwrap();
    let mut first = batch(1, false, outputs(&output_dir, 1, "one"));
    first.inputs[0].sha256 = label.clone();
    let mut second = batch(2, false, outputs(&output_dir, 2, "two"));
    second.inputs[0].sha256 = label.clone();
    fs::write(
        output_dir.join("1 summary Toko Alfa.pdf"),
        b"account summary",
    )
    .unwrap();
    fs::write(dir.join("day summary.pdf"), b"day summary").unwrap();
    fs::create_dir_all(output_dir.join(".partial")).unwrap();
    fs::write(
        output_dir.join(".partial").join("1-A Sepatu x2.pdf"),
        b"stopped",
    )
    .unwrap();
    // Written at the top by hand after v0.7: other bytes than the journal names.
    fs::write(dir.join("2-A Sepatu x2.pdf"), b"not this").unwrap();
    let mut journal = Journal::open(&folder).unwrap();
    journal
        .commit(
            vec![
                RecordBody::BatchSaved(first.clone()),
                RecordBody::BatchSaved(second.clone()),
            ],
            APP,
        )
        .unwrap();
    let journal_bytes = fs::read(dir.join("journal.jsonl")).unwrap();
    assert!(is_v0_7(&dir));
    assert!(migrations(data.path()).unwrap()[0].v0_7, "the start asks");

    let mut seen = Vec::new();
    flatten_date(&folder, &journal, &mut |file, files| {
        seen.push((file, files))
    })
    .unwrap();

    for saved in [&first, &second] {
        let input = input_path(&folder, saved.batch, PdfId::new(1), &label);
        assert_eq!(
            fs::read(input).unwrap(),
            b"label PDF",
            "batch {}",
            saved.batch
        );
        for output in &saved.outputs {
            assert_eq!(sha256_of(&dir.join(&output.file)).unwrap(), output.sha256);
        }
    }
    assert_eq!(
        fs::read(dir.join("Day Summary.pdf")).unwrap(),
        b"day summary"
    );
    assert_eq!(
        fs::read(dir.join("1 Summary - Toko Alfa.pdf")).unwrap(),
        b"account summary"
    );
    assert_eq!(
        fs::read(dir.join(".partial").join("1-A Sepatu x2.pdf")).unwrap(),
        b"stopped"
    );
    let backup = dir.join(V0_7_BACKUP);
    assert_eq!(
        fs::read(backup.join("replaced").join("2-A Sepatu x2.pdf")).unwrap(),
        b"not this"
    );
    for kept in ["input/labels_1.pdf", "input/other.pdf", "day summary.pdf"] {
        assert!(backup.join(kept).is_file(), "{kept} is kept");
    }
    assert!(!input_dir.is_dir() && !output_dir.is_dir());
    assert!(!is_v0_7(&dir));
    // Two names of one label PDF, six outputs and two summaries.
    assert_eq!(seen.len(), 10);
    assert_eq!(seen.last(), Some(&(10, 10)));
    assert_eq!(fs::read(dir.join("journal.jsonl")).unwrap(), journal_bytes);
    assert!(
        migrations(data.path())
            .unwrap()
            .iter()
            .all(|found| !found.v0_7)
    );

    // A conversion that stopped before its end is finished by the next; what is in place is left.
    fs::remove_file(backup.join("backup complete.txt")).unwrap();
    fs::remove_file(dir.join("1-Z Lainnya x1.pdf")).unwrap();
    assert!(is_v0_7(&dir));
    flatten_date(&folder, &journal, &mut |_, _| {}).unwrap();
    assert_eq!(
        sha256_of(&dir.join("1-Z Lainnya x1.pdf")).unwrap(),
        first.outputs[1].sha256
    );
    assert!(!is_v0_7(&dir));
    assert_eq!(
        sha256_of(&input_path(&folder, BatchId::new(1), PdfId::new(1), &label)).unwrap(),
        label
    );
}
