//! The batch summary: the text printed and written to `summary.txt`.

use std::path::Path;

use packing_engine::plan::{PickResult, REST_CODE};

/// The pieces the batch summary is built from (03 › *Screen summary*).
pub struct SummaryInput<'a> {
    pub day: &'a str,
    pub files: usize,
    pub pages: usize,
    pub orders: usize,
    pub slip_pages: usize,
    pub picks: &'a [PickResult],
    pub duplicates: usize,
    pub warnings: &'a [String],
    /// The check before saving's errors (07 › *Check before saving*), printed when the batch was
    /// saved anyway.
    pub check_errors: &'a [String],
    /// The check before saving's warnings.
    pub check_warnings: &'a [String],
    pub day_dir: &'a Path,
    pub packing_lists: &'a [String],
    /// The developer's `--quiet-ids`: keep the warning count, drop the detail lines that name
    /// orders and tracking IDs.
    pub quiet_ids: bool,
}

/// The batch summary (03 › *Screen summary*), the text printed and written to `summary.txt`.
pub fn summary_text(input: &SummaryInput<'_>) -> String {
    let file_word = if input.files == 1 {
        "label file"
    } else {
        "label files"
    };
    let mut lines = vec![format!(
        "Day {} · {} {} · no CSV",
        input.day, input.files, file_word
    )];
    lines.push(format!(
        "  Pages: {} read · {} orders",
        input.pages, input.orders
    ));
    let items = if input.slip_pages > 0 {
        format!("slips ({} pages)", input.slip_pages)
    } else {
        "none".to_string()
    };
    lines.push(format!("  Item data: {items}"));
    for pick in input.picks {
        let kind = if pick.code == REST_CODE {
            "Rest"
        } else {
            "Pick"
        };
        lines.push(format!(
            "  {kind} {}  {}: {} orders, {} runs → {}",
            pick.code,
            pick.name,
            pick.orders.len(),
            pick.runs.len(),
            pick.file_name
        ));
    }
    lines.push(format!("  Duplicates: {}", input.duplicates));
    if input.warnings.is_empty() {
        lines.push("  Warnings: none".to_string());
    } else {
        lines.push(format!("  Warnings: {}", input.warnings.len()));
        if !input.quiet_ids {
            for warning in input.warnings {
                lines.push(format!("    - {warning}"));
            }
        }
    }
    // The check before saving's two lists, kept apart (07 › *Check before saving*).
    if !input.check_errors.is_empty() {
        lines.push(format!("  Check errors: {}", input.check_errors.len()));
        if !input.quiet_ids {
            for error in input.check_errors {
                lines.push(format!("    - {error}"));
            }
        }
    }
    if !input.check_warnings.is_empty() {
        lines.push(format!("  Check warnings: {}", input.check_warnings.len()));
        if !input.quiet_ids {
            for warning in input.check_warnings {
                lines.push(format!("    - {warning}"));
            }
        }
    }
    let pdf_word = if input.picks.len() == 1 {
        "saved PDF"
    } else {
        "saved PDFs"
    };
    let packing = if input.packing_lists.is_empty() {
        "no packing list".to_string()
    } else {
        input.packing_lists.join(", ")
    };
    lines.push(format!(
        "Written: {}/  ({} {}, {})",
        input.day_dir.display(),
        input.picks.len(),
        pdf_word,
        packing
    ));
    lines.join("\n") + "\n"
}
