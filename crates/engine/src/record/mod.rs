//! The records of the journal (records.md): what one line of `journal.jsonl` says. The engine
//! builds a batch's record from its plan; `packing-store` seals, appends and folds them. A field
//! may be added without a new schema version; a field that changes meaning may not.

mod time;

use std::fmt;

use jiff::civil::Date;
use serde::{Deserialize, Serialize};

use crate::ids::{BatchId, CourierCode, OrderId, PdfId, TrackingId};

pub use self::time::Datetime;

/// The schema version every journal's `started` record carries.
pub const SCHEMA: &str = "1.0";

/// One line of the journal: the numbering every record carries, then its own fields.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Record {
    pub seq: u32,
    pub time: Datetime,
    /// The version of the app that wrote the line.
    pub app: String,
    #[serde(flatten)]
    pub body: RecordBody,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RecordBody {
    Started(Started),
    BatchSaved(BatchSaved),
    BatchReverted(BatchReverted),
    PrintedMarked(PrintedMarked),
    OutputRestored(OutputRestored),
    ReprintMade(ReprintMade),
    Recovered(Recovered),
}

impl RecordBody {
    /// The record's `type`, as the line writes it.
    pub fn kind(&self) -> &'static str {
        match self {
            RecordBody::Started(_) => "started",
            RecordBody::BatchSaved(_) => "batch_saved",
            RecordBody::BatchReverted(_) => "batch_reverted",
            RecordBody::PrintedMarked(_) => "printed_marked",
            RecordBody::OutputRestored(_) => "output_restored",
            RecordBody::ReprintMade(_) => "reprint_made",
            RecordBody::Recovered(_) => "recovered",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Started {
    pub schema: String,
    pub namespace: String,
    pub date: Date,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BatchSaved {
    pub batch: BatchId,
    /// True: replaces the newest live batch under the same number.
    pub amends: bool,
    /// In reading order.
    pub inputs: Vec<InputPdf>,
    /// The rule used, as it was.
    pub sort_rule: LabelSortRule,
    /// The distinct contents of the batch; an order names one by index.
    pub contents: Vec<OrderContent>,
    /// In printing order: group by group, content by content, then reading position.
    pub orders: Vec<SavedOrder>,
    pub left_out: Vec<LeftOutCopy>,
    /// The group PDFs in group order, then the summary PDF.
    pub outputs: Vec<OutputFile>,
    pub warnings: Vec<Finding>,
    /// The verifier's errors; only on a batch saved anyway.
    pub errors: Vec<Finding>,
}

/// A label download, stored once as `input/{sha256}.pdf`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InputPdf {
    pub pdf: PdfId,
    pub sha256: Sha256,
    /// The name it had when it was chosen.
    pub filename: String,
    pub size: u64,
    pub page_count: u16,
    /// Of the chosen file.
    pub create_time: Datetime,
    pub modify_time: Datetime,
}

/// A named list of label groups, as a batch used it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LabelSortRule {
    pub name: String,
    pub label_groups: Vec<LabelGroupRule>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LabelGroupRule {
    pub code: String,
    pub name: String,
    /// None: takes every order left; only the last group.
    pub filter: Option<String>,
}

/// What an order is packed with: its lines as (name, variation, quantity).
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct OrderContent {
    /// Sorted by name, variant, quantity.
    pub items: Vec<ItemPick>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ItemPick {
    pub name: String,
    /// Empty for `Default` or none.
    pub variant: String,
    pub quantity: u16,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SavedOrder {
    pub order_id: OrderId,
    pub tracking_id: Option<TrackingId>,
    pub courier: Option<CourierCode>,
    pub courier_clue: CourierClue,
    /// Index into the batch's `contents`.
    pub content: u16,
    /// The Seller SKU of each of its content's items, in their order.
    pub skus: Vec<Option<String>>,
    /// The `Qty Total` the slip prints; none on the Shopee-style slip.
    pub total_quantity: Option<u16>,
    pub ship_by_date: Option<Date>,
    /// The instant-service label's pickup code.
    pub pickup_code: Option<String>,
    /// The input that holds the copy written.
    pub pdf: PdfId,
    pub page_start: u16,
    pub page_end: u16,
    /// The label group's code.
    pub group: String,
}

/// The clue that named an order's courier.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CourierClue {
    pub kind: CourierClueKind,
    /// The hash, text or tag that named the courier.
    pub value: Option<String>,
    /// The tracking-ID form that agreed with it.
    pub form: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CourierClueKind {
    Logo,
    Shape,
    Text,
    Form,
    Tag,
    None,
}

impl CourierClueKind {
    pub fn as_str(self) -> &'static str {
        match self {
            CourierClueKind::Logo => "logo",
            CourierClueKind::Shape => "shape",
            CourierClueKind::Text => "text",
            CourierClueKind::Form => "form",
            CourierClueKind::Tag => "tag",
            CourierClueKind::None => "none",
        }
    }
}

/// A copy of an order that was not written, and why.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LeftOutCopy {
    pub order_id: OrderId,
    pub pdf: PdfId,
    pub page_start: u16,
    pub page_end: u16,
    #[serde(flatten)]
    pub reason: LeftOutReason,
}

/// Stored as `reason` and `kept`: the copy written for a repeat, the batch that saved it earlier.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "reason", content = "kept", rename_all = "snake_case")]
pub enum LeftOutReason {
    Repeat { pdf: PdfId, page_start: u16 },
    SavedEarlier { batch: BatchId },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutputFile {
    /// The name in `output/`.
    pub file: String,
    /// The label group's code; none for the summary PDF.
    pub group: Option<String>,
    pub pages: u16,
    pub size: u64,
    pub sha256: Sha256,
}

/// A warning or an error, as shown, with what it names.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Finding {
    pub kind: FindingKind,
    pub pdf: Option<PdfId>,
    pub page: Option<u16>,
    pub order_id: Option<OrderId>,
    pub text: String,
}

/// What a finding is about (`slip_total_differs`, `courier_unknown`, `page_doubled`): the reader
/// and the verifier name their own.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FindingKind(String);

impl FindingKind {
    pub fn new(kind: impl Into<String>) -> Self {
        Self(kind.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchReverted {
    pub batch: BatchId,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrintedMarked {
    pub batch: BatchId,
    pub group: String,
    pub printed: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutputRestored {
    pub batch: BatchId,
    pub file: String,
    pub pages: u16,
    pub size: u64,
    pub sha256: Sha256,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReprintMade {
    /// 1, 2, … within the date.
    pub number: u8,
    pub file: String,
    pub pages: u16,
    pub size: u64,
    pub sha256: Sha256,
    /// In printing order.
    pub orders: Vec<ReprintedOrder>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReprintedOrder {
    pub order_id: OrderId,
    /// The date the order was saved.
    pub date: Date,
    pub batch: BatchId,
    /// Its input: `{namespace}/{date}/input/{sha256}.pdf`.
    pub sha256: Sha256,
    pub page_start: u16,
    pub page_end: u16,
}

/// The cut-off tail of an append that did not finish.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Recovered {
    pub dropped_bytes: u64,
    /// The bytes as UTF-8, invalid ones replaced.
    pub dropped_text: String,
}

/// A file's SHA-256 as 64 lower-case hex characters: an input's or an output's name and proof.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Sha256(String);

impl Sha256 {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for Sha256 {
    type Error = String;

    fn try_from(hex: String) -> Result<Self, String> {
        let valid = hex.len() == 64 && hex.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'));
        if valid {
            Ok(Self(hex))
        } else {
            Err(format!("{hex:?} is not a SHA-256 in lower-case hex"))
        }
    }
}

impl From<Sha256> for String {
    fn from(sha: Sha256) -> String {
        sha.0
    }
}

impl fmt::Display for Sha256 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests;
