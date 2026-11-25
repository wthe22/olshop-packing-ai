//! What the owner's migration would change in the data folder (journal.md, *When a date opens*):
//! views of another `schema`, and live batches saved before the receiver's area was read. Finding
//! them reads only; a damaged journal is passed over, as nothing may be appended to it.

use std::path::Path;

use jiff::civil::Date;
use packing_engine::ids::BatchId;

use crate::StoreError;
use crate::date::DateFolder;
use crate::dates::dates;
use crate::fold::Fold;
use crate::inputs::input_path;
use crate::journal::Journal;
use crate::views::{SESSION_SCHEMA, views_schema};

/// A date the migration would change.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DateMigration {
    pub date: Date,
    /// The `schema` of its `session.toml`, when another than this version writes.
    pub older_views: Option<String>,
    /// Its batches whose orders' areas would be read, and how many orders that is.
    pub batches: Vec<BatchId>,
    pub orders: usize,
}

/// The dates of the data folder the migration would change, oldest first.
pub fn migrations(data_dir: &Path) -> Result<Vec<DateMigration>, StoreError> {
    let mut found = Vec::new();
    for date in dates(data_dir)? {
        let folder = DateFolder::new(data_dir, date);
        let journal = Journal::open(&folder)?;
        if journal.damage().is_some() || journal.fold().last_seq == 0 {
            continue;
        }
        let older_views = views_schema(folder.dir()).filter(|schema| schema != SESSION_SCHEMA);
        let batches = areas_to_read(&folder, journal.fold());
        if older_views.is_none() && batches.is_empty() {
            continue;
        }
        let orders = batches
            .iter()
            .map(|batch| orders_without_area(journal.fold(), *batch))
            .sum();
        found.push(DateMigration {
            date,
            older_views,
            batches,
            orders,
        });
    }
    Ok(found)
}

/// The live batches with an order saved before its area was read whose label PDFs are all in the
/// date folder: the only ones the migration can read.
pub fn areas_to_read(folder: &DateFolder, fold: &Fold) -> Vec<BatchId> {
    fold.live
        .iter()
        .filter(|(batch, live)| {
            orders_without_area(fold, **batch) > 0
                && live
                    .saved
                    .inputs
                    .iter()
                    .all(|input| input_path(folder, **batch, input.pdf, &input.sha256).is_file())
        })
        .map(|(batch, _)| *batch)
        .collect()
}

fn orders_without_area(fold: &Fold, batch: BatchId) -> usize {
    fold.live[&batch]
        .saved
        .orders
        .iter()
        .filter(|order| order.area.is_none())
        .count()
}
