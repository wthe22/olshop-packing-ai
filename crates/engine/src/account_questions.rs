//! The account question (new-batch.md, *The shop prompt*): each copy given its account, and the
//! senders no account lists, which the prompt asks about.

use std::collections::BTreeSet;

use crate::accounts::{Account, NO_ACCOUNT, OrderAccount, account_of, same_shop};
use crate::copies::Copy;
use crate::ids::OrderId;

/// A sender no account lists, one per name read (spaces and upper and lower case ignored).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShopQuestion {
    /// The name as first read.
    pub name: String,
    pub orders: usize,
    /// The inputs that hold them, by their place in the reading order.
    pub inputs: Vec<usize>,
}

/// Give every copy its account: the one that lists the shop on its label; [`NO_ACCOUNT`] for a label
/// with no sender. A sender no account lists gets none until the prompt is answered.
pub fn assign_accounts(copies: &mut [Copy], accounts: &[Account]) {
    for copy in copies {
        copy.account = match &copy.sender {
            Some(sender) => account_of(&sender.text, accounts),
            None => Some(OrderAccount {
                account: NO_ACCOUNT.to_string(),
                shop: None,
            }),
        };
    }
}

/// The senders of the copies with no account, by the order first read.
pub fn account_questions(copies: &[Copy]) -> Vec<ShopQuestion> {
    let mut shops: Vec<(ShopQuestion, BTreeSet<&OrderId>)> = Vec::new();
    for copy in copies.iter().filter(|copy| copy.account.is_none()) {
        let Some(sender) = &copy.sender else {
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
    shops.into_iter().map(|(question, _)| question).collect()
}
