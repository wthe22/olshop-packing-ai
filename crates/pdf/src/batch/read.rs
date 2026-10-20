//! Reading the chosen label files into a draft: open the files, read every page, group the pages
//! into orders, then apply the duplicate guard (steps 2-4).

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::Sender;

use packing_engine::day::{DayState, Duplicate, duplicates_warning};
use packing_engine::label::CourierTable;
use packing_engine::orders::{self, LogoAnswer, ReadOrder, UnseenLogo, Warning};

use crate::worker::{PdfWorker, Progress};

use super::check::FileCheck;
use super::error::BatchError;

/// A batch that has been read but not saved: the files,
/// the orders left after the duplicate guard, the duplicates and the warnings.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Draft {
    pub files: Vec<FileCheck>,
    pub orders: Vec<ReadOrder>,
    pub duplicates: Vec<Duplicate>,
    pub warnings: Vec<String>,
    /// Pages read over all files.
    pub pages: usize,
    /// Orders read, before the duplicate guard.
    pub total_orders: usize,
    /// Pages that carry a packing slip.
    pub slip_pages: usize,
    /// The header-band images the courier data does not know, for the logo prompt.
    /// Each is asked about once per batch and then leaves the list.
    pub unseen_logos: Vec<UnseenLogo>,
    /// The read warnings, kept structured so the logo prompt can recompute the ones that depend
    /// on the courier; [`Draft::warnings`] is their display text plus the duplicate
    /// warning.
    pub read_warnings: Vec<Warning>,
}

impl Draft {
    /// Apply the owner's answer about one unseen logo (the logo prompt of the New batch
    /// screen): the orders carrying the logo are named where the two clues agree
    /// ([`UnseenLogo::apply`] on `table`), the logo leaves `unseen_logos`, and the warnings that
    /// depend on the courier are recomputed (the unknown-courier line shrinks or goes). Returns
    /// how many orders the answer named; an unknown `hash` changes nothing.
    pub fn answer_logo(&mut self, hash: &str, answer: &LogoAnswer, table: &CourierTable) -> usize {
        let Some(index) = self
            .unseen_logos
            .iter()
            .position(|entry| entry.hash == hash)
        else {
            return 0;
        };
        let entry = self.unseen_logos.remove(index);
        let named = entry.apply(&mut self.orders, answer, table);
        self.refresh_courier_warning();
        named
    }

    /// Rebuild the warnings after the couriers changed: the unknown-courier line is recomputed
    /// from the orders (the message names them by tracking ID) and keeps its place among the
    /// read warnings; the `Qty Total` and tracking-ID lines do not depend on the courier.
    fn refresh_courier_warning(&mut self) {
        let mut warnings: Vec<Warning> = self
            .read_warnings
            .iter()
            .filter(|warning| !matches!(warning, Warning::UnknownCourier { .. }))
            .cloned()
            .collect();
        if let Some(unknown) = Warning::unknown_courier(&self.orders) {
            // Read puts the unknown-courier line after the `Qty Total` ones and before the
            // tracking-ID ones (`orders::read_batch`); keep that place.
            let at = warnings
                .iter()
                .position(|warning| matches!(warning, Warning::TrackingIdMissing { .. }))
                .unwrap_or(warnings.len());
            warnings.insert(at, unknown);
        }
        self.read_warnings = warnings;
        self.warnings = self
            .read_warnings
            .iter()
            .map(|warning| warning.to_string())
            .collect();
        if !self.duplicates.is_empty() {
            self.warnings.push(duplicates_warning(&self.duplicates));
        }
    }
}

/// Everything one read of a batch needs: the label files, the day state it is checked against,
/// the read's cancel flag and progress, and the data folder's courier data.
pub struct ReadRequest<'a> {
    pub worker: &'a PdfWorker,
    pub files: &'a [FileCheck],
    pub state: &'a DayState,
    pub cancel: Arc<AtomicBool>,
    pub progress: Sender<Progress>,
    pub couriers: &'a CourierTable,
}

/// Open the files on the worker, read every page with progress and a cancel flag, group the
/// pages into orders, then apply the duplicate guard (steps 2-4). `state` supplies the
/// duplicate guard (amend passes its state without the batch);
/// `couriers` is the data folder's courier data.
pub fn read(request: &ReadRequest<'_>) -> Result<Draft, BatchError> {
    let paths: Vec<PathBuf> = request.files.iter().map(|file| file.path.clone()).collect();
    request.worker.open(&paths)?;
    let items = request.worker.read(
        request.cancel.clone(),
        request.progress.clone(),
        request.couriers.clone(),
    )?;
    let pages = items.len();

    let names: Vec<String> = request.files.iter().map(|file| file.name.clone()).collect();
    let name_refs: Vec<&str> = names.iter().map(String::as_str).collect();
    let reading = orders::read_batch(items, &name_refs, request.couriers)?;

    let slip_pages = reading
        .orders
        .iter()
        .filter(|order| order.has_slip)
        .map(|order| order.pages.len())
        .sum();

    let (kept, duplicates) = request.state.drop_saved(&reading.orders);
    if kept.is_empty() {
        if reading.orders.is_empty() {
            return Err(BatchError::Message("no orders were read".to_string()));
        }
        return Err(BatchError::AllSaved {
            orders: reading.orders.len(),
            // Every read order was a duplicate, so the list names a batch.
            batch: duplicates
                .first()
                .map(|dup| dup.batch)
                .expect("every order was a duplicate"),
        });
    }

    let mut warnings: Vec<String> = reading
        .warnings
        .iter()
        .map(|warning| warning.to_string())
        .collect();
    if !duplicates.is_empty() {
        warnings.push(duplicates_warning(&duplicates));
    }

    let total_orders = reading.orders.len();
    Ok(Draft {
        files: request.files.to_vec(),
        orders: kept,
        duplicates,
        warnings,
        pages,
        total_orders,
        slip_pages,
        unseen_logos: reading.unseen_logos,
        read_warnings: reading.warnings,
    })
}
