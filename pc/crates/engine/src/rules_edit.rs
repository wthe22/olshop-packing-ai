//! Categories editor support: condition text <-> boxes tree, and saving `categories.toml`
//! keeping its comments and unchanged condition texts (08 › 5. Categories).
//!
//! The boxes view (08 › 5) is a group of rows. A group is *All of these* (`and`) or *Any of
//! these* (`or`) and holds rows and other groups; a row is one comparison `field · operator ·
//! value`. A `not` flag sits on rows and groups. The tree is built with the engine's own parser
//! and turned back into text by its printer (04 › *How the app edits conditions*), so the window
//! has no second parser.
//!
//! `RulesDocument` edits the file through `toml_edit`, so comments and the original text of the
//! entries that were not changed survive a save (08 › 5: "comments kept", "a category not
//! changed keeps its text exactly as in the file, line breaks included").

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use toml_edit::{ArrayOfTables, DocumentMut, Item, Table};

use crate::orders::Order;
use crate::rules::{
    Category, Condition, NumberOp, RulesError, TextOp, evaluate, format_condition, load_rules,
    parse_condition,
};

/// A small `RulesError` (the field is public, the constructor in `rules` is not).
fn error(message: impl Into<String>) -> RulesError {
    RulesError {
        message: message.into(),
    }
}

/// The TOML item for a condition text: a literal string (`'…'`) like the file format, so the
/// double quotes inside the condition need no escaping; a basic string only when the text holds
/// a `'` (04 › *File format*).
fn when_item(text: &str) -> Item {
    if !text.contains('\'')
        && let Ok(value) = format!("'{text}'").parse::<toml_edit::Value>()
    {
        return Item::Value(value);
    }
    toml_edit::value(text)
}

// ------------------------------------------------------------------ boxes tree

/// A group's kind: **All of these** (`and`) or **Any of these** (`or`) (08 › 5).
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

/// The boxes of one condition: a root group (08 › 5: "Every condition starts as one *All of
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
/// kind; `not not …` cannot fold, so it wraps the inner node in a group (08 › 5: every valid
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

/// Parse a condition and show it as boxes (08 › 5; the parser of `rules`).
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

/// A group as a condition; an empty group is an error (08 › 5: "This group is empty"). A group
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

/// Show boxes as condition text (the printer of `rules`, so boxes and text agree — 08 › 5).
/// The result is re-parsed so a hand-built tree with an unknown field or a bad date/time value
/// gives the same error as the text view.
pub fn tree_to_condition(tree: &Tree) -> Result<String, RulesError> {
    let condition = group_condition(&tree.root)?;
    let text = format_condition(&condition);
    parse_condition(&text)?;
    Ok(text)
}

// ----------------------------------------------------------------- live counts

/// The orders each category takes in the open batch, in file order from what is left (08 › 5
/// "In batch 3"). The same first-match, take-from-what-is-left rule as `plan.rs`: a category
/// without a condition takes every order still left (the rest).
pub fn counts(categories: &[Category], orders: &[Order]) -> Vec<usize> {
    let mut remaining: Vec<&Order> = orders.iter().collect();
    let mut result = Vec::with_capacity(categories.len());
    for category in categories {
        let mut left = Vec::with_capacity(remaining.len());
        let mut taken = 0usize;
        for order in &remaining {
            let matches = match &category.condition {
                None => true,
                Some(condition) => evaluate(condition, order),
            };
            if matches {
                taken += 1;
            } else {
                left.push(*order);
            }
        }
        result.push(taken);
        remaining = left;
    }
    result
}

// ------------------------------------------------------------ editable document

/// One category entry as written in the file (04 › *File format*: `code`, `name`, `when`; those
/// are the only per-entry keys).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub code: String,
    pub name: String,
    /// The `when` text exactly as in the file; `None` when the entry has no `when`.
    pub when: Option<String>,
}

/// `categories.toml` as an editable document (08 › 5, *Edit file as text* / *Save*). Comments and
/// the text of the entries that were not changed are kept byte-for-byte; a changed condition is
/// written in the printer's one-line canonical form.
pub struct RulesDocument {
    doc: DocumentMut,
}

impl FromStr for RulesDocument {
    type Err = RulesError;

    /// Parse the text of `categories.toml`, keeping its comments and formatting.
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let doc =
            DocumentMut::from_str(text).map_err(|e| error(format!("categories.toml: {e}")))?;
        let document = Self { doc };
        document.array()?; // the file must hold `[[category]]` tables
        Ok(document)
    }
}

impl fmt::Display for RulesDocument {
    /// The file text as it would be written (comments and formatting kept).
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.doc)
    }
}

impl RulesDocument {
    fn array(&self) -> Result<&ArrayOfTables, RulesError> {
        match self.doc.get("category") {
            Some(item) => item
                .as_array_of_tables()
                .ok_or_else(|| error("categories.toml: \"category\" must be an array of tables")),
            None => Err(error("categories.toml: no categories")),
        }
    }

    fn array_mut(&mut self) -> Result<&mut ArrayOfTables, RulesError> {
        match self.doc.get_mut("category") {
            Some(item) => item
                .as_array_of_tables_mut()
                .ok_or_else(|| error("categories.toml: \"category\" must be an array of tables")),
            None => Err(error("categories.toml: no categories")),
        }
    }

