//! `expected-labels.json`: what reading the fixtures must give, written from the same orders the
//! pages are drawn from. Keys keep the page order, so the file is written by hand rather than
//! through a map that sorts them.

use serde_json::{Value, json};

use crate::orders::{Label, Row};

/// The answer file's text: one entry per order, in page order.
pub fn expected_labels(labels: &[&Label]) -> String {
    let entries: Vec<String> = labels
        .iter()
        .map(|label| {
            format!(
                "  {}: {}",
                Value::from(label.order_id.as_str()),
                indented(&entry(label))
            )
        })
        .collect();
    format!("{{\n{}\n}}\n", entries.join(",\n"))
}

fn entry(label: &Label) -> Vec<(&'static str, Value)> {
    let (lines, qty_total, message) = match &label.slip {
        Some(slip) => (
            slip.rows.iter().map(line).collect(),
            json!(slip.qty_total),
            slip.message.unwrap_or_default(),
        ),
        None => (Vec::new(), Value::Null, ""),
    };
    vec![
        ("pages", json!(label.pages())),
        ("tracking_id", json!(label.tracking_id)),
        ("courier", json!(label.courier)),
        (
            "ship_by",
            label
                .ship_by
                .as_deref()
                .map_or(Value::Null, |s| json!(iso(s))),
        ),
        ("has_slip", json!(label.slip.is_some())),
        ("lines", Value::Array(lines)),
        ("qty_total", qty_total),
        ("customer_message", json!(message)),
    ]
}

/// A line as the reader stores it: the slip's `Default` variation is no variation.
fn line(row: &Row) -> Value {
    let variation = if row.variation == "Default" {
        ""
    } else {
        row.variation
    };
    json!([row.name, variation, row.seller_sku, row.quantity])
}

/// `dd/mm/yyyy HH:MM` as `yyyy-mm-ddTHH:MM:00`.
fn iso(printed: &str) -> String {
    let (date, time) = printed.split_once(' ').expect("date and time");
    let parts: Vec<&str> = date.split('/').collect();
    format!("{}-{}-{}T{time}:00", parts[2], parts[1], parts[0])
}

/// An entry's fields in their order, two levels in, every array element on its own line.
fn indented(fields: &[(&str, Value)]) -> String {
    let body: Vec<String> = fields
        .iter()
        .map(|(key, value)| format!("    {}: {}", Value::from(*key), pretty(value, 4)))
        .collect();
    format!("{{\n{}\n  }}", body.join(",\n"))
}

fn pretty(value: &Value, indent: usize) -> String {
    match value {
        Value::Array(items) if !items.is_empty() => {
            let inner = " ".repeat(indent + 2);
            let items: Vec<String> = items
                .iter()
                .map(|item| format!("{inner}{}", pretty(item, indent + 2)))
                .collect();
            format!("[\n{}\n{}]", items.join(",\n"), " ".repeat(indent))
        }
        other => other.to_string(),
    }
}

/// `expected-repeat.json`: the copies of the repeat inputs in reading order, and what keeping the
/// first copy of each order keeps and leaves out (each left-out copy with the copy kept instead).
pub fn expected_repeat(inputs: &[Vec<Label>], names: &[&str]) -> String {
    let mut copies = Vec::new();
    let mut first: Vec<(&str, usize)> = Vec::new();
    let mut kept = Vec::new();
    let mut left_out = Vec::new();
    for (input, labels) in inputs.iter().enumerate() {
        let mut page = 1;
        for label in labels {
            let index = copies.len();
            let pages = label.pages();
            let lines: Vec<Value> = label
                .slip
                .as_ref()
                .map(|slip| slip.rows.iter().map(line).collect())
                .unwrap_or_default();
            copies.push(json!({
                "order_id": label.order_id,
                "input": input,
                "page_start": page,
                "page_end": page + pages - 1,
                "lines": lines,
            }));
            page += pages;
            match first.iter().find(|(id, _)| *id == label.order_id) {
                Some((_, at)) => left_out.push(json!({ "copy": index, "kept": at })),
                None => {
                    first.push((&label.order_id, index));
                    kept.push(index);
                }
            }
        }
    }
    let answer = json!({
        "inputs": names,
        "copies": copies,
        "kept": kept,
        "left_out": left_out,
    });
    let mut text = serde_json::to_string_pretty(&answer).expect("plain JSON");
    text.push('\n');
    text
}
