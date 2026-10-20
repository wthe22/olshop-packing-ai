//! The `state.json` model and its atomic read/write: the batches, their saved PDFs, every saved
//! order, and the numbers a new batch takes (07 › *`state.json`*, *Day folder*, *Save*).

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::groupings::Grouping;
use crate::ids::{BatchNumber, OrderId, PdfNumber, TrackingId};
use crate::names;

use super::error::{DayError, io_error};
use super::fingerprint::Fingerprint;

/// A label file read in a batch: name, size, XXH3-64 and page count (07 › *`state.json`*).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileEntry {
    pub name: String,
    pub size: u64,
    /// XXH3-64 of the whole file, 16 lowercase hex characters (07 › *`state.json`*).
    #[serde(default)]
    pub xxh3: String,
    pub pages: usize,
    /// The SHA-256 a day written before 2.20 stored here. Such a day still opens; the value is
    /// never matched, since the "used twice" hint only matters inside one day, and it is never
    /// written back: a saved `state.json` carries `xxh3` only.
    #[serde(rename = "sha256", default, skip_serializing)]
    pub legacy_sha256: Option<String>,
}

/// One saved PDF of a batch (07 › *`state.json`*).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PdfEntry {
    pub number: PdfNumber,
    pub code: String,
    pub name: String,
    pub file: String,
    pub orders: usize,
    pub runs: usize,
}

/// Where the batch's packing list goes (07 › *Packing list*).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Scope {
    PerPdf,
    Whole,
    None,
}

/// A packing-list layout.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Layout {
    Full,
    Summary,
    Pick,
}

impl Layout {
    /// The name the command line and the packing-list file names use.
    pub fn as_str(self) -> &'static str {
        match self {
            Layout::Full => "full",
            Layout::Summary => "summary",
            Layout::Pick => "pick",
        }
    }

    /// Whether this layout prints each run's tracking IDs.
    pub fn tracking(self) -> bool {
        self == Layout::Full
    }
}

impl std::fmt::Display for Layout {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for Layout {
    type Err = String;

    fn from_str(name: &str) -> Result<Self, Self::Err> {
        match name {
            "full" => Ok(Layout::Full),
            "summary" => Ok(Layout::Summary),
            "pick" => Ok(Layout::Pick),
            other => Err(other.to_string()),
        }
    }
}

impl std::str::FromStr for Scope {
    type Err = String;

    fn from_str(name: &str) -> Result<Self, Self::Err> {
        match name {
            "per-pdf" | "per_pdf" | "perpdf" => Ok(Scope::PerPdf),
            "whole" => Ok(Scope::Whole),
            "none" => Ok(Scope::None),
            other => Err(other.to_string()),
        }
    }
}

/// The batch's packing-list choice (07 › *Packing list*).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackingListChoice {
    pub scope: Scope,
    pub layouts: Vec<Layout>,
}

impl Default for PackingListChoice {
    fn default() -> Self {
        Self {
            scope: Scope::Whole,
            layouts: vec![Layout::Pick],
        }
    }
}

/// One item line of a saved order, as the Search panel finds it (07 › *`state.json`*): the slip's
/// `name`, `variation`, `seller_sku` and `quantity`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SavedItem {
    pub name: String,
    #[serde(default)]
    pub variation: String,
    #[serde(default)]
    pub seller_sku: String,
    pub quantity: i64,
}

