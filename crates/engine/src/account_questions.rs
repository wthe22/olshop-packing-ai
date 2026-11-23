//! The account question (new-batch.md, *The shop prompt*): each copy given its account, and the
//! questions about the copies left without one — a sender no account lists, a file whose labels print
//! none, with the account its other orders share.

use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;

use crate::accounts::{Account, OrderAccount, account_of, same_shop};
use crate::copies::Copy;
use crate::ids::OrderId;

/// What gives a copy its account besides its label.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AccountChoices {
    /// The account chosen for the labels with no sender of each input, by its place in the
    /// reading order; for this read only.
    pub no_sender: BTreeMap<usize, String>,
    /// The accounts of the batch an amend redoes, by order: a label with no sender keeps its own.
    pub recorded: BTreeMap<OrderId, String>,
}

/// A sender no account lists, one per name read (spaces and upper and lower case ignored).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShopQuestion {
    /// The name as first read.
    pub name: String,
    pub orders: usize,
    /// The inputs that hold them, by their place in the reading order.
    pub inputs: Vec<usize>,
}

/// An input whose labels print no sender and whose account is not chosen yet.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NoSenderQuestion {
    pub input: usize,
    /// Its orders with no sender, and all its orders.
    pub orders: usize,
    pub of_orders: usize,
    /// Where its orders that print a sender all go, when they go to one place: the likely answer.
    pub others: Option<Others>,
}

/// One account, or one sender the same questions ask about.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Others {
    Account(String),
    Shop(String),
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AccountQuestions {
    pub shops: Vec<ShopQuestion>,
    pub no_sender: Vec<NoSenderQuestion>,
}

impl AccountQuestions {
    pub fn is_empty(&self) -> bool {
        self.shops.is_empty() && self.no_sender.is_empty()
    }
}

/// Give every copy its account: the one that lists the shop on its label; for a label with no
/// sender, the account chosen for its input, else the one its amended record names. The rest get
/// none until the prompt is answered.
pub fn assign_accounts(copies: &mut [Copy], accounts: &[Account], choices: &AccountChoices) {
    for copy in copies {
        copy.account = match &copy.sender {
            Some(sender) => account_of(&sender.text, accounts),
            None => choices
                .no_sender
                .get(&copy.input)
                .or_else(|| choices.recorded.get(&copy.order_id))
                .map(|account| OrderAccount {
                    account: account.clone(),
                    shop: None,
                }),
        };
    }
}

/// The questions about the copies with no account, shops by the order first read, then the inputs.
pub fn account_questions(copies: &[Copy]) -> AccountQuestions {
    let mut shops: Vec<(ShopQuestion, BTreeSet<&OrderId>)> = Vec::new();
    let mut no_sender: BTreeMap<usize, BTreeSet<&OrderId>> = BTreeMap::new();
    for copy in copies.iter().filter(|copy| copy.account.is_none()) {
        let Some(sender) = &copy.sender else {
            no_sender
                .entry(copy.input)
                .or_default()
                .insert(&copy.order_id);
            continue;
        };
        let at = match shops
            .iter()
            .position(|(q, _)| same_shop(&q.name, &sender.name))
        {
            Some(at) => at,
            None => {
                let question = ShopQuestion {
                    name: sender.name.clone(),
                    orders: 0,
                    inputs: Vec::new(),
                };
                shops.push((question, BTreeSet::new()));
                shops.len() - 1
            }
        };
        let (question, orders) = &mut shops[at];
        orders.insert(&copy.order_id);
        question.orders = orders.len();
        if !question.inputs.contains(&copy.input) {
            question.inputs.push(copy.input);
            question.inputs.sort_unstable();
        }
    }
    let shops: Vec<ShopQuestion> = shops.into_iter().map(|(question, _)| question).collect();
    let no_sender = no_sender
        .into_iter()
        .map(|(input, orders)| NoSenderQuestion {
            input,
            orders: orders.len(),
            of_orders: copies
                .iter()
                .filter(|copy| copy.input == input)
                .map(|copy| &copy.order_id)
                .collect::<BTreeSet<_>>()
                .len(),
            others: others_of(copies, input, &shops),
        })
        .collect();
    AccountQuestions { shops, no_sender }
}

/// Where the orders of `input` that print a sender go: their account, or the sender asked about;
/// `None` when they go to more than one place, or there are none.
fn others_of(copies: &[Copy], input: usize, shops: &[ShopQuestion]) -> Option<Others> {
    let mut found: Option<Others> = None;
    for copy in copies.iter().filter(|copy| copy.input == input) {
        let Some(sender) = &copy.sender else {
            continue;
        };
        let here = match &copy.account {
            Some(account) => Others::Account(account.account.clone()),
            None => Others::Shop(
                shops
                    .iter()
                    .find(|shop| same_shop(&shop.name, &sender.name))?
                    .name
                    .clone(),
            ),
        };
        match &found {
            Some(seen) if *seen != here => return None,
            _ => found = Some(here),
        }
    }
    found
}
