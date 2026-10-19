//! The evaluator: whether an order's item or order-level fields match a condition, with no
//! value meaning no match for dates and times.

use std::collections::BTreeSet;

use jiff::civil::{Date, DateTime};

use super::condition::{Condition, NumberOp, TextOp};
use super::parser::{is_clock_form, is_date_form, is_datetime_form};
use crate::ids::TrackingId;

fn text_match(field_value: &str, op: TextOp, literal: &str) -> bool {
    let value = field_value.trim().to_lowercase();
    let want = literal.to_lowercase();
    match op {
        TextOp::Contains => value.contains(&want),
        TextOp::Equals => value == want,
        TextOp::StartsWith => value.starts_with(&want),
    }
}

fn number_match(field_value: i64, op: NumberOp, literal: i64) -> bool {
    match op {
        NumberOp::Eq => field_value == literal,
        NumberOp::Ne => field_value != literal,
        NumberOp::Lt => field_value < literal,
        NumberOp::Le => field_value <= literal,
        NumberOp::Gt => field_value > literal,
        NumberOp::Ge => field_value >= literal,
    }
}

fn ord_match<T: Ord>(left: &T, op: NumberOp, right: &T) -> bool {
    match op {
        NumberOp::Eq => left == right,
        NumberOp::Ne => left != right,
        NumberOp::Lt => left < right,
        NumberOp::Le => left <= right,
        NumberOp::Gt => left > right,
        NumberOp::Ge => left >= right,
    }
}

enum TimeLiteral {
    Date(Date),
    Clock(i8, i8),
    DateTime(DateTime),
}

/// Only called on a canonical, already-validated value; `None` would mean a hand-built AST.
fn time_literal(value: &str) -> Option<TimeLiteral> {
    if is_datetime_form(value) {
        let date: Date = value[..10].parse().ok()?;
        let hour: i8 = value[11..13].parse().ok()?;
        let minute: i8 = value[14..16].parse().ok()?;
        Some(TimeLiteral::DateTime(date.at(hour, minute, 0, 0)))
    } else if is_date_form(value) {
        Some(TimeLiteral::Date(value.parse().ok()?))
    } else if is_clock_form(value) {
        Some(TimeLiteral::Clock(
            value[..2].parse().ok()?,
            value[3..5].parse().ok()?,
        ))
    } else {
        None
    }
}

/// No value means no match, for every operator (04-rules-file "Date and time values").
fn time_match(field_value: Option<DateTime>, op: NumberOp, literal: &str) -> bool {
    let Some(field_value) = field_value else {
        return false;
    };
    match time_literal(literal) {
        Some(TimeLiteral::Date(right)) => ord_match(&field_value.date(), op, &right),
        Some(TimeLiteral::Clock(hour, minute)) => ord_match(
            &(field_value.hour(), field_value.minute()),
            op,
            &(hour, minute),
        ),
        // Exact moment: the field keeps its seconds, so 14:00:30 < "14:00" is false.
        Some(TimeLiteral::DateTime(right)) => ord_match(&field_value, op, &right),
        None => false,
    }
}

fn item_text(order: &crate::orders::Order, field: &str, op: TextOp, literal: &str) -> bool {
    order.lines.iter().any(|line| {
        let value = match field {
            "name" => &line.name,
            "display_name" => &line.display_name,
            "variation" => &line.variation,
            "seller_sku" => &line.seller_sku,
            _ => return false,
        };
        text_match(value, op, literal)
    })
}

/// Evaluate a condition against an order (item fields: "at least one line matches").
pub fn evaluate(cond: &Condition, order: &crate::orders::Order) -> bool {
    match cond {
        Condition::Not(operand) => !evaluate(operand, order),
        Condition::And(operands) => operands.iter().all(|o| evaluate(o, order)),
        Condition::Or(operands) => operands.iter().any(|o| evaluate(o, order)),
        Condition::Text { field, op, value } => match field.as_str() {
            "courier" => text_match(order.courier.as_deref().unwrap_or(""), *op, value),
            "order_id" => text_match(order.order_id.as_str(), *op, value),
            // An order with no tracking ID matches text as the empty string.
            "tracking_id" => text_match(
                order.tracking_id.as_ref().map_or("", TrackingId::as_str),
                *op,
                value,
            ),
            "name" | "display_name" | "variation" | "seller_sku" => {
                item_text(order, field, *op, value)
            }
            // sku_id / product_category / channel are CSV-only and never parsed in the app.
            _ => false,
        },
        Condition::Number { field, op, value } => match field.as_str() {
            "line_quantity" => order
                .lines
                .iter()
                .any(|line| number_match(line.quantity, *op, *value)),
            "total_quantity" => number_match(order.total_quantity(), *op, *value),
            "distinct_items" => number_match(order.distinct_items(), *op, *value),
            _ => false,
        },
        Condition::Time { field, op, value } => match field.as_str() {
            "ship_by" => time_match(order.ship_by, *op, value),
            _ => false,
        },
    }
}

/// Every field name appearing in the condition.
pub fn fields_used(cond: &Condition) -> BTreeSet<String> {
    let mut used = BTreeSet::new();
    collect_fields(cond, &mut used);
    used
}

fn collect_fields(cond: &Condition, used: &mut BTreeSet<String>) {
    match cond {
        Condition::Text { field, .. }
        | Condition::Number { field, .. }
        | Condition::Time { field, .. } => {
            used.insert(field.clone());
        }
        Condition::Not(operand) => collect_fields(operand, used),
        Condition::And(operands) | Condition::Or(operands) => {
            for operand in operands {
                collect_fields(operand, used);
            }
        }
    }
}
