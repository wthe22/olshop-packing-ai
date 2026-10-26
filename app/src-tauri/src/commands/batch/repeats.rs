//! Repeated orders (duplicates.md): the batch's repeats as the screen shows them — the groups of
//! files that repeat each other, each file's copies kept and left out, each repeated order's
//! copies and the one kept — and the owner's choices sent back.

use std::collections::BTreeSet;

use packing_batch::Draft;
use packing_engine::ids::OrderId;
use packing_engine::repeats::{Choices, Difference, GroupChoice, LeftOutBecause, RepeatGroup};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct RepeatsView {
    /// Orders that appear more than once.
    orders: usize,
    /// Copies beyond one per order, and their pages.
    extra_copies: usize,
    extra_pages: usize,
    /// Files that hold a repeat, of all the batch's files.
    files: usize,
    all_files: usize,
    groups: Vec<GroupView>,
}

#[derive(Serialize)]
pub struct GroupView {
    number: usize,
    files: Vec<FileView>,
    choice: ChoiceView,
    /// An order of the group has its copy chosen by hand.
    by_hand: bool,
    /// *Take file f out*, for each file whose repeated orders all have a copy in another file.
    take_out: Vec<TakeOutView>,
    orders: Vec<OrderView>,
}

/// One file of a group: its number in the batch (from 1), and its copies of repeated orders.
#[derive(Serialize)]
pub struct FileView {
    file: usize,
    name: String,
    pages: u16,
    copies: usize,
    kept: usize,
    left_out: usize,
}

#[derive(Serialize)]
pub struct TakeOutView {
    file: usize,
    /// The file's orders that are in no other file: the choice is shown, not offered.
    only_here: usize,
}

#[derive(Serialize)]
pub struct OrderView {
    order_id: String,
    tracking_id: Option<String>,
    /// None: every page of every copy prints the same.
    difference: Option<DifferenceView>,
    copies: Vec<CopyView>,
    /// The copy that stays, by its `copy`.
    kept: usize,
    by_hand: bool,
}

#[derive(Serialize)]
pub struct CopyView {
    /// Its place in the batch's copies: what a choice by hand names.
    copy: usize,
    file: usize,
    page_start: u16,
}

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DifferenceView {
    TrackingId,
    LineCount,
    PageCount,
    Pages,
}

/// A group's choice, as the screen shows and sends it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ChoiceView {
    KeepFirst,
    KeepLast,
    TakeOut { file: usize },
}

/// The owner's choices: one per group, and the orders chosen by hand.
#[derive(Debug, Deserialize)]
pub struct RepeatChoices {
    groups: Vec<GroupChoiceInput>,
    by_hand: Vec<ByHand>,
}

#[derive(Debug, Deserialize)]
pub struct GroupChoiceInput {
    group: usize,
    choice: ChoiceView,
}

#[derive(Debug, Deserialize)]
pub struct ByHand {
    order_id: String,
    copy: usize,
}

impl RepeatChoices {
    pub(super) fn choices(&self) -> Choices {
        Choices {
            groups: self
                .groups
                .iter()
                .map(|group| {
                    let choice = match group.choice {
                        ChoiceView::KeepFirst => GroupChoice::KeepFirst,
                        ChoiceView::KeepLast => GroupChoice::KeepLast,
                        ChoiceView::TakeOut { file } => {
                            GroupChoice::TakeOut(file.saturating_sub(1))
                        }
                    };
                    (group.group, choice)
                })
                .collect(),
            by_hand: self
                .by_hand
                .iter()
                .map(|order| (OrderId::new(&order.order_id), order.copy))
                .collect(),
        }
    }
}

/// The batch's repeats; none when every order has one copy.
pub(super) fn repeats_view(draft: &Draft) -> Option<RepeatsView> {
    let orders = &draft.orders;
    if orders.repeats.is_empty() {
        return None;
    }
    let (repeated, extra_copies, extra_pages) = orders.repeat_counts(&draft.copies);
    let groups: Vec<GroupView> = orders.groups.iter().map(|g| group_view(draft, g)).collect();
    Some(RepeatsView {
        orders: repeated,
        extra_copies,
        extra_pages,
        files: groups.iter().map(|g| g.files.len()).sum(),
        all_files: draft.files.len(),
        groups,
    })
}

fn group_view(draft: &Draft, group: &RepeatGroup) -> GroupView {
    let copies = &draft.copies;
    let repeats: Vec<_> = group
        .repeats
        .iter()
        .map(|&at| &draft.orders.repeats[at])
        .collect();
    let left: BTreeSet<usize> = draft
        .orders
        .left_out
        .iter()
        .filter(|left| matches!(left.because, LeftOutBecause::Repeat { .. }))
        .map(|left| left.copy)
        .collect();
    let in_group: Vec<usize> = repeats.iter().flat_map(|r| r.copies.clone()).collect();
    let files = group
        .inputs
        .iter()
        .map(|&input| {
            let here: Vec<usize> = in_group
                .iter()
                .copied()
                .filter(|&c| copies[c].input == input)
                .collect();
            let left_out = here.iter().filter(|c| left.contains(c)).count();
            let file = &draft.files[input];
            FileView {
                file: input + 1,
                name: file.filename.clone(),
                pages: file.page_count,
                copies: here.len(),
                kept: here.len() - left_out,
                left_out,
            }
        })
        .collect();
    let take_out = group
        .inputs
        .iter()
        .filter(|&&input| {
            repeats
                .iter()
                .all(|r| r.copies.iter().any(|&c| copies[c].input != input))
        })
        .map(|&input| TakeOutView {
            file: input + 1,
            only_here: draft
                .orders
                .kept
                .iter()
                .filter(|&&c| copies[c].input == input && !in_group.contains(&c))
                .count(),
        })
        .collect();
    let orders: Vec<OrderView> = repeats
        .iter()
        .map(|repeat| {
            let kept = repeat
                .copies
                .iter()
                .copied()
                .find(|c| !left.contains(c))
                .unwrap_or(repeat.copies[0]);
            OrderView {
                order_id: repeat.order_id.to_string(),
                tracking_id: copies[repeat.copies[0]]
                    .tracking_id
                    .as_ref()
                    .map(ToString::to_string),
                difference: repeat.difference.map(|difference| match difference {
                    Difference::TrackingId => DifferenceView::TrackingId,
                    Difference::LineCount => DifferenceView::LineCount,
                    Difference::PageCount => DifferenceView::PageCount,
                    Difference::Pages => DifferenceView::Pages,
                }),
                copies: repeat
                    .copies
                    .iter()
                    .map(|&c| CopyView {
                        copy: c,
                        file: copies[c].input + 1,
                        page_start: copies[c].page_start,
                    })
                    .collect(),
                kept,
                by_hand: draft.choices.by_hand.contains_key(&repeat.order_id),
            }
        })
        .collect();
    let choice = match draft
        .choices
        .groups
        .get(&group.number)
        .copied()
        .unwrap_or_default()
    {
        GroupChoice::KeepFirst => ChoiceView::KeepFirst,
        GroupChoice::KeepLast => ChoiceView::KeepLast,
        GroupChoice::TakeOut(input) => ChoiceView::TakeOut { file: input + 1 },
    };
    GroupView {
        number: group.number,
        files,
        choice,
        by_hand: orders.iter().any(|order| order.by_hand),
        take_out,
        orders,
    }
}
