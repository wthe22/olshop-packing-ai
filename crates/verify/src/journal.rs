//! V1: the journal on disk and the lines about to be appended — every line sealed, `seq` with no
//! gap, the new lines after the last one — and the date's live batches as the journal says.

use std::collections::BTreeMap;

use crate::Findings;
use crate::record::{Batch, Head, Reverted, lines, sealed};

/// What the journal and the appended lines say: the live batches before the new lines, and each
/// new line's type and JSON.
pub(crate) struct Lines<'a> {
    pub live: BTreeMap<u8, Batch>,
    pub appended: Vec<(String, &'a [u8])>,
}

/// What the journal and the appended lines say: the batch being saved and the other live batches.
pub(crate) struct Journal {
    pub batch: Batch,
    /// The live batches before this one, by number, with their records.
    pub live: BTreeMap<u8, Batch>,
}

/// Read and check the journal and the lines to append. `None` when the appended lines hold no
/// batch record to check against.
pub(crate) fn read(journal: &[u8], append: &[u8], findings: &mut Findings) -> Option<Journal> {
    let Lines { mut live, appended } = read_lines(journal, append, findings)?;
    let batch = appended
        .iter()
        .rev()
        .find(|(kind, _)| kind == "batch_saved")
        .and_then(|(_, json)| serde_json::from_slice::<Batch>(json).ok())?;
    if batch.amends {
        live.remove(&batch.batch);
    }
    Some(Journal { batch, live })
}

/// V1: every line sealed and `seq` with no gap, the new lines after the last one. `None` when a
/// line is not a record at all.
pub(crate) fn read_lines<'a>(
    journal: &[u8],
    append: &'a [u8],
    findings: &mut Findings,
) -> Option<Lines<'a>> {
    let mut live: BTreeMap<u8, Batch> = BTreeMap::new();
    let mut last_seq = 0u32;
    let on_disk = lines(journal);
    let count = on_disk.len();
    for (index, (number, line, complete)) in on_disk.into_iter().enumerate() {
        let json = complete.then(|| sealed(line)).flatten();
        let Some(json) = json else {
            if index + 1 < count {
                findings.error(format!("the journal changed: line {number} is not sealed"));
            }
            // A torn last line: the commit cuts it.
            continue;
        };
        if !apply(json, number, &mut last_seq, &mut live, findings) {
            return None;
        }
    }
    let mut appended = Vec::new();
    for (number, line, complete) in lines(append) {
        let Some(json) = complete.then(|| sealed(line)).flatten() else {
            findings.error(format!("the new line {number} is not sealed"));
            return None;
        };
        let head: Head = serde_json::from_slice(json).ok()?;
        if head.seq != last_seq + 1 {
            findings.error(format!(
                "the journal changed: the new record is seq {} after seq {last_seq}",
                head.seq
            ));
        }
        last_seq = head.seq;
        appended.push((head.kind, json));
    }
    Some(Lines { live, appended })
}

/// Apply one sealed line of the journal on disk; false when it is not a record at all.
fn apply(
    json: &[u8],
    number: usize,
    last_seq: &mut u32,
    live: &mut BTreeMap<u8, Batch>,
    findings: &mut Findings,
) -> bool {
    let Ok(head) = serde_json::from_slice::<Head>(json) else {
        findings.error(format!(
            "the journal changed: line {number} is not a record"
        ));
        return false;
    };
    if head.seq != *last_seq + 1 {
        findings.error(format!(
            "the journal changed: line {number} is seq {} after seq {last_seq}",
            head.seq
        ));
    }
    *last_seq = head.seq;
    match head.kind.as_str() {
        "batch_saved" => {
            if let Ok(batch) = serde_json::from_slice::<Batch>(json) {
                live.insert(batch.batch, batch);
            }
        }
        "batch_reverted" => {
            if let Ok(reverted) = serde_json::from_slice::<Reverted>(json) {
                live.remove(&reverted.batch);
            }
        }
        _ => {}
    }
    true
}
