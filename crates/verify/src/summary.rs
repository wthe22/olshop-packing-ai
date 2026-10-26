//! V6: the summary PDF's printed counts, read back as text, equal the record's — the orders, units
//! and groups of its header, the ledger of its facts page, and each group PDF's orders, in group
//! order.

use std::path::Path;
use std::sync::OnceLock;

use lopdf::Document;
use regex::Regex;

use crate::Findings;
use crate::files::sha256_and_size;
use crate::record::Batch;

fn counts_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(r"(\d+) orders\D+(\d+) units\D+(\d+) groups").expect("built-in counts regex")
    })
}

/// A group's line by its fixed tail: the file name before it is not read, because lopdf reads no
/// text of a character the summary draws with its CID font (an `é` in a name).
fn group_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(r"(\d+) orders\D+\d+ pages\D+\d+ order contents").expect("built-in group regex")
    })
}

fn ledger_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(r"(\d+) pages read\D+(\d+) written\D+(\d+) left out")
            .expect("built-in ledger regex")
    })
}

pub(crate) fn check_summary(batch: &Batch, output_dir: &Path, findings: &mut Findings) {
    let Some(summary) = batch.outputs.iter().find(|output| output.group.is_none()) else {
        findings.error("the record names no summary PDF".to_string());
        return;
    };
    let path = output_dir.join(&summary.file);
    match sha256_and_size(&path) {
        None => {
            findings.error(format!("{} was not written", summary.file));
            return;
        }
        Some((sha, size)) if sha != summary.sha256 || size != summary.size => {
            findings.error(format!("{} is not the file the record names", summary.file));
        }
        Some(_) => {}
    }
    let text = Document::load(&path).ok().and_then(|doc| {
        let pages: Vec<u32> = doc.get_pages().keys().copied().collect();
        doc.extract_text(&pages).ok()
    });
    let Some(text) = text else {
        findings.error(format!("{} cannot be read as text", summary.file));
        return;
    };
    let text = text.split_whitespace().collect::<Vec<_>>().join(" ");

    let orders = batch.orders.len() as u64;
    let units: u64 = batch
        .orders
        .iter()
        .filter_map(|order| batch.contents.get(order.content as usize))
        .flat_map(|content| &content.items)
        .map(|item| item.quantity as u64)
        .sum();
    let groups = batch.outputs.iter().filter(|o| o.group.is_some()).count() as u64;
    let read: u64 = batch
        .inputs
        .iter()
        .map(|input| input.page_count as u64)
        .sum();
    let pages = |start: u16, end: u16| (end - start + 1) as u64;
    let written: u64 = batch
        .orders
        .iter()
        .map(|o| pages(o.page_start, o.page_end))
        .sum();
    let left: u64 = batch
        .left_out
        .iter()
        .map(|l| pages(l.page_start, l.page_end))
        .sum();

    let mut compare = |what: &str, re: &Regex, want: [u64; 3]| match re.captures(&text) {
        None => findings.error(format!("{} prints no {what}", summary.file)),
        Some(caps) => {
            let printed: Vec<u64> = (1..=3)
                .map(|i| caps[i].parse().unwrap_or(u64::MAX))
                .collect();
            if printed != want {
                findings.error(format!(
                    "{} prints the {what} {printed:?}, the record says {want:?}",
                    summary.file
                ));
            }
        }
    };
    compare(
        "orders, units and groups",
        counts_re(),
        [orders, units, groups],
    );
    compare("ledger", ledger_re(), [read, written, left]);

    let printed: Vec<Option<usize>> = group_re()
        .captures_iter(&text)
        .map(|caps| caps[1].parse().ok())
        .collect();
    let groups: Vec<_> = batch.outputs.iter().filter(|o| o.group.is_some()).collect();
    if printed.len() != groups.len() {
        findings.error(format!(
            "{} prints {} group lines, the record has {} group PDFs",
            summary.file,
            printed.len(),
            groups.len()
        ));
    }
    for (at, output) in groups.iter().enumerate() {
        let want = batch
            .orders
            .iter()
            .filter(|order| Some(&order.group) == output.group.as_ref())
            .count();
        let printed = printed.get(at).copied().flatten();
        if printed != Some(want) {
            findings.error(format!(
                "{} prints {printed:?} orders for {}, the record says {want}",
                summary.file, output.file
            ));
        }
    }
}
