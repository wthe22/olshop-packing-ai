//! The migration the owner starts (journal.md, *When a date opens*): each order saved before the
//! receiver's area was read gets it from its batch's stored label PDFs, one `areas_filled` record
//! per batch, and the date's views are written again in this version's schema. The caller holds
//! the data folder's lock.

use std::collections::BTreeMap;
use std::sync::mpsc::Sender;

use packing_engine::ids::{BatchId, OrderId};
use packing_engine::label::find_area;
use packing_engine::record::{Area, AreasFilled, BatchSaved, OrderArea, RecordBody};
use packing_engine::workflows::Workflows;
use packing_pdf::PdfError;
use packing_pdf::read;
use packing_pdf::worker::PdfWorker;
use packing_store::{DateFolder, Journal, areas_to_read, input_path, sha256_of, write_views};

use crate::BatchError;

pub struct MigrateRequest<'a> {
    pub worker: &'a PdfWorker,
    /// The rules, for the page reader's logos.
    pub workflows: &'a Workflows,
    pub app: &'a str,
    pub progress: Sender<AreaProgress>,
}

/// How far the reading of a batch's areas has got: its label pages read of those to read.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AreaProgress {
    pub batch: BatchId,
    pub page: usize,
    pub pages: usize,
}

/// A batch whose areas were not read, and why; the next migration tries it again.
#[derive(Debug)]
pub struct SkippedBatch {
    pub batch: BatchId,
    pub error: BatchError,
}

/// Migrate the date of `journal`: the areas of every batch [`areas_to_read`] names, then the views.
/// A batch that cannot be read is skipped and returned; a commit or a view that cannot be written
/// stops the date.
pub fn migrate_date(
    folder: &DateFolder,
    journal: &mut Journal,
    request: &MigrateRequest<'_>,
) -> Result<Vec<SkippedBatch>, BatchError> {
    let mut skipped = Vec::new();
    for batch in areas_to_read(folder, journal.fold()) {
        let saved = journal.fold().live[&batch].saved.clone();
        match read_areas(folder, &saved, request) {
            Ok(filled) => {
                journal.commit(vec![RecordBody::AreasFilled(filled)], request.app)?;
            }
            Err(error) => skipped.push(SkippedBatch { batch, error }),
        }
    }
    write_views(folder.dir(), journal.fold())?;
    Ok(skipped)
}

/// The area of each order of `saved` that has none, read from the first page of its copy.
fn read_areas(
    folder: &DateFolder,
    saved: &BatchSaved,
    request: &MigrateRequest<'_>,
) -> Result<AreasFilled, BatchError> {
    let wanted: Vec<_> = saved
        .orders
        .iter()
        .filter(|order| order.area.is_none())
        .collect();
    let pages = wanted.len();
    let mut areas: BTreeMap<OrderId, Area> = BTreeMap::new();
    for input in &saved.inputs {
        let path = input_path(folder, saved.batch, input.pdf, &input.sha256);
        if !path.is_file() || sha256_of(&path)? != input.sha256 {
            return Err(BatchError::InputChanged {
                file: input.filename.clone(),
                date: folder.date(),
            });
        }
        let on_input: Vec<(OrderId, usize)> = wanted
            .iter()
            .filter(|order| order.pdf == input.pdf)
            .map(|order| (order.order_id.clone(), usize::from(order.page_start) - 1))
            .collect();
        if on_input.is_empty() {
            continue;
        }
        let (rules, progress) = (request.workflows.clone(), request.progress.clone());
        let (batch, read_before) = (saved.batch, areas.len());
        let read = request.worker.run(move |pdfium| {
            let file = read::open(pdfium, &path)?;
            let mut read = Vec::with_capacity(on_input.len());
            for (order_id, page) in on_input {
                read.push((order_id, find_area(&file.page_text(page, &rules)?)));
                let page = read_before + read.len();
                let _ = progress.send(AreaProgress { batch, page, pages });
            }
            Ok::<_, PdfError>(read)
        })??;
        areas.extend(read);
    }
    let orders = wanted
        .iter()
        .filter_map(|order| {
            let area = areas.remove(&order.order_id)?;
            Some(OrderArea {
                order_id: order.order_id.clone(),
                area,
            })
        })
        .collect();
    Ok(AreasFilled {
        batch: saved.batch,
        orders,
    })
}
