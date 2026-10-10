//! The plan, the picks from what is left, the plan edits, the runs and the numbering
//! (07 › *What happens in one batch* step 5-6, *Plan*; 03 › *Picks*, *A saved PDF*,
//! *Numbering and file names*).
//!
//! The plan starts as the `categories.toml` entries in file order. Each entry takes its matching
//! orders from what is left, in order; the leftovers become the final rest pick
//! (`? Uncategorised`). Every change recounts the whole plan at once.

use std::fmt;

use crate::names::{self, NameError};
use crate::orders::ReadOrder;
use crate::rules::{Category, Condition, evaluate};

/// The rest pick's code and name when the last entry still has a condition (07 › *Plan*).
pub const REST_CODE: &str = "?";
pub const REST_NAME: &str = "Uncategorised";

/// The rest pick (the final `SavedPdf` of the reference implementation).
pub fn rest_category() -> Category {
    Category {
        code: REST_CODE.to_string(),
        name: REST_NAME.to_string(),
        condition: None,
        when: None,
    }
}

/// An invalid plan edit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlanError {
    /// A row index is past the end of the plan.
    Index { index: usize, len: usize },
    /// A move target is past the end of the plan.
    Position { position: usize, len: usize },
    /// A renamed saved PDF has an invalid name.
    Name(NameError),
}

impl fmt::Display for PlanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PlanError::Index { index, len } => {
                write!(f, "no row {index}: the plan has {len} rows")
            }
            PlanError::Position { position, len } => {
                write!(f, "cannot move to position {position} of {len} rows")
            }
            PlanError::Name(err) => err.fmt(f),
        }
    }
}

impl std::error::Error for PlanError {}

impl From<NameError> for PlanError {
    fn from(err: NameError) -> Self {
        PlanError::Name(err)
    }
}

/// One row of the plan: a pick from the rules file or one added for this batch only.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlanEntry {
    pub category: Category,
    /// Skipped for this batch only; its orders fall to the rows below (07 › *Plan*).
    pub skipped: bool,
    /// The owner's whole saved-PDF name; `None` = the suggested name (07 › *Plan*).
    pub file_name: Option<String>,
}

impl PlanEntry {
    fn new(category: Category) -> Self {
        Self {
            category,
            skipped: false,
            file_name: None,
        }
    }
}

/// The ordered rows of one batch's plan.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Plan {
    entries: Vec<PlanEntry>,
}

impl Plan {
    /// The rules applied in order: one row per `categories.toml` entry (08 › *Plan*).
    pub fn from_categories(categories: &[Category]) -> Self {
        Self {
            entries: categories.iter().cloned().map(PlanEntry::new).collect(),
        }
    }

