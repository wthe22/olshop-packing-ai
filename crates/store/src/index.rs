//! The search index (reprint.md): every order of a workspace's live batches, folded from every
//! journal of the workspace, with the fields a filter reads and its trace back to the input page,
//! and every date's reprints. It is only read: every decision is still taken from a date's own fold.

use std::collections::BTreeMap;
use std::ops::RangeInclusive;
use std::path::Path;

use jiff::civil::Date;
use packing_engine::ids::{BatchId, OrderId};
use packing_engine::record::{BatchSaved, ItemPick, ReprintMade, SavedOrder};
use packing_engine::rules::{Condition, FilterFields, FilterLine, evaluate_filter};
use packing_engine::workflows::Courier;

use crate::StoreError;
use crate::date::DateFolder;
use crate::fold::{Damage, Fold};
use crate::journal::Journal;
use crate::workspaces::dates;

/// One saved order as Search finds it.
#[derive(Clone, Debug, PartialEq)]
pub struct IndexedOrder {
    pub date: Date,
    pub batch: BatchId,
    pub order: SavedOrder,
    /// Its content's items, in their order.
    pub items: Vec<ItemPick>,
    /// The group PDF that holds it.
    pub file: String,
    /// The pages it fills in that group PDF, counted from 1.
    pub file_pages: RangeInclusive<u16>,
    /// The name its input had when it was chosen.
    pub input: String,
}

impl FilterFields for IndexedOrder {
    fn lines(&self) -> Vec<FilterLine<'_>> {
        self.items
            .iter()
            .enumerate()
            .map(|(at, item)| FilterLine {
                name: &item.name,
                variation: &item.variant,
                seller_sku: self
                    .order
                    .skus
                    .get(at)
                    .and_then(Option::as_deref)
                    .unwrap_or_default(),
                quantity: i64::from(item.quantity),
            })
            .collect()
    }

    fn courier(&self) -> &str {
        self.order.courier.as_ref().map_or("", |code| code.as_str())
    }

    fn order_id(&self) -> &str {
        self.order.order_id.as_str()
    }

    fn tracking_id(&self) -> &str {
        self.order.tracking_id.as_ref().map_or("", |id| id.as_str())
    }

    fn total_quantity(&self) -> Option<i64> {
        self.order.total_quantity.map(i64::from)
    }

    fn ship_by(&self) -> Option<Date> {
        self.order.ship_by_date
    }
}

/// A reprint that printed an order again.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReprintMark {
    /// The date the reprint was made on.
    pub date: Date,
    pub number: u8,
}

/// The orders of every date of one workspace, by date, each date in printing order.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SearchIndex {
    dates: BTreeMap<Date, Vec<IndexedOrder>>,
    /// The reprints of each date that has any, in the order they were made.
    reprints: BTreeMap<Date, Vec<ReprintMade>>,
    /// The dates whose journal is damaged: left out, and named in the panel.
    pub damaged: BTreeMap<Date, Damage>,
}

impl SearchIndex {
    /// Fold every journal of `workspace`. Nothing is written: a torn tail or `.partial/` waits for
    /// the date's own open.
    pub fn build(data_dir: &Path, workspace: &str) -> Result<Self, StoreError> {
        let mut index = SearchIndex::default();
        for date in dates(data_dir, workspace)? {
            let journal = Journal::open(&DateFolder::new(data_dir, workspace, date))?;
            match journal.damage() {
                Some(damage) => {
                    index.damaged.insert(date, damage.clone());
                }
                None => index.refresh(journal.fold()),
            }
        }
        Ok(index)
    }

    /// Take a date's orders from its fold as it is now: after a commit, or when it opens.
    pub fn refresh(&mut self, fold: &Fold) {
        let orders: Vec<IndexedOrder> = fold
            .live
            .iter()
            .flat_map(|(batch, live)| indexed(fold.date, *batch, &live.saved))
            .collect();
        self.damaged.remove(&fold.date);
        if orders.is_empty() {
            self.dates.remove(&fold.date);
        } else {
            self.dates.insert(fold.date, orders);
        }
        if fold.reprints.is_empty() {
            self.reprints.remove(&fold.date);
        } else {
            let made = fold.reprints.iter().map(|r| r.made.clone()).collect();
            self.reprints.insert(fold.date, made);
        }
    }

    /// The order saved on `date` under `order_id`, while a live batch holds it.
    pub fn find(&self, date: Date, order_id: &OrderId) -> Option<&IndexedOrder> {
        self.dates
            .get(&date)?
            .iter()
            .find(|order| &order.order.order_id == order_id)
    }

    /// Every reprint that printed this order again, oldest first.
    pub fn reprints_of(&self, order: &IndexedOrder) -> Vec<ReprintMark> {
        self.reprints
            .iter()
            .flat_map(|(date, made)| made.iter().map(move |reprint| (*date, reprint)))
            .filter(|(_, reprint)| {
                reprint.orders.iter().any(|printed| {
                    printed.date == order.date && printed.order_id == order.order.order_id
                })
            })
            .map(|(date, reprint)| ReprintMark {
                date,
                number: reprint.number,
            })
            .collect()
    }

    /// The orders a filter matches on the dates of `dates`, in printing order; only those dates
    /// are looked at. `couriers` name the couriers the filter compares.
    pub fn search(
        &self,
        filter: &Condition,
        dates: RangeInclusive<Date>,
        couriers: &[Courier],
    ) -> Vec<&IndexedOrder> {
        self.dates
            .range(dates)
            .flat_map(|(_, orders)| orders)
            .filter(|order| evaluate_filter(filter, *order, couriers))
            .collect()
    }

    /// The number of saved orders of each date that has any.
    pub fn counts(&self) -> BTreeMap<Date, usize> {
        self.dates
            .iter()
            .map(|(date, orders)| (*date, orders.len()))
            .collect()
    }
}

/// The orders of a batch; they are in printing order, so each one's pages in its group PDF follow
/// the pages of the orders of its group before it.
fn indexed(date: Date, batch: BatchId, saved: &BatchSaved) -> Vec<IndexedOrder> {
    let mut written: BTreeMap<&str, u16> = BTreeMap::new();
    saved
        .orders
        .iter()
        .map(|order| {
            let before = written.entry(order.group.as_str()).or_default();
            let first = *before + 1;
            *before += order.page_end - order.page_start + 1;
            (order, first..=*before)
        })
        .map(|(order, file_pages)| IndexedOrder {
            date,
            batch,
            items: saved
                .contents
                .get(order.content as usize)
                .map(|content| content.items.clone())
                .unwrap_or_default(),
            file: saved
                .outputs
                .iter()
                .find(|output| output.group.as_deref() == Some(order.group.as_str()))
                .map_or_else(String::new, |output| output.file.clone()),
            file_pages,
            input: saved
                .inputs
                .iter()
                .find(|input| input.pdf == order.pdf)
                .map_or_else(String::new, |input| input.filename.clone()),
            order: order.clone(),
        })
        .collect()
}
