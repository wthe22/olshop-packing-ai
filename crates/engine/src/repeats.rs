//! Steps 7 and 8 of the workflow (03-plan.md, duplicates.md): the copies of a batch become its
//! orders. An order saved in a live batch of the date loses every copy; an order with two or more
//! copies is a repeat, and exactly one of its copies stays — by the owner's choice per group of
//! inputs, the first by default. Every other copy is left out, and why is kept.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::copies::Copy;
use crate::ids::{BatchId, OrderId};

/// What the copies of a repeated order differ in, the first difference that can be named.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Difference {
    TrackingId,
    LineCount,
    PageCount,
    /// The pages print something else, but none of the above.
    Pages,
}

/// An order with two or more copies, by their indexes in the batch's copies (reading order).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Repeat {
    pub order_id: OrderId,
    pub copies: Vec<usize>,
    /// None: every page of every copy prints the same.
    pub difference: Option<Difference>,
}

/// Inputs that repeat each other: one repeated order has copies in two of them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RepeatGroup {
    /// `G1`, `G2`, … by the group's lowest input.
    pub number: usize,
    pub inputs: Vec<usize>,
    pub repeats: Vec<usize>,
}

/// The owner's choice for one group of inputs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum GroupChoice {
    #[default]
    KeepFirst,
    KeepLast,
    /// Leave out the copies of the group's repeated orders that lie in this input.
    TakeOut(usize),
}

/// The choices of a batch: one per group (by number), and an order's copy chosen by hand, which
/// overrides its group's choice.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Choices {
    pub groups: BTreeMap<usize, GroupChoice>,
    pub by_hand: BTreeMap<OrderId, usize>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LeftOutBecause {
    /// A copy of a repeat; the copy kept instead.
    Repeat { kept: usize },
    /// The order is in this live batch of the date already.
    SavedEarlier { batch: BatchId },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LeftOut {
    pub copy: usize,
    pub because: LeftOutBecause,
}

/// The orders of a batch, one copy each, and the copies left out.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Orders {
    /// One copy per order, in reading order.
    pub kept: Vec<usize>,
    pub left_out: Vec<LeftOut>,
    pub repeats: Vec<Repeat>,
    pub groups: Vec<RepeatGroup>,
}

impl Orders {
    /// The orders that repeat, the copies beyond one per order, and their pages.
    pub fn repeat_counts(&self, copies: &[Copy]) -> (usize, usize, usize) {
        let extra: Vec<&LeftOut> = self
            .left_out
            .iter()
            .filter(|left| matches!(left.because, LeftOutBecause::Repeat { .. }))
            .collect();
        let pages = extra
            .iter()
            .map(|left| {
                let copy = &copies[left.copy];
                (copy.page_end - copy.page_start + 1) as usize
            })
            .sum();
        (self.repeats.len(), extra.len(), pages)
    }
}

/// A choice that cannot apply; nothing is chosen.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ChoiceError {
    /// A repeated order of the input has no copy in another input of the group.
    CannotTakeOut { group: usize, input: usize },
    /// The copy chosen by hand is not a copy of that order.
    NotACopy { order_id: OrderId, copy: usize },
}

impl fmt::Display for ChoiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChoiceError::CannotTakeOut { group, input } => write!(
                f,
                "input {} of G{group} cannot be taken out: an order of it has no other copy",
                input + 1
            ),
            ChoiceError::NotACopy { order_id, copy } => {
                write!(f, "copy {copy} is not a copy of order {order_id}")
            }
        }
    }
}

