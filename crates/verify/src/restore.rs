//! A restored output (reprint.md, *Restore*): checked as at the save against its live batch's
//! record, with the size and SHA-256 its `output_restored` names — V2 and V5 for a group PDF, V6
//! for the summary PDF.

use std::collections::BTreeMap;
use std::path::Path;

use serde::Deserialize;

use crate::record::{Batch, Output};
use crate::{Findings, files, summary};

#[derive(Deserialize)]
struct Restored {
    batch: u8,
    file: String,
    pages: u16,
    size: u64,
    sha256: String,
}

pub(crate) fn check_restore(
    json: &[u8],
    live: &BTreeMap<u8, Batch>,
    input_dir: &Path,
    output_dir: &Path,
    findings: &mut Findings,
) {
    let Ok(restored) = serde_json::from_slice::<Restored>(json) else {
        findings.error("the restore's record cannot be read".to_string());
        return;
    };
    let Some(batch) = live.get(&restored.batch) else {
        findings.error(format!("batch {} is not live", restored.batch));
        return;
    };
    let Some(original) = batch.outputs.iter().find(|o| o.file == restored.file) else {
        findings.error(format!(
            "{} is not an output of batch {}",
            restored.file, restored.batch
        ));
        return;
    };
    let output = Output {
        file: restored.file.clone(),
        group: original.group.clone(),
        pages: restored.pages,
        size: restored.size,
        sha256: restored.sha256.clone(),
    };
    let mut checked = batch.clone();
    if output.group.is_some() {
        checked.outputs = vec![output];
        let inputs = files::check_inputs(&checked, input_dir, findings);
        files::check_group_pdfs(&checked, &inputs, output_dir, findings);
    } else {
        checked.outputs.retain(|o| o.group.is_some());
        checked.outputs.push(output);
        summary::check_summary(&checked, output_dir, findings);
    }
}
