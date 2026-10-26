//! A reprint (reprint.md): V2 for every input it copies from, in its own date's folder, and V5 for
//! its file — page *k* has the digest of the source page the record puts at *k*, and the page
//! count adds up.

use std::collections::BTreeMap;
use std::path::Path;

use lopdf::Document;
use serde::Deserialize;

use crate::Findings;
use crate::digest::Digester;
use crate::files::{output_pages, sha256_and_size};
use crate::record::Output;

#[derive(Deserialize)]
struct Reprint {
    file: String,
    pages: u16,
    size: u64,
    sha256: String,
    orders: Vec<Reprinted>,
}

#[derive(Deserialize)]
struct Reprinted {
    order_id: String,
    date: String,
    sha256: String,
    page_start: u16,
    page_end: u16,
}

pub(crate) fn check_reprint(
    json: &[u8],
    workspace_dir: &Path,
    output_dir: &Path,
    findings: &mut Findings,
) {
    let Ok(reprint) = serde_json::from_slice::<Reprint>(json) else {
        findings.error("the reprint's record cannot be read".to_string());
        return;
    };
    let mut inputs: BTreeMap<(&str, &str), Option<Vec<u64>>> = BTreeMap::new();
    let mut due: Vec<(Option<u64>, String)> = Vec::new();
    for order in &reprint.orders {
        let pages = inputs
            .entry((order.date.as_str(), order.sha256.as_str()))
            .or_insert_with(|| input_pages(workspace_dir, order, findings));
        for page in order.page_start..=order.page_end {
            let digest = pages
                .as_ref()
                .and_then(|pages| pages.get(page as usize - 1))
                .copied();
            let what = format!(
                "page {page} of the input of order {} ({})",
                order.order_id, order.date
            );
            due.push((digest, what));
        }
    }
    if due.len() != reprint.pages as usize {
        findings.error(format!(
            "the reprint's orders have {} pages, its record says {}",
            due.len(),
            reprint.pages
        ));
    }
    let output = Output {
        file: reprint.file.clone(),
        group: None,
        pages: reprint.pages,
        size: reprint.size,
        sha256: reprint.sha256.clone(),
    };
    let found = output_pages(&output, output_dir, findings);
    for (at, (digest, what)) in due.iter().enumerate() {
        match (digest, found.get(at)) {
            (None, _) => {}
            (Some(_), None) => findings.error(format!("{what} is not in {}", reprint.file)),
            (Some(due), Some(found)) if due == found => {}
            (Some(_), Some(_)) => {
                findings.error(format!("page {} of {} is not {what}", at + 1, reprint.file))
            }
        }
    }
}

/// The page digests of an order's input, after V2.
fn input_pages(
    workspace_dir: &Path,
    order: &Reprinted,
    findings: &mut Findings,
) -> Option<Vec<u64>> {
    let path = workspace_dir
        .join(&order.date)
        .join("input")
        .join(format!("{}.pdf", order.sha256));
    let input = format!("the input of order {} ({})", order.order_id, order.date);
    match sha256_and_size(&path) {
        None => {
            findings.error(format!("{input} is not in its date's inputs"));
            return None;
        }
        Some((sha, _)) if sha != order.sha256 => {
            findings.error(format!("{input} has changed"));
            return None;
        }
        Some(_) => {}
    }
    let Ok(doc) = Document::load(&path) else {
        findings.error(format!("{input} cannot be read as a PDF"));
        return None;
    };
    Some(Digester::new(&doc).pages())
}
