//! The check before saving on real files: a saved batch reads back
//! clean, and a tampered written file is caught — a doubled page, a dropped page, an order in
//! two saved PDFs and in none, a swapped page order (a warning) and the smaller seams of a page
//! count and a packing-list count.
//!
//! Each defect test asserts the finding the check produces, so removing the check makes the test
//! fail.

mod common;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc;

use packing_engine::day::{DayState, PackingListChoice};
use packing_engine::groupings::Grouping;
use packing_engine::ids::{BatchNumber, OrderId, PdfNumber, TrackingId};
use packing_engine::orders::ReadOrder;
use packing_engine::plan::{CategoryResult, Plan, PlanResult};
use packing_engine::rules::load_rules;
use packing_engine::verify::{Check, Finding};
use packing_pdf::batch::{self, Draft};
use packing_pdf::packing_list::{
    Layout, PackingLine, PackingList, RunOrder, RunRows, SavedPdfSection,
};
use packing_pdf::worker::PdfWorker;

/// A batch read from `testdata/labels-slip.pdf`, saved into a fresh day folder.
struct Saved {
    dir: PathBuf,
    worker: PdfWorker,
    draft: Draft,
    plan: PlanResult,
    packing: PackingListChoice,
    folder: PathBuf,
}

/// Read and save the testdata labels; `None` when pdfium or the Arial font is missing.
fn saved(tag: &str) -> Option<Saved> {
    let vendor = common::vendor_dir()?;
    if !common::fonts_available() {
        eprintln!("skipped: Arial font not installed");
        return None;
    }
    let dir = std::env::temp_dir().join(format!("packing-verify-{}-{tag}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);

    let state = DayState::new("2026-10-07");
    let worker = PdfWorker::spawn(vendor).expect("spawn worker");
    let labels = common::testdata("labels-slip.pdf");
    let checks =
        batch::check_files(&worker, std::slice::from_ref(&labels), &state).expect("check files");
    let (progress, _rx) = mpsc::channel();
    let draft = batch::read(&batch::ReadRequest {
        worker: &worker,
        files: &checks,
        state: &state,
        cancel: Arc::new(AtomicBool::new(false)),
        progress,
        couriers: packing_engine::label::courier_data(),
    })
    .expect("read the labels");

    let categories = load_rules(
        &std::fs::read_to_string(common::testdata("categories.toml")).expect("rules file"),
    )
    .expect("parse the rules");
    let plan = Plan::from_categories(&categories).count(&draft.orders, PdfNumber::new(1));
    let packing = PackingListChoice::default();

    let mut day = DayState::new("2026-10-07");
    let (progress, _rx) = mpsc::channel();
    let result = batch::save(
        &batch::SaveRequest {
            worker: &worker,
            draft: &draft,
            plan: &plan,
            packing: &packing,
            grouping: &Grouping::default(),
            day: "2026-10-07",
            day_dir: &dir,
            save_anyway: false,
            progress,
        },
        &mut day,
    )
    .expect("save");
    assert!(
        result.saved,
        "a clean save must rename: {:?}",
        result.errors
    );
    assert!(
        result.errors.is_empty() && result.warnings.is_empty(),
        "a clean save has no findings: {result:?}"
    );

    Some(Saved {
        dir,
        worker,
        draft,
        plan,
        packing,
        folder: result.folder,
    })
}

impl Saved {
    fn category(&self, number: i64) -> &CategoryResult {
        self.plan
            .categories
            .iter()
            .find(|category| category.number == PdfNumber::new(number))
            .expect("a pick")
    }

    /// A pick's written file path.
    fn file(&self, number: i64) -> PathBuf {
        self.folder.join(&self.category(number).file_name)
    }

    /// A pick's pages in run order, as `(file index, page index)` into the label files.
    fn pages(&self, number: i64) -> Vec<(usize, usize)> {
        let index = self.index();
        let mut pages = Vec::new();
        for run in &self.category(number).runs {
            for id in &run.orders {
                let order = index.get(id).expect("an order of the pick");
                for page in &order.pages {
                    pages.push((page.file, page.page));
                }
            }
        }
        pages
    }

    /// The Order ID of every page of a pick, in run order.
    fn page_orders(&self, number: i64) -> Vec<OrderId> {
        let index = self.index();
        let mut ids = Vec::new();
        for run in &self.category(number).runs {
            for id in &run.orders {
                let order = index.get(id).expect("an order of the pick");
                for _ in &order.pages {
                    ids.push(id.clone());
                }
            }
        }
        ids
    }

    fn index(&self) -> HashMap<&OrderId, &ReadOrder> {
        self.draft
            .orders
            .iter()
            .map(|order| (&order.order.order_id, order))
            .collect()
    }

    /// Run the check over the batch folder on its own.
    fn check(&self) -> Check {
        batch::check_saved(
            &self.worker,
            &self.folder,
            &self.draft,
            &self.plan,
            &self.packing,
        )
        .expect("check")
    }

    /// Replace a written saved PDF with the same label pages in another order, through the
    /// worker (the label documents are still open, so `write_pages` copies from them).
    fn rewrite(&self, number: i64, pages: &[(usize, usize)]) {
        let path = self.file(number);
        let temp = self.folder.join(format!("{number}-rewrite.tmp.pdf"));
        self.worker.write_pages(pages, &temp).expect("rewrite");
        std::fs::remove_file(&path).expect("remove the old file");
        std::fs::rename(&temp, &path).expect("rename the rewritten file");
    }

    fn finish(self) {
        let dir = self.dir.clone();
        drop(self);
        let _ = std::fs::remove_dir_all(dir);
    }
}

/// The first order of a pick whose pages are not shared with another pick, for the two-picks test.
fn first_order(category: &CategoryResult) -> &OrderId {
    &category.runs[0].orders[0]
}

#[test]
fn a_saved_batch_reads_back_clean() {
    let Some(saved) = saved("clean") else {
        return;
    };
    let check = saved.check();
    assert!(
        check.is_empty(),
        "a clean save reads back clean: {check:#?}"
    );
    saved.finish();
}

#[test]
fn a_doubled_page_is_reported() {
    let Some(saved) = saved("doubled") else {
        return;
    };
    let mut pages = saved.pages(1);
    // Copy the first page a second time.
    pages.insert(0, pages[0]);
    saved.rewrite(1, &pages);

    let check = saved.check();
    assert!(
        check
            .errors
            .iter()
            .any(|finding| matches!(finding, Finding::DoubledPageInOnePdf { .. })),
        "a doubled page is an error: {check:#?}"
    );
    saved.finish();
}

#[test]
fn a_dropped_page_is_reported() {
    let Some(saved) = saved("dropped") else {
        return;
    };
    let mut pages = saved.pages(1);
    // Drop the last page.
    pages.pop();
    saved.rewrite(1, &pages);

    let check = saved.check();
    assert!(
        check
            .errors
            .iter()
            .any(|finding| matches!(finding, Finding::MissingPage { .. })),
        "a dropped page is an error: {check:#?}"
    );
    saved.finish();
}

#[test]
fn an_order_in_two_saved_pdfs_is_reported() {
    let Some(saved) = saved("two-pdfs") else {
        return;
    };
    let first = saved.pages(1)[0];
    let mut second = saved.pages(2);
    // Put pick 1's first page into pick 2 too.
    second.insert(0, first);
    saved.rewrite(2, &second);

    let check = saved.check();
    assert!(
        check
            .errors
            .iter()
            .any(|finding| matches!(finding, Finding::OrderInTwoPdfs { .. })),
        "an order in two saved PDFs is an error: {check:#?}"
    );
    saved.finish();
}

#[test]
fn an_order_in_no_saved_pdf_is_reported() {
    let Some(saved) = saved("no-pdf") else {
        return;
    };
    let order = first_order(saved.category(1));
    // Write pick 1 without that order's pages at all.
    let pages: Vec<(usize, usize)> = saved
        .pages(1)
        .into_iter()
        .zip(saved.page_orders(1))
        .filter(|(_, id)| id != order)
        .map(|(page, _)| page)
        .collect();
    saved.rewrite(1, &pages);

    let check = saved.check();
    assert!(
        check
            .errors
            .iter()
            .any(|finding| matches!(finding, Finding::OrderInNoPdf { .. })),
        "an order in no saved PDF is an error: {check:#?}"
    );
    saved.finish();
}

#[test]
fn a_wrong_label_on_a_page_is_reported() {
    let Some(saved) = saved("wrong-label") else {
        return;
    };
    // Put pick 2's first page in place of pick 1's first page.
    let foreign = saved.pages(2)[0];
    let mut pages = saved.pages(1);
    pages[0] = foreign;
    saved.rewrite(1, &pages);

    let check = saved.check();
    assert!(
        check
            .errors
            .iter()
            .any(|finding| matches!(finding, Finding::WrongOrderLabelOnPage { .. })),
        "a page holding another pick's label is an error: {check:#?}"
    );
    saved.finish();
}

#[test]
fn a_swapped_page_order_is_a_warning_only() {
    let Some(saved) = saved("swapped") else {
        return;
    };
    // Two pages of two different orders, swapped: every page is still there, in another order.
    let orders = saved.page_orders(1);
    let first = (0..orders.len())
        .find(|index| orders[*index] != orders[0])
        .expect("pick 1 has two distinct orders");
    let mut pages = saved.pages(1);
    pages.swap(0, first);
    saved.rewrite(1, &pages);

    let check = saved.check();
    assert!(
        check.errors.is_empty(),
        "a swapped order is no error: {check:#?}"
    );
    assert!(
        check
            .warnings
            .iter()
            .any(|finding| matches!(finding, Finding::PageOrderDiffers { .. })),
        "a swapped page order is a warning: {check:#?}"
    );
    saved.finish();
}

#[test]
fn a_changed_page_count_is_reported() {
    let Some(saved) = saved("page-count") else {
        return;
    };
    let mut pages = saved.pages(1);
    pages.pop();
    saved.rewrite(1, &pages);

    let check = saved.check();
    assert!(
        check
            .errors
            .iter()
            .any(|finding| matches!(finding, Finding::PageCountDiffersFromPlan { .. })),
        "a page count that is not the plan's is an error: {check:#?}"
    );
    saved.finish();
}

#[test]
fn a_changed_packing_list_count_is_reported() {
    let Some(saved) = saved("list-count") else {
        return;
    };
    // Overwrite the packing list with one that prints different counts.
    let list = saved.worker.write_packing_list(
        Layout::Pick,
        &saved.folder.join("packing-list.pdf"),
        &wrong_list(),
    );
    assert!(list.is_ok(), "{list:?}");

    let check = saved.check();
    assert!(
        check
            .errors
            .iter()
            .any(|finding| matches!(finding, Finding::PackingListCountsDiffer { .. })),
        "a packing-list count that is not the plan's is an error: {check:#?}"
    );
    saved.finish();
}

/// A minimal packing list whose counts cannot be the plan's.
fn wrong_list() -> PackingList {
    PackingList {
        day: "2026-10-07".to_string(),
        batch: Some(BatchNumber::new(1)),
        printed: "07:40".to_string(),
        sections: vec![SavedPdfSection {
            number: PdfNumber::new(1),
            code: "A".to_string(),
            name: "Sepatu".to_string(),
            runs: vec![RunRows {
                number: 1,
                orders: vec![RunOrder {
                    tracking_id: Some(TrackingId::new("JY0000001234")),
                    lines: vec![PackingLine {
                        name: "Sepatu".to_string(),
                        variation: String::new(),
                        quantity: 1,
                        display_name: "Sepatu".to_string(),
                    }],
                }],
            }],
        }],
    }
}
