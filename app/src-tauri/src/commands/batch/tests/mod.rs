//! The commands' tests: the fixtures the groups share, and one module per scenario group.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc;

use packing_engine::groupings::Grouping;
use packing_engine::ids::{OrderId, TrackingId};
use packing_engine::orders::{Line, Order, ReadOrder, UnseenLogo, Warning, display_name};
use packing_engine::plan::{Plan, REST_CODE};
use packing_engine::rules::{Category, catch_all_category};
use packing_pdf::batch::Draft;
use packing_pdf::worker::{PdfWorker, SaveProgress, SaveStage};

use crate::commands::day::{remember_last_grouping, settings_load};
use crate::{CATEGORIES_FILE, COURIERS_FILE, OpenDraft};

use super::read::{load_categories, load_couriers, read_draft};

use super::*;

/// The pinned `pdfium.dll` directory, or `None` when it has not been downloaded.
fn vendor_dir() -> Option<PathBuf> {
    let vendor = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../vendor");
    vendor.join("pdfium.dll").exists().then_some(vendor)
}

/// `testdata/` at the repository root.
fn testdata(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../testdata")
        .join(name)
}

fn temp_dir(label: &str) -> PathBuf {
    let dir =
        std::env::temp_dir().join(format!("packing-app-batch-{}-{label}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("temp dir");
    dir
}

fn app_state(folder: &Path) -> AppState {
    let state = AppState::new();
    state.set_data_folder(folder.to_path_buf());
    state.set_day("2026-10-07");
    assert!(state.ensure_worker(), "run tools/get-pdfium.sh");
    state
}

mod amend_tests;
mod logo_tests;
mod plan_tests;
mod read_tests;

fn line(name: &str, variation: &str, quantity: i64) -> Line {
    Line {
        name: name.to_string(),
        variation: variation.to_string(),
        seller_sku: String::new(),
        quantity,
        display_name: display_name(name, variation),
    }
}

fn read_order(id: &str, tracking: &str, lines: Vec<Line>) -> ReadOrder {
    ReadOrder {
        order: Order {
            order_id: OrderId::new(id),
            tracking_id: (!tracking.is_empty()).then(|| TrackingId::new(tracking)),
            courier: Some("J&T Express".to_string()),
            ship_by: None,
            lines,
            customer_message: String::new(),
        },
        pages: vec![packing_engine::orders::PageRef { file: 0, page: 0 }],
        position: 1,
        has_slip: true,
        qty_total: None,
    }
}

fn category(code: &str, name: &str, when: Option<&str>) -> Category {
    Category {
        code: code.to_string(),
        name: name.to_string(),
        condition: when.map(|text| parse_condition(text).expect("condition")),
        when: when.map(str::to_string),
    }
}

/// Three sepatu, two spion, one leftover; two categories with conditions so the leftovers
/// become the generated rest row.
fn orders() -> Vec<ReadOrder> {
    vec![
        read_order("o1", "t1", vec![line("Sepatu A", "", 1)]),
        read_order("o2", "t2", vec![line("Sepatu B", "", 1)]),
        read_order("o3", "t3", vec![line("Sepatu C", "", 1)]),
        read_order("o4", "t4", vec![line("Spion X", "", 1)]),
        read_order("o5", "t5", vec![line("Spion Y", "", 1)]),
        read_order("o6", "t6", vec![line("Lain", "", 1)]),
    ]
}

fn sepatu_plan() -> Plan {
    Plan::from_categories(&[
        category("A", "Sepatu", Some("name contains \"sepatu\"")),
        category("B", "Spion", Some("name contains \"spion\"")),
    ])
}
