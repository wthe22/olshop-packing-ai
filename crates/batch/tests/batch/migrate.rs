//! The Update (the migration the owner starts): a batch saved by v0.4 gets its area, weight, shop
//! and the courier it lacked from its stored label PDFs; a v0.7 date is made flat.

use super::*;

/// The batch as a version that read neither the area nor the weight saved it, on `day` of the same data folder,
/// with its label PDFs as `inputs` gives them.
fn saved_before_reading(
    day: &Day,
    on: i8,
    record: &BatchSaved,
    inputs: impl Fn(&PdfId) -> Vec<u8>,
) -> (DateFolder, Journal) {
    let folder = DateFolder::new(day.folder.dir().parent().unwrap(), date(2026, 10, on));
    fs::create_dir_all(folder.dir()).unwrap();
    for input in &record.inputs {
        let path = input_path(&folder, record.batch, input.pdf, &input.sha256);
        fs::write(path, inputs(&input.pdf)).unwrap();
    }
    // As v0.4 saved it: no area, weight or shop but the account, J&T Express as `jnt`; and the
    // first order without its courier, the second without its tracking ID.
    let mut before = record.clone();
    for order in &mut before.orders {
        order.area = None;
        order.weight = None;
        order.shop = None;
        order.account_of_2_0 = Some("An account".to_string());
        if order
            .courier
            .as_ref()
            .is_some_and(|code| code.as_str() == "jnt-express")
        {
            order.courier = Some(CourierCode::new("jnt"));
        }
    }
    before.orders[0].courier = None;
    before.orders[0].courier_clue = CourierClue {
        kind: CourierClueKind::None,
        value: None,
        form: None,
    };
    before.orders[1].tracking_id = None;
    let mut journal = Journal::open(&folder).unwrap();
    journal
        .commit(vec![RecordBody::BatchSaved(before)], APP)
        .unwrap();
    (folder, journal)
}

/// The date folder as v0.7 kept it: the label PDFs in `input/` under the names they were chosen
/// by, the outputs of `day` in `output/`.
fn as_v0_7(day: &Day, folder: &DateFolder, record: &BatchSaved) {
    let (input, output) = (folder.dir().join("input"), folder.dir().join("output"));
    fs::create_dir_all(&input).unwrap();
    fs::create_dir_all(&output).unwrap();
    for pdf in &record.inputs {
        let stored = input_path(folder, record.batch, pdf.pdf, &pdf.sha256);
        fs::rename(stored, input.join(&pdf.filename)).unwrap();
    }
    for file in &record.outputs {
        fs::copy(day.folder.dir().join(&file.file), output.join(&file.file)).unwrap();
    }
}

#[test]
fn a_batch_saved_before_the_area_and_weight_were_read_gets_them_from_its_stored_label_pdfs() {
    let Some(mut day) = day("batch-migrate") else {
        return skipped();
    };
    let draft = day.read_inputs().unwrap();
    let record = saved(day.save(&draft, false)).record;
    let stored = |pdf: &PdfId| {
        let input = record
            .inputs
            .iter()
            .find(|input| input.pdf == *pdf)
            .unwrap();
        fs::read(input_path(&day.folder, record.batch, *pdf, &input.sha256)).unwrap()
    };
    let (progress, seen) = mpsc::channel();
    let request = MigrateRequest {
        worker: &day.worker,
        workflows: &day.workflows,
        app: APP,
        progress,
    };

    let (older, mut journal) = saved_before_reading(&day, 9, &record, stored);
    let journal_before = fs::read(older.dir().join("journal.jsonl")).unwrap();
    assert!(
        migrate_date(&older, &mut journal, &request)
            .unwrap()
            .skipped
            .is_empty()
    );
    assert_eq!(
        journal.fold().live[&record.batch].saved.orders,
        record.orders,
        "each order has the area, the weight, the shop and the courier the save read"
    );
    let pages: Vec<usize> = seen
        .try_iter()
        .map(|step| match step {
            MigrateProgress::Reading { page, .. } => page,
            moving => panic!("{moving:?}"),
        })
        .collect();
    assert_eq!(pages, (1..=record.orders.len()).collect::<Vec<_>>());
    let data_dir = day.folder.dir().parent().unwrap();
    assert!(
        migrations(data_dir).unwrap().is_empty(),
        "nothing is left to do"
    );
    let area = record.orders[0].area.clone().unwrap();
    let csv = fs::read_to_string(older.dir().join("orders.csv")).unwrap();
    assert!(csv.contains(&area.province.unwrap()), "the views show it");
    let update = older.dir().join(format!("update v{APP}"));
    assert_eq!(
        fs::read(update.join("journal.jsonl")).unwrap(),
        journal_before,
        "the journal is copied before it changes"
    );
    let listed = fs::read_to_string(update.join(UPDATE_LOG)).unwrap();
    assert!(listed.contains("kind = \"kept\"") && listed.contains("file = \"day.toml\""));
    let after = sha256_of(&older.dir().join("journal.jsonl")).unwrap();
    assert!(listed.contains(after.as_str()), "{listed}");

    let (changed, mut journal) = saved_before_reading(&day, 10, &record, |_| b"changed".to_vec());
    let last_seq = journal.fold().last_seq;
    let migrated = migrate_date(&changed, &mut journal, &request).unwrap();
    let skipped = migrated.skipped;
    assert_eq!(migrated.read, 0);
    assert_eq!(skipped.len(), 1);
    assert!(matches!(skipped[0].error, BatchError::InputChanged { .. }));
    assert_eq!(
        journal.fold().last_seq,
        last_seq,
        "a batch it cannot read is left as it was"
    );
    assert!(changed.dir().join("day.toml").is_file());

    let (flat, mut journal) = saved_before_reading(&day, 11, &record, stored);
    as_v0_7(&day, &flat, &record);
    let found = migrations(data_dir).unwrap();
    assert!(
        found
            .iter()
            .any(|date| date.v0_7 && date.batches == [record.batch]),
        "{found:?}"
    );
    assert_eq!(migrate_date(&flat, &mut journal, &request).unwrap().read, 1);
    let moving = seen
        .try_iter()
        .filter(|step| matches!(step, MigrateProgress::Moving { .. }))
        .count();
    assert_eq!(moving, record.inputs.len() + record.outputs.len());
    assert_eq!(
        journal.fold().live[&record.batch].saved.orders,
        record.orders,
        "the label PDFs moved up are read"
    );
    for output in &record.outputs {
        assert!(flat.dir().join(&output.file).is_file(), "{}", output.file);
    }
    assert!(
        !flat.dir().join(V0_7_BACKUP).exists(),
        "no copy of the label PDFs"
    );
    assert!(!flat.dir().join("input").exists() && !flat.dir().join("output").exists());
    let listed = fs::read_to_string(flat.dir().join(format!("update v{APP}")).join(UPDATE_LOG));
    assert!(listed.unwrap().contains("kind = \"moved\""));
}
