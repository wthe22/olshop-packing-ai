//! The Accounts screen's and the shop prompt's changes written into `config.toml`
//! (rules-file/config.md): the file's accounts replaced, everything else in it kept as it is. An
//! account keeps its table, so a key this version does not know and the owner's comments stay.

use std::str::FromStr;

use toml_edit::{Array, ArrayOfTables, DocumentMut, Item, Table};

use super::edit::{free_positions, set_value};
use super::{Workflows, WorkflowsError, load_workflows};
use crate::accounts::Account;

/// The config's text with its accounts replaced by `accounts`, written before the sort rules, and
/// the rules it now holds with `parser`. The owner's comments on the sort rules stay.
pub fn write_accounts(
    parser: &str,
    text: &str,
    accounts: &[Account],
) -> Result<(String, Workflows), WorkflowsError> {
    load_workflows(parser, text)?;
    let mut doc = DocumentMut::from_str(text).expect("a file the rules loaded from is TOML");
    let rules = doc.remove("label_sort_rules");
    let old: Vec<Table> = doc
        .remove("accounts")
        .as_ref()
        .and_then(Item::as_array_of_tables)
        .map(|tables| tables.iter().cloned().collect())
        .unwrap_or_default();
    if !accounts.is_empty() {
        let mut written = ArrayOfTables::new();
        for (account, kept) in accounts.iter().zip(kept_tables(&old, accounts)) {
            let mut table = kept.unwrap_or_default();
            set_value(&mut table, "name", account.name.as_str().into());
            let shops: Array = account.shops.iter().map(String::as_str).collect();
            set_value(&mut table, "shops", shops.into());
            written.push(table);
        }
        doc.insert("accounts", Item::ArrayOfTables(written));
    }
    if let Some(rules) = rules {
        doc.insert("label_sort_rules", rules);
    }
    free_positions(&mut doc);
    let written = doc.to_string();
    let workflows = load_workflows(parser, &written)?;
    Ok((written, workflows))
}

/// Each account's table as the file had it: found by its name, or, for the one account renamed,
/// the old table no account names any more, in order. Every change but a rename keeps the names,
/// adds accounts at the end or removes some; none is a new account.
fn kept_tables(old: &[Table], accounts: &[Account]) -> Vec<Option<Table>> {
    let name = |table: &Table| table.get("name").and_then(Item::as_str).map(str::to_string);
    let names: Vec<&str> = accounts
        .iter()
        .map(|account| account.name.as_str())
        .collect();
    let mut unnamed = old
        .iter()
        .filter(|table| name(table).is_none_or(|name| !names.contains(&name.as_str())));
    accounts
        .iter()
        .map(|account| {
            let by_name = old
                .iter()
                .find(|table| name(table).as_deref() == Some(account.name.as_str()));
            by_name.or_else(|| unnamed.next()).cloned()
        })
        .collect()
}
