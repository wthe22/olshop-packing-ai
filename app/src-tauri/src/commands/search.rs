//! Search and Reprint (search.md, reprint.md): saved orders of the open workspace found by a
//! filter on the search index, each with its trace line and the reprints that printed it again;
//! one PDF of the chosen orders' pages written into today's date; and a reprint's orders as Search
//! shows them.

use std::collections::BTreeSet;
use std::ops::RangeInclusive;

use jiff::civil::Date;
use packing_batch::{ReprintOutcome, ReprintRequest, reprint as write_reprint};
use packing_engine::ids::{BatchId, OrderId};
use packing_engine::record::{CourierClue, CourierClueKind};
use packing_engine::rules::parse_filter;
use packing_store::{IndexedOrder, SearchIndex};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State};

use crate::commands::day::date_of;
use crate::{APP_VERSION, AppState, today};

/// The dates a search covers, both ends included, as `2026-10-07`.
#[derive(Deserialize)]
pub struct DateRange {
    pub(crate) from: String,
    pub(crate) to: String,
}

impl DateRange {
    fn dates(&self) -> Result<RangeInclusive<Date>, String> {
        let (from, to) = (date_of(&self.from)?, date_of(&self.to)?);
        if from > to {
            return Err(format!(
                "The dates run backwards: {} is after {}.",
                self.from, self.to
            ));
        }
        Ok(from..=to)
    }
}

#[derive(Serialize)]
pub struct SearchView {
    /// The search index is still being built: no rows yet.
    loading: bool,
    orders: usize,
    batches: usize,
    rows: Vec<OrderRow>,
}

#[derive(Serialize)]
pub struct OrderRow {
    day: String,
    order_id: String,
    tracking_id: Option<String>,
    /// The courier's code; none when no courier was named.
    courier: Option<String>,
    items: Vec<ItemView>,
    batch: BatchId,
    /// `<batch>-<code>`.
    group: String,
    /// The group PDF's file name, for *Open*.
    file: String,
    trace: Vec<TracePart>,
    /// The reprints that printed it again, oldest first.
    reprints: Vec<ReprintMarkView>,
}

/// A piece of a row's trace line; a logo's hash or a tracking-ID form is `code`, which the window
/// sets in monospace.
#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct TracePart {
    text: String,
    code: bool,
}

impl TracePart {
    fn text(text: impl Into<String>) -> Self {
        TracePart {
            text: text.into(),
            code: false,
        }
    }

    fn code(text: impl Into<String>) -> Self {
        TracePart {
            text: text.into(),
            code: true,
        }
    }
}

#[derive(Serialize)]
pub struct ReprintMarkView {
    /// The date the reprint was made on.
    day: String,
    number: u8,
}

#[derive(Serialize)]
pub struct ItemView {
    name: String,
    quantity: u16,
}

/// The saved orders `filter` matches on the dates of `range`; none searches every date.
pub(crate) fn search(
    state: &AppState,
    filter: &str,
    range: Option<&DateRange>,
) -> Result<SearchView, String> {
    let filter = parse_filter(filter).map_err(|error| error.to_string())?;
    let dates = match range {
        Some(range) => range.dates()?,
        None => Date::MIN..=Date::MAX,
    };
    let rows = state.index().with(|index| {
        index
            .search(&filter, dates)
            .into_iter()
            .map(|order| row(index, order))
            .collect::<Vec<_>>()
    })?;
    let Some(rows) = rows else {
        return Ok(SearchView {
            loading: true,
            orders: 0,
            batches: 0,
            rows: Vec::new(),
        });
    };
    let batches: BTreeSet<(&str, BatchId)> = rows
        .iter()
        .map(|row| (row.day.as_str(), row.batch))
        .collect();
    Ok(SearchView {
        loading: false,
        orders: rows.len(),
        batches: batches.len(),
        rows,
    })
}

fn row(index: &SearchIndex, order: &IndexedOrder) -> OrderRow {
    let saved = &order.order;
    OrderRow {
        day: order.date.to_string(),
        order_id: saved.order_id.to_string(),
        tracking_id: saved.tracking_id.as_ref().map(ToString::to_string),
        courier: saved.courier.as_ref().map(|code| code.as_str().to_string()),
        items: order
            .items
            .iter()
            .map(|item| ItemView {
                name: item.name.clone(),
                quantity: item.quantity,
            })
            .collect(),
        batch: order.batch,
        group: format!("{}-{}", order.batch, saved.group),
        file: order.file.clone(),
        trace: trace_line(order),
        reprints: index
            .reprints_of(order)
            .into_iter()
            .map(|mark| ReprintMarkView {
                day: mark.date.to_string(),
                number: mark.number,
            })
            .collect(),
    }
}

/// `from <input> p. 12–13 · courier jnt by logo 39b9… + form J[XY]\d{10} · batch 2 · 2-A`, the
/// hash and the form as `code`.
pub(crate) fn trace_line(order: &IndexedOrder) -> Vec<TracePart> {
    let saved = &order.order;
    let pages = if saved.page_start == saved.page_end {
        format!("p. {}", saved.page_start)
    } else {
        format!("p. {}–{}", saved.page_start, saved.page_end)
    };
    let mut parts = vec![TracePart::text(format!("from {} {pages} · ", order.input))];
    match &saved.courier {
        None => parts.push(TracePart::text("no courier named")),
        Some(code) => {
            parts.push(TracePart::text(format!("courier {}", code.as_str())));
            parts.extend(clue_parts(&saved.courier_clue));
        }
    }
    parts.push(TracePart::text(format!(
        " · batch {} · {}-{}",
        order.batch, order.batch, saved.group
    )));
    parts
}

