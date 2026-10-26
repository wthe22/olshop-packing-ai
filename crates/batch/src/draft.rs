//! A batch read but not saved (steps 4–10): the chosen files read into copies, the copies into
//! orders by the already-saved guard and the repeat choices, and the orders into the plan's label
//! groups. A choice works out the orders again from the copies.

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::Sender;

use packing_engine::batch_record::saved_orders;
use packing_engine::copies::{Copy, read_copies};
use packing_engine::ids::{BatchId, OrderId};
use packing_engine::label::courier_data;
use packing_engine::orders::UnseenLogo;
use packing_engine::repeats::{Choices, Orders, sort_out};
use packing_engine::sorting::{PlanGroup, SortedBatch, plan_groups, sort_batch};
use packing_engine::workflows::{SortRule, Workflows};
use packing_pdf::read;
use packing_pdf::worker::{PdfWorker, Progress};
use packing_store::Fold;

use crate::{BatchError, ChosenFile};

pub struct ReadRequest<'a> {
    pub worker: &'a PdfWorker,
    /// In reading order.
    pub files: Vec<ChosenFile>,
    pub workflows: &'a Workflows,
    pub rule: &'a SortRule,
    pub fold: &'a Fold,
    /// The batch an amend redoes: its own orders are not saved earlier.
    pub redo: Option<BatchId>,
    pub cancel: Arc<AtomicBool>,
    pub progress: Sender<Progress>,
}

#[derive(Clone, Debug)]
pub struct Draft {
    pub files: Vec<ChosenFile>,
    pub copies: Vec<Copy>,
    pub unseen_logos: Vec<UnseenLogo>,
    /// The orders of the date's other live batches.
    pub saved: BTreeMap<OrderId, BatchId>,
    pub choices: Choices,
    pub orders: Orders,
    pub rule_name: String,
    pub plan: Vec<PlanGroup>,
    /// The batch an amend redoes.
    pub redo: Option<BatchId>,
}

impl Draft {
    /// Keep the copies `choices` names; nothing changes when one cannot apply.
    pub fn choose(&mut self, choices: Choices) -> Result<(), BatchError> {
        self.orders = sort_out(&self.copies, &self.saved, &choices)?;
        self.choices = choices;
        Ok(())
    }

    /// The batch's number on a date whose fold is `fold`: the batch it redoes, or the next one.
    pub fn batch(&self, fold: &Fold) -> BatchId {
        self.redo.unwrap_or_else(|| fold.next_batch())
    }

    pub fn sorted(&self, batch: BatchId) -> SortedBatch {
        sort_batch(&self.copies, &self.orders.kept, &self.plan, batch)
    }
}

/// Read the chosen files on the worker and work out the draft. A page that cannot be placed, or a
/// batch whose every order was saved earlier, stops it.
pub fn read_draft(request: ReadRequest<'_>) -> Result<Draft, BatchError> {
    let paths: Vec<PathBuf> = request.files.iter().map(|f| f.path.clone()).collect();
    let cancel = request.cancel;
    let progress = request.progress;
    let pages = request.worker.run(move |pdfium| {
        read::read_all(
            pdfium,
            &paths,
            &cancel,
            courier_data(),
            &mut |page, pages| {
                let _ = progress.send(Progress { page, pages });
            },
        )
    })??;
    let names: Vec<&str> = request.files.iter().map(|f| f.filename.as_str()).collect();
    let read = read_copies(pages, &names, request.workflows)?;
    let saved = saved_orders(
        request
            .fold
            .live
            .iter()
            .filter(|(batch, _)| Some(**batch) != request.redo)
            .map(|(_, live)| &live.saved),
    );
    let choices = Choices::default();
    let orders = sort_out(&read.copies, &saved, &choices)?;
    if orders.kept.is_empty() {
        let ids: BTreeSet<&OrderId> = read.copies.iter().map(|c| &c.order_id).collect();
        return Err(BatchError::AllSavedEarlier { orders: ids.len() });
    }
    Ok(Draft {
        files: request.files,
        copies: read.copies,
        unseen_logos: read.unseen_logos,
        saved,
        choices,
        orders,
        rule_name: request.rule.name.clone(),
        plan: plan_groups(request.rule),
        redo: request.redo,
    })
}
