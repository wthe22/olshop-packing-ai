//! Condition text <-> boxes tree: the group, row
//! and value shapes the window shows, built with the engine's parser and printer.

use serde::{Deserialize, Serialize};

use crate::rules::{Condition, NumberOp, RulesError, TextOp, format_condition, parse_condition};

use super::error;

/// A group's kind: **All of these** (`and`) or **Any of these** (`or`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GroupKind {
    All,
    Any,
}

/// A group: its kind, a **not** flag and its rows and nested groups.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Group {
    pub kind: GroupKind,
    pub not: bool,
    pub children: Vec<Node>,
}

/// One item of a group: a row or a nested group.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Node {
    Group(Group),
    Row(Row),
}

/// One comparison `field · operator · value`, with a **not** flag.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Row {
    pub not: bool,
    /// The field name as the printer writes it (lowercased), e.g. `name`.
    pub field: String,
    /// The operator as its canonical text (`contains`, `equals`, `starts_with`, `=`, `!=`,
    /// `<`, `<=`, `>`, `>=`).
    pub operator: String,
    pub value: RowValue,
}

/// A row's value, tagged by the field type (text, whole number, date/time).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RowValue {
    Text(String),
    Number(i64),
    Time(String),
}

/// The boxes of one condition: a root group ("Every condition starts as one *All of
/// these* group").
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tree {
    pub root: Group,
}

fn text_op_str(op: TextOp) -> &'static str {
    match op {
        TextOp::Contains => "contains",
        TextOp::Equals => "equals",
        TextOp::StartsWith => "starts_with",
    }
}

fn text_op_parse(text: &str) -> Option<TextOp> {
    match text {
        "contains" => Some(TextOp::Contains),
        "equals" => Some(TextOp::Equals),
        "starts_with" => Some(TextOp::StartsWith),
        _ => None,
    }
}

fn number_op_str(op: NumberOp) -> &'static str {
    match op {
        NumberOp::Eq => "=",
        NumberOp::Ne => "!=",
        NumberOp::Lt => "<",
        NumberOp::Le => "<=",
        NumberOp::Gt => ">",
        NumberOp::Ge => ">=",
    }
}

fn number_op_parse(text: &str) -> Option<NumberOp> {
    match text {
        "=" => Some(NumberOp::Eq),
        "!=" => Some(NumberOp::Ne),
        "<" => Some(NumberOp::Lt),
        "<=" => Some(NumberOp::Le),
        ">" => Some(NumberOp::Gt),
        ">=" => Some(NumberOp::Ge),
        _ => None,
    }
}

/// A comparison as a row (the `not` flag stays `false`; the caller folds `not` in).
fn leaf_row(cond: &Condition) -> Row {
    match cond {
        Condition::Text { field, op, value } => Row {
            not: false,
            field: field.clone(),
            operator: text_op_str(*op).to_string(),
            value: RowValue::Text(value.clone()),
        },
        Condition::Number { field, op, value } => Row {
            not: false,
            field: field.clone(),
            operator: number_op_str(*op).to_string(),
            value: RowValue::Number(*value),
        },
        Condition::Time { field, op, value } => Row {
            not: false,
            field: field.clone(),
            operator: number_op_str(*op).to_string(),
            value: RowValue::Time(value.clone()),
        },
        Condition::Not(_) | Condition::And(_) | Condition::Or(_) => {
            unreachable!("only a comparison has a row")
        }
    }
}

