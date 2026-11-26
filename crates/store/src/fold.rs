//! The fold (journal.md): the records applied in order, each checked by its rule. It gives the
//! live batches, the inputs of the date, the sent and printed marks, the reprints and the next
//! numbers; every decision is taken from it.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use jiff::civil::Date;
use packing_engine::ids::{BatchId, OrderId, PdfId, TrackingId};
use packing_engine::record::{
    AreasFilled, BatchSaved, Datetime, JOURNAL_SCHEMA, OutputFile, OutputRestored, Record,
    RecordBody, ReprintMade, SavedOrder, Sha256, WeightsFilled,
};
use packing_engine::schema::{Schema, SchemaFit};

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
    /// `started` names another date than the folder.
    OtherDate {
        date: Date,
    },
    /// A record that breaks its rule of the fold.
    Rule(String),
    /// `started` names a newer major schema: a newer copy's journal, which this copy does not read.
    Newer {
        schema: String,
        app: String,
    },
}

impl fmt::Display for DamageReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DamageReason::BadSeal => f.write_str("its seal does not match"),
            DamageReason::Unreadable => f.write_str("it is not a record"),
            DamageReason::SeqGap { expected, found } => {
                write!(f, "seq {found} where {expected} was due")
            }
            DamageReason::OtherDate { date } => write!(f, "it names the date {date}"),
            DamageReason::Rule(rule) => f.write_str(rule),
            DamageReason::Newer { schema, app } => write!(
                f,
                "it is of a newer format (schema {schema}, Packing v{app}): this copy reads {}",
                JOURNAL_SCHEMA.family()
            ),
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
    /// The label groups marked as sent, each with the time of its `print_sent`.
    pub sent: BTreeMap<String, Datetime>,
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

