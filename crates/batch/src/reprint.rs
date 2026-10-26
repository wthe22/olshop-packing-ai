//! Step 19 (reprint.md): the chosen saved orders' label pages, unchanged and in their original
//! printing order, written into today's date as one file from the stored inputs, verified, then
//! recorded as `reprint_made`. No batch changes.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use jiff::civil::Date;
use packing_engine::ids::OrderId;
use packing_engine::record::{RecordBody, ReprintMade, ReprintedOrder};
use packing_pdf::worker::PdfWorker;
use packing_store::{
    DateFolder, Fold, Journal, StoreError, discard_partial, input_path, move_forward,
    settle_partial, sha256_of, write_views,
};
use packing_verify::{Findings, ReprintCheck, verify_reprint};

use crate::BatchError;
use crate::written::{copy_pages, described, partial};

pub struct ReprintRequest<'a> {
    pub worker: &'a PdfWorker,
    pub data_dir: &'a Path,
    /// Today's date of the workspace: the reprint is written and recorded there.
    pub today: &'a DateFolder,
    /// Each order with the date it was saved on.
    pub orders: &'a [(Date, OrderId)],
    pub app: &'a str,
}

#[derive(Debug)]
pub enum ReprintOutcome {
    Made(ReprintMade),
    /// The verifier found something: nothing was kept.
    Findings(Findings),
}

/// A chosen order as its live batch wrote it, and where it sits in the printing order.
struct Source {
    place: (Date, u8, usize),
    order: ReprintedOrder,
    path: PathBuf,
}

/// Write, verify and record a reprint; `journal` is today's.
pub fn reprint(
    request: &ReprintRequest<'_>,
    journal: &mut Journal,
) -> Result<ReprintOutcome, BatchError> {
    let workspace = request.today.workspace();
    let mut sources = sources(request, journal.fold())?;
    sources.sort_by_key(|source| source.place);

    let fold = journal.fold().clone();
    let dir = request.today.dir();
    settle_partial(dir, &fold)?;
    let number = u8::try_from(fold.reprints.len() + 1).map_err(|_| BatchError::TooManyReprints)?;
    let out = partial(dir)?.join(format!("reprint {number} x{}.pdf", sources.len()));
    let mut paths: Vec<PathBuf> = Vec::new();
    let mut pages = Vec::new();
    for source in &sources {
        let at = paths
            .iter()
            .position(|p| *p == source.path)
            .unwrap_or_else(|| {
                paths.push(source.path.clone());
                paths.len() - 1
            });
        let order = &source.order;
        pages.extend((order.page_start..=order.page_end).map(|p| (at, p as usize - 1)));
    }
    let count = pages.len();
    if let Err(error) = copy_pages(request.worker, paths, pages, out.clone()) {
        discard_partial(dir)?;
        return Err(error);
    }
    let file = described(&out, None, count)?;
    let made = ReprintMade {
        number,
        file: file.file,
        pages: file.pages,
        size: file.size,
        sha256: file.sha256,
        orders: sources.into_iter().map(|source| source.order).collect(),
    };

    let pending = journal.prepare(vec![RecordBody::ReprintMade(made.clone())], request.app)?;
    let findings = verify_reprint(&ReprintCheck {
        journal: &journal.bytes()?,
        append: &pending.bytes,
        workspace_dir: &request.data_dir.join(workspace),
        output_dir: &packing_store::partial_dir(dir),
    });
    if !findings.is_clean() {
        discard_partial(dir)?;
        return Ok(ReprintOutcome::Findings(findings));
    }
    if let Err(error) = journal.append_pending(pending) {
        settle_partial(dir, journal.fold())?;
        return Err(error.into());
    }
    move_forward(dir)?;
    write_views(dir, journal.fold())?;
    Ok(ReprintOutcome::Made(made))
}

/// Each chosen order from its date's fold (not the index): the copy its live batch wrote, whose
/// input must still be the bytes the journal names.
fn sources(request: &ReprintRequest<'_>, today: &Fold) -> Result<Vec<Source>, BatchError> {
    let mut by_date: BTreeMap<Date, Vec<&OrderId>> = BTreeMap::new();
    for (date, order) in request.orders {
        by_date.entry(*date).or_default().push(order);
    }
    let mut sources = Vec::new();
    for (date, orders) in by_date {
        let folder = DateFolder::new(request.data_dir, request.today.workspace(), date);
        let opened;
        let fold = if date == today.date {
            today
        } else {
            opened = Journal::open(&folder)?;
            if let Some(damage) = opened.damage() {
                return Err(StoreError::Damaged(damage.clone()).into());
            }
            opened.fold()
        };
        for order_id in orders {
            sources.push(source(fold, &folder, order_id)?);
        }
    }
    Ok(sources)
}

fn source(fold: &Fold, folder: &DateFolder, order_id: &OrderId) -> Result<Source, BatchError> {
    let not_saved = || BatchError::NotSaved {
        order_id: order_id.clone(),
        date: fold.date,
    };
    let (batch, place, saved) = fold
        .live
        .iter()
        .find_map(|(batch, live)| {
            let place = live
                .saved
                .orders
                .iter()
                .position(|o| &o.order_id == order_id)?;
            Some((*batch, place, &live.saved))
        })
        .ok_or_else(not_saved)?;
    let order = &saved.orders[place];
    let input = saved
        .inputs
        .iter()
        .find(|input| input.pdf == order.pdf)
        .ok_or_else(not_saved)?;
    let path = input_path(folder.dir(), &input.sha256);
    if !path.is_file() || sha256_of(&path)? != input.sha256 {
        return Err(BatchError::InputUnusable {
            file: input.filename.clone(),
            date: fold.date,
        });
    }
    Ok(Source {
        place: (fold.date, batch.get(), place),
        order: ReprintedOrder {
            order_id: order_id.clone(),
            date: fold.date,
            batch,
            sha256: input.sha256.clone(),
            page_start: order.page_start,
            page_end: order.page_end,
        },
        path,
    })
}
