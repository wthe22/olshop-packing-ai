//! The Summary screen (ui/summary.md): a batch's or the date's summary as its parts, read from the
//! open date and written nowhere, so it works read-only and on an archived date.

use packing_batch::{BatchSummaryRequest, batch_summary_parts, day_summary_parts};
use packing_engine::ids::BatchId;
use packing_pdf::summary_view::SummaryView;
use tauri::{AppHandle, Manager};

use super::date_of;
use crate::{APP_VERSION, AppState, settings};

/// The summary of `batch`, or of the date when none, of all accounts or of `account`.
pub(crate) fn summary_of(
    state: &AppState,
    day: &str,
    batch: Option<BatchId>,
    account: Option<&str>,
) -> Result<SummaryView, String> {
    let date = date_of(day)?;
    let (_, workflows) = settings::workflows(state.program())?;
    state.with_data(|data| {
        let fold = data.open_day(date)?.journal.fold();
        let (couriers, accounts) = (&workflows.couriers, &workflows.accounts);
        match batch {
            Some(batch) => batch_summary_parts(&BatchSummaryRequest {
                fold,
                batch,
                account,
                couriers,
                accounts,
                app: APP_VERSION,
            }),
            None => day_summary_parts(fold, couriers, accounts, account),
        }
        .map_err(|error| error.to_string())
    })
}

/// **Summary** and **Day summary**: the summary to show, read on its own thread.
#[tauri::command]
pub async fn summary_view(
    app: AppHandle,
    day: String,
    batch: Option<BatchId>,
    account: Option<String>,
) -> Result<SummaryView, String> {
    tauri::async_runtime::spawn_blocking(move || {
        summary_of(&app.state::<AppState>(), &day, batch, account.as_deref())
    })
    .await
    .map_err(|error| format!("the summary task stopped: {error}"))?
}
