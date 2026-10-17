//! The day: `state.json`, the duplicate guard, the file fingerprints, revert and amend
//! (07 › *Day folder*, *`state.json`*, *Revert and amend*).
//!
//! One `state.json` per day folder holds the batches, their saved PDFs and every saved order.
//! The file is written whole (to `state.json.tmp`, then replaced) so a crash never leaves half
//! a file. The engine owns the state; deleting a batch folder is [`delete_batch_folder`].

use std::fmt;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};

use jiff::Zoned;
use serde::{Deserialize, Serialize};
use xxhash_rust::xxh3::Xxh3;

use crate::names;
use crate::orders::ReadOrder;

/// A `state.json` could not be read or written, or a batch folder deleted.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DayError {
    /// `state.json` cannot be read: `<message>` (the reference implementation's `PickError`).
    Read { path: PathBuf, message: String },
    /// A folder cannot be written (08 › *Messages*: "Cannot write to `<folder>`: `<reason>`").
    Write { folder: PathBuf, message: String },
    /// A file is open in another program (08 › *Messages*: Windows refuses the delete/rename).
    FileInUse { path: PathBuf },
}

impl fmt::Display for DayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DayError::Read { path, message } => {
                write!(f, "cannot read {}: {message}", path.display())
            }
            DayError::Write { folder, message } => {
                write!(f, "Cannot write to {}: {message}", folder.display())
            }
            DayError::FileInUse { path } => write!(
                f,
                "{} is open in another program. Close it (e.g. the PDF viewer) and press Save again.",
                path.display()
            ),
        }
    }
}

impl std::error::Error for DayError {}

/// Windows `ERROR_SHARING_VIOLATION` / `ERROR_ACCESS_DENIED`: a file is open elsewhere.
fn in_use(error: &std::io::Error) -> bool {
    matches!(error.raw_os_error(), Some(32) | Some(5))
}

fn io_error(path: &Path, error: std::io::Error) -> DayError {
    if in_use(&error) {
        DayError::FileInUse {
            path: path.to_path_buf(),
        }
    } else {
        DayError::Write {
            folder: path.to_path_buf(),
            message: error.to_string(),
        }
    }
}

// ---------------------------------------------------------------------- state.json

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
    pub number: i64,
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
    pub order_id: String,
    pub tracking_id: String,
    pub batch: i64,
    pub pdf: i64,
    pub run: usize,
    /// The order's lines, each with its `name`, `variation`, `seller_sku` and `quantity`. Written
    /// at save (07 › *`state.json`*); empty on an order saved before search existed, which is
    /// found by Order ID or tracking ID only (*items not stored*, [Search](../ui/search.md)).
    #[serde(default)]
    pub items: Vec<SavedItem>,
    /// The courier named for the order; empty when it is unknown.
    #[serde(default)]
    pub courier: String,
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

