//! Steps 16 and 17 (05-after-save.md): the Printed tick and the revert of the newest live batch,
//! each one append; the views follow.

use packing_engine::ids::BatchId;
use packing_engine::record::{BatchReverted, PrintedMarked, RecordBody};
use packing_store::{
    DateFolder, Journal, discard_partial, move_aside, settle_partial, write_views,
};

use crate::BatchError;

/// Mark a label group of a live batch as printed, or not. A note only: nothing depends on it.
pub fn set_printed(
    folder: &DateFolder,
    journal: &mut Journal,
    mark: PrintedMarked,
    app: &str,
) -> Result<(), BatchError> {
    journal.commit(vec![RecordBody::PrintedMarked(mark)], app)?;
    write_views(folder.dir(), journal.fold())?;
    Ok(())
}

/// Revert the newest live batch: its outputs moved aside, `batch_reverted` appended, then the
/// outputs deleted. A file Windows refuses to move stops it with the file named, and the date is
/// as before.
pub fn revert(
    folder: &DateFolder,
    journal: &mut Journal,
    app: &str,
) -> Result<BatchId, BatchError> {
    let dir = folder.dir();
    let fold = journal.fold();
    let batch = fold.newest_live().ok_or(BatchError::NoBatch)?;
    let files: Vec<String> = fold.live[&batch]
        .saved
        .outputs
        .iter()
        .map(|output| output.file.clone())
        .collect();
    settle_partial(dir, fold)?;
    move_aside(dir, &files)?;
    let reverted = RecordBody::BatchReverted(BatchReverted { batch });
    if let Err(error) = journal.commit(vec![reverted], app) {
        settle_partial(dir, journal.fold())?;
        return Err(error.into());
    }
    discard_partial(dir)?;
    write_views(dir, journal.fold())?;
    Ok(batch)
}
