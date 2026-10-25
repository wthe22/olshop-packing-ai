//! V2, the inputs as the record names them, and V5, each group PDF page by page: its page *k* must
//! have the digest of the source page the record puts at *k*.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::Read;
use std::path::Path;

use lopdf::Document;
use sha2::{Digest, Sha256};

use crate::Findings;
use crate::digest::Digester;
use crate::record::{Batch, Output};

/// A file's SHA-256 in lower-case hex, and its size.
pub(crate) fn sha256_and_size(path: &Path) -> Option<(String, u64)> {
    let mut file = fs::File::open(path).ok()?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0u8; 1 << 16];
    let mut size = 0u64;
    loop {
        let read = file.read(&mut buffer).ok()?;
        if read == 0 {
            break;
        }
        size += read as u64;
        hasher.update(&buffer[..read]);
    }
    Some((format!("{:x}", hasher.finalize()), size))
}

/// The page digests of each input the record names, by `PdfID`, after V2.
pub(crate) fn check_inputs(
    batch: &Batch,
    input_dir: &Path,
    findings: &mut Findings,
) -> BTreeMap<u8, Vec<u64>> {
    let mut digests = BTreeMap::new();
    for input in &batch.inputs {
        let path = input_dir.join(format!("{}.pdf", input.sha256));
        let changed = |findings: &mut Findings, why: &str| {
            findings.error(format!("{}, an input of the batch, {why}", input.filename))
        };
        match sha256_and_size(&path) {
            None => {
                changed(findings, "is not in the date's inputs");
                continue;
            }
            Some((sha, _)) if sha != input.sha256 => {
                changed(findings, "has changed");
                continue;
            }
            Some(_) => {}
        }
        let Ok(doc) = Document::load(&path) else {
            changed(findings, "cannot be read as a PDF");
            continue;
        };
        let pages = Digester::new(&doc).pages();
        if pages.len() != input.page_count as usize {
            changed(
                findings,
                &format!(
                    "has {} pages, the record says {}",
                    pages.len(),
                    input.page_count
                ),
            );
        }
        digests.insert(input.pdf, pages);
    }
    digests
}

/// A source page the record puts in an output.
struct Due {
    output: usize,
    position: usize,
    pdf: u8,
    page: u16,
    order: String,
}

pub(crate) fn check_group_pdfs(
    batch: &Batch,
    inputs: &BTreeMap<u8, Vec<u64>>,
    output_dir: &Path,
    findings: &mut Findings,
) {
    let groups: Vec<&Output> = batch.outputs.iter().filter(|o| o.group.is_some()).collect();
    let mut due: BTreeMap<u64, Vec<Due>> = BTreeMap::new();
    let mut found: BTreeMap<u64, Vec<(usize, usize)>> = BTreeMap::new();
    for (index, output) in groups.iter().enumerate() {
        let mut position = 0;
        for order in batch
            .orders
            .iter()
            .filter(|o| Some(&o.group) == output.group.as_ref())
        {
            for page in order.page_start..=order.page_end {
                let digest = inputs
                    .get(&order.pdf)
                    .and_then(|pages| pages.get(page as usize - 1));
                if let Some(digest) = digest {
                    due.entry(*digest).or_default().push(Due {
                        output: index,
                        position,
                        pdf: order.pdf,
                        page,
                        order: order.order_id.clone(),
                    });
                }
                position += 1;
            }
        }
        for (position, digest) in output_pages(output, output_dir, findings)
            .into_iter()
            .enumerate()
        {
            found.entry(digest).or_default().push((index, position));
        }
    }
    let source_of: BTreeMap<u64, (u8, u16)> = inputs
        .iter()
        .flat_map(|(pdf, pages)| {
            pages
                .iter()
                .enumerate()
                .map(move |(at, digest)| (*digest, (*pdf, at as u16 + 1)))
        })
        .collect();
    let name = |pdf: u8| crate::ledger::input_name(batch, pdf);
    let place = |(output, position): (usize, usize)| {
        format!("{} page {}", groups[output].file, position + 1)
    };
    let mut reordered: BTreeSet<usize> = BTreeSet::new();
    let mut wrong: BTreeSet<usize> = BTreeSet::new();
    let digests: BTreeSet<u64> = due.keys().chain(found.keys()).copied().collect();
    for digest in digests {
        let expected = due.get(&digest).map(Vec::as_slice).unwrap_or_default();
        let actual = found.get(&digest).map(Vec::as_slice).unwrap_or_default();
        let Some(first) = expected.first() else {
            for &at in actual {
                wrong.insert(at.0);
                match source_of.get(&digest) {
                    Some((pdf, page)) => findings.error(format!(
                        "{} is page {page} of {}, which the record does not write",
                        place(at),
                        name(*pdf)
                    )),
                    None => findings.error(format!("{} is a page of no input", place(at))),
                }
            }
            continue;
        };
        let what = format!(
            "page {} of {} (order {})",
            first.page,
            name(first.pdf),
            first.order
        );
        if actual.is_empty() {
            findings.error(format!("{what} is in no output"));
        } else if actual.len() != expected.len() {
            let places: Vec<String> = actual.iter().map(|&at| place(at)).collect();
            findings.error(format!(
                "{what} is written {} times: {}",
                actual.len(),
                places.join(", ")
            ));
            wrong.extend(actual.iter().map(|at| at.0));
        } else {
            let mut due_outputs: Vec<usize> = expected.iter().map(|d| d.output).collect();
            let mut found_outputs: Vec<usize> = actual.iter().map(|at| at.0).collect();
            due_outputs.sort_unstable();
            found_outputs.sort_unstable();
            if due_outputs != found_outputs {
                findings.error(format!(
                    "{what} is in {}, the record puts it in {}",
                    place(actual[0]),
                    groups[first.output].file
                ));
                wrong.extend(found_outputs);
            } else if expected
                .iter()
                .zip(actual)
                .any(|(due, at)| (due.output, due.position) != *at)
            {
                reordered.extend(found_outputs);
            }
        }
    }
    for output in reordered.difference(&wrong) {
        findings.warning(format!(
            "{}: every page is there, but not in the printing order",
            groups[*output].file
        ));
    }
}

/// The digests of an output's pages, after its SHA-256, size and page count are checked.
fn output_pages(output: &Output, output_dir: &Path, findings: &mut Findings) -> Vec<u64> {
    let path = output_dir.join(&output.file);
    let Some((sha, size)) = sha256_and_size(&path) else {
        findings.error(format!("{} was not written", output.file));
        return Vec::new();
    };
    if sha != output.sha256 || size != output.size {
        findings.error(format!("{} is not the file the record names", output.file));
    }
    let Ok(doc) = Document::load(&path) else {
        findings.error(format!("{} cannot be read as a PDF", output.file));
        return Vec::new();
    };
    let pages = Digester::new(&doc).pages();
    if pages.len() != output.pages as usize {
        findings.error(format!(
            "{} has {} pages, the record says {}",
            output.file,
            pages.len(),
            output.pages
        ));
    }
    pages
}