/// One saved batch (07 › *`state.json`*).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchEntry {
    pub batch: i64,
    pub save_time: String,
    pub files: Vec<FileEntry>,
    pub pdfs: Vec<PdfEntry>,
    pub packing_list: PackingListChoice,
    /// The saved-PDF numbers of the batch whose **Printed** tick is set (07 › *`state.json`*);
    /// written when a tick is clicked; absent loads as empty. A note only: nothing is enforced,
    /// and it belongs to the batch, so a revert takes it and an amend writes the batch afresh.
    #[serde(default)]
    pub printed: Vec<i64>,
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
    pub fn is_printed(&self, pdf: i64) -> bool {
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
    pub fn next_batch_number(&self) -> i64 {
        self.batches
            .iter()
            .map(|batch| batch.batch)
            .max()
            .unwrap_or(0)
            + 1
    }

    /// The day's next saved-PDF number, after the last batch's last number (07 › *Save* step 1).
    pub fn next_pdf_number(&self) -> i64 {
        let from_batches = self
            .batches
            .iter()
            .flat_map(|batch| batch.pdfs.iter().map(|pdf| pdf.number))
            .max();
        let from_saved = self.saved.iter().map(|order| order.pdf).max();
        from_batches.max(from_saved).unwrap_or(0) + 1
    }

    /// The batch that read this file (`used in batch n`, 07 › *New batch*), by XXH3-64 and size.
    pub fn used_in_batch(&self, fingerprint: &Fingerprint) -> Option<i64> {
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

    fn saved_entry(&self, order_id: &str) -> Option<&SavedOrder> {
        self.saved
            .iter()
            .filter(|order| order.order_id == order_id)
            .min_by_key(|order| order.batch)
    }

    /// The duplicate guard: drop the orders already saved today and list them with the batch,
    /// saved-PDF number and time they were first saved (07 › *What happens in one batch* step 4).
    pub fn drop_saved(&self, orders: &[ReadOrder]) -> (Vec<ReadOrder>, Vec<Duplicate>) {
        let mut kept = Vec::new();
        let mut duplicates = Vec::new();
        for order in orders {
            match self.saved_entry(&order.order.order_id) {
                None => kept.push(order.clone()),
                Some(entry) => duplicates.push(Duplicate {
                    order_id: entry.order_id.clone(),
                    tracking_id: entry.tracking_id.clone(),
                    batch: entry.batch,
                    pdf: entry.pdf,
                    time: self
                        .batches
                        .iter()
                        .find(|batch| batch.batch == entry.batch)
                        .map(|batch| batch.save_time.clone())
                        .unwrap_or_default(),
                }),
            }
        }
        (kept, duplicates)
    }

    /// Undo the last batch: remove it and its orders from the state. Returns the removed batch
    /// (the caller deletes its folder with [`delete_batch_folder`]); the numbers are reused
    /// (07 › *Revert and amend*).
    pub fn revert_last(&mut self) -> Option<BatchEntry> {
        let last = self.batches.iter().map(|batch| batch.batch).max()?;
        let index = self.batches.iter().position(|batch| batch.batch == last)?;
        let removed = self.batches.remove(index);
        self.saved.retain(|order| order.batch != last);
        Some(removed)
    }

    /// Tick or untick saved-PDF row `pdf` of batch `batch` and write the day's `state.json`
    /// (07 › *`state.json`*; 08 › *1. Day*): the tick is a note only. Ticking twice leaves one
    /// entry, unticking removes it, and a batch or saved-PDF number the day does not hold changes
    /// nothing (nothing is written then). The file is written whole, as [`save`](Self::save) does
    /// (to `state.json.tmp`, then replace).
    pub fn set_printed(
        &mut self,
        day_dir: &Path,
        batch: i64,
        pdf: i64,
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

    /// Start an amend of the last batch (07 › *Revert and amend*): the `download/` file paths in
    /// their stored order, and the state with the last batch removed (the duplicate guard then
    /// ignores the batch's own orders, and the numbers are reused).
    pub fn amend(&self, day_dir: &Path) -> Option<Amend> {
        let last = self.batches.iter().map(|batch| batch.batch).max()?;
        let entry = self
            .batches
            .iter()
            .find(|batch| batch.batch == last)?
            .clone();
        let download = day_dir
            .join(names::batch_folder(last))
            .join(names::DOWNLOAD_DIR);
        let files = entry
            .files
            .iter()
            .map(|file| download.join(&file.name))
            .collect();
        let mut state = self.clone();
        state.revert_last();
        Some(Amend {
            batch: last,
            files,
            state,
        })
    }
}

/// An amend in progress: the last batch's `download/` files and the state without it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Amend {
    pub batch: i64,
    pub files: Vec<PathBuf>,
    pub state: DayState,
}

/// An order already saved today (07 › *What happens in one batch* step 4).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Duplicate {
    pub order_id: String,
    pub tracking_id: String,
    pub batch: i64,
    pub pdf: i64,
    pub time: String,
}

impl fmt::Display for Duplicate {
    // 08 › *Messages*: the warning lists `<tracking ID>` (batch `<b>`, PDF `<k>`, `<time>`).
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} (batch {}, PDF {}, {})",
            self.tracking_id, self.batch, self.pdf, self.time
        )
    }
}

/// The warning for the orders left out by the duplicate guard (08 › *Messages*).
pub fn duplicates_warning(duplicates: &[Duplicate]) -> String {
    let list: Vec<String> = duplicates.iter().map(|dup| dup.to_string()).collect();
    format!(
        "{} orders were already saved today and are left out: {}",
        duplicates.len(),
        list.join(", ")
    )
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

// ------------------------------------------------------------------- fingerprints

/// A label file's identity: size and XXH3-64 of its bytes (07 › *New batch*).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fingerprint {
    pub size: u64,
    pub xxh3: String,
}

