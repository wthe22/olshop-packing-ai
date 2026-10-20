//! Fixtures the crates' tests share: made-up orders, product lines and categories, the temp and
//! `testdata/` folders, and the lock that serialises PDFium, which is not thread-safe.
//!
//! The builders hold made-up values only, so a test never copies a real ID, name or address from
//! `samples/`. They are plain functions; a test with a fixture of its own keeps it.

use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};
use std::time::{SystemTime, UNIX_EPOCH};

use packing_engine::ids::{OrderId, TrackingId};
use packing_engine::orders::{Line, Order, PageRef, ReadOrder, display_name};
use packing_engine::rules::{Category, parse_condition};

/// The pinned `pdfium.dll` directory, or `None` when it has not been downloaded.
pub fn vendor_dir() -> Option<PathBuf> {
    let vendor = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../vendor");
    vendor.join("pdfium.dll").exists().then_some(vendor)
}

/// A file in the shared `testdata/` folder.
pub fn testdata(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../testdata")
        .join(name)
}

/// A fresh folder under the system temp directory, empty at the start. Tests of one test binary
/// share a process, so `label` tells one test's folder from another's.
pub fn temp_dir(label: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("packing-test-{}-{label}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("temp dir");
    dir
}

/// A unique folder under the system temp directory, removed when the test ends; `tag` tells one
/// test's folder from another's.
pub struct TempDir(PathBuf);

impl TempDir {
    pub fn new(tag: &str) -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("packing-test-{tag}-{}-{nanos}", std::process::id()));
        std::fs::create_dir_all(&path).expect("create temp dir");
        TempDir(path)
    }

    pub fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// One product line with no variation, its display name equal to the name.
pub fn line(name: &str, quantity: i64) -> Line {
    Line {
        name: name.to_string(),
        variation: String::new(),
        seller_sku: String::new(),
        quantity,
        display_name: name.to_string(),
    }
}

/// One product line of a product with a variation, its display name `name — variation`.
pub fn line_with_variation(name: &str, variation: &str, quantity: i64) -> Line {
    Line {
        name: name.to_string(),
        variation: variation.to_string(),
        seller_sku: String::new(),
        quantity,
        display_name: display_name(name, variation),
    }
}

/// One order of a single line, on the same made-up order and tracking IDs every time.
pub fn order(name: &str, quantity: i64, courier: Option<&str>) -> Order {
    Order {
        order_id: OrderId::new("580000000000000999"),
        tracking_id: Some(TrackingId::new("JY0000009999")),
        courier: courier.map(str::to_string),
        ship_by: None,
        lines: vec![line(name, quantity)],
        customer_message: String::new(),
    }
}

/// One read order of the given lines, carrying one page of the first file. An empty `tracking` is
/// an order without a tracking ID.
pub fn read_order(id: &str, tracking: &str, lines: Vec<Line>) -> ReadOrder {
    ReadOrder {
        order: Order {
            order_id: OrderId::new(id),
            tracking_id: (!tracking.is_empty()).then(|| TrackingId::new(tracking)),
            courier: Some("J&T Express".to_string()),
            ship_by: None,
            lines,
            customer_message: String::new(),
        },
        pages: vec![PageRef { file: 0, page: 0 }],
        position: 1,
        has_slip: true,
        qty_total: None,
    }
}

/// One category entry; a `when` of `None` is the last entry, which takes the rest.
pub fn category(code: &str, name: &str, when: Option<&str>) -> Category {
    Category {
        code: code.to_string(),
        name: name.to_string(),
        condition: when.map(|text| parse_condition(text).expect("parse category")),
        when: when.map(str::to_string),
    }
}

/// PDFium is not thread-safe and the test harness runs tests on several threads.
static PDFIUM_LOCK: Mutex<()> = Mutex::new(());

/// The guard one test holds while it is the only one using PDFium; the lock is released when the
/// test ends. A poisoned lock is taken anyway: the guard only says who is using PDFium.
pub struct PdfiumGuard {
    _held: MutexGuard<'static, ()>,
}

/// Take the PDFium lock. A test that binds PDFium, or starts a worker that binds it, holds the
/// guard for its whole run so no second bind runs beside it.
pub fn pdfium_lock() -> PdfiumGuard {
    PdfiumGuard {
        _held: PDFIUM_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()),
    }
}
