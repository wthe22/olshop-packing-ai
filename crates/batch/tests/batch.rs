//! The batch steps on the repeat fixture (an order printed twice in a row, an order in both
//! inputs): read → plan → save → verify → reopen, a second read of the same files, revert, the
//! Printed tick, amend, and a finding that stops the save until *Save anyway*.
//!
//! Skipped with a message when `pdfium.dll` has not been downloaded (tools/get-pdfium.sh).

use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc;

use jiff::civil::date;
use packing_batch::{
    BatchError, Draft, ReadRequest, SaveOutcome, SaveRequest, Saved, amend, check_files,
    read_draft, revert, save, set_printed,
};
use packing_engine::ids::BatchId;
use packing_engine::record::PrintedMarked;
use packing_engine::workflows::{BUNDLED_WORKFLOWS, SortRule, Workflows, load_workflows};
use packing_pdf::worker::PdfWorker;
use packing_store::{DateFolder, Journal, Settled, input_path, open_date, partial_dir};
use testkit::{PdfiumGuard, TempDir, testdata, vendor_dir};

const INPUTS: [&str; 2] = ["labels-repeat-1.pdf", "labels-repeat-2.pdf"];
const APP: &str = "0.2.0";
const GROUP_FILE: &str = "1-Z Lainnya x4.pdf";

/// A date of a fresh data folder, opened, with the worker and the bundled rules.
struct Day {
    folder: DateFolder,
    journal: Journal,
    workflows: Workflows,
    worker: PdfWorker,
    _guard: PdfiumGuard,
    _data: TempDir,
}

fn day(tag: &str) -> Option<Day> {
    let vendor = vendor_dir()?;
    let guard = testkit::pdfium_lock();
    let data = TempDir::new(tag);
    let folder = DateFolder::new(data.path(), "Toko Contoh", date(2026, 10, 8));
    let journal = open_date(&folder).unwrap().journal;
    Some(Day {
        journal,
        folder,
        workflows: load_workflows(BUNDLED_WORKFLOWS).unwrap(),
        worker: PdfWorker::spawn(vendor).unwrap(),
        _guard: guard,
        _data: data,
    })
}

impl Day {
    fn read(
        &self,
        paths: &[PathBuf],
        rule: &SortRule,
        redo: Option<BatchId>,
    ) -> Result<Draft, BatchError> {
        let files = check_files(&self.worker, paths, self.journal.fold())?;
        let (progress, _) = mpsc::channel();
        read_draft(ReadRequest {
            worker: &self.worker,
            files,
            workflows: &self.workflows,
            rule,
            fold: self.journal.fold(),
            redo,
            cancel: Arc::new(AtomicBool::new(false)),
            progress,
        })
    }

    fn read_inputs(&self) -> Result<Draft, BatchError> {
        let paths: Vec<PathBuf> = INPUTS.iter().map(|name| testdata(name)).collect();
        let rule = self.workflows.sort_rule("default").unwrap().clone();
        self.read(&paths, &rule, None)
    }

    fn save(&mut self, draft: &Draft, save_anyway: bool) -> SaveOutcome {
        save(
            &SaveRequest {
                worker: &self.worker,
                folder: &self.folder,
                draft,
                save_anyway,
                app: APP,
                progress: &|_| {},
            },
            &mut self.journal,
        )
        .unwrap()
    }

    fn output(&self, file: &str) -> PathBuf {
        self.folder.dir().join("output").join(file)
    }
}

fn saved(outcome: SaveOutcome) -> Saved {
    match outcome {
        SaveOutcome::Saved(saved) => saved,
        SaveOutcome::Findings(findings) => panic!("the verifier found {findings:?}"),
    }
}

fn skipped() {
    eprintln!("skipped: pdfium.dll not downloaded (tools/get-pdfium.sh)");
}

#[test]
fn a_batch_is_saved_through_the_journal_and_opens_again() {
    let Some(mut day) = day("batch-save") else {
        return skipped();
    };
    let draft = day.read_inputs().unwrap();
    let saved = saved(day.save(&draft, false));
    assert!(saved.after_commit.is_empty(), "{:?}", saved.after_commit);
    let record = saved.record;
    assert_eq!((record.batch, record.amends), (BatchId::new(1), false));
    assert_eq!((record.orders.len(), record.left_out.len()), (4, 2));
    let files: Vec<&str> = record.outputs.iter().map(|o| o.file.as_str()).collect();
    assert_eq!(files, [GROUP_FILE, "1 summary.pdf"]);
    for output in &record.outputs {
        assert!(
            day.output(&output.file).is_file(),
            "{} moved in",
            output.file
        );
    }
    assert!(!partial_dir(day.folder.dir()).exists());
    for input in &record.inputs {
        assert!(input_path(day.folder.dir(), &input.sha256).is_file());
    }

    let opened = open_date(&day.folder).unwrap();
    let fold = opened.journal.fold();
    assert_eq!(fold.last_seq, 2, "started, then the batch");
    assert_eq!(fold.live[&BatchId::new(1)].saved, record);
    assert!(opened.inputs.is_empty() && opened.outputs.is_empty());
    assert_eq!(opened.settled, Settled::Nothing);
    assert!(!opened.views_written, "the save wrote the views");
}