/// One order saved by a batch; the duplicate guard reads these and the Search panel finds them by
/// the fields kept here (07 › *`state.json`*).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SavedOrder {
    pub order_id: OrderId,
    /// The courier's parcel number; `None` when the label printed none. `state.json` stores the
    /// empty string for that, so the field is written back the same way.
    #[serde(
        default,
        serialize_with = "serialize_tracking_id",
        deserialize_with = "deserialize_tracking_id"
    )]
    pub tracking_id: Option<TrackingId>,
    pub batch: BatchNumber,
    pub pdf: PdfNumber,
    pub run: usize,
    /// The order's lines, each with its `name`, `variation`, `seller_sku` and `quantity`. Written
    /// at save (07 › *`state.json`*); empty on an order saved before search existed, which is
    /// found by Order ID or tracking ID only (*items not stored*, [Search](../ui/search.md)).
    #[serde(default)]
    pub items: Vec<SavedItem>,
    /// The courier named for the order; `None` when it is unknown. `state.json` stores the
    /// empty string for that, so the field is written back the same way.
    #[serde(
        default,
        serialize_with = "serialize_courier",
        deserialize_with = "deserialize_courier"
    )]
    pub courier: Option<String>,
    /// The order's total quantity (the sum of its lines).
    #[serde(default)]
    pub total_quantity: i64,
    /// The number of lines of the order.
    #[serde(default)]
    pub distinct_items: i64,
    /// The label's *In transit by*, as `YYYY-MM-DD HH:MM:SS` text; absent when the label had none.
    #[serde(default)]
    pub ship_by: Option<String>,
}

/// A saved order's tracking ID as `state.json` keeps it: text, and the empty string when the
/// label printed none.
fn serialize_tracking_id<S>(
    tracking_id: &Option<TrackingId>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(tracking_id.as_ref().map_or("", TrackingId::as_str))
}

fn deserialize_tracking_id<'de, D>(deserializer: D) -> Result<Option<TrackingId>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let text = String::deserialize(deserializer)?;
    Ok((!text.is_empty()).then(|| TrackingId::new(text)))
}

/// A saved order's courier as `state.json` keeps it: text, and the empty string when the
/// courier is unknown.
fn serialize_courier<S>(courier: &Option<String>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(courier.as_deref().unwrap_or(""))
}

fn deserialize_courier<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let text = String::deserialize(deserializer)?;
    Ok((!text.is_empty()).then_some(text))
}

/// One saved batch (07 › *`state.json`*).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchEntry {
    pub batch: BatchNumber,
    pub save_time: String,
    /// The grouping the batch was packed with (07 › *`state.json`*; 04 › *Sets*); written at
    /// save and absent in an older file, which loads as `default`. An **Amend** reads the
    /// batch's own grouping again, whatever grouping was used last.
    #[serde(rename = "set", default)]
    pub grouping: Grouping,
    pub files: Vec<FileEntry>,
    pub pdfs: Vec<PdfEntry>,
    pub packing_list: PackingListChoice,
    /// The saved-PDF numbers of the batch whose **Printed** tick is set (07 › *`state.json`*);
    /// written when a tick is clicked; absent loads as empty. A note only: nothing is enforced,
    /// and it belongs to the batch, so a revert takes it and an amend writes the batch afresh.
    #[serde(default)]
    pub printed: Vec<PdfNumber>,
    /// The read warnings of the batch, as shown on the Plan screen (07 › *`state.json`*), plus
    /// the check before saving's warnings; the Day screen *Warnings ▸* reads them without
    /// re-reading the files. Defaulted so an older file still loads.
    #[serde(default)]
    pub warnings: Vec<String>,
    /// The check before saving's errors, kept apart from the warnings (07 › *Check before
    /// saving*); present only when the batch was saved anyway. Defaulted so an older file loads.
    #[serde(default)]
    pub errors: Vec<String>,
}

impl BatchEntry {
    /// Whether saved-PDF row `pdf` of this batch is ticked (07 › *`state.json`*).
    pub fn is_printed(&self, pdf: PdfNumber) -> bool {
        self.printed.contains(&pdf)
    }
}

/// The day's `state.json`: everything needed to continue, revert or amend a batch.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DayState {
    pub day: String,
    #[serde(default)]
    pub batches: Vec<BatchEntry>,
    #[serde(default)]
    pub saved: Vec<SavedOrder>,
}

/// The path of a day folder's `state.json`.
pub fn state_path(day_dir: &Path) -> PathBuf {
    day_dir.join(names::STATE_FILE)
}

impl DayState {
    /// An empty state for a day; `day` is the day folder's name.
    pub fn new(day: impl Into<String>) -> Self {
        Self {
            day: day.into(),
            batches: Vec::new(),
            saved: Vec::new(),
        }
    }

