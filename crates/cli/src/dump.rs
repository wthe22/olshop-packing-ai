//! `packing-cli dump`: what reading the label files gives, as JSON, with nothing saved — the
//! local baseline of the owner's sample files is made of it. Read with the bundled rules, whatever
//! the program folder holds, so the same files always give the same text.

use std::path::PathBuf;
use std::sync::atomic::AtomicBool;

use packing_engine::copies::{Copy, read_copies};
use packing_engine::workflows::{Workflows, courier_name, load_bundled};
use packing_pdf::read;
use packing_pdf::worker::PdfWorker;
use serde_json::{Value, json};

use crate::error::CliError;
use crate::options::{Options, pdfium_dir};

pub fn run(options: &Options) -> Result<(), CliError> {
    let labels = options.labels_in_order()?;
    let worker = PdfWorker::spawn(pdfium_dir(options.pdfium.as_deref()))?;
    print!("{}", dump_json(&worker, &labels)?);
    Ok(())
}

/// The files, the counts, every copy, the header images the bundled rules do not know and the
/// read warnings, as pretty JSON.
pub fn dump_json(worker: &PdfWorker, paths: &[PathBuf]) -> Result<String, CliError> {
    let workflows = load_bundled();
    let rules = workflows.clone();
    let owned = paths.to_vec();
    let pages = worker.run(move |pdfium| {
        read::read_all(
            pdfium,
            &owned,
            &AtomicBool::new(false),
            &rules,
            &mut |_, _| {},
        )
    })??;
    let names: Vec<String> = paths
        .iter()
        .map(|path| {
            path.file_name()
                .map_or_else(String::new, |n| n.to_string_lossy().into_owned())
        })
        .collect();
    let name_refs: Vec<&str> = names.iter().map(String::as_str).collect();
    let files: Vec<Value> = names
        .iter()
        .enumerate()
        .map(|(at, name)| {
            let count = pages.iter().filter(|(file, _, _)| *file == at).count();
            json!({ "name": name, "pages": count })
        })
        .collect();
    let page_count = pages.len();
    let read =
        read_copies(pages, &name_refs, &workflows).map_err(|stop| CliError::Batch(stop.into()))?;
    let copies: Vec<Value> = read
        .copies
        .iter()
        .map(|copy| copy_json(copy, &workflows))
        .collect();
    let warnings: Vec<String> = read
        .copies
        .iter()
        .filter_map(|copy| {
            let (printed, sum) = copy.total_differs()?;
            Some(format!(
                "order {}: the label prints a total of {printed}, its lines add up to {sum}",
                copy.order_id
            ))
        })
        .collect();
    let unseen_logos: Vec<&str> = read
        .unseen_logos
        .iter()
        .map(|logo| logo.hash.as_str())
        .collect();
    let value = json!({
        "files": files,
        "pages": page_count,
        "copies": copies,
        "unseen_logos": unseen_logos,
        "warnings": warnings,
    });
    let mut text = serde_json::to_string_pretty(&value).expect("JSON of plain values");
    text.push('\n');
    Ok(text)
}

fn copy_json(copy: &Copy, workflows: &Workflows) -> Value {
    let pages: Vec<[usize; 2]> = (copy.page_start..=copy.page_end)
        .map(|page| [copy.input, usize::from(page)])
        .collect();
    let lines: Vec<Value> = copy
        .lines
        .iter()
        .map(|line| json!([line.name, line.variation, line.seller_sku, line.quantity]))
        .collect();
    json!({
        "order_id": copy.order_id.as_str(),
        "pages": pages,
        "tracking_id": copy.tracking_id.as_ref().map(|id| id.as_str()),
        "courier": copy
            .courier
            .as_ref()
            .map(|code| courier_name(&workflows.couriers, code.as_str())),
        "ship_by_date": copy.ship_by_date.map(|date| date.to_string()),
        "sender": copy.sender.as_ref().map(|sender| sender.name.as_str()),
        "lines": lines,
        "qty_total": copy.total_quantity,
    })
}

#[cfg(test)]
mod tests {
    use std::fs;

    use serde_json::Value;
    use testkit::{testdata, vendor_dir};

    use super::{PdfWorker, dump_json};

    /// Every order of the fixture with the lines, total, tracking ID and page count of its answer,
    /// and the wrong printed total warned.
    #[test]
    fn the_fixture_dumps_as_its_answer() {
        let Some(vendor) = vendor_dir() else {
            eprintln!("skipped: pdfium.dll not downloaded (tools/get-pdfium.sh)");
            return;
        };
        let _guard = testkit::pdfium_lock();
        let worker = PdfWorker::spawn(vendor).expect("spawn worker");
        let text = dump_json(&worker, &[testdata("labels-slip.pdf")]).expect("dump");
        let dump: Value = serde_json::from_str(&text).expect("JSON");
        let answer: Value = serde_json::from_str(
            &fs::read_to_string(testdata("expected-labels.json")).expect("read the answer"),
        )
        .expect("parse the answer");

        assert_eq!(dump["files"][0]["pages"], 24);
        let copies = dump["copies"].as_array().expect("copies");
        assert_eq!(copies.len(), 23);
        for copy in copies {
            let id = copy["order_id"].as_str().expect("order_id");
            let want = &answer[id];
            for field in ["lines", "qty_total", "tracking_id"] {
                assert_eq!(copy[field], want[field], "{id}: {field}");
            }
            assert_eq!(
                copy["pages"].as_array().unwrap().len(),
                want["pages"],
                "{id}"
            );
        }
        let warnings = dump["warnings"].as_array().expect("warnings");
        assert!(
            warnings
                .iter()
                .any(|warning| warning.as_str().unwrap().contains("580000000000000901")),
            "the wrong printed total is warned: {warnings:?}"
        );
    }
}