#[test]
fn saved_orders_are_left_out_until_the_batch_is_reverted() {
    let Some(mut day) = day("batch-revert") else {
        return skipped();
    };
    let draft = day.read_inputs().unwrap();
    let first = saved(day.save(&draft, false)).record;
    match day.read_inputs() {
        Err(BatchError::AllSavedEarlier { orders }) => assert_eq!(orders, 4),
        other => panic!("a second read of the same files: {other:?}"),
    }

    assert_eq!(
        revert(&day.folder, &mut day.journal, APP).unwrap(),
        BatchId::new(1)
    );
    assert!(day.journal.fold().live.is_empty());
    assert!(!day.output(GROUP_FILE).exists() && !day.output("1 summary.pdf").exists());
    assert!(!partial_dir(day.folder.dir()).exists());
    assert!(matches!(
        revert(&day.folder, &mut day.journal, APP),
        Err(BatchError::NoBatch)
    ));

    let draft = day.read_inputs().unwrap();
    let again = saved(day.save(&draft, false)).record;
    assert_eq!(again.batch, BatchId::new(1), "the number is free again");

    let redo = amend(&day.folder, day.journal.fold(), &day.workflows).unwrap();
    let paths: Vec<PathBuf> = redo.files.iter().map(|f| f.path.clone()).collect();
    let stale = day.read(&paths, &redo.rule, Some(redo.batch)).unwrap();
    revert(&day.folder, &mut day.journal, APP).unwrap();
    let refused = save(
        &SaveRequest {
            worker: &day.worker,
            folder: &day.folder,
            draft: &stale,
            save_anyway: false,
            app: APP,
            progress: &|_| {},
        },
        &mut day.journal,
    );
    assert!(
        matches!(refused, Err(BatchError::NotNewest)),
        "an amend of a batch reverted meanwhile: {refused:?}"
    );
    let numbers = |record: &packing_engine::record::BatchSaved| -> Vec<_> {
        record.inputs.iter().map(|input| input.pdf).collect()
    };
    assert_eq!(
        numbers(&again),
        numbers(&first),
        "an input keeps its number"
    );
}

#[test]
fn an_amend_saves_the_newest_batch_again_under_its_number() {
    let Some(mut day) = day("batch-amend") else {
        return skipped();
    };
    let draft = day.read_inputs().unwrap();
    let first = saved(day.save(&draft, false)).record;
    let mark = PrintedMarked {
        batch: first.batch,
        group: "Z".into(),
        printed: true,
    };
    set_printed(&day.folder, &mut day.journal, mark, APP).unwrap();
    assert!(day.journal.fold().live[&first.batch].printed.contains("Z"));

    let amend = amend(&day.folder, day.journal.fold(), &day.workflows).unwrap();
    assert_eq!(amend.batch, first.batch);
    let paths: Vec<PathBuf> = amend.files.iter().map(|f| f.path.clone()).collect();
    assert!(paths.iter().all(|path| path.starts_with(day.folder.dir())));
    let mut draft = day.read(&paths, &amend.rule, Some(amend.batch)).unwrap();
    draft.keep_as_recorded(&first).unwrap();
    let again = saved(day.save(&draft, false)).record;

    assert_eq!((again.batch, again.amends), (first.batch, true));
    assert_eq!(again.orders, first.orders, "the same copies kept");
    assert_eq!(again.left_out, first.left_out);
    let live = &day.journal.fold().live[&first.batch];
    assert!(live.printed.is_empty(), "an amend starts its ticks empty");
    assert_eq!(day.journal.fold().last_seq, 4);
    assert!(day.output(GROUP_FILE).is_file());
}

#[test]
fn a_finding_stops_the_save_until_save_anyway() {
    let Some(mut day) = day("batch-finding") else {
        return skipped();
    };
    let draft = day.read_inputs().unwrap();
    // `input/` already holds bytes under the first input's name that are not its bytes.
    let stored = input_path(day.folder.dir(), &draft.files[0].sha256);
    fs::create_dir_all(stored.parent().unwrap()).unwrap();
    let mut bytes = fs::read(testdata(INPUTS[0])).unwrap();
    bytes.extend_from_slice(b"\n% changed\n");
    fs::write(&stored, bytes).unwrap();

    let findings = match day.save(&draft, false) {
        SaveOutcome::Findings(findings) => findings,
        SaveOutcome::Saved(_) => panic!("a changed input must be found"),
    };
    assert!(!findings.errors.is_empty(), "{findings:?}");
    assert_eq!(day.journal.fold().last_seq, 0, "nothing was saved");
    assert!(!partial_dir(day.folder.dir()).exists());
    assert!(!day.folder.dir().join("output").join(GROUP_FILE).exists());

    let record = saved(day.save(&draft, true)).record;
    let texts: Vec<&str> = record.errors.iter().map(|f| f.text.as_str()).collect();
    assert_eq!(
        texts,
        findings
            .errors
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>()
    );
    assert_eq!(
        day.journal.fold().live[&record.batch].saved.errors,
        record.errors
    );
}
