//! Fixtures the crates' tests share: made-up orders, product lines, categories and a batch record,
//! the temp and `testdata/` folders, and the lock that serialises PDFium, which is not thread-safe.
//!
//! The builders hold made-up values only, so a test never copies a real ID, name or address from
//! `samples/`. They are plain functions; a test with a fixture of its own keeps it.

use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};
use std::time::{SystemTime, UNIX_EPOCH};

use packing_engine::accounts::{Account, OrderAccount};
use packing_engine::copies::Copy;
use packing_engine::ids::OrderId;
use packing_engine::record::{CourierClue, CourierClueKind};
use packing_engine::slip::SlipLine;
use packing_engine::workflows::{Workflows, load_bundled};

mod record;

pub use record::{batch, sha, time};

/// The pinned `pdfium.dll` directory, or `None` when it has not been downloaded.
pub fn vendor_dir() -> Option<PathBuf> {
    let vendor = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../vendor");
    vendor.join("pdfium.dll").exists().then_some(vendor)
}

/// The rules the program carries: the couriers, their clues and the sort rule `default`.
pub fn bundled_workflows() -> Workflows {
    load_bundled()
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

/// A slip line of a v0.2 copy.
pub fn slip_line(name: &str, variation: &str, seller_sku: &str, quantity: i64) -> SlipLine {
    SlipLine {
        name: name.to_string(),
        variation: variation.to_string(),
        seller_sku: seller_sku.to_string(),
        quantity,
    }
}

/// One account per shop, named after it.
pub fn shop_accounts(shops: &[&str]) -> Vec<Account> {
    shops
        .iter()
        .map(|shop| Account {
            name: shop.to_string(),
            shops: vec![shop.to_string()],
        })
        .collect()
}

/// A one-page v0.2 copy of `order_id` at `page` of `input`, with no tracking ID, courier or
/// printed total; a test sets what it needs.
pub fn copy(order_id: &str, input: usize, page: u16, lines: Vec<SlipLine>) -> Copy {
    Copy {
        order_id: OrderId::new(order_id),
        input,
        page_start: page,
        page_end: page,
        tracking_id: None,
        courier: None,
        courier_clue: CourierClue {
            kind: CourierClueKind::None,
            value: None,
            form: None,
        },
        ship_by_date: None,
        pickup_code: None,
        lines,
        total_quantity: None,
        fingerprints: vec![0],
        sender: None,
        account: Some(OrderAccount {
            account: "Toko Alfa".into(),
            shop: Some("Toko Alfa".into()),
        }),
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