    /// Read the day's `state.json`; a missing file is an empty state for the folder's day
    /// (the reference implementation's `load_state`).
    pub fn load(day_dir: &Path) -> Result<Self, DayError> {
        let path = state_path(day_dir);
        if !path.is_file() {
            return Ok(Self::new(folder_day(day_dir)));
        }
        let text = fs::read_to_string(&path).map_err(|error| DayError::Read {
            path: path.clone(),
            message: error.to_string(),
        })?;
        serde_json::from_str(&text).map_err(|error| DayError::Read {
            path: path.clone(),
            message: error.to_string(),
        })
    }

    /// Write the whole state: `state.json.tmp`, then replace, so a crash never leaves half a
    /// file (07 › *`state.json`*).
    pub fn save(&self, day_dir: &Path) -> Result<(), DayError> {
        fs::create_dir_all(day_dir).map_err(|error| io_error(day_dir, error))?;
        let mut text = serde_json::to_string_pretty(self).expect("state serializes");
        text.push('\n');
        let path = state_path(day_dir);
        let tmp = day_dir.join(format!("{}.tmp", names::STATE_FILE));
        fs::write(&tmp, text).map_err(|error| io_error(&tmp, error))?;
        replace(&tmp, &path).map_err(|error| io_error(&path, error))
    }

    /// The next batch number `1, 2, 3, …` within the day.
    pub fn next_batch_number(&self) -> BatchNumber {
        self.batches
            .iter()
            .map(|batch| batch.batch)
            .max()
            .map_or(BatchNumber::new(1), BatchNumber::next)
    }

    /// The day's next saved-PDF number, after the last batch's last number (07 › *Save* step 1).
    pub fn next_pdf_number(&self) -> PdfNumber {
        let from_batches = self
            .batches
            .iter()
            .flat_map(|batch| batch.pdfs.iter().map(|pdf| pdf.number))
            .max();
        let from_saved = self.saved.iter().map(|order| order.pdf).max();
        from_batches
            .max(from_saved)
            .map_or(PdfNumber::new(1), PdfNumber::next)
    }

    /// The batch that read this file (`used in batch n`, 07 › *New batch*), by XXH3-64 and size.
    pub fn used_in_batch(&self, fingerprint: &Fingerprint) -> Option<BatchNumber> {
        self.batches
            .iter()
            .find(|batch| {
                batch.files.iter().any(|file| {
                    file.xxh3.eq_ignore_ascii_case(&fingerprint.xxh3)
                        && file.size == fingerprint.size
                })
            })
            .map(|batch| batch.batch)
    }

    /// Tick or untick saved-PDF row `pdf` of batch `batch` and write the day's `state.json`
    /// (07 › *`state.json`*; 08 › *1. Day*): the tick is a note only. Ticking twice leaves one
    /// entry, unticking removes it, and a batch or saved-PDF number the day does not hold changes
    /// nothing (nothing is written then). The file is written whole, as [`save`](Self::save) does
    /// (to `state.json.tmp`, then replace).
    pub fn set_printed(
        &mut self,
        day_dir: &Path,
        batch: BatchNumber,
        pdf: PdfNumber,
        printed: bool,
    ) -> Result<(), DayError> {
        let Some(entry) = self.batches.iter_mut().find(|entry| entry.batch == batch) else {
            return Ok(());
        };
        if !entry.pdfs.iter().any(|row| row.number == pdf) {
            return Ok(());
        }
        if printed {
            if !entry.printed.contains(&pdf) {
                entry.printed.push(pdf);
            }
        } else {
            entry.printed.retain(|number| *number != pdf);
        }
        self.save(day_dir)
    }
}

fn folder_day(day_dir: &Path) -> String {
    day_dir
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// Replace `to` with `from`, over an existing `to` if needed.
fn replace(from: &Path, to: &Path) -> std::io::Result<()> {
    match fs::rename(from, to) {
        Ok(()) => Ok(()),
        Err(_) if to.exists() => {
            fs::remove_file(to)?;
            fs::rename(from, to)
        }
        Err(error) => Err(error),
    }
}
