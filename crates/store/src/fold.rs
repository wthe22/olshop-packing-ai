//! The fold (journal.md): the records applied in order, each checked by its rule. It gives the
//! live batches, the inputs of the date, the printed marks, the reprints and the next numbers;
//! every decision is taken from it.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use jiff::civil::Date;
use packing_engine::ids::{BatchId, PdfId};
use packing_engine::record::{
    BatchSaved, Datetime, OutputFile, OutputRestored, Record, RecordBody, ReprintMade, SCHEMA,
    Sha256,
};

/// Where a journal is damaged: the line, counted from 1, and why.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Damage {
    pub line: usize,
    pub reason: DamageReason,
}

impl fmt::Display for Damage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}: {}", self.line, self.reason)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DamageReason {
    /// A line before the last whose seal does not match.
    BadSeal,
    /// A sealed line that is not a record.
    Unreadable,
    SeqGap {
        expected: u32,
        found: u32,
    },
    /// `started` names another namespace or date than the folder.
    OtherFolder {
        namespace: String,
        date: Date,
    },
    /// A record that breaks its rule of the fold.
    Rule(String),
}

impl fmt::Display for DamageReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DamageReason::BadSeal => f.write_str("its seal does not match"),
            DamageReason::Unreadable => f.write_str("it is not a record"),
            DamageReason::SeqGap { expected, found } => {
                write!(f, "seq {found} where {expected} was due")
            }
            DamageReason::OtherFolder { namespace, date } => {
                write!(f, "it names the owner {namespace:?} and the date {date}")
            }
            DamageReason::Rule(rule) => f.write_str(rule),
        }
    }
}

/// A batch from its `batch_saved` until a `batch_reverted` of it.
#[derive(Clone, Debug, PartialEq)]
pub struct LiveBatch {
    /// The newest record of the batch: an amend's replaces the earlier one.
    pub saved: BatchSaved,
    /// The batch's first `batch_saved`, and its newest.
    pub create_time: Datetime,
    pub modify_time: Datetime,
    /// The label groups marked as printed.
    pub printed: BTreeSet<String>,
    /// The newest `output_restored` of each output.
    pub restored: BTreeMap<String, OutputRestored>,
}

impl LiveBatch {
    /// The size of an output as the newest record that wrote it names.
    pub fn output_size(&self, output: &OutputFile) -> u64 {
        self.restored
            .get(&output.file)
            .map_or(output.size, |restored| restored.size)
    }
}

