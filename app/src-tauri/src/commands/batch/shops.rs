//! The account question (new-batch.md): the senders no account lists and the files with labels that
//! show no sender; the owner's answer, its shops and new accounts written into `config.toml` under
//! the data folder's lock and the copies given their accounts again.

use std::collections::BTreeMap;

use packing_batch::Draft;
use packing_engine::account_questions::Others;
use packing_engine::accounts::{NameError, ShopAccount, add_shop, check_shop_name, create_account};
use serde::{Deserialize, Serialize};

use crate::AppState;
use crate::settings;

use super::read::read_result;
use super::views::ReadResult;

/// The question's rows and the accounts its lists offer; no rows when every order has an account.
#[derive(Serialize)]
pub struct ShopPromptView {
    accounts: Vec<String>,
    shops: Vec<ShopRowView>,
    no_sender: Vec<NoSenderRowView>,
}

/// A sender no account lists: the name as first read and its orders.
#[derive(Serialize)]
struct ShopRowView {
    name: String,
    orders: usize,
}

/// A file whose labels show no sender: its place in the reading order, its name, how many of its
/// orders show none, and where its other orders go when they go to one place.
#[derive(Serialize)]
struct NoSenderRowView {
    input: usize,
    file: String,
    orders: usize,
    others: Option<Others>,
}

/// *Continue*: every row's answer.
#[derive(Deserialize)]
pub struct ShopAnswer {
    shops: Vec<ShopRow>,
    /// The account of each file's labels with no sender, by the file's place; for this read only.
    no_sender: BTreeMap<usize, ShopAccount>,
}

/// A sender's answer: the name as read, and its account.
#[derive(Deserialize)]
struct ShopRow {
    read: String,
    account: ShopAccount,
}

pub(super) fn shop_prompt(draft: &Draft) -> ShopPromptView {
    let questions = draft.account_questions();
    ShopPromptView {
        accounts: draft.accounts.iter().map(|a| a.name.clone()).collect(),
        shops: questions
            .shops
            .into_iter()
            .map(|shop| ShopRowView {
                name: shop.name,
                orders: shop.orders,
            })
            .collect(),
        no_sender: questions
            .no_sender
            .into_iter()
            .map(|file| NoSenderRowView {
                input: file.input,
                file: draft.files[file.input].filename.clone(),
                orders: file.orders,
                others: file.others,
            })
            .collect(),
    }
}

/// The name of the account `account` puts a row in.
fn name_of(account: &ShopAccount) -> &String {
    match account {
        ShopAccount::Listed(name) | ShopAccount::New(name) => name,
    }
}

/// Write the answered shops and new accounts into the accounts, read again first so another copy's
/// change is kept, and give the draft's copies their accounts. A new account is made by the one row
/// that names it as new, before the rows that take it as listed. A shop another copy listed
/// meanwhile keeps that account; any other refused name writes nothing and says which row.
pub(super) fn answer_shop(state: &AppState, answer: &ShopAnswer) -> Result<ReadResult, String> {
    let (_, workflows) = settings::workflows(state.program())?;
    state.with_draft(|open| {
        let accounts = state.write_data(|_, _| {
            settings::change_accounts(state.program(), |accounts| {
                for account in answer.no_sender.values() {
                    if let ShopAccount::New(name) = account {
                        create_account(accounts, name)
                            .map_err(|error| format!("{name}: {error}."))?;
                    }
                }
                let (new, listed): (Vec<&ShopRow>, Vec<&ShopRow>) = answer
                    .shops
                    .iter()
                    .partition(|row| matches!(row.account, ShopAccount::New(_)));
                for row in new.into_iter().chain(listed) {
                    match check_shop_name(&row.read, accounts) {
                        Err(NameError::ShopListed { .. }) => continue,
                        checked => checked.map_err(|error| format!("{}: {error}.", row.read))?,
                    }
                    add_shop(accounts, &row.read, &row.account)
                        .map_err(|error| format!("{}: {error}.", row.read))?;
                }
                let listed = |name: &String| accounts.iter().any(|a| &a.name == name);
                match answer
                    .no_sender
                    .values()
                    .map(name_of)
                    .find(|name| !listed(name))
                {
                    Some(name) => Err(format!("There is no account \"{name}\".")),
                    None => Ok(()),
                }
            })
        })?;
        let no_sender = answer
            .no_sender
            .iter()
            .map(|(input, account)| (*input, name_of(account).clone()))
            .collect();
        open.draft.answer_accounts(accounts, no_sender);
        state.with_data(|data| {
            let day = data.open_day(open.date)?;
            Ok(read_result(open, day.journal.fold(), &workflows))
        })
    })
}
