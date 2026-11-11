//! The Accounts screen (ui/accounts.md): the accounts of `packing-settings.toml`, made, renamed,
//! given their shops and deleted under the data folder's lock, and one account's dates from the
//! search index.

use packing_engine::accounts::{
    Account, create_account, delete_account, rename_account, set_shops,
};
use serde::Serialize;
use tauri::State;

use crate::{AppState, settings, today};

/// One account's dates, newest first; none while the search index is still being built.
#[derive(Debug, Serialize)]
pub struct AccountDatesView {
    dates: Vec<AccountDateRow>,
    loading: bool,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
struct AccountDateRow {
    date: String,
    today: bool,
    orders: usize,
    batches: usize,
}

pub(crate) fn account_dates_of(
    state: &AppState,
    account: &str,
) -> Result<AccountDatesView, String> {
    let today = today();
    let dates = state.index().with(|index| {
        index
            .account_dates(account)
            .into_iter()
            .map(|day| AccountDateRow {
                date: day.date.to_string(),
                today: day.date == today,
                orders: day.orders,
                batches: day.batches,
            })
            .collect::<Vec<_>>()
    })?;
    Ok(AccountDatesView {
        loading: dates.is_none(),
        dates: dates.unwrap_or_default(),
    })
}

/// Change the accounts under the data folder's lock and return them as written.
pub(crate) fn change(
    state: &AppState,
    change: impl FnOnce(&mut Vec<Account>) -> Result<(), String>,
) -> Result<Vec<Account>, String> {
    state.write_data(|_, _| settings::change_accounts(state.program(), change))
}

fn told(error: impl std::fmt::Display) -> String {
    let text = error.to_string();
    let mut chars = text.chars();
    let first = chars.next().map(|c| c.to_uppercase().to_string());
    format!("{}{}.", first.unwrap_or_default(), chars.as_str())
}

/// The accounts, in the order they were made.
#[tauri::command]
pub fn accounts_list(state: State<'_, AppState>) -> Result<Vec<Account>, String> {
    settings::accounts(state.program())
}

#[tauri::command]
pub fn account_dates(
    state: State<'_, AppState>,
    account: String,
) -> Result<AccountDatesView, String> {
    account_dates_of(&state, &account)
}

/// **+ New account**: an account with no shop.
#[tauri::command]
pub fn account_new(state: State<'_, AppState>, name: String) -> Result<Vec<Account>, String> {
    change(&state, |accounts| {
        create_account(accounts, &name).map_err(told)
    })
}

/// **Rename…**: orders saved before keep the name they were saved with.
#[tauri::command]
pub fn account_rename(
    state: State<'_, AppState>,
    from: String,
    to: String,
) -> Result<Vec<Account>, String> {
    change(&state, |accounts| {
        rename_account(accounts, &from, &to).map_err(told)
    })
}

/// **Save**: the account's shops as the fields list them; a refused one is named.
#[tauri::command]
pub fn account_set_shops(
    state: State<'_, AppState>,
    account: String,
    shops: Vec<String>,
) -> Result<Vec<Account>, String> {
    change(&state, |accounts| {
        set_shops(accounts, &account, &shops).map_err(|(at, error)| match shops.get(at) {
            Some(shop) => format!("{shop}: {error}."),
            None => told(error),
        })
    })
}

/// **Delete account**: its shops are asked about again, its saved orders keep its name.
#[tauri::command]
pub fn account_delete(state: State<'_, AppState>, name: String) -> Result<Vec<Account>, String> {
    change(&state, |accounts| {
        delete_account(accounts, &name).map_err(told)
    })
}

#[cfg(test)]
mod tests;
