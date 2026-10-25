//! The independent verifier (verification.md): checks a batch's outputs against the record about
//! to be committed. It reads the record and the journal as JSON into types of its own, takes its
//! expected side from the inputs and the record, never from the files it checks, and uses no other
//! crate of the workspace and no PDFium.

mod digest;
mod files;
mod journal;
mod ledger;
mod record;
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

/// The page digests of a PDF, in page order: what a page draws, however it is stored.
pub fn page_digests(path: &Path) -> Option<Vec<u64>> {
    let doc = lopdf::Document::load(path).ok()?;
    Some(digest::Digester::new(&doc).pages())
}
