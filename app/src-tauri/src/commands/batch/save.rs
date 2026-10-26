//! *Save batch* through `packing-batch` on the date the draft was read for: saved, the draft is
//! forgotten; stopped by the verifier, the draft stays for *Save anyway* or *Cancel*.

use packing_batch::{SaveOutcome, SaveProgress, SaveRequest, save};

use crate::{APP_VERSION, AppState};

use super::views::SaveResultView;

pub(super) fn save_draft(
    state: &AppState,
    save_anyway: bool,
    on_progress: &dyn Fn(SaveProgress),
) -> Result<SaveResultView, String> {
    let (batch, outcome) = state.with_draft(|open| {
        state.with_data(|data| {
            let day = data.open_day(open.date)?;
            let batch = open.draft.batch(day.journal.fold());
            let outcome = state
                .with_worker(|worker| {
                    save(
                        &SaveRequest {
                            worker,
                            folder: &day.folder,
                            draft: &open.draft,
                            save_anyway,
                            app: APP_VERSION,
                            progress: on_progress,
                        },
                        &mut day.journal,
                    )
                })?
                .map_err(|error| error.to_string())?;
            if let SaveOutcome::Saved(saved) = &outcome {
                day.outputs
                    .retain(|problem| problem.batch != saved.record.batch);
                state.index().refresh(day.journal.fold());
            }
            Ok((batch, outcome))
        })
    })?;
    Ok(match outcome {
        SaveOutcome::Saved(saved) => {
            state.clear_draft();
            let record = saved.record;
            SaveResultView {
                batch: record.batch,
                pdfs: record.outputs.iter().filter(|o| o.group.is_some()).count(),
                saved: true,
                errors: record.errors.iter().map(|f| f.text.clone()).collect(),
                warnings: Vec::new(),
                after_save: saved.after_commit.iter().map(ToString::to_string).collect(),
            }
        }
        SaveOutcome::Findings(findings) => SaveResultView {
            batch,
            pdfs: 0,
            saved: false,
            errors: findings.errors,
            warnings: findings.warnings,
            after_save: Vec::new(),
        },
    })
}