/// ` by logo 39b9… + form J[XY]\d{10}`: what named the courier; nothing when no clue is kept.
fn clue_parts(clue: &CourierClue) -> Vec<TracePart> {
    let mut parts = Vec::new();
    if let Some(value) = &clue.value {
        parts.push(TracePart::text(format!(" by {} ", clue.kind.as_str())));
        parts.push(match clue.kind {
            CourierClueKind::Logo | CourierClueKind::Shape => TracePart::code(value),
            _ => TracePart::text(value),
        });
    }
    if let Some(form) = &clue.form {
        let joint = if parts.is_empty() { " by" } else { " +" };
        parts.push(TracePart::text(format!("{joint} form ")));
        parts.push(TracePart::code(form));
    }
    parts
}

#[tauri::command]
pub fn search_orders(
    state: State<'_, AppState>,
    filter: String,
    range: Option<DateRange>,
) -> Result<SearchView, String> {
    search(&state, &filter, range.as_ref())
}

/// One ticked row: the order and the date it was saved on.
#[derive(Deserialize)]
pub struct ReprintChoice {
    pub(crate) day: String,
    pub(crate) order_id: String,
}

#[derive(Serialize)]
pub struct ReprintView {
    made: Option<MadeView>,
    /// The verifier's findings: nothing was kept.
    errors: Vec<String>,
}

#[derive(Serialize)]
pub struct MadeView {
    number: u8,
    file: String,
    path: String,
    orders: usize,
    pages: u16,
}

pub(crate) fn reprint_orders(
    state: &AppState,
    choices: &[ReprintChoice],
) -> Result<ReprintView, String> {
    let orders = choices
        .iter()
        .map(|choice| Ok((date_of(&choice.day)?, OrderId::new(&choice.order_id))))
        .collect::<Result<Vec<_>, String>>()?;
    state.with_data(|data| {
        let data_dir = data.dir.clone();
        let day = data.open_day(today())?;
        let outcome = state
            .with_worker(|worker| {
                write_reprint(
                    &ReprintRequest {
                        worker,
                        data_dir: &data_dir,
                        today: &day.folder,
                        orders: &orders,
                        app: APP_VERSION,
                    },
                    &mut day.journal,
                )
            })?
            .map_err(|error| error.to_string())?;
        state.index().refresh(day.journal.fold());
        Ok(match outcome {
            ReprintOutcome::Made(made) => ReprintView {
                made: Some(MadeView {
                    number: made.number,
                    path: day
                        .folder
                        .dir()
                        .join("output")
                        .join(&made.file)
                        .display()
                        .to_string(),
                    file: made.file,
                    orders: made.orders.len(),
                    pages: made.pages,
                }),
                errors: Vec::new(),
            },
            ReprintOutcome::Findings(findings) => ReprintView {
                made: None,
                errors: findings
                    .errors
                    .into_iter()
                    .chain(findings.warnings)
                    .collect(),
            },
        })
    })
}

/// **Reprint `<n>` orders**, on its own thread.
#[tauri::command]
pub async fn reprint(app: AppHandle, orders: Vec<ReprintChoice>) -> Result<ReprintView, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        reprint_orders(&state, &orders)
    })
    .await
    .map_err(|error| format!("the reprint task stopped: {error}"))?
}

/// One order of a reprint: where it was saved, and the order as Search shows it.
#[derive(Serialize)]
pub struct ReprintedView {
    order_id: String,
    /// The date it was saved on.
    day: String,
    batch: BatchId,
    /// None once no live batch holds it (its batch was reverted since).
    found: Option<OrderRow>,
}

#[derive(Serialize)]
pub struct ReprintedOrdersView {
    /// The search index is still being built: no orders yet.
    loading: bool,
    /// In printing order.
    orders: Vec<ReprintedView>,
}

pub(crate) fn reprinted_orders(
    state: &AppState,
    day: &str,
    number: u8,
) -> Result<ReprintedOrdersView, String> {
    let date = date_of(day)?;
    let printed = state.with_data(|data| {
        let open = data.open_day(date)?;
        open.journal
            .fold()
            .reprints
            .iter()
            .find(|reprint| reprint.made.number == number)
            .map(|reprint| reprint.made.orders.clone())
            .ok_or_else(|| format!("{day} has no reprint {number}."))
    })?;
    let orders = state.index().with(|index| {
        printed
            .iter()
            .map(|order| ReprintedView {
                order_id: order.order_id.to_string(),
                day: order.date.to_string(),
                batch: order.batch,
                found: index
                    .find(order.date, &order.order_id)
                    .map(|found| row(index, found)),
            })
            .collect()
    })?;
    Ok(match orders {
        Some(orders) => ReprintedOrdersView {
            loading: false,
            orders,
        },
        None => ReprintedOrdersView {
            loading: true,
            orders: Vec::new(),
        },
    })
}

/// The orders of reprint `number` of `day`, for the Day screen's *Orders ▸*.
#[tauri::command]
pub fn reprint_orders_of(
    state: State<'_, AppState>,
    day: String,
    number: u8,
) -> Result<ReprintedOrdersView, String> {
    reprinted_orders(&state, &day, number)
}
