//! The condition types: the tree an entry's `when` parses into, the two operator sets, the
//! field table that says which fields exist and how deep they can be used, and one entry of
//! `categories.toml`.

/// A text comparison operator (`contains` | `equals` | `starts_with`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextOp {
    Contains,
    Equals,
    StartsWith,
}

impl TextOp {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            TextOp::Contains => "contains",
            TextOp::Equals => "equals",
            TextOp::StartsWith => "starts_with",
        }
    }

    pub(super) fn parse(text: &str) -> Option<Self> {
        match text {
            "contains" => Some(TextOp::Contains),
            "equals" => Some(TextOp::Equals),
            "starts_with" => Some(TextOp::StartsWith),
            _ => None,
        }
    }
}

/// The operator of a text field with no value: `courier is_empty`.
pub const IS_EMPTY: &str = "is_empty";

/// A number/date-time comparison operator (`=` | `!=` | `<` | `<=` | `>` | `>=`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NumberOp {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

impl NumberOp {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            NumberOp::Eq => "=",
            NumberOp::Ne => "!=",
            NumberOp::Lt => "<",
            NumberOp::Le => "<=",
            NumberOp::Gt => ">",
            NumberOp::Ge => ">=",
        }
    }

    pub(super) fn parse(text: &str) -> Option<Self> {
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
}

/// Parsed condition (AST root). `field` names are stored lowercased; a text/date-time value
/// keeps its original characters (only string escapes are resolved).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Condition {
    Text {
        field: String,
        op: TextOp,
        value: String,
    },
    Number {
        field: String,
        op: NumberOp,
        value: i64,
    },
    /// `field is_empty`: a text field with no value (spaces count as none).
    Empty {
        field: String,
    },
    /// `value` is canonical: `YYYY-MM-DD`, `HH:MM` or `YYYY-MM-DD HH:MM`.
    Time {
        field: String,
        op: NumberOp,
        value: String,
    },
    Not(Box<Condition>),
    /// Flattened: no direct `And` child.
    And(Vec<Condition>),
    /// Flattened: no direct `Or` child.
    Or(Vec<Condition>),
}

impl Condition {
    pub(super) fn precedence(&self) -> u8 {
        match self {
            Condition::Or(_) => 1,
            Condition::And(_) => 2,
            Condition::Not(_) => 3,
            _ => 4,
        }
    }
}

/// One entry of `categories.toml` = one pick (04-rules-file).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Category {
    /// 1-3 letters or digits, unique.
    pub code: String,
    /// Heading text.
    pub name: String,
    /// The parsed `when`; `None` only when the entry has no `when` (last entry / the rest).
    pub condition: Option<Condition>,
    /// The original `when` text as written in the file; `None` when the entry has no `when`.
    pub when: Option<String>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum FieldType {
    Text,
    Number,
    Time,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Level {
    Item,
    Order,
    App,
}

/// field -> (type, level). Level `App` (category/batch/group) is never allowed here.
pub(super) const FIELDS: &[(&str, FieldType, Level)] = &[
    ("name", FieldType::Text, Level::Item),
    ("display_name", FieldType::Text, Level::Item),
    ("variation", FieldType::Text, Level::Item),
    ("sku_id", FieldType::Text, Level::Item),
    ("seller_sku", FieldType::Text, Level::Item),
    ("product_category", FieldType::Text, Level::Item),
    ("line_quantity", FieldType::Number, Level::Item),
    ("total_quantity", FieldType::Number, Level::Order),
    ("distinct_items", FieldType::Number, Level::Order),
    ("courier", FieldType::Text, Level::Order),
    ("channel", FieldType::Text, Level::Order),
    ("order_id", FieldType::Text, Level::Order),
    ("tracking_id", FieldType::Text, Level::Order),
    ("ship_by", FieldType::Time, Level::Order),
    ("paid_time", FieldType::Time, Level::Order),
    ("rts_time", FieldType::Time, Level::Order),
    ("created_time", FieldType::Time, Level::Order),
    ("category", FieldType::Text, Level::App),
    ("batch", FieldType::Number, Level::App),
    ("group", FieldType::Number, Level::App),
];

/// Fields whose value comes only from the orders CSV; the PC app reads no CSV, so a condition
/// using one is an error (07-pc-app "Fields the conditions can use").
pub(super) const CSV_ONLY_FIELDS: &[&str] = &[
    "sku_id",
    "product_category",
    "channel",
    "paid_time",
    "rts_time",
    "created_time",
];

pub(super) fn field_type(field: &str) -> Option<(FieldType, Level)> {
    FIELDS
        .iter()
        .find(|(name, _, _)| *name == field)
        .map(|(_, t, l)| (*t, *l))
}
