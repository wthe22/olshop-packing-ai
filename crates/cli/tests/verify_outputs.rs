//! The verifier on outputs PDFium wrote: the repeat fixture read, planned and recorded by the v0.2
//! chain, its group PDF and summary PDF written by `packing-pdf`, its record sealed by `packing-store`. Written
//! right, the set passes; with a page doubled, a page missing, two pages swapped or a foreign page,
//! the verifier says so. Each case fails when its check is removed.
//!
//! Skipped with a message when `pdfium.dll` has not been downloaded (tools/get-pdfium.sh).

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;

use jiff::civil::date;
use packing_engine::batch_record::{BatchFacts, batch_saved};
use packing_engine::copies::read_copies;
use packing_engine::ids::{BatchId, PdfId};
use packing_engine::label::courier_data;
use packing_engine::record::{BatchSaved, Datetime, InputPdf, OutputFile, RecordBody};
use packing_engine::repeats::{Choices, sort_out};
use packing_engine::sorting::{plan_groups, sort_batch};
use packing_engine::workflows::{BUNDLED_WORKFLOWS, load_workflows};
use packing_pdf::summary_pdf::{SummaryFacts, write_summary};
use packing_pdf::{read, write};
use packing_store::{DateFolder, Journal, encode_line, input_path, sha256_of, store_input};
use packing_verify::{BatchCheck, Findings, verify_batch};
use pdfium_render::prelude::Pdfium;
use testkit::{TempDir, testdata, vendor_dir};

const INPUTS: [&str; 2] = ["labels-repeat-1.pdf", "labels-repeat-2.pdf"];

/// The repeat fixture as a draft: its inputs stored in a date folder, its record without outputs,
/// and the pages each group PDF must hold, as (input, page index) in printing order.
struct Draft {
    folder: DateFolder,
    record: BatchSaved,
    pages: Vec<(String, Vec<(usize, usize)>)>,
}

fn draft(pdfium: &Pdfium, data: &Path) -> Draft {
    let folder = DateFolder::new(data, "Toko Contoh", date(2026, 10, 8));
    let paths: Vec<PathBuf> = INPUTS.iter().map(|name| testdata(name)).collect();
    let pages = read::read_all(
        pdfium,
        &paths,
        &AtomicBool::new(false),
        courier_data(),
        &mut |_, _| {},
    )
    .unwrap();
    let workflows = load_workflows(BUNDLED_WORKFLOWS).unwrap();
    let copies = read_copies(pages, &INPUTS, &workflows).unwrap().copies;
    let orders = sort_out(&copies, &BTreeMap::new(), &Choices::default()).unwrap();
    let rule = workflows.sort_rule("default").unwrap();
    let plan = plan_groups(rule);
    let sorted = sort_batch(&copies, &orders.kept, &plan, BatchId::new(1));
    let time = Datetime::parse("2026-10-08T07:00:00+07:00").unwrap();
    let inputs: Vec<InputPdf> = paths
        .iter()
        .enumerate()
        .map(|(at, path)| {
            let stored = store_input(folder.dir(), path).unwrap();
            InputPdf {
                pdf: PdfId::new(at as u8 + 1),
                sha256: stored.sha256,
                filename: INPUTS[at].to_string(),
                size: stored.size,
                page_count: [6, 2][at],
                create_time: time.clone(),
                modify_time: time.clone(),
            }
        })
        .collect();
    let record = batch_saved(BatchFacts {
        batch: BatchId::new(1),
        amends: false,
        inputs: &inputs,
        rule_name: &rule.name,
        plan: &plan,
        copies: &copies,
        orders: &orders,
        sorted: &sorted,
        outputs: Vec::new(),
    });
    let pages = sorted
        .groups
        .iter()
        .map(|group| {
            let pages = group
                .contents
                .iter()
                .flat_map(|block| &block.orders)
                .flat_map(|&at| {
                    let copy = &copies[at];
                    (copy.page_start..=copy.page_end).map(move |p| (copy.input, p as usize - 1))
                })
                .collect();
            (group.file.clone(), pages)
        })
        .collect();
    Draft {
        folder,
        record,
        pages,
    }
}

