//! The condition parser: the date and clock forms a time value may take, and the recursive
//! descent that turns tokens into a flattened condition.

use jiff::civil::Date;

use super::condition::{
    CSV_ONLY_FIELDS, Condition, FieldType, Level, NumberOp, TextOp, field_type,
};
use super::error::RulesError;
use super::tokenizer::{Token, TokenKind, tokenize};

// -------------------------------------------------------------------- parser

pub(super) fn valid_calendar_date(text: &str) -> bool {
    text.parse::<Date>().is_ok()
}

pub(super) fn valid_clock(text: &str) -> bool {
    // Called only on a `HH:MM` (len 5) shape.
    let hour: u8 = text[..2].parse().unwrap_or(255);
    let minute: u8 = text[3..5].parse().unwrap_or(255);
    hour <= 23 && minute <= 59
}

pub(super) fn is_date_form(text: &str) -> bool {
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

pub(super) fn is_clock_form(text: &str) -> bool {
    let b: Vec<char> = text.chars().collect();
    b.len() == 5
        && b[0].is_ascii_digit()
        && b[1].is_ascii_digit()
        && b[2] == ':'
        && b[3].is_ascii_digit()
        && b[4].is_ascii_digit()
}

pub(super) fn is_datetime_form(text: &str) -> bool {
    let b: Vec<char> = text.chars().collect();
    b.len() == 16
        && is_date_form(&b[..10].iter().collect::<String>())
        && b[10] == ' '
        && is_clock_form(&b[11..].iter().collect::<String>())
}

/// Canonical form of a date/time value, or `None` when malformed. Only the three grammar forms
/// are accepted and each is already canonical, so the canonical form is the value itself.
pub(super) fn normalise_time_value(text: &str) -> Option<String> {
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

const TIME_ERROR: &str =
    "expected a date \"YYYY-MM-DD\", a time \"HH:MM\" or a date and time \"YYYY-MM-DD HH:MM\"";
const CSV_ERROR: &str = "needs the orders CSV, which the PC app does not read";

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
            FieldType::Text => self.parse_text_comparison(&field)?,
            FieldType::Time => self.parse_time_comparison(&field)?,
            FieldType::Number => self.parse_number_comparison(&field)?,
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

    fn parse_text_comparison(&mut self, field: &str) -> Result<Condition, RulesError> {
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
        Ok(Condition::Text {
            field: field.to_string(),
            op,
            value: value_token.text,
        })
    }

    fn parse_time_comparison(&mut self, field: &str) -> Result<Condition, RulesError> {
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
        Ok(Condition::Time {
            field: field.to_string(),
            op,
            value,
        })
    }

    fn parse_number_comparison(&mut self, field: &str) -> Result<Condition, RulesError> {
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
        Ok(Condition::Number {
            field: field.to_string(),
            op,
            value: parsed,
        })
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
