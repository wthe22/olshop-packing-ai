//! Search and Reprint (search.md, reprint.md): saved orders of the open workspace found by a
//! filter on the search index, each with where it came from and the reprints that printed it again;
//! one PDF of the chosen orders' pages written into today's date; and a reprint's orders as Search
//! shows them.

use std::collections::BTreeSet;
use std::ops::RangeInclusive;

use jiff::civil::Date;
use packing_batch::{ReprintOutcome, ReprintRequest, reprint as write_reprint};
use packing_engine::ids::{BatchId, OrderId};
use packing_engine::rules::parse_filter;
use packing_engine::workflows::{Courier, courier_name};
use packing_store::{IndexedOrder, SearchIndex};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State};

use crate::commands::day::date_of;
use crate::settings;
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
    /// The courier's name; none when no courier was named.
    courier: Option<String>,
    items: Vec<ItemView>,
    batch: BatchId,
    /// `<batch>-<code>`.
    group: String,
    /// The group PDF's file name, for *Open*.
    file: String,
    /// The pages the order fills in that group PDF, counted from 1.
    first_page: u16,
    last_page: u16,
    /// The batch's last save, `09:12`.
    batch_time: String,
    /// The input PDF it was read from, by the name it had when it was chosen, and its pages there.
    input: String,
    input_first_page: u16,
    input_last_page: u16,
    /// The reprints that printed it again, oldest first.
    reprints: Vec<ReprintMarkView>,
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
    let (_, workflows) = settings::workflows(state.program())?;
    let couriers = &workflows.couriers;
    let dates = match range {
        Some(range) => range.dates()?,
        None => Date::MIN..=Date::MAX,
    };
    let rows = state.index().with(|index| {
        index
            .search(&filter, dates, couriers)
            .into_iter()
            .map(|order| row(index, order, couriers))
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

fn row(index: &SearchIndex, order: &IndexedOrder, couriers: &[Courier]) -> OrderRow {
    let saved = &order.order;
    OrderRow {
        day: order.date.to_string(),
        order_id: saved.order_id.to_string(),
        tracking_id: saved.tracking_id.as_ref().map(ToString::to_string),
        courier: saved
            .courier
            .as_ref()
            .map(|code| courier_name(couriers, code.as_str()).to_string()),
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
        first_page: *order.file_pages.start(),
        last_page: *order.file_pages.end(),
        batch_time: order.batch_time.zoned().strftime("%H:%M").to_string(),
        input: order.input.clone(),
        input_first_page: saved.page_start,
        input_last_page: saved.page_end,
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
    let (_, workflows) = settings::workflows(state.program())?;
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
                    .map(|found| row(index, found, &workflows.couriers)),
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
