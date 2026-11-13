//! The shop prompt (new-batch.md): the senders no account lists and the files with labels that show
//! no sender; the owner's answer, its shops written into `config.toml` under the data
//! folder's lock and the copies given their accounts again.

use std::collections::BTreeMap;

use packing_batch::Draft;
use packing_engine::account_questions::{ShopAnswerError, check_shop_answer};
use packing_engine::accounts::{NameError, ShopAccount, add_shop};
use serde::{Deserialize, Serialize};

use crate::AppState;
use crate::settings;

use super::read::read_result;
use super::views::ReadResult;

/// The prompt's rows and the accounts its lists offer; no rows when every order has an account.
#[derive(Serialize)]
pub struct ShopPromptView {
    accounts: Vec<String>,
    shops: Vec<ShopRowView>,
    no_sender: Vec<NoSenderRowView>,
}

/// A sender no account lists: the name as first read, its orders and how many files hold them.
#[derive(Serialize)]
struct ShopRowView {
    name: String,
    orders: usize,
    files: usize,
}

/// A file whose labels show no sender: its place in the reading order, its name, and how many of
/// its orders show none.
#[derive(Serialize)]
struct NoSenderRowView {
    input: usize,
    file: String,
    orders: usize,
    of_orders: usize,
}

/// *Continue*: every row's answer.
#[derive(Deserialize)]
pub struct ShopAnswer {
    shops: Vec<ShopRow>,
    /// The account of each file's labels with no sender, by the file's place; for this read only.
    no_sender: BTreeMap<usize, String>,
}

/// A sender's answer: the name as read, the name kept or put right in the field, and its account.
#[derive(Deserialize)]
struct ShopRow {
    read: String,
    name: String,
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
                files: shop.inputs.len(),
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
                of_orders: file.of_orders,
            })
            .collect(),
    }
}

/// Write the answered shops into the accounts, read again first so another copy's change is kept,
/// and give the draft's copies their accounts. A shop another copy listed meanwhile keeps that
/// account; any other refused name writes nothing and says which row.
pub(super) fn answer_shop(state: &AppState, answer: &ShopAnswer) -> Result<ReadResult, String> {
    let (_, workflows) = settings::workflows(state.program())?;
    state.with_draft(|open| {
        let accounts = state.write_data(|_, _| {
            settings::change_accounts(state.program(), |accounts| {
                for row in &answer.shops {
                    match check_shop_answer(&open.draft.copies, &row.read, &row.name, accounts) {
                        Err(ShopAnswerError::Name(NameError::ShopListed { .. })) => continue,
                        checked => checked.map_err(|error| format!("{}: {error}.", row.name))?,
                    }
                    add_shop(accounts, &row.name, &row.account)
                        .map_err(|error| format!("{}: {error}.", row.name))?;
                }
                let listed = |name: &&String| accounts.iter().any(|a| &a.name == *name);
                match answer.no_sender.values().find(|name| !listed(name)) {
                    Some(name) => Err(format!("There is no account \"{name}\".")),
                    None => Ok(()),
                }
            })
        })?;
        open.draft
            .answer_accounts(accounts, answer.no_sender.clone());
        state.with_data(|data| {
            let day = data.open_day(open.date)?;
            Ok(read_result(open, day.journal.fold(), &workflows))
        })
    })
}
