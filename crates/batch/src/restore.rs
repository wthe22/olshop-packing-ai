//! Step 20 (reprint.md, *Restore*): an output of a live batch that is missing or changed, written
//! again from `input/` and the batch's record — a group PDF by the reprint path, the summary PDF as
//! at the save — verified, then recorded as `output_restored` with its new size and SHA-256.

use std::path::PathBuf;

use packing_engine::record::{OutputRestored, RecordBody};
use packing_pdf::summary_pdf::{SummaryFacts, write_summary};
use packing_pdf::worker::PdfWorker;
use packing_store::{
    DateFolder, Journal, discard_partial, input_path, move_forward, partial_dir, settle_partial,
    write_views,
};
use packing_verify::{Findings, RestoreCheck, verify_restore};

use crate::BatchError;
use crate::written::{copy_pages, described, partial};

pub struct RestoreRequest<'a> {
    pub worker: &'a PdfWorker,
    pub folder: &'a DateFolder,
    /// The output's name in `output/`.
    pub file: &'a str,
    pub app: &'a str,
}

#[derive(Debug)]
pub enum RestoreOutcome {
    Restored(OutputRestored),
    /// The verifier found something: nothing was kept.
    Findings(Findings),
}

pub fn restore(
    request: &RestoreRequest<'_>,
    journal: &mut Journal,
) -> Result<RestoreOutcome, BatchError> {
    let fold = journal.fold().clone();
    let (batch, live) = fold
        .live
        .iter()
        .find(|(_, live)| live.saved.outputs.iter().any(|o| o.file == request.file))
        .ok_or_else(|| BatchError::NoOutput {
            file: request.file.to_string(),
        })?;
    let saved = &live.saved;
    let output = saved
        .outputs
        .iter()
        .find(|o| o.file == request.file)
        .expect("found above");
    let dir = request.folder.dir();
    settle_partial(dir, &fold)?;
    let out = partial(dir)?.join(request.file);

    let written = match &output.group {
        Some(group) => {
            let sources: Vec<PathBuf> = saved
                .inputs
                .iter()
                .map(|input| input_path(dir, &input.sha256))
                .collect();
            let pages: Vec<(usize, usize)> = saved
                .orders
                .iter()
                .filter(|order| &order.group == group)
                .flat_map(|order| {
                    let at = saved.inputs.iter().position(|i| i.pdf == order.pdf);
                    (order.page_start..=order.page_end)
                        .filter_map(move |p| at.map(|at| (at, p as usize - 1)))
                })
                .collect();
            let count = pages.len();
            copy_pages(request.worker, sources, pages, out.clone()).map(|()| count)
        }
        None => {
            let (namespace, date, app) =
                (fold.namespace.clone(), fold.date, request.app.to_string());
            let printed = jiff::Zoned::now().strftime("%H:%M").to_string();
            let (record, target) = (saved.clone(), out.clone());
            request
                .worker
                .run(move |pdfium| {
                    write_summary(
                        pdfium,
                        &target,
                        &SummaryFacts {
                            namespace: &namespace,
                            date,
                            record: &record,
                            printed: &printed,
                            app: &app,
                        },
                    )
                })
                .map_err(BatchError::from)
                .and_then(|pages| pages.map_err(BatchError::from))
        }
    };
    let pages = match written {
        Ok(pages) => pages,
        Err(error) => {
            discard_partial(dir)?;
            return Err(error);
        }
    };
    let file = described(&out, output.group.as_deref(), pages)?;
    let restored = OutputRestored {
        batch: *batch,
        file: file.file,
        pages: file.pages,
        size: file.size,
        sha256: file.sha256,
    };

    let pending = journal.prepare(
        vec![RecordBody::OutputRestored(restored.clone())],
        request.app,
    )?;
    let findings = verify_restore(&RestoreCheck {
        journal: &journal.bytes()?,
        append: &pending.bytes,
        input_dir: &dir.join("input"),
        output_dir: &partial_dir(dir),
    });
    if !findings.is_clean() {
        discard_partial(dir)?;
        return Ok(RestoreOutcome::Findings(findings));
    }
    if let Err(error) = journal.append_pending(pending) {
        settle_partial(dir, journal.fold())?;
        return Err(error.into());
    }
    move_forward(dir)?;
    write_views(dir, journal.fold())?;
    Ok(RestoreOutcome::Restored(restored))
}
