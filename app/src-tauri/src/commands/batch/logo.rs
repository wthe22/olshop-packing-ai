//! The logo prompt (new-batch.md): the header images the rules do not know, each with the logo as
//! printed, and the owner's answer written into `packing-workflows.toml`, after which the draft's
//! copies are read again with the rules it now holds.

use std::collections::HashMap;

use packing_batch::Draft;
use packing_engine::orders::LogoAnswer;
use packing_engine::workflows::{LogoIs, remember_logo};
use packing_pdf::read;
use packing_pdf::worker::PdfWorker;

use crate::settings;
use crate::{AppState, OpenDraft};

use super::read::read_result;
use super::views::{LogoView, ReadResult};

/// The unseen logos the owner has not passed over, in first-seen order.
pub(super) fn logo_views(open: &OpenDraft) -> Vec<LogoView> {
    open.draft
        .unseen_logos
        .iter()
        .filter(|logo| !open.skipped_logos.contains(&logo.hash))
        .map(|logo| LogoView {
            hash: logo.hash.clone(),
            pages: logo.pages,
            orders: logo.orders(),
            picture: open.logo_pictures.get(&logo.hash).cloned().flatten(),
        })
        .collect()
}

/// Each unseen logo read back from the first page carrying it, as a PNG data URL; none when its
/// page cannot be read.
pub(super) fn read_pictures(worker: &PdfWorker, draft: &Draft) -> HashMap<String, Option<String>> {
    draft
        .unseen_logos
        .iter()
        .map(|logo| (logo.hash.clone(), picture(worker, draft, &logo.hash)))
        .collect()
}

fn picture(worker: &PdfWorker, draft: &Draft, hash: &str) -> Option<String> {
    let (file, page, _) = draft
        .pages
        .iter()
        .find(|(_, _, text)| text.logos.iter().any(|logo| logo.hash == hash))?;
    let path = draft.files.get(*file)?.path.clone();
    let (page, hash) = (*page, hash.to_string());
    let png = worker
        .run(move |pdfium| read::open(pdfium, &path)?.band_picture(page, &hash))
        .ok()?
        .ok()??;
    Some(data_url(&png))
}

/// Apply the owner's answer about the image `hash` and return the read as it now is.
pub(super) fn answer_logo(
    state: &AppState,
    hash: &str,
    answer: &LogoAnswer,
) -> Result<ReadResult, String> {
    let program = state.program();
    let (text, mut workflows) = settings::workflows(program)?;
    let logo = match answer {
        LogoAnswer::Courier { name } => Some(LogoIs::Courier(
            workflows
                .couriers
                .iter()
                .find(|courier| &courier.name == name)
                .map(|courier| courier.code.clone())
                .ok_or_else(|| format!("There is no courier \"{name}\"."))?,
        )),
        LogoAnswer::NotCourier => Some(LogoIs::NotCourier),
        LogoAnswer::Skip => None,
    };
    if let Some(logo) = &logo {
        let (written, rules) =
            remember_logo(&text, hash, logo).map_err(|error| error.to_string())?;
        settings::write_workflows(program, &written)?;
        workflows = rules;
    }
    state.with_draft(|open| {
        match logo {
            Some(_) => open
                .draft
                .reclassify(&workflows)
                .map_err(|error| error.to_string())?,
            None => open.skipped_logos.push(hash.to_string()),
        }
        state.with_data(|data| {
            let day = data.open_day(open.date)?;
            Ok(read_result(open, day.journal.fold(), &workflows))
        })
    })
}

/// A PNG as a data URL (`data:image/png;base64,…`), for the prompt's `<img>`.
fn data_url(png: &[u8]) -> String {
    format!("data:image/png;base64,{}", base64(png))
}

/// Standard base64 (RFC 4648, with padding); the app has no base64 dependency for this one use.
fn base64(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let b = [
            chunk[0],
            chunk.get(1).copied().unwrap_or(0),
            chunk.get(2).copied().unwrap_or(0),
        ];
        let n = (u32::from(b[0]) << 16) | (u32::from(b[1]) << 8) | u32::from(b[2]);
        for (i, shift) in [18, 12, 6, 0].into_iter().enumerate() {
            if i <= chunk.len() {
                out.push(ALPHABET[((n >> shift) & 63) as usize] as char);
            } else {
                out.push('=');
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    #[test]
    fn base64_is_the_standard_alphabet_with_padding() {
        for (bytes, text) in [
            (&b""[..], ""),
            (b"f", "Zg=="),
            (b"fo", "Zm8="),
            (b"foo", "Zm9v"),
            (b"foob", "Zm9vYg=="),
            (&[0xfb, 0xff][..], "+/8="),
        ] {
            assert_eq!(super::base64(bytes), text);
        }
    }
}