    pub fn entries(&self) -> &[PlanEntry] {
        &self.entries
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    fn check_index(&self, index: usize) -> Result<(), PlanError> {
        if index < self.entries.len() {
            Ok(())
        } else {
            Err(PlanError::Index {
                index,
                len: self.entries.len(),
            })
        }
    }

    /// Skip a pick for this batch only (08 › *Plan*).
    pub fn skip(&mut self, index: usize) -> Result<(), PlanError> {
        self.set_skipped(index, true)
    }

    /// Take a skipped pick back into the plan.
    pub fn unskip(&mut self, index: usize) -> Result<(), PlanError> {
        self.set_skipped(index, false)
    }

    pub fn set_skipped(&mut self, index: usize, skipped: bool) -> Result<(), PlanError> {
        self.check_index(index)?;
        self.entries[index].skipped = skipped;
        Ok(())
    }

    /// Move a row one place up; the top row cannot move.
    pub fn move_up(&mut self, index: usize) -> Result<(), PlanError> {
        if index == 0 {
            return Ok(());
        }
        self.check_index(index)?;
        self.entries.swap(index - 1, index);
        Ok(())
    }

    /// Move a row one place down; the bottom row cannot move.
    pub fn move_down(&mut self, index: usize) -> Result<(), PlanError> {
        self.check_index(index)?;
        if index + 1 < self.entries.len() {
            self.entries.swap(index, index + 1);
        }
        Ok(())
    }

    /// Move the row at `from` to `to` (the target position after removal).
    pub fn move_to(&mut self, from: usize, to: usize) -> Result<(), PlanError> {
        self.check_index(from)?;
        if to > self.entries.len() - 1 {
            return Err(PlanError::Position {
                position: to,
                len: self.entries.len(),
            });
        }
        let entry = self.entries.remove(from);
        self.entries.insert(to, entry);
        Ok(())
    }

    /// Add a one-off pick at position `at` (08 › *Plan*: above the rest row).
    pub fn add(&mut self, at: usize, category: Category) -> Result<(), PlanError> {
        if at > self.entries.len() {
            return Err(PlanError::Position {
                position: at,
                len: self.entries.len(),
            });
        }
        self.entries.insert(at, PlanEntry::new(category));
        Ok(())
    }

    /// Remove a row (the Categories screen's 🗑; not the rules file).
    pub fn remove(&mut self, index: usize) -> Result<PlanEntry, PlanError> {
        self.check_index(index)?;
        Ok(self.entries.remove(index))
    }

    /// Change a pick's condition for this batch only.
    pub fn set_condition(
        &mut self,
        index: usize,
        condition: Option<Condition>,
    ) -> Result<(), PlanError> {
        self.check_index(index)?;
        self.entries[index].category.condition = condition;
        Ok(())
    }

    /// Set the whole saved-PDF name for a pick (07 › *Plan*; whole name editable).
    pub fn rename(&mut self, index: usize, name: &str) -> Result<(), PlanError> {
        self.check_index(index)?;
        self.entries[index].file_name = Some(names::normalize_saved_pdf_name(name)?);
        Ok(())
    }

    /// The counts and runs of every pick: the rows in order, each taking its matching orders
    /// from what is left; the leftovers are the rest pick. Numbering starts at `first_number`
    /// and only picks that take orders get a number (03 › *Picks*).
    pub fn count(&self, orders: &[ReadOrder], first_number: i64) -> PlanResult {
        let mut remaining: Vec<&ReadOrder> = orders.iter().collect();
        let mut picks: Vec<PickResult> = Vec::new();
        let mut number = first_number;

        for entry in &self.entries {
            if entry.skipped {
                continue;
            }
            let (taken, left) = split_matching(&remaining, entry.category.condition.as_ref());
            remaining = left;
            if taken.is_empty() {
                continue;
            }
            picks.push(build_pick(number, entry, &taken));
            number += 1;
        }

        if !remaining.is_empty() {
            let rest = PlanEntry::new(rest_category());
            picks.push(build_pick(number, &rest, &remaining));
        }

        PlanResult { picks }
    }
}

/// Split the orders into the ones a condition takes and the ones left (a `None` condition takes
/// every order = the rest entry).
fn split_matching<'a>(
    orders: &[&'a ReadOrder],
    condition: Option<&Condition>,
) -> (Vec<&'a ReadOrder>, Vec<&'a ReadOrder>) {
    let mut taken = Vec::new();
    let mut left = Vec::new();
    for order in orders {
        let matches = match condition {
            None => true,
            Some(cond) => evaluate(cond, &order.order),
        };
        if matches {
            taken.push(*order);
        } else {
            left.push(*order);
        }
    }
    (taken, left)
}

fn build_pick(number: i64, entry: &PlanEntry, orders: &[&ReadOrder]) -> PickResult {
    let category = &entry.category;
    let file_name = entry.file_name.clone().unwrap_or_else(|| {
        names::suggested_saved_pdf_name(number, &category.code, &category.name, orders.len())
    });
    PickResult {
        number,
        code: category.code.clone(),
        name: category.name.clone(),
        orders: orders
            .iter()
            .map(|order| order.order.order_id.clone())
            .collect(),
        runs: runs(orders),
        pages: orders.iter().map(|order| order.pages.len()).sum(),
        file_name,
    }
}

/// An order's contents key: per line (name, variation, quantity), sorted (`Order::contents`).
type ContentsKey = Vec<(String, String, i64)>;

/// Re-order a pick's orders into runs: identical contents together, runs sorted by (number of
/// orders descending, contents text ascending); inside a run the download page order (03 ›
/// *A saved PDF*).
fn runs(orders: &[&ReadOrder]) -> Vec<RunResult> {
    let mut groups: Vec<(ContentsKey, Vec<&ReadOrder>)> = Vec::new();
    for order in orders {
        let key = order.order.contents();
        match groups.iter_mut().find(|(existing, _)| *existing == key) {
            Some((_, group)) => group.push(*order),
            None => groups.push((key, vec![*order])),
        }
    }
    groups.sort_by(|a, b| b.1.len().cmp(&a.1.len()).then_with(|| a.0.cmp(&b.0)));
    groups
        .into_iter()
        .enumerate()
        .map(|(index, (_, group))| RunResult {
            number: index + 1,
            orders: group
                .iter()
                .map(|order| order.order.order_id.clone())
                .collect(),
        })
        .collect()
}

/// One run inside a saved PDF: `number` is 1-based in printing order (written `3-05`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RunResult {
    pub number: usize,
    /// The order IDs in download order.
    pub orders: Vec<String>,
}

/// One pick: the saved PDF it will become (07 › *Plan*).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PickResult {
    pub number: i64,
    pub code: String,
    pub name: String,
    /// The order IDs in download order.
    pub orders: Vec<String>,
    pub runs: Vec<RunResult>,
    /// The label pages of the pick's orders.
    pub pages: usize,
    /// The suggested name, or the owner's name.
    pub file_name: String,
}

/// Every pick of a plan, in printing order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlanResult {
    pub picks: Vec<PickResult>,
}

impl PlanResult {
    pub fn total_orders(&self) -> usize {
        self.picks.iter().map(|pick| pick.orders.len()).sum()
    }

    pub fn total_pages(&self) -> usize {
        self.picks.iter().map(|pick| pick.pages).sum()
    }

    pub fn file_names(&self) -> Vec<String> {
        self.picks
            .iter()
            .map(|pick| pick.file_name.clone())
            .collect()
    }

    /// Refuse two saved PDFs of one batch with the same name (07 › *Plan*).
    pub fn check_file_names(&self) -> Result<(), NameError> {
        names::check_unique_names(&self.file_names())
    }
}