/// Condition -> node. `not` folds onto a comparison (a row's flag) or onto a group of the same
/// kind; `not not …` cannot fold, so it wraps the inner node in a group (every valid
/// text can be shown as boxes).
fn to_node(cond: &Condition) -> Node {
    match cond {
        Condition::Text { .. } | Condition::Number { .. } | Condition::Time { .. } => {
            Node::Row(leaf_row(cond))
        }
        Condition::And(operands) => Node::Group(Group {
            kind: GroupKind::All,
            not: false,
            children: operands.iter().map(to_node).collect(),
        }),
        Condition::Or(operands) => Node::Group(Group {
            kind: GroupKind::Any,
            not: false,
            children: operands.iter().map(to_node).collect(),
        }),
        Condition::Not(inner) => match inner.as_ref() {
            Condition::Text { .. } | Condition::Number { .. } | Condition::Time { .. } => {
                let mut row = leaf_row(inner);
                row.not = true;
                Node::Row(row)
            }
            Condition::And(operands) => Node::Group(Group {
                kind: GroupKind::All,
                not: true,
                children: operands.iter().map(to_node).collect(),
            }),
            Condition::Or(operands) => Node::Group(Group {
                kind: GroupKind::Any,
                not: true,
                children: operands.iter().map(to_node).collect(),
            }),
            Condition::Not(_) => Node::Group(Group {
                kind: GroupKind::All,
                not: true,
                children: vec![to_node(inner)],
            }),
        },
    }
}

/// Parse a condition and show it as boxes.
pub fn condition_to_tree(text: &str) -> Result<Tree, RulesError> {
    let condition = parse_condition(text)?;
    let node = to_node(&condition);
    let root = match node {
        Node::Group(group) => group,
        row @ Node::Row(_) => Group {
            kind: GroupKind::All,
            not: false,
            children: vec![row],
        },
    };
    Ok(Tree { root })
}

/// A group as a condition; an empty group is an error ("This group is empty"). A group
/// with a single child is that child, not a one-element `and`/`or`: the printer would wrap a
/// one-element `and`/`or` in parentheses under a `not` (`not not a` must stay `not not a`, not
/// `not (not a)`).
fn group_condition(group: &Group) -> Result<Condition, RulesError> {
    if group.children.is_empty() {
        return Err(error("this group is empty"));
    }
    let mut operands = Vec::with_capacity(group.children.len());
    for child in &group.children {
        operands.push(node_condition(child)?);
    }
    let inner = if operands.len() == 1 {
        operands.pop().expect("one operand")
    } else {
        match group.kind {
            GroupKind::All => Condition::And(operands),
            GroupKind::Any => Condition::Or(operands),
        }
    };
    Ok(if group.not {
        Condition::Not(Box::new(inner))
    } else {
        inner
    })
}

fn node_condition(node: &Node) -> Result<Condition, RulesError> {
    match node {
        Node::Group(group) => group_condition(group),
        Node::Row(row) => {
            let leaf = row_condition(row)?;
            Ok(if row.not {
                Condition::Not(Box::new(leaf))
            } else {
                leaf
            })
        }
    }
}

fn row_condition(row: &Row) -> Result<Condition, RulesError> {
    let field = row.field.trim().to_lowercase();
    match &row.value {
        RowValue::Text(value) => {
            let op = text_op_parse(&row.operator.trim().to_lowercase()).ok_or_else(|| {
                error(format!(
                    "unknown text operator \"{}\" for \"{field}\"",
                    row.operator
                ))
            })?;
            Ok(Condition::Text {
                field,
                op,
                value: value.clone(),
            })
        }
        RowValue::Number(value) => {
            let op = number_op(row, &field)?;
            Ok(Condition::Number {
                field,
                op,
                value: *value,
            })
        }
        RowValue::Time(value) => {
            let op = number_op(row, &field)?;
            Ok(Condition::Time {
                field,
                op,
                value: value.clone(),
            })
        }
    }
}

fn number_op(row: &Row, field: &str) -> Result<NumberOp, RulesError> {
    number_op_parse(row.operator.trim()).ok_or_else(|| {
        error(format!(
            "unknown number operator \"{}\" for \"{field}\"",
            row.operator
        ))
    })
}

/// Show boxes as condition text (the printer of `rules`, so boxes and text agree).
/// The result is re-parsed so a hand-built tree with an unknown field or a bad date/time value
/// gives the same error as the text view.
pub fn tree_to_condition(tree: &Tree) -> Result<String, RulesError> {
    let condition = group_condition(&tree.root)?;
    let text = format_condition(&condition);
    parse_condition(&text)?;
    Ok(text)
}
