//! A date folder of v0.7 made flat: each file moved up under its v0.8 name and listed in the
//! Update's `update.toml`, not copied; a conversion of v0.8.8 that stopped is finished from its backup.

use std::fs;

use jiff::civil::date;
use packing_engine::ids::{BatchId, PdfId};
use packing_engine::record::{RecordBody, Sha256};
use packing_store::{
    DateFolder, Journal, UPDATE_LOG, UpdateLog, UpdateStep, V0_7_BACKUP, flatten_date, input_path,
    is_v0_7, migrations, sha256_of,
};
use testkit::TempDir;

use crate::common::{APP, batch, outputs};

#[test]
fn a_v0_7_date_folder_is_made_flat_by_moves_each_listed() {
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

    let mut log = UpdateLog::open(&dir, APP).unwrap();
    let mut seen = Vec::new();
    flatten_date(&folder, &journal, &mut log, &mut |file, files| {
        seen.push((file, files))
    })
    .unwrap();

    let names: Vec<_> = [&first, &second]
        .map(|saved| input_path(&folder, saved.batch, PdfId::new(1), &label))
        .to_vec();
    for name in &names {
        assert_eq!(fs::read(name).unwrap(), b"label PDF");
    }
    for saved in [&first, &second] {
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
    let update = dir.join(format!("update v{APP}"));
    assert_eq!(
        fs::read(update.join("replaced").join("2-A Sepatu x2.pdf")).unwrap(),
        b"not this"
    );
    assert!(
        !input_dir.join("labels_1.pdf").exists(),
        "moved, not copied"
    );
    assert!(input_dir.join("other.pdf").is_file(), "no journal names it");
    assert!(!output_dir.exists(), "emptied, so removed");
    let names_there: Vec<String> = fs::read_dir(&dir)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert!(
        names_there.iter().any(|name| name == "Day Summary.pdf")
            && !names_there.iter().any(|name| name == "day summary.pdf"),
        "renamed even where the case is all that differs: {names_there:?}"
    );
    assert!(!dir.join(V0_7_BACKUP).exists());
    assert!(!is_v0_7(&dir), "update.toml lists it flat");

    assert_moves_listed(&log, &names, &label);
    let listed = fs::read_to_string(update.join(UPDATE_LOG)).unwrap();
    assert!(listed.contains("flattened = true"), "{listed}");
    assert!(listed.contains("from = \"input/labels_1.pdf\""), "{listed}");
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
    // Undone by the owner's script, the date is of v0.7 again.
    fs::rename(&update, dir.join(format!("update v{APP} reverted"))).unwrap();
    assert!(is_v0_7(&dir));
}

/// The moves of the first test, each listed; a copy only for the label PDF's second name.
fn assert_moves_listed(log: &UpdateLog, names: &[std::path::PathBuf], label: &Sha256) {
    let relative =
        |path: &std::path::Path| path.file_name().unwrap().to_string_lossy().into_owned();
    let steps = log.steps();
    assert!(steps.contains(&UpdateStep::Moved {
        from: "input/labels_1.pdf".into(),
        to: relative(&names[0]),
    }));
    assert!(steps.contains(&UpdateStep::Copied {
        from: relative(&names[0]),
        to: relative(&names[1]),
        sha256: label.clone(),
    }));
    assert!(steps.contains(&UpdateStep::Moved {
        from: "2-A Sepatu x2.pdf".into(),
        to: format!("update v{APP}/replaced/2-A Sepatu x2.pdf"),
    }));
    assert!(steps.contains(&UpdateStep::Moved {
        from: "output/.partial".into(),
        to: ".partial".into(),
    }));
    let copies = steps
        .iter()
        .filter(|step| matches!(step, UpdateStep::Copied { .. }))
        .count();
    assert_eq!(copies, 1, "only a second name is a copy");
}

#[test]
fn a_conversion_of_v0_8_8_that_stopped_is_finished_from_its_backup() {
    let data = TempDir::new("flatten-stopped");
    let folder = DateFolder::new(data.path(), date(2026, 10, 8));
    let dir = folder.dir().to_path_buf();
    let backup = dir.join(V0_7_BACKUP);
    let output_dir = backup.join("output");
    fs::create_dir_all(backup.join("input")).unwrap();
    fs::write(backup.join("input").join("labels_1.pdf"), b"label PDF").unwrap();
    let label = sha256_of(&backup.join("input").join("labels_1.pdf")).unwrap();
    let mut first = batch(1, false, outputs(&output_dir, 1, "one"));
    first.inputs[0].sha256 = label.clone();
    // The stopped conversion had copied this one up already.
    let done = &first.outputs[0].file;
    fs::copy(output_dir.join(done), dir.join(done)).unwrap();
    let mut journal = Journal::open(&folder).unwrap();
    journal
        .commit(vec![RecordBody::BatchSaved(first.clone())], APP)
        .unwrap();
    assert!(is_v0_7(&dir));

    let mut log = UpdateLog::open(&dir, APP).unwrap();
    flatten_date(&folder, &journal, &mut log, &mut |_, _| {}).unwrap();

    let input = input_path(&folder, BatchId::new(1), PdfId::new(1), &label);
    assert_eq!(sha256_of(&input).unwrap(), label);
    for output in &first.outputs {
        assert_eq!(sha256_of(&dir.join(&output.file)).unwrap(), output.sha256);
    }
    assert!(output_dir.join(done).is_file(), "in place already: left");
    assert!(!is_v0_7(&dir));
}
