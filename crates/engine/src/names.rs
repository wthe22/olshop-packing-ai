//! File and folder names (07 › *Day folder*; 03 › *Numbering and file names*).
//!
//! The suggested saved-PDF name is `<n> <code> <name> ×<orders>.pdf`; the owner may replace the
//! whole name (`normalize_saved_pdf_name`). Characters Windows forbids in a name become `-`,
//! and the `<name>` part is cut at [`NAME_LIMIT`] characters ending in `…`.

use std::fmt;

use jiff::civil::Date;

/// `<name>` limit; keeps file names clear of the Windows path limit (03).
pub const NAME_LIMIT: usize = 100;

/// The batch's day state (07 › *`state.json`*).
pub const STATE_FILE: &str = "state.json";
/// The day folder with scope `whole` and one layout (07 › *Day folder*).
pub const PACKING_LIST_FILE: &str = "packing-list.pdf";
/// The batch summary text.
pub const SUMMARY_FILE: &str = "summary.txt";
/// The batch's copy of the label files read; amend reads them back (07 › *Day folder*).
pub const DOWNLOAD_DIR: &str = "download";

/// A saved-PDF or batch name is invalid.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NameError {
    /// The name is empty (after trimming and dropping the extension).
    Empty,
    /// Two picks would write the same file name; `first`/`second` are 0-based positions.
    Duplicate {
        name: String,
        first: usize,
        second: usize,
    },
}

impl fmt::Display for NameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NameError::Empty => f.write_str("the file name is empty"),
            NameError::Duplicate {
                name,
                first,
                second,
            } => write!(
                f,
                "saved PDFs {} and {} have the same name \"{}\"",
                first + 1,
                second + 1,
                name
            ),
        }
    }
}

impl std::error::Error for NameError {}

fn is_illegal(c: char) -> bool {
    matches!(c, '\\' | '/' | ':' | '*' | '?' | '"' | '<' | '>' | '|')
}

/// Characters Windows forbids in a file name (`\ / : * ? " < > |`) become `-` (07 › *Day folder*).
pub fn sanitize_file_name(name: &str) -> String {
    name.chars()
        .map(|c| if is_illegal(c) { '-' } else { c })
        .collect()
}

/// Cut a name part at [`NAME_LIMIT`] characters, ending in `…` (03, 07).
fn cut_name(name: &str) -> String {
    if name.chars().count() > NAME_LIMIT {
        let mut cut: String = name.chars().take(NAME_LIMIT - 1).collect();
        cut.push('…');
        cut
    } else {
        name.to_string()
    }
}

/// The suggested saved-PDF name `<n> <code> <name> ×<orders>.pdf` (07 › *Plan*): the `<name>`
/// part is cut at [`NAME_LIMIT`], then illegal characters in the whole name become `-` (the code
/// `?` of the rest pick becomes `-`).
pub fn suggested_saved_pdf_name(number: i64, code: &str, name: &str, orders: usize) -> String {
    let name = cut_name(name);
    sanitize_file_name(&format!("{number} {code} {name} ×{orders}.pdf"))
}

/// The owner's whole saved-PDF name: trimmed, missing `.pdf` added, illegal characters become
/// `-`, the stem cut at [`NAME_LIMIT`]. An empty name is [`NameError::Empty`].
pub fn normalize_saved_pdf_name(name: &str) -> Result<String, NameError> {
    let trimmed = name.trim();
    let stem = if trimmed.len() >= 4 && trimmed[trimmed.len() - 4..].eq_ignore_ascii_case(".pdf") {
        &trimmed[..trimmed.len() - 4]
    } else {
        trimmed
    };
    let stem = sanitize_file_name(&cut_name(stem.trim_end()));
    if stem.is_empty() {
        return Err(NameError::Empty);
    }
    Ok(format!("{stem}.pdf"))
}

/// A batch folder name `batch <b>` (07 › *Day folder*).
pub fn batch_folder(batch: i64) -> String {
    format!("batch {batch}")
}

/// The temporary folder a save is written into before it is renamed (07 › *Save*).
pub fn temp_batch_folder(batch: i64) -> String {
    format!("batch {batch}.partial")
}

/// A day folder name `YYYY-MM-DD`.
pub fn day_folder(date: Date) -> String {
    date.strftime("%Y-%m-%d").to_string()
}

/// A packing-list file name (07 › *Day folder*): `number` is the saved-PDF number for scope
/// `per-pdf` (`None` for `whole`); `layout` is added only when several layouts are written.
pub fn packing_list_file_name(number: Option<i64>, layout: Option<&str>) -> String {
    let mut name = String::from("packing-list");
    if let Some(number) = number {
        name.push_str(&format!("-{number}"));
    }
    if let Some(layout) = layout {
        name.push_str(&format!("-{layout}"));
    }
    name.push_str(".pdf");
    name
}

/// Refuse two saved PDFs of one batch writing the same name (07 › *Plan*). The comparison is
/// case-insensitive, as Windows file names are.
pub fn check_unique_names(names: &[String]) -> Result<(), NameError> {
    for (second, name) in names.iter().enumerate() {
        if let Some(first) = names[..second]
            .iter()
            .position(|other| other.eq_ignore_ascii_case(name))
        {
            return Err(NameError::Duplicate {
                name: name.clone(),
                first,
                second,
            });
        }
    }
    Ok(())
}
