//! The commands' tests: the fixtures the groups share, and one module per scenario group.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc;

use packing_engine::groupings::Grouping;
use packing_engine::orders::{ReadOrder, UnseenLogo, Warning};
use packing_engine::plan::{Plan, REST_CODE};
use packing_engine::rules::catch_all_category;
use packing_pdf::batch::Draft;
use packing_pdf::worker::{PdfWorker, SaveProgress, SaveStage};
use testkit::{
    PdfiumGuard, category, line_with_variation, read_order, temp_dir, testdata, vendor_dir,
};

use crate::commands::day::{remember_last_grouping, settings_load};
use crate::{CATEGORIES_FILE, COURIERS_FILE, OpenDraft};

use super::read::{load_categories, load_couriers, read_draft};

use super::*;

/// A state with a PDFium worker, and the guard that keeps this test the only one using it. Hold
/// the guard for the whole test: PDFium is not thread-safe, and the harness runs tests in parallel.
fn app_state(folder: &Path) -> (AppState, PdfiumGuard) {
    let guard = testkit::pdfium_lock();
    let state = AppState::new();
    state.set_data_folder(folder.to_path_buf());
    state.set_day("2026-10-07");
    assert!(state.ensure_worker(), "run tools/get-pdfium.sh");
    (state, guard)
}

mod amend_tests;
mod logo_tests;
mod plan_tests;
mod read_tests;

/// Three sepatu, two spion, one leftover; two categories with conditions so the leftovers
/// become the generated rest row.
fn orders() -> Vec<ReadOrder> {
    vec![
        read_order("o1", "t1", vec![line_with_variation("Sepatu A", "", 1)]),
        read_order("o2", "t2", vec![line_with_variation("Sepatu B", "", 1)]),
        read_order("o3", "t3", vec![line_with_variation("Sepatu C", "", 1)]),
        read_order("o4", "t4", vec![line_with_variation("Spion X", "", 1)]),
        read_order("o5", "t5", vec![line_with_variation("Spion Y", "", 1)]),
        read_order("o6", "t6", vec![line_with_variation("Lain", "", 1)]),
    ]
}

fn sepatu_plan() -> Plan {
    Plan::from_categories(&[
        category("A", "Sepatu", Some("name contains \"sepatu\"")),
        category("B", "Spion", Some("name contains \"spion\"")),
    ])
}
