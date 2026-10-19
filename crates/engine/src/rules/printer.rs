//! The printer: a condition back to its canonical text, one space around the operators and
//! only the parentheses the precedence needs.

use super::condition::Condition;

fn escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn format_node(node: &Condition, min_prec: u8) -> String {
    let text = match node {
        Condition::Text { field, op, value } => {
            format!("{field} {} \"{}\"", op.as_str(), escape(value))
        }
        Condition::Number { field, op, value } => format!("{field} {} {value}", op.as_str()),
        Condition::Time { field, op, value } => {
            format!("{field} {} \"{}\"", op.as_str(), escape(value))
        }
        Condition::Not(operand) => format!("not {}", format_node(operand, 3)),
        Condition::And(operands) => operands
            .iter()
            .map(|o| format_node(o, 2))
            .collect::<Vec<_>>()
            .join(" and "),
        Condition::Or(operands) => operands
            .iter()
            .map(|o| format_node(o, 1))
            .collect::<Vec<_>>()
            .join(" or "),
    };
    if node.precedence() < min_prec {
        format!("({text})")
    } else {
        text
    }
}

/// Canonical text: lowercase keywords, one space around operators, minimal parentheses.
pub fn format_condition(cond: &Condition) -> String {
    format_node(cond, 0)
}
