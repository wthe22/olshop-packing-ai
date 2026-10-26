//! The logo prompt's answers written into `packing-workflows.toml` (courier.md): a logo the owner
//! names goes into that courier's classifier, an image answered *Not a courier* into
//! `not_courier`. The file is edited through `toml_edit`, so the owner's comments and every entry
//! the answer does not touch stay as they were.

use std::str::FromStr;

use toml_edit::{Array, ArrayOfTables, DocumentMut, Item, Table, Value};

use super::{Workflows, WorkflowsError, load_workflows};
use crate::ids::CourierCode;

/// What the owner said a header image is.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LogoIs {
    Courier(CourierCode),
    NotCourier,
}

/// The file's text with the image's byte hash remembered, and the rules it now holds. A hash
/// already remembered changes nothing; a courier without a classifier gets one.
pub fn remember_logo(
    text: &str,
    hash: &str,
    logo: &LogoIs,
) -> Result<(String, Workflows), WorkflowsError> {
    load_workflows(text)?;
    let mut doc = DocumentMut::from_str(text).expect("a file the rules loaded from is TOML");
    match logo {
        LogoIs::Courier(code) => {
            let classifiers = doc
                .entry("courier_classifier")
                .or_insert_with(|| Item::ArrayOfTables(ArrayOfTables::new()))
                .as_array_of_tables_mut()
                .expect("the rules loaded it as an array of tables");
            let at = classifiers.iter().position(|table| {
                table.get("courier_code").and_then(Item::as_str) == Some(code.as_str())
            });
            let at = at.unwrap_or_else(|| {
                let mut table = Table::new();
                table.insert("courier_code", toml_edit::value(code.as_str()));
                classifiers.push(table);
                classifiers.len() - 1
            });
            let table = classifiers.get_mut(at).expect("found or pushed above");
            push_hash(table, hash);
        }
        LogoIs::NotCourier => {
            let table = doc
                .entry("not_courier")
                .or_insert_with(|| Item::Table(Table::new()))
                .as_table_mut()
                .expect("the rules loaded it as a table");
            push_hash(table, hash);
        }
    }
    let written = doc.to_string();
    let workflows = load_workflows(&written)?;
    Ok((written, workflows))
}

/// Append `hash` to the table's `logos`, creating the list when it is absent.
fn push_hash(table: &mut Table, hash: &str) {
    let logos = table
        .entry("logos")
        .or_insert_with(|| toml_edit::value(Array::new()))
        .as_array_mut()
        .expect("the rules loaded `logos` as a list of strings");
    if !logos.iter().any(|known| known.as_str() == Some(hash)) {
        logos.push(Value::from(hash));
    }
}
