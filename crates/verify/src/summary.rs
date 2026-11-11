//! V6: the summary PDF's printed counts, read back as text, equal the record's — the orders, units
//! and groups of its header, the ledger of its facts page, and each group bar's orders and units,
//! in group order. An account's summary counts that account's orders and the whole batch's ledger.

use std::path::Path;
use std::sync::OnceLock;

use lopdf::Document;
use regex::Regex;

use crate::Findings;
use crate::files::sha256_and_size;
use crate::record::{Batch, Order};

fn counts_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(r"(\d+) orders\D+(\d+) units\D+(\d+) groups").expect("built-in counts regex")
    })
}

/// A group bar's counts, `<orders> orders · <units> units`, looked for after the ledger: the group
/// pages follow the facts pages, whose account rows read the same. The page header's counts read
/// the same too and go on to `<groups> groups`, which tells the two apart. The group's name before them is not
/// read: lopdf reads no text of a character the summary draws with its CID font (an `é` in a name).
fn group_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(r"(\d+) orders\D+(\d+) units(\D+\d+ groups)?").expect("built-in group regex")
    })
}

fn ledger_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(r"(\d+) pages read\D+(\d+) written\D+(\d+) left out")
            .expect("built-in ledger regex")
    })
}

/// The batch's summary PDF: the file the record names, printing the record's counts.
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
    check_counts(batch, batch, &path, &summary.file, findings);
}

/// An account's summary of `batch`: its header and group bars count the account's orders and the
/// groups that hold one; its ledger is the whole batch's.
pub(crate) fn check_account_summary(
    batch: &Batch,
    account: &str,
    path: &Path,
    findings: &mut Findings,
) {
    let file = path
        .file_name()
        .map_or_else(String::new, |name| name.to_string_lossy().to_string());
    if !path.is_file() {
        findings.error(format!("{file} was not written"));
        return;
    }
    let mut mine = batch.clone();
    mine.orders.retain(|order| order.account == account);
    mine.outputs.retain(|output| {
        output
            .group
            .as_ref()
            .is_some_and(|group| mine.orders.iter().any(|order| &order.group == group))
    });
    check_counts(&mine, batch, path, &file, findings);
}

/// `shown`'s orders, units, groups and group bars, and `whole`'s ledger, as `file` prints them.
fn check_counts(shown: &Batch, whole: &Batch, path: &Path, file: &str, findings: &mut Findings) {
    let text = Document::load(path).ok().and_then(|doc| {
        let pages: Vec<u32> = doc.get_pages().keys().copied().collect();
        doc.extract_text(&pages).ok()
    });
    let Some(text) = text else {
        findings.error(format!("{file} cannot be read as text"));
        return;
    };
    let text = text.split_whitespace().collect::<Vec<_>>().join(" ");

    let groups: Vec<_> = shown.outputs.iter().filter(|o| o.group.is_some()).collect();
    let read: u64 = whole
        .inputs
        .iter()
        .map(|input| input.page_count as u64)
        .sum();
    let pages = |start: u16, end: u16| (end - start + 1) as u64;
    let written: u64 = whole
        .orders
        .iter()
        .map(|o| pages(o.page_start, o.page_end))
        .sum();
    let left: u64 = whole
        .left_out
        .iter()
        .map(|l| pages(l.page_start, l.page_end))
        .sum();

    let mut compare = |what: &str, re: &Regex, want: [u64; 3]| match re.captures(&text) {
        None => findings.error(format!("{file} prints no {what}")),
        Some(caps) => {
            let printed: Vec<u64> = (1..=3)
                .map(|i| caps[i].parse().unwrap_or(u64::MAX))
                .collect();
            if printed != want {
                findings.error(format!(
                    "{file} prints the {what} {printed:?}, the record says {want:?}"
                ));
            }
        }
    };
    let all: Vec<&Order> = shown.orders.iter().collect();
    compare(
        "orders, units and groups",
        counts_re(),
        [all.len() as u64, units(shown, &all), groups.len() as u64],
    );
    compare("ledger", ledger_re(), [read, written, left]);

    let after_ledger = ledger_re()
        .find(&text)
        .map_or(text.len(), |ledger| ledger.end());
    let printed: Vec<Option<[u64; 2]>> = group_re()
        .captures_iter(&text[after_ledger..])
        .filter(|caps| caps.get(3).is_none())
        .map(|caps| Some([caps[1].parse().ok()?, caps[2].parse().ok()?]))
        .collect();
    if printed.len() != groups.len() {
        findings.error(format!(
            "{file} prints {} group bars, the record has {} group PDFs",
            printed.len(),
            groups.len()
        ));
    }
    for (at, output) in groups.iter().enumerate() {
        let orders: Vec<&Order> = shown
            .orders
            .iter()
            .filter(|order| Some(&order.group) == output.group.as_ref())
            .collect();
        let want = [orders.len() as u64, units(shown, &orders)];
        let printed = printed.get(at).copied().flatten();
        if printed != Some(want) {
            findings.error(format!(
                "{file} prints the orders and units {printed:?} for {}, the record says {want:?}",
                output.file
            ));
        }
    }
}

fn units(batch: &Batch, orders: &[&Order]) -> u64 {
    orders
        .iter()
        .filter_map(|order| batch.contents.get(order.content as usize))
        .flat_map(|content| &content.items)
        .map(|item| item.quantity as u64)
        .sum()
}
