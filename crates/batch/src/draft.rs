//! A batch read but not saved (steps 4–10): the chosen files read into copies, the copies into
//! orders by the already-saved guard and the repeat choices, and the orders into the plan's label
//! groups. A choice works out the orders again from the copies.

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::Sender;

use packing_engine::batch_record::{read_findings, saved_orders};
use packing_engine::copies::{Copy, read_copies};
use packing_engine::ids::{BatchId, OrderId};
use packing_engine::label::courier_data;
use packing_engine::orders::{PageItem, UnseenLogo};
use packing_engine::record::{Finding, InputPdf};
use packing_engine::repeats::{Choices, Orders, sort_out};
use packing_engine::sorting::{PlanGroup, SortedBatch, plan_groups, sort_batch};
use packing_engine::workflows::{Courier, SortRule, Workflows};
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
    /// Every page as read, so the copies can be read again with new rules.
    pub pages: Vec<PageItem>,
    pub copies: Vec<Copy>,
    pub unseen_logos: Vec<UnseenLogo>,
    /// The orders of the date's other live batches.
    pub saved: BTreeMap<OrderId, BatchId>,
    pub choices: Choices,
    pub orders: Orders,
    pub rule_name: String,
    pub plan: Vec<PlanGroup>,
    /// The couriers of the rules the copies were read with: the summary PDF names them.
    pub couriers: Vec<Courier>,
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
        sort_batch(
            &self.copies,
            &self.orders.kept,
            &self.plan,
            batch,
            &self.couriers,
        )
    }

    /// The inputs as a save on `fold`'s date numbers them: bytes the date knows keep their
    /// number, new ones take the next, in reading order.
    pub fn inputs(&self, fold: &Fold) -> Vec<InputPdf> {
        let mut next = fold.next_pdf();
        self.files
            .iter()
            .map(|file| {
                let pdf = fold.pdf_of(&file.sha256).unwrap_or_else(|| {
                    let pdf = next;
                    next = next.next();
                    pdf
                });
                InputPdf {
                    pdf,
                    sha256: file.sha256.clone(),
                    filename: file.filename.clone(),
                    size: file.size,
                    page_count: file.page_count,
                    create_time: file.create_time.clone(),
                    modify_time: file.modify_time.clone(),
                }
            })
            .collect()
    }

    /// The read's warnings of the orders kept.
    pub fn findings(&self, fold: &Fold) -> Vec<Finding> {
        read_findings(&self.copies, &self.orders.kept, &self.inputs(fold))
    }

    /// Read the copies again with `workflows` (a logo answered): the owner's choices stay where
    /// they still apply.
    pub fn reclassify(&mut self, workflows: &Workflows) -> Result<(), BatchError> {
        let names: Vec<&str> = self.files.iter().map(|f| f.filename.as_str()).collect();
        let read = read_copies(self.pages.clone(), &names, workflows)?;
        self.copies = read.copies;
        self.unseen_logos = read.unseen_logos;
        self.couriers = workflows.couriers.clone();
        self.orders = match sort_out(&self.copies, &self.saved, &self.choices) {
            Ok(orders) => orders,
            Err(_) => {
                self.choices = Choices::default();
                sort_out(&self.copies, &self.saved, &self.choices)?
            }
        };
        Ok(())
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
    let read = read_copies(pages.clone(), &names, request.workflows)?;
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
        pages,
        copies: read.copies,
        unseen_logos: read.unseen_logos,
        saved,
        choices,
        orders,
        rule_name: request.rule.name.clone(),
        plan: plan_groups(request.rule),
        couriers: request.workflows.couriers.clone(),
        redo: request.redo,
    })
}
