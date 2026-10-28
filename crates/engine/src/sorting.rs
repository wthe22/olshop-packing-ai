//! Steps 9 and 10 of the workflow (03-plan.md, groups.md): a batch's orders sorted into label
//! groups — each group, in order, takes the orders its filter matches from what is left; what is
//! left at the end is the rest — and inside each group into order contents, which give the
//! printing order and the file names.

use std::fmt;

use crate::copies::Copy;
use crate::ids::BatchId;
use crate::record::{ItemPick, OrderContent};
use crate::record::{LabelGroupRule, LabelSortRule};
use crate::rules::evaluate_filter;
use crate::workflows::{Courier, Filter, SortRule};

/// The rest group when the last group has a filter or was skipped: Windows allows no `?` in a
/// file name, so its file name has no code.
pub const REST_CODE: &str = "?";
pub const REST_NAME: &str = "Uncategorised";

/// A file name is cut at this many characters.
pub const NAME_LIMIT: usize = 100;

/// One group of a batch's plan: the sort rule's, or a one-off, in the order the owner left them.
#[derive(Clone, Debug)]
pub struct PlanGroup {
    pub code: String,
    pub name: String,
    /// None: takes every order left.
    pub filter: Option<Filter>,
    /// The owner's file name, without `.pdf`; none for the suggested one.
    pub file_name: Option<String>,
}

/// The plan as a saved batch keeps it: the rule's name and the groups as used.
pub fn plan_snapshot(rule_name: &str, plan: &[PlanGroup]) -> LabelSortRule {
    LabelSortRule {
        name: rule_name.to_string(),
        label_groups: plan
            .iter()
            .map(|group| LabelGroupRule {
                code: group.code.clone(),
                name: group.name.clone(),
                filter: group.filter.as_ref().map(|filter| filter.text.clone()),
            })
            .collect(),
    }
}

/// The plan a sort rule proposes: its groups in file order.
pub fn plan_groups(rule: &SortRule) -> Vec<PlanGroup> {
    rule.groups
        .iter()
        .map(|group| PlanGroup {
            code: group.code.clone(),
            name: group.name.clone(),
            filter: group.filter.clone(),
            file_name: None,
        })
        .collect()
}

/// The orders of one order content inside a group, in reading position.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContentBlock {
    pub content: OrderContent,
    /// Copy indexes.
    pub orders: Vec<usize>,
}

/// A label group that took orders, and its group PDF.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SortedGroup {
    pub code: String,
    pub name: String,
    /// The group PDF's name in `output/`, with `.pdf`.
    pub file: String,
    /// Most orders first, then by content.
    pub contents: Vec<ContentBlock>,
}

