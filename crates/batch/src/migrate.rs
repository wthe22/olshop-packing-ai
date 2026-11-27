//! The migration the owner starts (journal.md, *When a date opens*): a date folder of v0.7 is made
//! flat, each order saved before the receiver's area, the weight or the shop (v0.4) was read gets it
//! from its batch's stored label PDFs, in an `areas_filled`, a `weights_filled` and a `shops_filled`
//! record per batch, and the
//! date's views are written again in this version's schema (`session.toml` becomes `day.toml`). The journal
//! and the views are copied first, and every move is listed. The caller holds the data folder's lock.

use std::collections::BTreeMap;
use std::sync::mpsc::Sender;

use packing_engine::accounts::account_of;
use packing_engine::copies::label_courier;
use packing_engine::ids::{BatchId, CourierCode, OrderId, TrackingId};
use packing_engine::label::{find_area, find_sender, find_weight};
use packing_engine::record::{
    Area, AreasFilled, BatchSaved, CourierClue, CouriersFilled, OrderArea, OrderCourier, OrderShop,
    OrderWeight, RecordBody, SavedOrder, ShopsFilled, Weight, WeightsFilled,
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

/// What a label page gives the Update.
struct Read {
    area: Area,
    weight: Weight,
    shop: Option<String>,
    courier: (Option<TrackingId>, Option<CourierCode>, CourierClue),
}

/// What each order of `saved` lacks, read from the first page of its copy: an `areas_filled` record
/// for the orders without an area, a `weights_filled` one for those without a weight, a
/// `shops_filled` one for those saved by v0.4 (the listed shop the label shows, as a new batch finds
/// it, else the sender as printed), and a `couriers_filled` one for those the label now gives a
/// tracking ID or a courier they had none of.
fn read_labels(
    folder: &DateFolder,
    saved: &BatchSaved,
    request: &MigrateRequest<'_>,
) -> Result<Vec<RecordBody>, BatchError> {
    let wanted: Vec<_> = saved
        .orders
        .iter()
        .filter(|order| order.unread() || order.tracking_id.is_none() || order.courier.is_none())
        .collect();
    let pages = wanted.len();
    let mut read: BTreeMap<OrderId, Read> = BTreeMap::new();
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
                let shop = find_sender(&text).map(|sender| {
                    account_of(&sender.text, &rules.accounts)
                        .and_then(|account| account.shop)
                        .unwrap_or(sender.name)
                });
                let courier = label_courier(&rules, &text, &order_id);
                let (area, weight) = (find_area(&text), find_weight(&text));
                found.push((
                    order_id,
                    Read {
                        area,
                        weight,
                        shop,
                        courier,
                    },
                ));
                let page = read_before + found.len();
                let _ = progress.send(MigrateProgress::Reading { batch, page, pages });
            }
            Ok::<_, PdfError>(found)
        })??;
        read.extend(found);
    }
    let mut filled = Filled::default();
    for order in wanted {
        if let Some(label) = read.remove(&order.order_id) {
            filled.add(order, label);
        }
    }
    Ok(filled.records(saved.batch))
}

/// The records of one batch's read, each order in the batch's printing order.
#[derive(Default)]
struct Filled {
    areas: Vec<OrderArea>,
    weights: Vec<OrderWeight>,
    shops: Vec<OrderShop>,
    couriers: Vec<OrderCourier>,
}

impl Filled {
    fn add(&mut self, order: &SavedOrder, label: Read) {
        let order_id = order.order_id.clone();
        if order.area.is_none() {
            let area = label.area;
            self.areas.push(OrderArea {
                order_id: order_id.clone(),
                area,
            });
        }
        if order.weight.is_none() {
            let weight = label.weight;
            self.weights.push(OrderWeight {
                order_id: order_id.clone(),
                weight,
            });
        }
        if order.shop_unread() {
            let shop = label.shop;
            self.shops.push(OrderShop {
                order_id: order_id.clone(),
                shop,
            });
        }
        let (tracking_id, courier, courier_clue) = label.courier;
        let tracking_id = tracking_id.filter(|_| order.tracking_id.is_none());
        let courier = courier.filter(|_| order.courier.is_none());
        if tracking_id.is_some() || courier.is_some() {
            self.couriers.push(OrderCourier {
                order_id,
                tracking_id,
                courier,
                courier_clue,
            });
        }
    }

    fn records(self, batch: BatchId) -> Vec<RecordBody> {
        let mut records = Vec::new();
        if !self.areas.is_empty() {
            records.push(RecordBody::AreasFilled(AreasFilled {
                batch,
                orders: self.areas,
            }));
        }
        if !self.weights.is_empty() {
            records.push(RecordBody::WeightsFilled(WeightsFilled {
                batch,
                orders: self.weights,
            }));
        }
        if !self.shops.is_empty() {
            records.push(RecordBody::ShopsFilled(ShopsFilled {
                batch,
                orders: self.shops,
            }));
        }
        if !self.couriers.is_empty() {
            records.push(RecordBody::CouriersFilled(CouriersFilled {
                batch,
                orders: self.couriers,
            }));
        }
        records
    }
}
