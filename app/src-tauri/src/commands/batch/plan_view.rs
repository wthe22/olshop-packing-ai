//! The Plan screen's rows: the plan and the draft's orders mapped into the view, and the window's
//! plan edits applied to the open plan.

use std::collections::HashMap;

use packing_engine::groupings::Grouping;
use packing_engine::ids::PdfNumber;
use packing_engine::orders::{Order, ReadOrder, display_name};
use packing_engine::plan::{PickResult, Plan};
use packing_engine::rules::{Category, Condition, evaluate, parse_condition};

use super::views::{PlanEdit, PlanRow, PlanView, RunView};

/// Map the plan and the draft's orders into the Plan screen's rows (08 › *3. Plan*). Every entry
/// becomes a row (skipped and 0-order rows included); the leftovers, if any, become the final
/// `? Uncategorised` row. The numbers, runs and names come from [`Plan::count`] so the screen and
/// the saved PDFs agree. `set` is the picks set the plan was built from; the screen's header
/// names it.
pub(super) fn plan_view(
    plan: &Plan,
    orders: &[ReadOrder],
    first_number: PdfNumber,
    grouping: &Grouping,
) -> PlanView {
    let result = plan.count(orders, first_number);
    let entries = plan.entries();
    let index = order_index(orders);

    // The last entry when it has no condition is the rest row and stays last (08 › *3. Plan*).
    let rest_entry = match entries.last() {
        Some(last) if last.category.condition.is_none() => Some(entries.len() - 1),
        _ => None,
    };
    let add_at = rest_entry.unwrap_or(entries.len());

    let mut remaining: Vec<&ReadOrder> = orders.iter().collect();
    let mut picks = result.picks.iter();
    let mut rows: Vec<PlanRow> = Vec::with_capacity(entries.len() + 1);

    for (position, entry) in entries.iter().enumerate() {
        let is_rest = rest_entry == Some(position);
        let base =
            |number, orders: usize, runs, pages, file_name: String, empty, skipped| PlanRow {
                index: Some(position),
                number,
                code: entry.category.code.clone(),
                name: entry.category.name.clone(),
                condition: entry.category.when.clone(),
                orders,
                runs,
                pages,
                file_name,
                skipped,
                generated: false,
                is_rest,
                duplicate_of: None,
                can_use: true,
                can_move_up: position > 0 && !is_rest,
                can_move_down: position + 1 < entries.len()
                    && rest_entry != Some(position + 1)
                    && !is_rest,
                empty,
            };
        if entry.skipped {
            rows.push(base(None, 0, Vec::new(), 0, String::new(), false, true));
            continue;
        }
        let (taken, left) = split(&remaining, entry.category.condition.as_ref());
        remaining = left;
        if taken.is_empty() {
            rows.push(base(None, 0, Vec::new(), 0, String::new(), true, false));
        } else {
            let pick = picks.next().expect("a pick for every non-empty entry");
            rows.push(base(
                Some(pick.number),
                pick.orders.len(),
                run_views(pick, &index),
                pick.pages,
                pick.file_name.clone(),
                false,
                false,
            ));
        }
    }

    // The leftovers when no entry took them become `? Uncategorised` (08 › *3. Plan*).
    if !remaining.is_empty() {
        let pick = picks.next().expect("the rest pick");
        rows.push(PlanRow {
            index: None,
            number: Some(pick.number),
            code: pick.code.clone(),
            name: pick.name.clone(),
            condition: None,
            orders: pick.orders.len(),
            runs: run_views(pick, &index),
            pages: pick.pages,
            file_name: pick.file_name.clone(),
            skipped: false,
            generated: true,
            is_rest: true,
            duplicate_of: None,
            can_use: false,
            can_move_up: false,
            can_move_down: false,
            empty: false,
        });
    }

    // Duplicate saved-PDF names, case-insensitive as Windows is (08 › *3. Plan*).
    let mut seen: Vec<(String, PdfNumber)> = Vec::new();
    let mut has_duplicate_names = false;
    for row in &mut rows {
        let Some(number) = row.number else {
            continue;
        };
        let key = row.file_name.to_lowercase();
        if let Some((_, first)) = seen.iter().find(|(name, _)| *name == key) {
            row.duplicate_of = Some(*first);
            has_duplicate_names = true;
        } else {
            seen.push((key, number));
        }
    }

    PlanView {
        rows,
        has_duplicate_names,
        add_at,
        no_picks: plan.unconfigured(),
        grouping: grouping.clone(),
    }
}

fn order_index(orders: &[ReadOrder]) -> HashMap<&str, &ReadOrder> {
    orders
        .iter()
        .map(|order| (order.order.order_id.as_str(), order))
        .collect()
}

/// Split the orders into the ones a condition takes and the ones left; `None` takes every order
/// (the rest entry). Mirrors the engine's `count` so the row counts match the saved PDFs.
fn split<'a>(
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

/// A pick's runs as the expanded row shows them; the label is `<pick number>-<run number>`.
fn run_views(pick: &PickResult, index: &HashMap<&str, &ReadOrder>) -> Vec<RunView> {
    pick.runs
        .iter()
        .map(|run| RunView {
            label: format!("{}-{:02}", pick.number, run.number),
            contents: run
                .orders
                .first()
                .and_then(|id| index.get(id.as_str()))
                .map(|order| run_contents(&order.order))
                .unwrap_or_default(),
            orders: run.orders.len(),
        })
        .collect()
}

/// The identical contents of a run, as text: each line `display name ×quantity`.
fn run_contents(order: &Order) -> String {
    order
        .lines
        .iter()
        .map(|line| {
            format!(
                "{} ×{}",
                display_name(&line.name, &line.variation),
                line.quantity
            )
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// Apply the window's edits to the open plan in order (08 › *3. Plan*). A bad condition text or a
/// bad file name stops with the engine's message (line/col for a condition).
pub(super) fn apply_edits(plan: &mut Plan, edits: &[PlanEdit]) -> Result<(), String> {
    for edit in edits {
        match edit {
            PlanEdit::SetSkipped { index, skipped } => plan.set_skipped(*index, *skipped),
            PlanEdit::MoveUp { index } => plan.move_up(*index),
            PlanEdit::MoveDown { index } => plan.move_down(*index),
            PlanEdit::Add {
                at,
                code,
                name,
                condition,
            } => {
                let code = code.trim();
                if code.is_empty() {
                    return Err("the pick code must not be empty".to_string());
                }
                if name.trim().is_empty() {
                    return Err("the pick name must not be empty".to_string());
                }
                let condition = condition.trim();
                let node = parse_condition(condition).map_err(|error| error.to_string())?;
                let category = Category {
                    code: code.to_string(),
                    name: name.trim().to_string(),
                    condition: Some(node),
                    when: Some(condition.to_string()),
                };
                plan.add(*at, category)
            }
            PlanEdit::Rename { index, name } => plan.rename(*index, name),
        }
        .map_err(|error| error.to_string())?;
    }
    Ok(())
}