/// The batch's orders from its copies. `saved` holds the orders of the date's live batches.
pub fn sort_out(
    copies: &[Copy],
    saved: &BTreeMap<OrderId, BatchId>,
    choices: &Choices,
) -> Result<Orders, ChoiceError> {
    let mut by_order: Vec<(OrderId, Vec<usize>)> = Vec::new();
    for (index, copy) in copies.iter().enumerate() {
        match by_order.iter_mut().find(|(id, _)| *id == copy.order_id) {
            Some((_, list)) => list.push(index),
            None => by_order.push((copy.order_id.clone(), vec![index])),
        }
    }
    let mut kept = Vec::new();
    let mut left_out = Vec::new();
    let mut repeats = Vec::new();
    for (order_id, list) in by_order {
        if let Some(batch) = saved.get(&order_id) {
            left_out.extend(list.iter().map(|&copy| LeftOut {
                copy,
                because: LeftOutBecause::SavedEarlier { batch: *batch },
            }));
        } else if list.len() == 1 {
            kept.push(list[0]);
        } else {
            let difference = difference(copies, &list);
            repeats.push(Repeat {
                order_id,
                copies: list,
                difference,
            });
        }
    }
    let groups = groups(copies, &repeats);
    for group in &groups {
        let choice = choices
            .groups
            .get(&group.number)
            .copied()
            .unwrap_or_default();
        for &at in &group.repeats {
            let repeat = &repeats[at];
            let keep = match choices.by_hand.get(&repeat.order_id) {
                Some(&copy) if repeat.copies.contains(&copy) => copy,
                Some(&copy) => {
                    return Err(ChoiceError::NotACopy {
                        order_id: repeat.order_id.clone(),
                        copy,
                    });
                }
                None => chosen(copies, repeat, group, choice)?,
            };
            kept.push(keep);
            left_out.extend(
                repeat
                    .copies
                    .iter()
                    .filter(|&&copy| copy != keep)
                    .map(|&copy| LeftOut {
                        copy,
                        because: LeftOutBecause::Repeat { kept: keep },
                    }),
            );
        }
    }
    kept.sort_unstable();
    left_out.sort_by_key(|left| left.copy);
    Ok(Orders {
        kept,
        left_out,
        repeats,
        groups,
    })
}

fn chosen(
    copies: &[Copy],
    repeat: &Repeat,
    group: &RepeatGroup,
    choice: GroupChoice,
) -> Result<usize, ChoiceError> {
    match choice {
        GroupChoice::KeepFirst => Ok(repeat.copies[0]),
        GroupChoice::KeepLast => Ok(*repeat.copies.last().expect("a repeat has copies")),
        GroupChoice::TakeOut(input) => repeat
            .copies
            .iter()
            .copied()
            .find(|&copy| copies[copy].input != input)
            .ok_or(ChoiceError::CannotTakeOut {
                group: group.number,
                input,
            }),
    }
}

fn difference(copies: &[Copy], list: &[usize]) -> Option<Difference> {
    let first = &copies[list[0]];
    let others = list[1..].iter().map(|&at| &copies[at]);
    let mut found = None;
    for other in others {
        let this = if other.tracking_id != first.tracking_id {
            Some(Difference::TrackingId)
        } else if other.lines.len() != first.lines.len() {
            Some(Difference::LineCount)
        } else if other.fingerprints.len() != first.fingerprints.len() {
            Some(Difference::PageCount)
        } else if other.fingerprints != first.fingerprints {
            Some(Difference::Pages)
        } else {
            None
        };
        found = found.or(this);
    }
    found
}

/// The connected sets of inputs that repeated orders join, numbered by their lowest input.
fn groups(copies: &[Copy], repeats: &[Repeat]) -> Vec<RepeatGroup> {
    let mut sets: Vec<(BTreeSet<usize>, Vec<usize>)> = Vec::new();
    for (at, repeat) in repeats.iter().enumerate() {
        let inputs: BTreeSet<usize> = repeat.copies.iter().map(|&c| copies[c].input).collect();
        let (joined, apart): (Vec<_>, Vec<_>) = sets
            .into_iter()
            .partition(|(set, _)| !set.is_disjoint(&inputs));
        let mut merged = (inputs, vec![at]);
        for (set, members) in joined {
            merged.0.extend(set);
            merged.1.extend(members);
        }
        sets = apart;
        sets.push(merged);
    }
    sets.sort_by_key(|(set, _)| *set.iter().next().expect("a group has an input"));
    sets.into_iter()
        .enumerate()
        .map(|(index, (inputs, mut members))| {
            members.sort_unstable();
            RepeatGroup {
                number: index + 1,
                inputs: inputs.into_iter().collect(),
                repeats: members,
            }
        })
        .collect()
}
