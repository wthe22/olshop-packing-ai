//! Step 18 (05-after-save.md): what an amend reads — the newest live batch's inputs from `input/`
//! in the recorded order, with the sort rule of the batch's name as it is now, else the batch's own
//! snapshot — and, once read, each repeat's copy kept as the record has it.

use packing_engine::ids::BatchId;
use packing_engine::record::{BatchSaved, LabelSortRule, LeftOutReason};
use packing_engine::rules::parse_filter;
use packing_engine::workflows::{Filter, GroupRule, SortRule, Workflows};
use packing_store::{DateFolder, Fold, input_path};

use crate::files::used_in;
use crate::{BatchError, ChosenFile, Draft};

pub struct Amend {
    pub batch: BatchId,
    /// The batch's inputs in `input/`, in reading order.
    pub files: Vec<ChosenFile>,
    pub rule: SortRule,
}

pub fn amend(folder: &DateFolder, fold: &Fold, workflows: &Workflows) -> Result<Amend, BatchError> {
    let batch = fold.newest_live().ok_or(BatchError::NoBatch)?;
    let saved = &fold.live[&batch].saved;
    let files = saved
        .inputs
        .iter()
        .map(|input| ChosenFile {
            path: input_path(folder.dir(), &input.sha256),
            filename: input.filename.clone(),
            sha256: input.sha256.clone(),
            size: input.size,
            page_count: input.page_count,
            create_time: input.create_time.clone(),
            modify_time: input.modify_time.clone(),
            used_in: used_in(fold, &input.sha256, Some(batch)),
        })
        .collect();
    let rule = match workflows.sort_rule(&saved.sort_rule.name) {
        Some(rule) => rule.clone(),
        None => snapshot_rule(&saved.sort_rule)?,
    };
    Ok(Amend { batch, files, rule })
}

fn snapshot_rule(snapshot: &LabelSortRule) -> Result<SortRule, BatchError> {
    let groups = snapshot
        .label_groups
        .iter()
        .map(|group| {
            let filter = match &group.filter {
                Some(text) => Some(Filter {
                    text: text.clone(),
                    condition: parse_filter(text).map_err(BatchError::Rule)?,
                }),
                None => None,
            };
            Ok(GroupRule {
                code: group.code.clone(),
                name: group.name.clone(),
                filter,
            })
        })
        .collect::<Result<_, BatchError>>()?;
    Ok(SortRule {
        name: snapshot.name.clone(),
        enabled: true,
        groups,
    })
}

impl Draft {
    /// For each repeat `record` left out, keep the copy it kept, where that copy was read again.
    pub fn keep_as_recorded(&mut self, record: &BatchSaved) -> Result<(), BatchError> {
        let pdf_of = |input: usize| {
            record
                .inputs
                .iter()
                .find(|known| known.sha256 == self.files[input].sha256)
                .map(|known| known.pdf)
        };
        let mut choices = self.choices.clone();
        for left in &record.left_out {
            let LeftOutReason::Repeat { pdf, page_start } = left.reason else {
                continue;
            };
            let kept = self.copies.iter().position(|copy| {
                copy.order_id == left.order_id
                    && copy.page_start == page_start
                    && pdf_of(copy.input) == Some(pdf)
            });
            if let Some(kept) = kept {
                choices.by_hand.insert(left.order_id.clone(), kept);
            }
        }
        self.choose(choices)
    }
}
