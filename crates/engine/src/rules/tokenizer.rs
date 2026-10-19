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

pub(super) fn tokenize(text: &str) -> Result<Vec<Token>, RulesError> {
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

/// Characters that end an identifier or a bare word.
const IDENT_STOP: &[char] = &[' ', '\t', '\r', '\n', '(', ')', '"', '=', '<', '>', '!'];