/// What the journal's last record writes into the date folder, which settles its `.partial/`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LastWrites {
    Nothing,
    /// A record of a type this copy does not know: what it wrote is not known either.
    Unknown,
    /// `batch_saved`, `output_restored` or `reprint_made`: these files with their SHA-256.
    Files(Vec<(String, Sha256)>),
    /// `batch_reverted`: the reverted batch's files, moved aside.
    Reverted(Vec<String>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Fold {
    pub date: Date,
    /// The last record's `seq`; 0 for a date with no journal.
    pub last_seq: u32,
    pub live: BTreeMap<BatchId, LiveBatch>,
    /// Every input any record named, live or not.
    pub inputs: BTreeMap<PdfId, Sha256>,
    pub reprints: Vec<Reprint>,
    pub last_writes: LastWrites,
    /// The records of a newer copy's type, passed over.
    pub passed_over: Vec<PassedOver>,
}

/// A record whose `type` this copy does not know: a newer copy wrote it, and an older one may pass
/// over it (data/README.md, *Types, versions and names*).
#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize)]
pub struct PassedOver {
    pub seq: u32,
    /// The version of the app that wrote it.
    pub app: String,
    #[serde(rename = "type")]
    pub kind: String,
}

/// A reprint of the date and when it was made.
#[derive(Clone, Debug, PartialEq)]
pub struct Reprint {
    pub made: ReprintMade,
    pub create_time: Datetime,
}

impl Fold {
    pub fn new(date: Date) -> Self {
        Self {
            date,
            last_seq: 0,
            live: BTreeMap::new(),
            inputs: BTreeMap::new(),
            reprints: Vec::new(),
            last_writes: LastWrites::Nothing,
            passed_over: Vec::new(),
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
        self.check_seq(record.seq, matches!(record.body, RecordBody::Started(_)))?;
        let mut next = self.clone();
        next.last_writes = LastWrites::Nothing;
        next.apply_body(record)?;
        next.last_seq = record.seq;
        *self = next;
        Ok(())
    }

    /// Pass over a newer copy's record: only its `seq` is checked.
    pub fn pass_over(&mut self, record: &PassedOver) -> Result<(), DamageReason> {
        self.check_seq(record.seq, false)?;
        self.last_seq = record.seq;
        self.last_writes = LastWrites::Unknown;
        self.passed_over.push(record.clone());
        Ok(())
    }

    fn check_seq(&self, seq: u32, is_started: bool) -> Result<(), DamageReason> {
        let expected = self.last_seq + 1;
        if seq != expected {
            return Err(DamageReason::SeqGap {
                expected,
                found: seq,
            });
        }
        if (seq == 1) != is_started {
            return Err(rule("`started` is the first record, and only there"));
        }
        Ok(())
    }

    fn apply_body(&mut self, record: &Record) -> Result<(), DamageReason> {
        match &record.body {
            RecordBody::Started(started) => {
                check_journal_schema(&started.schema, &record.app)?;
                if started.date != self.date {
                    return Err(DamageReason::OtherDate { date: started.date });
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
            RecordBody::PrintSent(sent) => {
                let batch = self.group_batch(sent.batch, &sent.group)?;
                if sent.sent {
                    batch.sent.insert(sent.group.clone(), record.time.clone());
                } else {
                    batch.sent.remove(&sent.group);
                }
            }
            RecordBody::PrintedMarked(marked) => {
                let batch = self.group_batch(marked.batch, &marked.group)?;
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
                self.reprints.push(Reprint {
                    made: reprint.clone(),
                    create_time: record.time.clone(),
                });
                self.last_writes =
                    LastWrites::Files(vec![(reprint.file.clone(), reprint.sha256.clone())]);
            }
            RecordBody::AreasFilled(filled) => self.fill_areas(filled)?,
            RecordBody::WeightsFilled(filled) => self.fill_weights(filled)?,
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
                saved: with_pickup_tracking(saved),
                create_time,
                modify_time: time.clone(),
                sent: BTreeMap::new(),
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

    /// Each order named gets its area: an order of the batch that has none yet.
    fn fill_areas(&mut self, filled: &AreasFilled) -> Result<(), DamageReason> {
        for named in &filled.orders {
            let order = self.batch_order(filled.batch, &named.order_id)?;
            if order.area.is_some() {
                return Err(rule(format!("{} has its area already", named.order_id)));
            }
            order.area = Some(named.area.clone());
        }
        Ok(())
    }

    /// Each order named gets its weight: an order of the batch that has none yet.
    fn fill_weights(&mut self, filled: &WeightsFilled) -> Result<(), DamageReason> {
        for named in &filled.orders {
            let order = self.batch_order(filled.batch, &named.order_id)?;
            if order.weight.is_some() {
                return Err(rule(format!("{} has its weight already", named.order_id)));
            }
            order.weight = Some(named.weight);
        }
        Ok(())
    }

    fn batch_order(
        &mut self,
        batch: BatchId,
        order_id: &OrderId,
    ) -> Result<&mut SavedOrder, DamageReason> {
        self.live_batch(batch)?
            .saved
            .orders
            .iter_mut()
            .find(|order| order.order_id == *order_id)
            .ok_or_else(|| rule(format!("{order_id} is not an order of batch {batch}")))
    }

    fn live_batch(&mut self, batch: BatchId) -> Result<&mut LiveBatch, DamageReason> {
        self.live
            .get_mut(&batch)
            .ok_or_else(|| rule(format!("batch {batch} is not live")))
    }

    /// The live batch that has label group `group`.
    fn group_batch(&mut self, batch: BatchId, group: &str) -> Result<&mut LiveBatch, DamageReason> {
        let live = self.live_batch(batch)?;
        let has_group = live
            .saved
            .outputs
            .iter()
            .any(|output| output.group.as_deref() == Some(group));
        if !has_group {
            return Err(rule(format!("batch {batch} has no label group {group}")));
        }
        Ok(live)
    }
}

/// The record as the fold reads it: an order whose label printed no tracking ID and a pickup code
/// has the pickup code as its tracking ID, whichever version of the app saved it.
fn with_pickup_tracking(saved: &BatchSaved) -> BatchSaved {
    let mut saved = saved.clone();
    for order in &mut saved.orders {
        if order.tracking_id.is_none() {
            order.tracking_id = order.pickup_code.as_deref().map(TrackingId::new);
        }
    }
    saved
}

/// A journal of this copy's major reads, whatever its minor.
pub(crate) fn check_journal_schema(schema: &str, app: &str) -> Result<(), DamageReason> {
    match Schema::parse(schema).map(|found| JOURNAL_SCHEMA.fit(found)) {
        Some(SchemaFit::Reads) => Ok(()),
        Some(SchemaFit::Newer) => Err(DamageReason::Newer {
            schema: schema.to_string(),
            app: app.to_string(),
        }),
        Some(SchemaFit::Older) | None => Err(rule(format!(
            "schema {schema} is not one this copy reads ({})",
            JOURNAL_SCHEMA.family()
        ))),
    }
}

fn rule(text: impl Into<String>) -> DamageReason {
    DamageReason::Rule(text.into())
}
