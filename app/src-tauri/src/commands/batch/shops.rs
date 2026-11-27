//! The account question (new-batch.md): the senders no account lists; the owner's answer, its shops
//! and new accounts written into `config.toml` under the data folder's lock and the copies given
//! their accounts again.

use packing_batch::Draft;
use packing_engine::accounts::{NameError, ShopAccount, add_shop, check_shop_name};
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
}

/// A sender no account lists: the name as first read and its orders.
#[derive(Serialize)]
struct ShopRowView {
    name: String,
    orders: usize,
}

/// *Continue*: every row's answer.
#[derive(Deserialize)]
pub struct ShopAnswer {
    shops: Vec<ShopRow>,
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
            .into_iter()
            .map(|shop| ShopRowView {
                name: shop.name,
                orders: shop.orders,
            })
            .collect(),
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
                Ok(())
            })
        })?;
        open.draft.answer_accounts(accounts);
        state.with_data(|data| {
            let day = data.open_day(open.date)?;
            Ok(read_result(open, day.journal.fold(), &workflows))
        })
    })
}