fn described(path: &Path, group: Option<&str>, pages: u16) -> OutputFile {
    OutputFile {
        file: path.file_name().unwrap().to_string_lossy().into_owned(),
        group: group.map(str::to_string),
        pages,
        size: fs::metadata(path).unwrap().len(),
        sha256: sha256_of(path).unwrap(),
    }
}

/// Write the draft's group PDFs, each with `change` applied to its pages, then its summary; seal
/// the record as the commit would append it and verify.
fn verify(pdfium: &Pdfium, tag: &str, change: impl Fn(&mut Vec<(usize, usize)>)) -> Findings {
    let data = TempDir::new(tag);
    let Draft {
        folder,
        mut record,
        pages,
    } = draft(pdfium, data.path());
    let partial = folder.dir().join("output").join(".partial");
    fs::create_dir_all(&partial).unwrap();
    let mut sources: Vec<PathBuf> = record
        .inputs
        .iter()
        .map(|input| input_path(folder.dir(), &input.sha256))
        .collect();
    sources.push(testdata("labels-slip.pdf"));
    let opened = read::open_all(pdfium, &sources).unwrap();
    for (file, mut group_pages) in pages {
        change(&mut group_pages);
        // A name with characters the summary draws in its CID font, whose text lopdf cannot read.
        let out = partial.join(format!("Kaca é × {file}"));
        write::copy_pages(pdfium, &opened, &group_pages, &out).unwrap();
        record
            .outputs
            .push(described(&out, Some("Z"), group_pages.len() as u16));
    }
    let summary_path = partial.join("1 summary.pdf");
    let summary_pages = write_summary(
        pdfium,
        &summary_path,
        &SummaryFacts {
            workspace: folder.workspace(),
            date: folder.date(),
            record: &record,
            printed: "07:00",
            app: "0.2.0",
        },
    )
    .unwrap();
    record
        .outputs
        .push(described(&summary_path, None, summary_pages as u16));

    let journal = Journal::open(&folder).unwrap();
    assert_eq!(journal.fold().last_seq, 0);
    let started = packing_engine::record::Record {
        seq: 1,
        time: Datetime::now(),
        app: "0.2.0".into(),
        body: RecordBody::Started(packing_engine::record::Started {
            schema: "1.0".into(),
            workspace: "Toko Contoh".into(),
            date: folder.date(),
        }),
    };
    let saved = packing_engine::record::Record {
        seq: 2,
        time: Datetime::now(),
        app: "0.2.0".into(),
        body: RecordBody::BatchSaved(record),
    };
    let append: Vec<u8> = [encode_line(&started), encode_line(&saved)].concat();
    verify_batch(&BatchCheck {
        journal: b"",
        append: &append,
        input_dir: &folder.dir().join("input"),
        output_dir: &partial,
    })
}

fn bind() -> Option<(Pdfium, testkit::PdfiumGuard)> {
    let vendor = vendor_dir()?;
    let guard = testkit::pdfium_lock();
    Some((packing_pdf::bind(&vendor).expect("bind pdfium"), guard))
}

#[test]
fn outputs_pdfium_wrote_are_checked_page_by_page() {
    let Some((pdfium, _guard)) = bind() else {
        eprintln!("skipped: pdfium.dll not downloaded (tools/get-pdfium.sh)");
        return;
    };
    let right = verify(&pdfium, "verify-right", |_| {});
    assert!(
        right.is_clean(),
        "PDFium's copies have their source pages' digests: {right:?}"
    );

    type Change = fn(&mut Vec<(usize, usize)>);
    let cases: [(&str, Change, &str, bool); 4] = [
        ("doubled", |p| p.insert(1, p[0]), "is written 2 times", true),
        (
            "missing",
            |p| {
                p.pop();
            },
            "is in no output",
            true,
        ),
        (
            "swapped",
            |p| p.swap(0, 1),
            "not in the printing order",
            false,
        ),
        ("foreign", |p| p[0] = (2, 0), "is a page of no input", true),
    ];
    for (what, change, finding, error) in cases {
        let findings = verify(&pdfium, &format!("verify-{what}"), change);
        let list = if error {
            &findings.errors
        } else {
            &findings.warnings
        };
        assert!(
            list.iter().any(|f| f.contains(finding)),
            "{what}: {findings:?}"
        );
        if !error {
            assert!(
                findings.errors.is_empty(),
                "{what}: a warning alone: {findings:?}"
            );
        }
    }
}
