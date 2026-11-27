//! The migration the owner starts (journal.md, *When a date opens*): a date folder of v0.7 is made
//! flat, each order saved before the receiver's area or the weight was read gets it from its
//! batch's stored label PDFs, in an `areas_filled` and a `weights_filled` record per batch, and the
//! date's views are written again in this version's schema (`session.toml` becomes `day.toml`). The journal
//! and the views are copied first, and every move is listed. The caller holds the data folder's lock.

use std::collections::BTreeMap;
use std::sync::mpsc::Sender;

use packing_engine::ids::{BatchId, OrderId};
use packing_engine::label::{find_area, find_weight};
use packing_engine::record::{
    Area, AreasFilled, BatchSaved, OrderArea, OrderWeight, RecordBody, Weight, WeightsFilled,
};
use packing_engine::workflows::Workflows;
use packing_pdf::PdfError;
use packing_pdf::read;
use packing_pdf::worker::PdfWorker;
use packing_store::{
    DateFolder, Journal, UpdateLog, batches_to_read, flatten_date, input_path, is_v0_7, sha256_of,
    write_views_again,
};

use crate::BatchError;

pub struct MigrateRequest<'a> {
    pub worker: &'a PdfWorker,
    /// The rules, for the page reader's logos.
    pub workflows: &'a Workflows,
    pub app: &'a str,
    pub progress: Sender<MigrateProgress>,
}

/// How far the migration of a date has got: the files of v0.7 copied up of how many, or a batch's
/// label pages read of those to read.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MigrateProgress {
    Moving {
        file: usize,
        files: usize,
    },
    Reading {
        batch: BatchId,
        page: usize,
        pages: usize,
    },
}

/// What the migration of a date did: how many batches' labels it read, and the batches it skipped.
#[derive(Debug)]
pub struct DateMigrated {
    pub read: usize,
    pub skipped: Vec<SkippedBatch>,
}

/// A batch whose labels were not read, and why; the next migration tries it again.
#[derive(Debug)]
pub struct SkippedBatch {
    pub batch: BatchId,
    pub error: BatchError,
}

/// Migrate the date of `journal`: its folder made flat when it is of v0.7, the areas and weights of
/// every batch [`batches_to_read`] names, then the views. A batch that cannot be read is skipped and named; a
/// file that cannot be moved, a commit or a view that cannot be written stops the date.
pub fn migrate_date(
    folder: &DateFolder,
    journal: &mut Journal,
    request: &MigrateRequest<'_>,
) -> Result<DateMigrated, BatchError> {
    let mut log = UpdateLog::open(folder.dir(), request.app)?;
    log.keep_journal_and_views()?;
    if is_v0_7(folder.dir()) {
        flatten_date(folder, journal, &mut log, &mut |file, files| {
            let _ = request
                .progress
                .send(MigrateProgress::Moving { file, files });
        })?;
    }
    let mut migrated = DateMigrated {
        read: 0,
        skipped: Vec::new(),
    };
    for batch in batches_to_read(folder, journal.fold()) {
        let saved = journal.fold().live[&batch].saved.clone();
        match read_labels(folder, &saved, request) {
            Ok(filled) => {
                journal.commit(filled, request.app)?;
                log.journal_is_now()?;
                migrated.read += 1;
            }
            Err(error) => migrated.skipped.push(SkippedBatch { batch, error }),
        }
    }
    write_views_again(folder.dir(), journal.fold())?;
    Ok(migrated)
}

/// The area and the weight of each order of `saved` that has none, read from the first page of its
/// copy: an `areas_filled` record for the orders without an area, a `weights_filled` one for those
/// without a weight.
fn read_labels(
    folder: &DateFolder,
    saved: &BatchSaved,
    request: &MigrateRequest<'_>,
) -> Result<Vec<RecordBody>, BatchError> {
    let wanted: Vec<_> = saved
        .orders
        .iter()
        .filter(|order| order.area.is_none() || order.weight.is_none())
        .collect();
    let pages = wanted.len();
    let mut read: BTreeMap<OrderId, (Area, Weight)> = BTreeMap::new();
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
        let (batch, read_before) = (saved.batch, read.len());
        let found = request.worker.run(move |pdfium| {
            let file = read::open(pdfium, &path)?;
            let mut found = Vec::with_capacity(on_input.len());
            for (order_id, page) in on_input {
                let text = file.page_text(page, &rules)?;
                found.push((order_id, (find_area(&text), find_weight(&text))));
                let page = read_before + found.len();
                let _ = progress.send(MigrateProgress::Reading { batch, page, pages });
            }
            Ok::<_, PdfError>(found)
        })??;
        read.extend(found);
    }
    let mut areas = Vec::new();
    let mut weights = Vec::new();
    for order in wanted {
        let Some((area, weight)) = read.remove(&order.order_id) else {
            continue;
        };
        let order_id = order.order_id.clone();
        if order.area.is_none() {
            areas.push(OrderArea {
                order_id: order_id.clone(),
                area,
            });
        }
        if order.weight.is_none() {
            weights.push(OrderWeight { order_id, weight });
        }
    }
    let batch = saved.batch;
    let mut filled = Vec::new();
    if !areas.is_empty() {
        filled.push(RecordBody::AreasFilled(AreasFilled {
            batch,
            orders: areas,
        }));
    }
    if !weights.is_empty() {
        filled.push(RecordBody::WeightsFilled(WeightsFilled {
            batch,
            orders: weights,
        }));
    }
    Ok(filled)
}