    /// The entries in file order, with their values as written.
    pub fn entries(&self) -> Vec<Entry> {
        self.doc
            .get("category")
            .and_then(Item::as_array_of_tables)
            .map(|array| array.iter().map(entry_of).collect())
            .unwrap_or_default()
    }

    fn table_mut(&mut self, index: usize) -> Result<&mut Table, RulesError> {
        let array = self.array_mut()?;
        let len = array.len();
        array.get_mut(index).ok_or_else(|| {
            error(format!(
                "categories.toml: no category #{} of {len}",
                index + 1
            ))
        })
    }

    /// Change an entry's `code` (kept byte-for-byte when it does not change).
    pub fn set_code(&mut self, index: usize, code: &str) -> Result<(), RulesError> {
        self.set_string(index, "code", code)
    }

    /// Change an entry's `name` (kept byte-for-byte when it does not change).
    pub fn set_name(&mut self, index: usize, name: &str) -> Result<(), RulesError> {
        self.set_string(index, "name", name)
    }

    fn set_string(&mut self, index: usize, key: &str, value: &str) -> Result<(), RulesError> {
        let table = self.table_mut(index)?;
        if table.get(key).and_then(Item::as_str) == Some(value) {
            return Ok(());
        }
        table.insert(key, toml_edit::value(value));
        Ok(())
    }

    /// Set (or clear, with `None`) an entry's condition from its condition text. When the new
    /// condition's canonical form equals the old one the file text is left exactly as it was
    /// (08 › 5: "a category not changed keeps its text exactly as in the file").
    pub fn set_condition(
        &mut self,
        index: usize,
        condition: Option<&str>,
    ) -> Result<(), RulesError> {
        let new_canonical = match condition {
            None => None,
            Some(text) => Some(format_condition(&parse_condition(text)?)),
        };
        let table = self.table_mut(index)?;
        let old_canonical = table
            .get("when")
            .and_then(Item::as_str)
            .and_then(|text| parse_condition(text).ok())
            .map(|cond| format_condition(&cond));
        if new_canonical == old_canonical {
            return Ok(());
        }
        match new_canonical {
            Some(text) => {
                table.insert("when", when_item(&text));
            }
            None => {
                table.remove("when");
            }
        }
        Ok(())
    }

    /// Add an entry at position `index` (08 › 5: *+ Add category* adds one above the rest entry,
    /// i.e. at the last position when the last entry has no condition).
    pub fn add(
        &mut self,
        index: usize,
        code: &str,
        name: &str,
        condition: Option<&str>,
    ) -> Result<(), RulesError> {
        let when = match condition {
            None => None,
            Some(text) => Some(format_condition(&parse_condition(text)?)),
        };
        let mut table = Table::new();
        table.insert("code", toml_edit::value(code));
        table.insert("name", toml_edit::value(name));
        if let Some(text) = when {
            table.insert("when", when_item(&text));
        }

        let array = self.array_mut()?;
        let len = array.len();
        if index > len {
            return Err(error(format!(
                "categories.toml: cannot add at #{index}: the file has {len} categories"
            )));
        }
        array.insert(index, table);
        self.renumber();
        Ok(())
    }

    /// Delete an entry.
    pub fn remove(&mut self, index: usize) -> Result<(), RulesError> {
        let array = self.array_mut()?;
        let len = array.len();
        if index >= len {
            return Err(error(format!(
                "categories.toml: no category #{} of {len}",
                index + 1
            )));
        }
        array.remove(index);
        self.renumber();
        Ok(())
    }

    /// Move the entry at `from` to position `to` (the target position after removal).
    pub fn move_to(&mut self, from: usize, to: usize) -> Result<(), RulesError> {
        let array = self.array_mut()?;
        let len = array.len();
        if from >= len {
            return Err(error(format!(
                "categories.toml: no category #{} of {len}",
                from + 1
            )));
        }
        if to >= len {
            return Err(error(format!(
                "categories.toml: cannot move to #{} of {len}",
                to + 1
            )));
        }
        let table = array.remove(from);
        array.insert(to, table);
        self.renumber();
        Ok(())
    }

    /// Validate the document as it would be written (04 validation; errors carry the file's
    /// line/col). Returns the categories, ready for the plan.
    pub fn validate(&self) -> Result<Vec<Category>, RulesError> {
        load_rules(&self.to_string())
    }

    /// toml_edit orders array-of-tables entries by their recorded position, so after a reorder the
    /// positions must follow the new array order or the move would be undone when written.
    fn renumber(&mut self) {
        if let Some(item) = self.doc.get_mut("category")
            && let Some(array) = item.as_array_of_tables_mut()
        {
            for (index, table) in array.iter_mut().enumerate() {
                table.set_position(Some(index as isize));
            }
        }
    }
}

fn entry_of(table: &Table) -> Entry {
    Entry {
        code: table
            .get("code")
            .and_then(Item::as_str)
            .unwrap_or_default()
            .to_string(),
        name: table
            .get("name")
            .and_then(Item::as_str)
            .unwrap_or_default()
            .to_string(),
        when: table.get("when").and_then(Item::as_str).map(str::to_string),
    }
}