/// What the journal's last record writes into `output/`, which settles `output/.partial/`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LastWrites {
    Nothing,
    /// `batch_saved`, `output_restored` or `reprint_made`: these files with their SHA-256.
    Files(Vec<(String, Sha256)>),
    /// `batch_reverted`: the reverted batch's files, moved aside.
    Reverted(Vec<String>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Fold {
    pub namespace: String,
    pub date: Date,
    /// The last record's `seq`; 0 for a date with no journal.
    pub last_seq: u32,
    pub live: BTreeMap<BatchId, LiveBatch>,
    /// Every input any record named, live or not.
    pub inputs: BTreeMap<PdfId, Sha256>,
    pub reprints: Vec<ReprintMade>,
    pub last_writes: LastWrites,
}

impl Fold {
    pub fn new(namespace: impl Into<String>, date: Date) -> Self {
        Self {
            namespace: namespace.into(),
            date,
            last_seq: 0,
            live: BTreeMap::new(),
            inputs: BTreeMap::new(),
            reprints: Vec::new(),
            last_writes: LastWrites::Nothing,
        }
    }

    pub fn newest_live(&self) -> Option<BatchId> {
        self.live.keys().next_back().copied()
    }

    /// The number of a new batch: the highest live + 1.
    pub fn next_batch(&self) -> BatchId {
        self.newest_live().map_or(BatchId::FIRST, BatchId::next)
    }

    /// The number of an input not seen yet: the highest ever + 1.
    pub fn next_pdf(&self) -> PdfId {
        self.inputs
            .keys()
            .next_back()
            .map_or(PdfId::FIRST, |pdf| pdf.next())
    }

    /// The input already numbered for this content, if any.
    pub fn pdf_of(&self, sha256: &Sha256) -> Option<PdfId> {
        self.inputs
            .iter()
            .find(|(_, sha)| *sha == sha256)
            .map(|(pdf, _)| *pdf)
    }

    /// Apply one record by its rule; a record that breaks it leaves the fold unchanged.
    pub fn apply(&mut self, record: &Record) -> Result<(), DamageReason> {
        let expected = self.last_seq + 1;
        if record.seq != expected {
            return Err(DamageReason::SeqGap {
                expected,
                found: record.seq,
            });
        }
        let is_started = matches!(record.body, RecordBody::Started(_));
        if (record.seq == 1) != is_started {
            return Err(rule("`started` is the first record, and only there"));
        }
        let mut next = self.clone();
        next.last_writes = LastWrites::Nothing;
        next.apply_body(record)?;
        next.last_seq = record.seq;
        *self = next;
        Ok(())
    }

    fn apply_body(&mut self, record: &Record) -> Result<(), DamageReason> {
        match &record.body {
            RecordBody::Started(started) => {
                if started.schema != SCHEMA {
                    return Err(rule(format!("schema {} is not {SCHEMA}", started.schema)));
                }
                if started.namespace != self.namespace || started.date != self.date {
                    return Err(DamageReason::OtherFolder {
                        namespace: started.namespace.clone(),
                        date: started.date,
                    });
                }
            }
            RecordBody::BatchSaved(saved) => self.save_batch(saved, &record.time)?,
            RecordBody::BatchReverted(reverted) => {
                if self.newest_live() != Some(reverted.batch) {
                    return Err(rule(format!(
                        "batch {} reverted is not the newest live batch",
                        reverted.batch
                    )));
                }
                let batch = self.live.remove(&reverted.batch).expect("newest live");
                let files = batch.saved.outputs.iter().map(|o| o.file.clone()).collect();
                self.last_writes = LastWrites::Reverted(files);
            }
            RecordBody::PrintedMarked(marked) => {
                let batch = self.live_batch(marked.batch)?;
                let has_group = batch
                    .saved
                    .outputs
                    .iter()
                    .any(|output| output.group.as_deref() == Some(marked.group.as_str()));
                if !has_group {
                    return Err(rule(format!(
                        "batch {} has no label group {}",
                        marked.batch, marked.group
                    )));
                }
                if marked.printed {
                    batch.printed.insert(marked.group.clone());
                } else {
                    batch.printed.remove(&marked.group);
                }
            }
            RecordBody::OutputRestored(restored) => {
                let batch = self.live_batch(restored.batch)?;
                if !batch.saved.outputs.iter().any(|o| o.file == restored.file) {
                    return Err(rule(format!(
                        "{} is not an output of batch {}",
                        restored.file, restored.batch
                    )));
                }
                batch
                    .restored
                    .insert(restored.file.clone(), restored.clone());
                self.last_writes =
                    LastWrites::Files(vec![(restored.file.clone(), restored.sha256.clone())]);
            }
            RecordBody::ReprintMade(reprint) => {
                let due = self.reprints.len() + 1;
                if reprint.number as usize != due {
                    return Err(rule(format!(
                        "reprint {} where {due} was due",
                        reprint.number
                    )));
                }
                self.reprints.push(reprint.clone());
                self.last_writes =
                    LastWrites::Files(vec![(reprint.file.clone(), reprint.sha256.clone())]);
            }
            RecordBody::Recovered(_) => {}
        }
        Ok(())
    }

    fn save_batch(&mut self, saved: &BatchSaved, time: &Datetime) -> Result<(), DamageReason> {
        let newest = self.newest_live();
        let due = if saved.amends {
            newest
        } else {
            Some(self.next_batch())
        };
        if due != Some(saved.batch) {
            let due = if saved.amends {
                "the newest live batch"
            } else {
                "the next number"
            };
            return Err(rule(format!("batch {} is not {due}", saved.batch)));
        }
        for input in &saved.inputs {
            match self.inputs.get(&input.pdf) {
                Some(sha) if *sha != input.sha256 => {
                    return Err(rule(format!("input {} names another SHA-256", input.pdf)));
                }
                Some(_) => {}
                None => {
                    if self.pdf_of(&input.sha256).is_some() {
                        return Err(rule(format!(
                            "the SHA-256 of input {} has another number",
                            input.pdf
                        )));
                    }
                    self.inputs.insert(input.pdf, input.sha256.clone());
                }
            }
        }
        let create_time = match self.live.get(&saved.batch) {
            Some(earlier) if saved.amends => earlier.create_time.clone(),
            _ => time.clone(),
        };
        self.live.insert(
            saved.batch,
            LiveBatch {
                saved: saved.clone(),
                create_time,
                modify_time: time.clone(),
                printed: BTreeSet::new(),
                restored: BTreeMap::new(),
            },
        );
        self.last_writes = LastWrites::Files(
            saved
                .outputs
                .iter()
                .map(|output| (output.file.clone(), output.sha256.clone()))
                .collect(),
        );
        Ok(())
    }

    fn live_batch(&mut self, batch: BatchId) -> Result<&mut LiveBatch, DamageReason> {
        self.live
            .get_mut(&batch)
            .ok_or_else(|| rule(format!("batch {batch} is not live")))
    }
}

fn rule(text: impl Into<String>) -> DamageReason {
    DamageReason::Rule(text.into())
}