impl SortedGroup {
    pub fn order_count(&self) -> usize {
        self.contents.iter().map(|block| block.orders.len()).sum()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SortedBatch {
    pub batch: BatchId,
    /// The groups that took an order, in plan order; the rest last.
    pub groups: Vec<SortedGroup>,
}

impl SortedBatch {
    /// Every order as (group index, copy index), in printing order: group by group, content by
    /// content, then reading position.
    pub fn printing_order(&self) -> Vec<(usize, usize)> {
        self.groups
            .iter()
            .enumerate()
            .flat_map(|(group, sorted)| {
                sorted
                    .contents
                    .iter()
                    .flat_map(move |block| block.orders.iter().map(move |&copy| (group, copy)))
            })
            .collect()
    }

    /// `{batch} summary.pdf`.
    pub fn summary_file(&self) -> String {
        format!("{} summary.pdf", self.batch)
    }
}

/// An order's content — its lines as (name, variation, quantity), sorted — with the Seller SKU of
/// each item in that order, which an order content does not hold.
pub fn content_of(copy: &Copy) -> (OrderContent, Vec<Option<String>>) {
    let mut items: Vec<(ItemPick, Option<String>)> = copy
        .lines
        .iter()
        .map(|line| {
            let item = ItemPick {
                name: line.name.clone(),
                variant: line.variation.clone(),
                quantity: u16::try_from(line.quantity).unwrap_or(u16::MAX),
            };
            let sku = (!line.seller_sku.is_empty()).then(|| line.seller_sku.clone());
            (item, sku)
        })
        .collect();
    items.sort();
    let (items, skus) = items.into_iter().unzip();
    (OrderContent { items }, skus)
}

/// Sort the kept copies of a batch by its plan; `couriers` name the couriers a filter compares.
pub fn sort_batch(
    copies: &[Copy],
    kept: &[usize],
    plan: &[PlanGroup],
    batch: BatchId,
    couriers: &[Courier],
) -> SortedBatch {
    let mut remaining: Vec<usize> = kept.to_vec();
    remaining.sort_unstable();
    let mut taken: Vec<(&str, &str, Option<&str>, Vec<usize>)> = Vec::new();
    for group in plan {
        let (mine, rest): (Vec<usize>, Vec<usize>) = remaining.iter().partition(|&&copy| {
            group
                .filter
                .as_ref()
                .is_none_or(|filter| evaluate_filter(&filter.condition, &copies[copy], couriers))
        });
        remaining = rest;
        taken.push((&group.code, &group.name, group.file_name.as_deref(), mine));
    }
    if !remaining.is_empty() {
        taken.push((REST_CODE, REST_NAME, None, remaining));
    }
    let groups = taken
        .into_iter()
        .filter(|(_, _, _, orders)| !orders.is_empty())
        .map(|(code, name, file_name, orders)| {
            let contents = blocks(copies, &orders);
            let file = match file_name {
                Some(owner) => format!("{owner}.pdf"),
                None => suggested_file(batch, code, name, orders.len()),
            };
            SortedGroup {
                code: code.to_string(),
                name: name.to_string(),
                file,
                contents,
            }
        })
        .collect();
    SortedBatch { batch, groups }
}

fn blocks(copies: &[Copy], orders: &[usize]) -> Vec<ContentBlock> {
    let mut blocks: Vec<ContentBlock> = Vec::new();
    for &copy in orders {
        let (content, _) = content_of(&copies[copy]);
        match blocks.iter_mut().find(|block| block.content == content) {
            Some(block) => block.orders.push(copy),
            None => blocks.push(ContentBlock {
                content,
                orders: vec![copy],
            }),
        }
    }
    blocks.sort_by(|a, b| {
        b.orders
            .len()
            .cmp(&a.orders.len())
            .then_with(|| a.content.cmp(&b.content))
    });
    blocks
}

/// `{batch}-{code} {name} x{count}.pdf` by the default file-name rule; the rest has no code.
pub fn suggested_file(batch: BatchId, code: &str, name: &str, count: usize) -> String {
    let stem = if code == REST_CODE {
        format!("{batch} {name} x{count}")
    } else {
        format!("{batch}-{code} {name} x{count}")
    };
    format!("{}.pdf", default_file_name(&stem))
}

/// The default file-name rule (data/README.md): letters, digits, space, `-` and `_` stay, any
/// other character becomes a space, repeated spaces become one, the ends are trimmed, and the
/// name is cut at [`NAME_LIMIT`] characters.
pub fn default_file_name(text: &str) -> String {
    let spaced: String = text
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || matches!(c, ' ' | '-' | '_') {
                c
            } else {
                ' '
            }
        })
        .collect();
    let joined = spaced.split_whitespace().collect::<Vec<_>>().join(" ");
    joined
        .chars()
        .take(NAME_LIMIT)
        .collect::<String>()
        .trim_end()
        .to_string()
}

/// An owner's file name that cannot be used.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FileNameError {
    /// Empty, or with a character or a name Windows does not allow.
    NotAllowed { name: String },
    /// Two outputs of the batch would have the same name (upper and lower case count as the same).
    Twice { name: String },
}

impl fmt::Display for FileNameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FileNameError::NotAllowed { name } => {
                write!(f, "\"{name}\" is not a file name Windows allows")
            }
            FileNameError::Twice { name } => {
                write!(f, "two files of the batch are named \"{name}\"")
            }
        }
    }
}

/// The batch's outputs must be valid Windows file names, unique when case is ignored.
pub fn check_file_names(files: &[String]) -> Result<(), FileNameError> {
    let mut seen: Vec<String> = Vec::new();
    for file in files {
        let stem = file.strip_suffix(".pdf").unwrap_or(file);
        if !allowed_on_windows(stem) {
            return Err(FileNameError::NotAllowed { name: file.clone() });
        }
        let folded = file.to_lowercase();
        if seen.contains(&folded) {
            return Err(FileNameError::Twice { name: file.clone() });
        }
        seen.push(folded);
    }
    Ok(())
}

fn allowed_on_windows(stem: &str) -> bool {
    const RESERVED: [&str; 22] = [
        "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
        "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
    ];
    !stem.trim().is_empty()
        && !stem.ends_with(['.', ' '])
        && !stem.chars().any(|c| {
            c.is_control() || matches!(c, '\\' | '/' | ':' | '*' | '?' | '"' | '<' | '>' | '|')
        })
        && !RESERVED
            .iter()
            .any(|reserved| stem.eq_ignore_ascii_case(reserved))
}
