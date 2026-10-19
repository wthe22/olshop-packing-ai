//! The logo prompt: the unseen logos and their pictures as the prompt shows them, and the owner's
//! answer applied to the open draft and written into `couriers.toml`.

use std::collections::HashMap;
use std::path::Path;
use std::str::FromStr;

use packing_engine::couriers_edit::CouriersDocument;
use packing_engine::orders::UnseenLogo;
use packing_pdf::batch::Draft;
use packing_pdf::worker::PdfWorker;

use crate::{AppState, BUILTIN_COURIERS, COURIERS_FILE, OpenDraft};

use super::plan_view::plan_view;
use super::views::{LogoAnswerInput, LogoView, ReadResult};

/// The screen's read result (08 › *2. New batch*, *The logo prompt*, *3. Plan*): the draft's
/// counts and warnings, the couriers the prompt's buttons show, the unseen logos still to ask
/// about, and the plan view. Used both when a read finishes and after every logo answer, so the
/// screen always shows the current counts and warnings.
pub(super) fn read_result(open: &OpenDraft) -> ReadResult {
    ReadResult {
        files: open.draft.files.len(),
        pages: open.draft.pages,
        orders: open.draft.orders.len(),
        total_orders: open.draft.total_orders,
        already_saved: open.draft.duplicates.len(),
        warnings: open.draft.warnings.clone(),
        amend: open.amend.as_ref().map(|amend| amend.batch),
        // The prompt's buttons: the names of the data folder's `couriers.toml`, in file order
        // (D-LABEL-1, D-LABEL-4).
        couriers: open.couriers.names(),
        logos: logo_views(&open.draft, &open.logo_pictures),
        plan: plan_view(&open.plan, &open.draft.orders, open.first_number, &open.set),
    }
}

/// The unseen logos as the prompt shows them, in first-seen order (D-LABEL-4).
fn logo_views(draft: &Draft, pictures: &HashMap<String, Option<String>>) -> Vec<LogoView> {
    draft
        .unseen_logos
        .iter()
        .map(|entry| LogoView {
            hash: entry.hash.clone(),
            pages: entry.pages,
            orders: entry.orders(),
            picture: pictures.get(&entry.hash).cloned().flatten(),
        })
        .collect()
}

/// Read each unseen logo back from the first page carrying it, as a PNG data URL for the prompt
/// (08 › *The logo prompt*). Runs on the PDFium worker, so the label documents stay on their
/// thread. A logo whose page cannot be found or rendered gets `None`: the screen shows its hash.
pub(super) fn read_pictures(worker: &PdfWorker, draft: &Draft) -> HashMap<String, Option<String>> {
    let mut pictures = HashMap::with_capacity(draft.unseen_logos.len());
    for entry in &draft.unseen_logos {
        pictures.insert(entry.hash.clone(), first_picture(worker, draft, entry));
    }
    pictures
}

/// The logo as printed on the first page of the first order carrying it (the batch's kept orders,
/// so an already-saved order's page is not read).
fn first_picture(worker: &PdfWorker, draft: &Draft, entry: &UnseenLogo) -> Option<String> {
    for order in &draft.orders {
        if !entry.order_ids.iter().any(|id| id == &order.order.order_id) {
            continue;
        }
        for page in &order.pages {
            if let Ok(Some(png)) = worker.logo_picture(page.file, page.page, &entry.hash) {
                return Some(data_url(&png));
            }
        }
    }
    None
}

/// A PNG as a data URL (`data:image/png;base64,…`), for the prompt's `<img>`.
pub(super) fn data_url(png: &[u8]) -> String {
    format!("data:image/png;base64,{}", base64(png))
}

/// Standard base64 (RFC 4648, with padding). The app has no base64 dependency and the value is
/// one capped PNG of a logo.
pub(super) fn base64(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let b = [
            chunk[0],
            chunk.get(1).copied().unwrap_or(0),
            chunk.get(2).copied().unwrap_or(0),
        ];
        let n = (u32::from(b[0]) << 16) | (u32::from(b[1]) << 8) | u32::from(b[2]);
        out.push(ALPHABET[(n >> 18) as usize & 63] as char);
        out.push(ALPHABET[(n >> 12) as usize & 63] as char);
        out.push(if chunk.len() > 1 {
            ALPHABET[(n >> 6) as usize & 63] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            ALPHABET[n as usize & 63] as char
        } else {
            '='
        });
    }
    out
}

/// Answer the logo prompt about one unseen logo (08 › *2. New batch* → *The logo prompt*;
/// D-LABEL-4): the order's courier is filled where the logo's courier and the tracking-ID form
/// agree, the draft's warnings and the plan view are recounted, and the refreshed result goes
/// back to the screen. The taught hash is then written into the data folder's `couriers.toml`
/// (Q-LABEL-7 default) so the logo is never asked about again. An unknown hash changes nothing.
/// *Skip all* never comes here: it only stops the screen asking.
pub(super) fn answer_logo_impl(
    state: &AppState,
    hash: &str,
    answer: &LogoAnswerInput,
) -> Result<ReadResult, String> {
    let folder = state.data_folder()?;
    // Only a logo the draft really asked about is remembered; an unknown hash is written nowhere.
    let mut asked = false;
    let result = state.with_draft(|open| {
        asked = open
            .draft
            .unseen_logos
            .iter()
            .any(|entry| entry.hash == hash);
        open.draft
            .answer_logo(hash, &answer.to_engine(), &open.couriers);
        // The answered logo is spent; its picture is no longer needed.
        open.logo_pictures.remove(hash);
        read_result(open)
    })?;
    if asked {
        remember_logo(&folder, hash, answer)?;
    }
    Ok(result)
}

/// Write the hash the owner taught into the data folder's `couriers.toml`, comments and order
/// kept (D-LABEL-4, Q-LABEL-7 default): a courier name appends it to that courier's `logo` list,
/// *Not a courier* appends it to `not_courier`, *I don't know* (`Skip`) writes nothing. A file
/// the data folder does not have yet starts as the built-in list.
fn remember_logo(folder: &Path, hash: &str, answer: &LogoAnswerInput) -> Result<(), String> {
    if matches!(answer, LogoAnswerInput::Skip) {
        return Ok(());
    }
    let path = folder.join(COURIERS_FILE);
    let text = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => BUILTIN_COURIERS.to_string(),
        Err(error) => return Err(format!("cannot read {}: {error}", path.display())),
    };
    let mut document = CouriersDocument::from_str(&text).map_err(|error| error.to_string())?;
    match answer {
        LogoAnswerInput::Courier { name } => document.add_courier_logo(name, hash),
        LogoAnswerInput::NotCourier => document.add_not_courier(hash),
        LogoAnswerInput::Skip => Ok(()),
    }
    .map_err(|error| error.to_string())?;
    std::fs::write(&path, document.to_string())
        .map_err(|error| format!("Cannot write to {}: {error}.", path.display()))
}
