//! The Plan screen (03-plan.md, steps 9 and 10): the draft's groups as rows with their orders,
//! order contents and file names, and the owner's edits for this batch only — skip, move, add a
//! one-off group, rename a file. Every edit sorts the whole batch again.

use packing_engine::orders::display_name;
use packing_engine::rules::parse_filter;
use packing_engine::sorting::{PlanGroup, REST_CODE, SortedGroup};
use packing_engine::workflows::Filter;
use packing_store::Fold;

use crate::{OpenDraft, PlanEntry};

use super::views::{PlanEdit, PlanRow, PlanRowKind, PlanView, RunView};

pub(super) fn plan_view(open: &OpenDraft, fold: &Fold) -> PlanView {
    let draft = &open.draft;
    let sorted = draft.sorted(draft.batch(fold));
    let entries = &open.entries;
    let movable = movable(entries);
    let mut taken = sorted.groups.iter().peekable();
    let mut rows: Vec<PlanRow> = Vec::new();
    for (index, entry) in entries.iter().enumerate() {
        let group = if entry.skipped {
            None
        } else {
            taken.next_if(|group| group.code == entry.group.code)
        };
        let kind = if index < movable {
            PlanRowKind::Entry { index }
        } else {
            PlanRowKind::Rest { index }
        };
        let mut row = row(open, kind, &entry.group, group);
        row.skipped = entry.skipped;
        row.can_move_up = index < movable && index > 0;
        row.can_move_down = index + 1 < movable;
        rows.push(row);
    }
    if let Some(rest) = taken.find(|group| group.code == REST_CODE) {
        let generated = PlanGroup {
            code: rest.code.clone(),
            name: rest.name.clone(),
            filter: None,
            file_name: None,
        };
        rows.push(row(open, PlanRowKind::Generated, &generated, Some(rest)));
    }
    mark_duplicates(&mut rows);
    PlanView {
        has_duplicate_names: rows.iter().any(|row| row.duplicate_of.is_some()),
        rows,
        add_at: movable,
        no_groups: entries.len() == 1 && entries[0].group.filter.is_none(),
        sort_rule: draft.rule_name.clone(),
    }
}

/// The groups that move: all but a last one without a filter, which takes the rest.
fn movable(entries: &[PlanEntry]) -> usize {
    match entries.last() {
        Some(last) if last.group.filter.is_none() => entries.len() - 1,
        _ => entries.len(),
    }
}

fn row(
    open: &OpenDraft,
    kind: PlanRowKind,
    group: &PlanGroup,
    sorted: Option<&SortedGroup>,
) -> PlanRow {
    let copies = &open.draft.copies;
    let runs: Vec<RunView> = sorted
        .map(|sorted| {
            sorted
                .contents
                .iter()
                .map(|block| RunView {
                    contents: block
                        .content
                        .items
                        .iter()
                        .map(|item| {
                            format!(
                                "{} ×{}",
                                display_name(&item.name, &item.variant),
                                item.quantity
                            )
                        })
                        .collect::<Vec<_>>()
                        .join(", "),
                    orders: block.orders.len(),
                })
                .collect()
        })
        .unwrap_or_default();
    let pages = sorted.map_or(0, |sorted| {
        sorted
            .contents
            .iter()
            .flat_map(|block| &block.orders)
            .map(|&at| usize::from(copies[at].page_end - copies[at].page_start) + 1)
            .sum()
    });
    PlanRow {
        kind,
        code: group.code.clone(),
        name: group.name.clone(),
        condition: group.filter.as_ref().map(|filter| filter.text.clone()),
        orders: sorted.map_or(0, SortedGroup::order_count),
        runs,
        pages,
        file_name: sorted.map_or_else(String::new, |sorted| sorted.file.clone()),
        skipped: false,
        duplicate_of: None,
        can_move_up: false,
        can_move_down: false,
    }
}

/// A row whose file has the name of an earlier row's file (upper and lower case the same).
fn mark_duplicates(rows: &mut [PlanRow]) {
    for at in 0..rows.len() {
        if rows[at].file_name.is_empty() {
            continue;
        }
        let name = rows[at].file_name.to_lowercase();
        rows[at].duplicate_of = rows[..at]
            .iter()
            .find(|earlier| earlier.file_name.to_lowercase() == name)
            .map(|earlier| earlier.code.clone());
    }
}

/// Apply the edits in order, then sort by the plan they leave. An edit that names no group, or a
/// filter that does not parse, stops with its message and the plan as before.
pub(super) fn apply_edits(open: &mut OpenDraft, edits: &[PlanEdit]) -> Result<(), String> {
    let mut entries = open.entries.clone();
    for edit in edits {
        let movable = movable(&entries);
        match edit {
            PlanEdit::SetSkipped { index, skipped } => {
                entry(&mut entries, *index)?.skipped = *skipped
            }
            PlanEdit::MoveUp { index } if *index > 0 && *index < movable => {
                entries.swap(index - 1, *index);
            }
            PlanEdit::MoveDown { index } if index + 1 < movable => entries.swap(*index, index + 1),
            PlanEdit::MoveUp { .. } | PlanEdit::MoveDown { .. } => {}
            PlanEdit::Add {
                at,
                code,
                name,
                condition,
            } => {
                let parsed = parse_filter(condition).map_err(|error| error.to_string())?;
                let group = PlanGroup {
                    code: code.clone(),
                    name: name.clone(),
                    filter: Some(Filter {
                        text: condition.clone(),
                        condition: parsed,
                    }),
                    file_name: None,
                };
                let entry = PlanEntry {
                    group,
                    skipped: false,
                };
                entries.insert((*at).min(movable), entry);
            }
            PlanEdit::Rename { index, name } => {
                let stem = name.trim().trim_end_matches(".pdf").to_string();
                entry(&mut entries, *index)?.group.file_name = (!stem.is_empty()).then_some(stem);
            }
        }
    }
    open.entries = entries;
    open.apply_plan();
    Ok(())
}

fn entry(entries: &mut [PlanEntry], index: usize) -> Result<&mut PlanEntry, String> {
    entries
        .get_mut(index)
        .ok_or_else(|| format!("The plan has no group {}.", index + 1))
}

/// The plan's groups as the draft first sorts by them.
pub(super) fn entries_of(sorted_plan: &[PlanGroup]) -> Vec<PlanEntry> {
    sorted_plan
        .iter()
        .map(|group| PlanEntry {
            group: group.clone(),
            skipped: false,
        })
        .collect()
}
