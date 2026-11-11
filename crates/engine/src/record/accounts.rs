//! A batch record's orders by account (summary-pdf.md, *One account*): the orders and units of
//! each account, and the part of a batch one account's summary shows.

use super::{BatchSaved, Finding, SavedOrder};

/// An account's orders and units, as the Accounts sections and `session.toml` count them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AccountCount {
    pub account: String,
    pub orders: usize,
    pub units: u32,
}

/// The orders and units of each account over `records`, most orders first, then by name.
pub fn account_counts<'r>(records: impl IntoIterator<Item = &'r BatchSaved>) -> Vec<AccountCount> {
    let mut counts: Vec<AccountCount> = Vec::new();
    for record in records {
        for order in &record.orders {
            let at = match counts.iter().position(|c| c.account == order.account) {
                Some(at) => at,
                None => {
                    counts.push(AccountCount {
                        account: order.account.clone(),
                        orders: 0,
                        units: 0,
                    });
                    counts.len() - 1
                }
            };
            counts[at].orders += 1;
            counts[at].units += order_units(record, order);
        }
    }
    counts.sort_by(|a, b| b.orders.cmp(&a.orders).then(a.account.cmp(&b.account)));
    counts
}

/// The units of one order of `record`.
pub fn order_units(record: &BatchSaved, order: &SavedOrder) -> u32 {
    record.contents[order.content as usize]
        .items
        .iter()
        .map(|item| u32::from(item.quantity))
        .sum()
}

/// `record` as one account's summary shows it: that account's orders, the group PDFs that hold one
/// of them, the summary PDF, and the findings that name one of them. The inputs and the left-out
/// copies stay the batch's, which its ledger counts.
pub fn of_account(record: &BatchSaved, account: &str) -> BatchSaved {
    let mut mine = record.clone();
    mine.orders.retain(|order| order.account == account);
    mine.outputs.retain(|output| match &output.group {
        Some(group) => mine.orders.iter().any(|order| &order.group == group),
        None => true,
    });
    let names_mine = |finding: &Finding| {
        finding
            .order_id
            .as_ref()
            .is_some_and(|id| mine.orders.iter().any(|order| &order.order_id == id))
    };
    mine.warnings.retain(&names_mine);
    mine.errors.retain(&names_mine);
    mine
}
