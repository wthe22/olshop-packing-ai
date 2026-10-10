//! `categories.toml` and the condition language (04-rules-file): loading and validation, the
//! condition tokenizer/parser, the evaluator (item-level "any line" semantics) and the printer.
//!
//! The PC app reads no orders CSV, so the CSV-only fields are known but always an error
//! (07-pc-app "Fields the conditions can use"); the app scan-filter fields stay errors too.

use std::collections::BTreeSet;
use std::fmt;

use jiff::civil::{Date, DateTime};
use regex::Regex;

/// Invalid rules file or condition; the message names the place (`categories.toml`, line, col).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RulesError {
    pub message: String,
}

impl RulesError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for RulesError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for RulesError {}

/// A text comparison operator (`contains` | `equals` | `starts_with`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextOp {
    Contains,
    Equals,
    StartsWith,
}

impl TextOp {
    fn as_str(self) -> &'static str {
        match self {
            TextOp::Contains => "contains",
            TextOp::Equals => "equals",
            TextOp::StartsWith => "starts_with",
        }
    }

    fn parse(text: &str) -> Option<Self> {
        match text {
            "contains" => Some(TextOp::Contains),
            "equals" => Some(TextOp::Equals),
            "starts_with" => Some(TextOp::StartsWith),
            _ => None,
        }
    }
}

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
    fn as_str(self) -> &'static str {
        match self {
            NumberOp::Eq => "=",
            NumberOp::Ne => "!=",
            NumberOp::Lt => "<",
            NumberOp::Le => "<=",
            NumberOp::Gt => ">",
            NumberOp::Ge => ">=",
        }
    }

    fn parse(text: &str) -> Option<Self> {
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
    fn precedence(&self) -> u8 {
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

// --------------------------------------------------------------- field table

#[derive(Clone, Copy, PartialEq, Eq)]
enum FieldType {
    Text,
    Number,
    Time,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Level {
    Item,
    Order,
    App,
}

/// field -> (type, level). Level `App` (category/batch/group) is never allowed here.
const FIELDS: &[(&str, FieldType, Level)] = &[
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
const CSV_ONLY_FIELDS: &[&str] = &[
    "sku_id",
    "product_category",
    "channel",
    "paid_time",
    "rts_time",
    "created_time",
];

const TIME_ERROR: &str =
    "expected a date \"YYYY-MM-DD\", a time \"HH:MM\" or a date and time \"YYYY-MM-DD HH:MM\"";
const CSV_ERROR: &str = "needs the orders CSV, which the PC app does not read";
const IDENT_STOP: &[char] = &[' ', '\t', '\r', '\n', '(', ')', '"', '=', '<', '>', '!'];

fn field_type(field: &str) -> Option<(FieldType, Level)> {
    FIELDS
        .iter()
        .find(|(name, _, _)| *name == field)
        .map(|(_, t, l)| (*t, *l))
}

// ------------------------------------------------------------------ tokenizer

#[derive(Clone, Copy, PartialEq, Eq)]
enum TokenKind {
    Word,
    Str,
    Op,
    LParen,
    RParen,
    Eof,
}

#[derive(Clone)]
struct Token {
    kind: TokenKind,
    text: String,
    line: usize,
    col: usize,
}

fn advance(chars: &[char], i: &mut usize, line: &mut usize, col: &mut usize, k: usize) {
    for _ in 0..k {
        if *i < chars.len() && chars[*i] == '\n' {
            *line += 1;
            *col = 1;
        } else {
            *col += 1;
        }
        *i += 1;
    }
}

fn tokenize(text: &str) -> Result<Vec<Token>, RulesError> {
    let chars: Vec<char> = text.chars().collect();
    let n = chars.len();
    let mut i = 0usize;
    let mut line = 1usize;
    let mut col = 1usize;
    let mut tokens: Vec<Token> = Vec::new();

    while i < n {
        let ch = chars[i];
        if ch == ' ' || ch == '\t' || ch == '\r' || ch == '\n' {
            advance(&chars, &mut i, &mut line, &mut col, 1);
            continue;
        }
        if ch == '(' {
            tokens.push(Token {
                kind: TokenKind::LParen,
                text: "(".into(),
                line,
                col,
            });
            advance(&chars, &mut i, &mut line, &mut col, 1);
            continue;
        }
        if ch == ')' {
            tokens.push(Token {
                kind: TokenKind::RParen,
                text: ")".into(),
                line,
                col,
            });
            advance(&chars, &mut i, &mut line, &mut col, 1);
            continue;
        }
        if ch == '"' {
            let (start_line, start_col) = (line, col);
            advance(&chars, &mut i, &mut line, &mut col, 1); // opening quote
            let mut value = String::new();
            let mut closed = false;
            while i < n {
                let c = chars[i];
                if c == '"' {
                    advance(&chars, &mut i, &mut line, &mut col, 1);
                    closed = true;
                    break;
                }
                if c == '\\' {
                    let (esc_line, esc_col) = (line, col);
                    advance(&chars, &mut i, &mut line, &mut col, 1);
                    if i < n {
                        let e = chars[i];
                        if e == '"' {
                            value.push('"');
                        } else if e == '\\' {
                            value.push('\\');
                        } else {
                            return Err(RulesError::new(format!(
                                "line {esc_line}, col {esc_col}: bad escape sequence \"\\{e}\""
                            )));
                        }
                        advance(&chars, &mut i, &mut line, &mut col, 1);
                    }
                    continue;
                }
                value.push(c);
                advance(&chars, &mut i, &mut line, &mut col, 1);
            }
            if !closed {
                return Err(RulesError::new(format!(
                    "line {start_line}, col {start_col}: unterminated string"
                )));
            }
            tokens.push(Token {
                kind: TokenKind::Str,
                text: value,
                line: start_line,
                col: start_col,
            });
            continue;
        }
        if ch == '=' || ch == '<' || ch == '>' || ch == '!' {
            let nxt = if i + 1 < n { chars[i + 1] } else { '\0' };
            if nxt == '=' && ch != '=' {
                tokens.push(Token {
                    kind: TokenKind::Op,
                    text: format!("{ch}="),
                    line,
                    col,
                });
                advance(&chars, &mut i, &mut line, &mut col, 2);
            } else {
                tokens.push(Token {
                    kind: TokenKind::Op,
                    text: ch.to_string(),
                    line,
                    col,
                });
                advance(&chars, &mut i, &mut line, &mut col, 1);
            }
            continue;
        }
        let (start_line, start_col) = (line, col);
        let start = i;
        while i < n && !IDENT_STOP.contains(&chars[i]) {
            advance(&chars, &mut i, &mut line, &mut col, 1);
        }
        let word: String = chars[start..i].iter().collect();
        tokens.push(Token {
            kind: TokenKind::Word,
            text: word,
            line: start_line,
            col: start_col,
        });
    }

    tokens.push(Token {
        kind: TokenKind::Eof,
        text: String::new(),
        line,
        col,
    });
    Ok(tokens)
}

// -------------------------------------------------------------------- parser

fn valid_calendar_date(text: &str) -> bool {
    text.parse::<Date>().is_ok()
}

fn valid_clock(text: &str) -> bool {
    // Called only on a `HH:MM` (len 5) shape.
    let hour: u8 = text[..2].parse().unwrap_or(255);
    let minute: u8 = text[3..5].parse().unwrap_or(255);
    hour <= 23 && minute <= 59
}

fn is_date_form(text: &str) -> bool {
    let b: Vec<char> = text.chars().collect();
    b.len() == 10
        && b[0].is_ascii_digit()
        && b[1].is_ascii_digit()
        && b[2].is_ascii_digit()
        && b[3].is_ascii_digit()
        && b[4] == '-'
        && b[5].is_ascii_digit()
        && b[6].is_ascii_digit()
        && b[7] == '-'
        && b[8].is_ascii_digit()
        && b[9].is_ascii_digit()
}

fn is_clock_form(text: &str) -> bool {
    let b: Vec<char> = text.chars().collect();
    b.len() == 5
        && b[0].is_ascii_digit()
        && b[1].is_ascii_digit()
        && b[2] == ':'
        && b[3].is_ascii_digit()
        && b[4].is_ascii_digit()
}

fn is_datetime_form(text: &str) -> bool {
    let b: Vec<char> = text.chars().collect();
    b.len() == 16
        && is_date_form(&b[..10].iter().collect::<String>())
        && b[10] == ' '
        && is_clock_form(&b[11..].iter().collect::<String>())
}

/// Canonical form of a date/time value, or `None` when malformed. Only the three grammar forms
/// are accepted and each is already canonical, so the canonical form is the value itself.
fn normalise_time_value(text: &str) -> Option<String> {
    if is_date_form(text) {
        return valid_calendar_date(text).then(|| text.to_string());
    }
    if is_clock_form(text) {
        return valid_clock(text).then(|| text.to_string());
    }
    if is_datetime_form(text) {
        let date: String = text.chars().take(10).collect();
        let clock: String = text.chars().skip(11).collect();
        if valid_calendar_date(&date) && valid_clock(&clock) {
            return Some(text.to_string());
        }
    }
    None
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn peek(&self) -> &Token {
        &self.tokens[self.pos]
    }

    fn advance(&mut self) -> Token {
        let token = self.tokens[self.pos].clone();
        self.pos += 1;
        token
    }

    fn at_word(&self, word: &str) -> bool {
        let token = self.peek();
        token.kind == TokenKind::Word && token.text.eq_ignore_ascii_case(word)
    }

    fn parse_or(&mut self) -> Result<Condition, RulesError> {
        let mut operands = vec![self.parse_and()?];
        while self.at_word("or") {
            self.advance();
            operands.push(self.parse_and()?);
        }
        if operands.len() == 1 {
            return Ok(operands.pop().expect("one operand"));
        }
        let mut flat = Vec::new();
        for operand in operands {
            match operand {
                Condition::Or(inner) => flat.extend(inner),
                other => flat.push(other),
            }
        }
        Ok(Condition::Or(flat))
    }

    fn parse_and(&mut self) -> Result<Condition, RulesError> {
        let mut operands = vec![self.parse_not()?];
        while self.at_word("and") {
            self.advance();
            operands.push(self.parse_not()?);
        }
        if operands.len() == 1 {
            return Ok(operands.pop().expect("one operand"));
        }
        let mut flat = Vec::new();
        for operand in operands {
            match operand {
                Condition::And(inner) => flat.extend(inner),
                other => flat.push(other),
            }
        }
        Ok(Condition::And(flat))
    }

    fn parse_not(&mut self) -> Result<Condition, RulesError> {
        if self.at_word("not") {
            self.advance();
            return Ok(Condition::Not(Box::new(self.parse_not()?)));
        }
        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<Condition, RulesError> {
        if self.peek().kind == TokenKind::LParen {
            self.advance();
            let expr = self.parse_or()?;
            let closing = self.peek();
            if closing.kind != TokenKind::RParen {
                return Err(RulesError::new(format!(
                    "line {}, col {}: expected ')'",
                    closing.line, closing.col
                )));
            }
            self.advance();
            return Ok(expr);
        }
        self.parse_comparison()
    }

    fn parse_comparison(&mut self) -> Result<Condition, RulesError> {
        let token = self.peek().clone();
        let Some((ftype, level)) = field_type(&token.text.to_lowercase()) else {
            return Err(RulesError::new(format!(
                "line {}, col {}: unknown field \"{}\"",
                token.line, token.col, token.text
            )));
        };
        let field = token.text.to_lowercase();
        if level == Level::App {
            return Err(RulesError::new(format!(
                "line {}, col {}: field \"{field}\" is allowed in app scan filters only",
                token.line, token.col
            )));
        }
        self.advance();

        let node = match ftype {
            FieldType::Text => {
                let op_token = self.peek().clone();
                let op = (op_token.kind == TokenKind::Word)
                    .then(|| TextOp::parse(&op_token.text.to_lowercase()))
                    .flatten();
                let Some(op) = op else {
                    return Err(RulesError::new(format!(
                        "line {}, col {}: expected a text operator (contains, equals, starts_with) after \"{field}\"",
                        op_token.line, op_token.col
                    )));
                };
                self.advance();
                let value_token = self.peek().clone();
                if value_token.kind != TokenKind::Str {
                    return Err(RulesError::new(format!(
                        "line {}, col {}: expected a string after \"{field} {}\"",
                        value_token.line,
                        value_token.col,
                        op.as_str()
                    )));
                }
                self.advance();
                Condition::Text {
                    field: field.clone(),
                    op,
                    value: value_token.text,
                }
            }
            FieldType::Time => {
                let op_token = self.peek().clone();
                let op = (op_token.kind == TokenKind::Op)
                    .then(|| NumberOp::parse(&op_token.text))
                    .flatten();
                let Some(op) = op else {
                    return Err(RulesError::new(format!(
                        "line {}, col {}: expected a number operator (=, !=, <, <=, >, >=) after \"{field}\"",
                        op_token.line, op_token.col
                    )));
                };
                self.advance();
                let value_token = self.peek().clone();
                let value = if value_token.kind == TokenKind::Str {
                    normalise_time_value(&value_token.text)
                } else {
                    None
                };
                let Some(value) = value else {
                    return Err(RulesError::new(format!(
                        "line {}, col {}: {TIME_ERROR}",
                        value_token.line, value_token.col
                    )));
                };
                self.advance();
                Condition::Time {
                    field: field.clone(),
                    op,
                    value,
                }
            }
            FieldType::Number => {
                let op_token = self.peek().clone();
                let op = (op_token.kind == TokenKind::Op)
                    .then(|| NumberOp::parse(&op_token.text))
                    .flatten();
                let Some(op) = op else {
                    return Err(RulesError::new(format!(
                        "line {}, col {}: expected a number operator (=, !=, <, <=, >, >=) after \"{field}\"",
                        op_token.line, op_token.col
                    )));
                };
                self.advance();
                let value_token = self.peek().clone();
                let is_integer = value_token.kind == TokenKind::Word
                    && !value_token.text.is_empty()
                    && value_token.text.chars().all(|c| c.is_ascii_digit());
                if !is_integer {
                    return Err(RulesError::new(format!(
                        "line {}, col {}: expected an integer after \"{field} {}\"",
                        value_token.line,
                        value_token.col,
                        op.as_str()
                    )));
                }
                let parsed = value_token.text.parse::<i64>().map_err(|_| {
                    RulesError::new(format!(
                        "line {}, col {}: expected an integer after \"{field} {}\"",
                        value_token.line,
                        value_token.col,
                        op.as_str()
                    ))
                })?;
                self.advance();
                Condition::Number {
                    field: field.clone(),
                    op,
                    value: parsed,
                }
            }
        };

        // CSV-only fields are known but unusable in the PC app: report after the comparison is
        // structurally valid, so a malformed comparison still gives its own error.
        if CSV_ONLY_FIELDS.contains(&field.as_str()) {
            return Err(RulesError::new(format!(
                "line {}, col {}: field \"{field}\" {CSV_ERROR}",
                token.line, token.col
            )));
        }
        Ok(node)
    }
}

/// Parse a condition; on a bad text the error is `line L, col C: expected …`.
pub fn parse_condition(text: &str) -> Result<Condition, RulesError> {
    let tokens = tokenize(text)?;
    if tokens.first().map(|t| t.kind) == Some(TokenKind::Eof) {
        return Err(RulesError::new("line 1, col 1: expected a condition"));
    }
    let mut parser = Parser { tokens, pos: 0 };
    let node = parser.parse_or()?;
    let trailing = parser.peek();
    if trailing.kind != TokenKind::Eof {
        return Err(RulesError::new(format!(
            "line {}, col {}: unexpected text after the condition",
            trailing.line, trailing.col
        )));
    }
    Ok(node)
}

// ------------------------------------------------------------------- printer

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

// ----------------------------------------------------------------- evaluator

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
            "courier" => text_match(&order.courier, *op, value),
            "tracking_id" => text_match(&order.tracking_id, *op, value),
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

// ---------------------------------------------------------------- rules file

const RULES_FILE_NAME: &str = "categories.toml";
const CATEGORY_KEYS: &[&str] = &["code", "name", "when"];

fn valid_code(code: &str) -> bool {
    let chars: Vec<char> = code.chars().collect();
    (1..=3).contains(&chars.len()) && chars.iter().all(|c| c.is_ascii_alphanumeric())
}

fn category_header_re() -> Regex {
    Regex::new(r"(?m)^[ \t]*\[\[[ \t]*category[ \t]*\]\]").expect("constant regex compiles")
}

fn when_key_re() -> Regex {
    Regex::new(r"(?m)^[ \t]*when[ \t]*=").expect("constant regex compiles")
}

/// 1-based (line, col) of the condition text for the `index`-th `[[category]]` table. The TOML
/// crate reports no positions, so the raw file text is scanned: the index-th header, then its
/// `when` key, then the start of the string content.
fn locate_when_value(text: &str, index: usize) -> Option<(usize, usize)> {
    let headers: Vec<usize> = category_header_re()
        .find_iter(text)
        .map(|m| m.start())
        .collect();
    let start = *headers.get(index)?;
    let end = headers.get(index + 1).copied().unwrap_or(text.len());
    let chunk = &text[start..end];
    let key = when_key_re().find(chunk)?;
    let mut pos = chunk[key.start()..].find('=')? + key.start() + 1;
    while pos < chunk.len() && (chunk.as_bytes()[pos] == b' ' || chunk.as_bytes()[pos] == b'\t') {
        pos += 1;
    }
    let content = if chunk[pos..].starts_with("'''") || chunk[pos..].starts_with("\"\"\"") {
        let mut content = pos + 3;
        if content < chunk.len() && chunk.as_bytes()[content] == b'\n' {
            content += 1;
        }
        content
    } else if pos < chunk.len() && (chunk.as_bytes()[pos] == b'\'' || chunk.as_bytes()[pos] == b'"')
    {
        pos + 1
    } else {
        return None;
    };
    let absolute = start + content;
    let before = &text[..absolute];
    let line = before.matches('\n').count() + 1;
    let col = match before.rfind('\n') {
        Some(i) => before[i + 1..].chars().count() + 1,
        None => before.chars().count() + 1,
    };
    Some((line, col))
}

fn condition_pos_re() -> Regex {
    Regex::new(r"(?s)^line (\d+), col (\d+): (.*)$").expect("constant regex compiles")
}

/// Load the categories from the text of `categories.toml` (the app reads the file). Errors
/// carry the line/col inside the file, like the reference implementation.
pub fn load_rules(text: &str) -> Result<Vec<Category>, RulesError> {
    let table: toml::Table =
        toml::from_str(text).map_err(|e| RulesError::new(format!("{RULES_FILE_NAME}: {e}")))?;

    for key in table.keys() {
        if key != "category" {
            return Err(RulesError::new(format!(
                "{RULES_FILE_NAME}: unknown key \"{key}\""
            )));
        }
    }

    let raw = match table.get("category") {
        None => return Err(RulesError::new(format!("{RULES_FILE_NAME}: no categories"))),
        Some(toml::Value::Array(entries)) => entries,
        Some(_) => {
            return Err(RulesError::new(format!(
                "{RULES_FILE_NAME}: \"category\" must be an array of tables"
            )));
        }
    };
    if raw.is_empty() {
        return Err(RulesError::new(format!("{RULES_FILE_NAME}: no categories")));
    }

    let mut categories: Vec<Category> = Vec::new();
    let mut seen: BTreeSet<String> = BTreeSet::new();
    let count = raw.len();
    for (index, entry) in raw.iter().enumerate() {
        let toml::Value::Table(entry) = entry else {
            return Err(RulesError::new(format!(
                "{RULES_FILE_NAME}: category #{}: must be a table",
                index + 1
            )));
        };
        let code_value = entry.get("code");
        // Name the category by code when there is one, else by its position.
        let cid = match code_value {
            Some(toml::Value::String(s)) if !s.is_empty() => format!("\"{s}\""),
            _ => format!("#{}", index + 1),
        };

        for key in entry.keys() {
            if !CATEGORY_KEYS.contains(&key.as_str()) {
                return Err(RulesError::new(format!(
                    "{RULES_FILE_NAME}: category {cid}: unknown key \"{key}\""
                )));
            }
        }

        let Some(code_value) = code_value else {
            return Err(RulesError::new(format!(
                "{RULES_FILE_NAME}: category {cid}: missing \"code\""
            )));
        };
        let toml::Value::String(code) = code_value else {
            return Err(RulesError::new(format!(
                "{RULES_FILE_NAME}: category {cid}: \"code\" must be a string"
            )));
        };
        if code.is_empty() {
            return Err(RulesError::new(format!(
                "{RULES_FILE_NAME}: category {cid}: \"code\" must not be empty"
            )));
        }
        if !valid_code(code) {
            return Err(RulesError::new(format!(
                "{RULES_FILE_NAME}: category {cid}: \"code\" must be 1-3 letters or digits"
            )));
        }
        if seen.contains(code) {
            return Err(RulesError::new(format!(
                "{RULES_FILE_NAME}: category {cid}: duplicate \"code\" \"{code}\""
            )));
        }
        seen.insert(code.clone());

        let Some(name_value) = entry.get("name") else {
            return Err(RulesError::new(format!(
                "{RULES_FILE_NAME}: category {cid}: missing \"name\""
            )));
        };
        let toml::Value::String(name) = name_value else {
            return Err(RulesError::new(format!(
                "{RULES_FILE_NAME}: category {cid}: \"name\" must be a string"
            )));
        };
        if name.is_empty() {
            return Err(RulesError::new(format!(
                "{RULES_FILE_NAME}: category {cid}: \"name\" must not be empty"
            )));
        }

        let when_text = match entry.get("when") {
            None => None,
            Some(toml::Value::String(s)) => Some(s.clone()),
            Some(_) => {
                return Err(RulesError::new(format!(
                    "{RULES_FILE_NAME}: category {cid}: \"when\" must be a string"
                )));
            }
        };

        let condition = match &when_text {
            None => {
                if index != count - 1 {
                    return Err(RulesError::new(format!(
                        "{RULES_FILE_NAME}: category {cid}: missing \"when\" on a category that is not last"
                    )));
                }
                None
            }
            Some(when) => Some(parse_when(when, text, index, &cid)?),
        };

        categories.push(Category {
            code: code.clone(),
            name: name.clone(),
            condition,
            when: when_text,
        });
    }
    Ok(categories)
}

fn parse_when(
    when: &str,
    file_text: &str,
    index: usize,
    cid: &str,
) -> Result<Condition, RulesError> {
    match parse_condition(when) {
        Ok(node) => Ok(node),
        Err(e) => {
            let where_ = locate_when_value(file_text, index);
            let pos = condition_pos_re().captures(&e.message);
            if let (Some((start_line, start_col)), Some(caps)) = (where_, pos) {
                let cond_line: usize = caps[1].parse().unwrap_or(1);
                let cond_col: usize = caps[2].parse().unwrap_or(1);
                let rest = &caps[3];
                let (file_line, file_col) = if cond_line == 1 {
                    (start_line, start_col + cond_col - 1)
                } else {
                    (start_line + cond_line - 1, cond_col)
                };
                return Err(RulesError::new(format!(
                    "{RULES_FILE_NAME}, line {file_line}, col {file_col}: category {cid} (when): {rest}"
                )));
            }
            Err(RulesError::new(format!(
                "{RULES_FILE_NAME}: category {cid} (when): {}",
                e.message
            )))
        }
    }
}

/// The first entry whose condition matches the order (an entry without a condition matches
/// every order); `None` means no entry matched (the "rest").
pub fn categorize<'a>(
    order: &crate::orders::Order,
    categories: &'a [Category],
) -> Option<&'a Category> {
    categories
        .iter()
        .find(|category| match &category.condition {
            None => true,
            Some(cond) => evaluate(cond, order),
        })
}
