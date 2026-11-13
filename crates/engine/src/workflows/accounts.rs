//! The Accounts screen's and the shop prompt's changes written into `config.toml`
//! (rules-file/config.md): the file's accounts replaced, everything else in it kept as it is.

use std::str::FromStr;

use toml_edit::{Array, ArrayOfTables, DocumentMut, Item, Table};

use super::edit::free_positions;
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
    doc.remove("accounts");
    if !accounts.is_empty() {
        let mut written = ArrayOfTables::new();
        for account in accounts {
            let mut table = Table::new();
            table.insert("name", toml_edit::value(account.name.as_str()));
            let shops: Array = account.shops.iter().map(String::as_str).collect();
            table.insert("shops", toml_edit::value(shops));
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
