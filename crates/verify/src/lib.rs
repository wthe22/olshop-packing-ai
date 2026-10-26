//! The independent verifier (verification.md): checks a batch's outputs against the record about
//! to be committed. It reads the record and the journal as JSON into types of its own, takes its
//! expected side from the inputs and the record, never from the files it checks, and uses no other
//! crate of the workspace and no PDFium.

mod digest;
mod files;
mod journal;
mod ledger;
mod record;
mod reprint;
mod restore;
mod summary;

use std::path::Path;

/// What a batch is checked against: the journal as on disk, the lines its commit appends (the
/// batch's record last), the date's `input/` and the folder that holds the new outputs.
pub struct BatchCheck<'a> {
    pub journal: &'a [u8],
    pub append: &'a [u8],
    pub input_dir: &'a Path,
    pub output_dir: &'a Path,
}

/// What a reprint is checked against: today's journal as on disk, the `reprint_made` line its
/// commit appends, the namespace's folder (each order's input is `{date}/input/{sha256}.pdf` in
/// it) and the folder that holds the new file.
pub struct ReprintCheck<'a> {
    pub journal: &'a [u8],
    pub append: &'a [u8],
    pub namespace_dir: &'a Path,
    pub output_dir: &'a Path,
}

/// What a restored output is checked against: the date's journal as on disk, the
/// `output_restored` line its commit appends, the date's `input/` and the folder that holds the
/// file written again.
pub struct RestoreCheck<'a> {
    pub journal: &'a [u8],
    pub append: &'a [u8],
    pub input_dir: &'a Path,
    pub output_dir: &'a Path,
}

/// The findings, errors and warnings apart, each in plain words.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Findings {
    pub errors: Vec<String>,
    /// Only the right pages of a group PDF in another order.
    pub warnings: Vec<String>,
}

impl Findings {
    pub fn is_clean(&self) -> bool {
        self.errors.is_empty() && self.warnings.is_empty()
    }

    pub(crate) fn error(&mut self, text: String) {
        self.errors.push(text);
    }

    pub(crate) fn warning(&mut self, text: String) {
        self.warnings.push(text);
    }
}

/// Run V1 to V6 on a batch before its commit.
pub fn verify_batch(check: &BatchCheck<'_>) -> Findings {
    let mut findings = Findings::default();
    let Some(journal) = journal::read(check.journal, check.append, &mut findings) else {
        findings.error("the new lines hold no batch record to check".to_string());
        return findings;
    };
    let batch = &journal.batch;
    let inputs = files::check_inputs(batch, check.input_dir, &mut findings);
    ledger::check_ledger(batch, &mut findings);
    ledger::check_orders(batch, &journal.live, &mut findings);
    files::check_group_pdfs(batch, &inputs, check.output_dir, &mut findings);
    summary::check_summary(batch, check.output_dir, &mut findings);
    findings
}

/// V1, then V2 and V5 on a reprint before its commit.
pub fn verify_reprint(check: &ReprintCheck<'_>) -> Findings {
    let mut findings = Findings::default();
    match appended(check.journal, check.append, "reprint_made", &mut findings) {
        Some((json, _)) => {
            reprint::check_reprint(json, check.namespace_dir, check.output_dir, &mut findings);
        }
        None => findings.error("the new lines hold no reprint record to check".to_string()),
    }
    findings
}

/// V1, then the checks of the save for the one output a restore writes again.
pub fn verify_restore(check: &RestoreCheck<'_>) -> Findings {
    let mut findings = Findings::default();
    match appended(
        check.journal,
        check.append,
        "output_restored",
        &mut findings,
    ) {
        Some((json, live)) => restore::check_restore(
            json,
            &live,
            check.input_dir,
            check.output_dir,
            &mut findings,
        ),
        None => findings.error("the new lines hold no restore record to check".to_string()),
    }
    findings
}

/// The JSON of the last new line of type `kind`, and the live batches before the new lines.
fn appended<'a>(
    journal: &[u8],
    append: &'a [u8],
    kind: &str,
    findings: &mut Findings,
) -> Option<(&'a [u8], std::collections::BTreeMap<u8, record::Batch>)> {
    let lines = journal::read_lines(journal, append, findings)?;
    let json = lines
        .appended
        .iter()
        .rev()
        .find(|(found, _)| found == kind)
        .map(|(_, json)| *json)?;
    Some((json, lines.live))
}

/// The page digests of a PDF, in page order: what a page draws, however it is stored.
pub fn page_digests(path: &Path) -> Option<Vec<u64>> {
    let doc = lopdf::Document::load(path).ok()?;
    Some(digest::Digester::new(&doc).pages())
}
