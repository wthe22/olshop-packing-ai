//! The tokenizer of a condition text: words, quoted strings, operators and parentheses, each
//! with its line and column for the error messages.

use super::error::RulesError;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum TokenKind {
    Word,
    Str,
    Op,
    LParen,
    RParen,
    Eof,
}

#[derive(Clone)]
pub(super) struct Token {
    pub(super) kind: TokenKind,
    pub(super) text: String,
    pub(super) line: usize,
    pub(super) col: usize,
}

/// Where the tokenizer is in the text, with the line and column the errors name.
struct Cursor {
    i: usize,
    line: usize,
    col: usize,
}

impl Cursor {
    fn advance(&mut self, chars: &[char], k: usize) {
        for _ in 0..k {
            if self.i < chars.len() && chars[self.i] == '\n' {
                self.line += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }
            self.i += 1;
        }
    }
}

/// A `"…"` literal, read from its opening quote.
fn scan_string(chars: &[char], cursor: &mut Cursor) -> Result<Token, RulesError> {
    let (start_line, start_col) = (cursor.line, cursor.col);
    cursor.advance(chars, 1); // opening quote
    let mut value = String::new();
    let mut closed = false;
    while cursor.i < chars.len() {
        let c = chars[cursor.i];
        if c == '"' {
            cursor.advance(chars, 1);
            closed = true;
            break;
        }
        if c == '\\' {
            let (esc_line, esc_col) = (cursor.line, cursor.col);
            cursor.advance(chars, 1);
            if cursor.i < chars.len() {
                let e = chars[cursor.i];
                if e == '"' {
                    value.push('"');
                } else if e == '\\' {
                    value.push('\\');
                } else {
                    return Err(RulesError::new(format!(
                        "line {esc_line}, col {esc_col}: bad escape sequence \"\\{e}\""
                    )));
                }
                cursor.advance(chars, 1);
            }
            continue;
        }
        value.push(c);
        cursor.advance(chars, 1);
    }
    if !closed {
        return Err(RulesError::new(format!(
            "line {start_line}, col {start_col}: unterminated string"
        )));
    }
    Ok(Token {
        kind: TokenKind::Str,
        text: value,
        line: start_line,
        col: start_col,
    })
}

/// An operator: `=`, `<`, `>`, `!` or the two-character forms `<=`, `>=` and `!=`.
fn scan_operator(chars: &[char], cursor: &mut Cursor) -> Token {
    let ch = chars[cursor.i];
    let nxt = if cursor.i + 1 < chars.len() {
        chars[cursor.i + 1]
    } else {
        '\0'
    };
    let (text, width) = if nxt == '=' && ch != '=' {
        (format!("{ch}="), 2)
    } else {
        (ch.to_string(), 1)
    };
    let token = Token {
        kind: TokenKind::Op,
        text,
        line: cursor.line,
        col: cursor.col,
    };
    cursor.advance(chars, width);
    token
}

pub(super) fn tokenize(text: &str) -> Result<Vec<Token>, RulesError> {
    let chars: Vec<char> = text.chars().collect();
    let mut cursor = Cursor {
        i: 0,
        line: 1,
        col: 1,
    };
    let mut tokens: Vec<Token> = Vec::new();

    while cursor.i < chars.len() {
        let ch = chars[cursor.i];
        if ch == ' ' || ch == '\t' || ch == '\r' || ch == '\n' {
            cursor.advance(&chars, 1);
            continue;
        }
        if ch == '(' || ch == ')' {
            let kind = if ch == '(' {
                TokenKind::LParen
            } else {
                TokenKind::RParen
            };
            tokens.push(Token {
                kind,
                text: ch.to_string(),
                line: cursor.line,
                col: cursor.col,
            });
            cursor.advance(&chars, 1);
            continue;
        }
        if ch == '"' {
            tokens.push(scan_string(&chars, &mut cursor)?);
            continue;
        }
        if ch == '=' || ch == '<' || ch == '>' || ch == '!' {
            tokens.push(scan_operator(&chars, &mut cursor));
            continue;
        }
        let (start_line, start_col) = (cursor.line, cursor.col);
        let start = cursor.i;
        while cursor.i < chars.len() && !IDENT_STOP.contains(&chars[cursor.i]) {
            cursor.advance(&chars, 1);
        }
        let word: String = chars[start..cursor.i].iter().collect();
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
        line: cursor.line,
        col: cursor.col,
    });
    Ok(tokens)
}

/// Characters that end an identifier or a bare word.
const IDENT_STOP: &[char] = &[' ', '\t', '\r', '\n', '(', ')', '"', '=', '<', '>', '!'];