/// The size and XXH3-64 of a file, streamed in chunks so a big file is not held in memory.
pub fn fingerprint(path: &Path) -> Result<Fingerprint, DayError> {
    let mut file = File::open(path).map_err(|error| DayError::Read {
        path: path.to_path_buf(),
        message: error.to_string(),
    })?;
    let mut hasher = Xxh3::new();
    let mut size = 0u64;
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let read = file.read(&mut buffer).map_err(|error| DayError::Read {
            path: path.to_path_buf(),
            message: error.to_string(),
        })?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
        size += read as u64;
    }
    Ok(Fingerprint {
        size,
        xxh3: format!("{:016x}", hasher.digest()),
    })
}

// ----------------------------------------------------------------------- save

/// Everything a finished batch adds to the day, except the files a closure writes (07 › *Save*).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BatchWrite {
    pub batch: i64,
    pub save_time: String,
    pub files: Vec<FileEntry>,
    pub pdfs: Vec<PdfEntry>,
    pub packing_list: PackingListChoice,
    /// The read warnings of the batch, stored in `state.json` (07 › *`state.json`*).
    pub warnings: Vec<String>,
    pub saved: Vec<SavedOrder>,
}

/// The current time as `state.json` stores it: local time with the UTC offset
/// (`2026-10-07T07:40:12+07:00`).
pub fn now_save_time() -> String {
    Zoned::now().strftime("%Y-%m-%dT%H:%M:%S%:z").to_string()
}

/// The check before saving's findings, stored with the batch (07 › *Check before saving*):
/// errors and warnings, kept apart. Empty when the check found nothing.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Findings {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

/// Write a batch completely or not at all (07 › *Save*): create `batch <b>.partial/`, let
/// `fill` write the saved PDFs, the packing list, `summary.txt` and `download/` into it and
/// return the check's [`Findings`], rename it to `batch <b>` (replacing an amended batch), then
/// add the batch to `state.json`. On any error before the rename the temporary folder is deleted
/// and the state is unchanged.
pub fn save_batch<F>(
    day_dir: &Path,
    state: &mut DayState,
    write: &BatchWrite,
    fill: F,
) -> Result<PathBuf, DayError>
where
    F: FnOnce(&Path) -> Result<Findings, DayError>,
{
    let temp = day_dir.join(names::temp_batch_folder(write.batch));
    if temp.exists() {
        fs::remove_dir_all(&temp).map_err(|error| io_error(&temp, error))?;
    }
    fs::create_dir_all(&temp).map_err(|error| io_error(&temp, error))?;

    let findings = match fill(&temp) {
        Ok(findings) => findings,
        Err(error) => {
            let _ = fs::remove_dir_all(&temp);
            return Err(error);
        }
    };

    let final_dir = day_dir.join(names::batch_folder(write.batch));
    if final_dir.exists() {
        fs::remove_dir_all(&final_dir).map_err(|error| io_error(&final_dir, error))?;
    }
    if let Err(error) = fs::rename(&temp, &final_dir) {
        let _ = fs::remove_dir_all(&temp);
        return Err(io_error(&final_dir, error));
    }

    let mut warnings = write.warnings.clone();
    warnings.extend(findings.warnings);
    state.batches.push(BatchEntry {
        batch: write.batch,
        save_time: write.save_time.clone(),
        files: write.files.clone(),
        pdfs: write.pdfs.clone(),
        packing_list: write.packing_list.clone(),
        // A batch written by a save (or written again by an amend) starts with no ticks: the
        // amend writes the batch afresh (07 › *`state.json`*).
        printed: Vec::new(),
        warnings,
        errors: findings.errors,
    });
    state.saved.extend(write.saved.iter().cloned());
    state.save(day_dir)?;
    Ok(final_dir)
}

/// Delete a batch folder (07 › *Revert* step 1); a file open in another program is a
/// [`DayError::FileInUse`].
pub fn delete_batch_folder(day_dir: &Path, batch: i64) -> Result<(), DayError> {
    let path = day_dir.join(names::batch_folder(batch));
    if !path.exists() {
        return Ok(());
    }
    fs::remove_dir_all(&path).map_err(|error| io_error(&path, error))
}
