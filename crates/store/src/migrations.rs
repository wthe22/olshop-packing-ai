//! What the owner's migration would change in the data folder (journal.md, *When a date opens*):
//! date folders of v0.7 to make flat, views of an older `schema` or still named `session.toml`, and
//! live batches saved before the receiver's area or the weight was read. Finding them reads only; a
//! damaged journal is passed over, as nothing may be appended to it.

use std::path::Path;

use jiff::civil::Date;
use packing_engine::ids::BatchId;

use crate::StoreError;
use crate::date::DateFolder;
use crate::dates::dates;
use crate::flatten::is_v0_7;
use crate::fold::Fold;
use crate::inputs::input_path;
use crate::journal::Journal;
use crate::views::older_views;

/// A date the migration would change.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DateMigration {
    pub date: Date,
    /// Its files are still in v0.7's `input/` and `output/` (or a conversion stopped).
    pub v0_7: bool,
    /// The `schema` of its views when they are written again: a `session.toml`, or an older
    /// `day.toml`.
    pub older_views: Option<String>,
    /// Its batches whose labels would be read for the area or the weight, and how many orders that is.
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
        let v0_7 = is_v0_7(folder.dir());
        let older_views = older_views(folder.dir());
        // A date of v0.7 has its label PDFs in the date folder once it is flat.
        let batches = if v0_7 {
            unread(journal.fold()).collect()
        } else {
            batches_to_read(&folder, journal.fold())
        };
        if !v0_7 && older_views.is_none() && batches.is_empty() {
            continue;
        }
        let orders = batches
            .iter()
            .map(|batch| orders_unread(journal.fold(), *batch))
            .sum();
        found.push(DateMigration {
            date,
            v0_7,
            older_views,
            batches,
            orders,
        });
    }
    Ok(found)
}

/// The live batches with an order saved before its area or weight was read whose label PDFs are
/// all in the date folder: the only ones the migration can read.
pub fn batches_to_read(folder: &DateFolder, fold: &Fold) -> Vec<BatchId> {
    unread(fold)
        .filter(|batch| {
            fold.live[batch]
                .saved
                .inputs
                .iter()
                .all(|input| input_path(folder, *batch, input.pdf, &input.sha256).is_file())
        })
        .collect()
}

/// The live batches with an order saved before its area or weight was read.
fn unread(fold: &Fold) -> impl Iterator<Item = BatchId> + '_ {
    fold.live
        .keys()
        .copied()
        .filter(|batch| orders_unread(fold, *batch) > 0)
}

fn orders_unread(fold: &Fold, batch: BatchId) -> usize {
    fold.live[&batch]
        .saved
        .orders
        .iter()
        .filter(|order| order.area.is_none() || order.weight.is_none())
        .count()
}
