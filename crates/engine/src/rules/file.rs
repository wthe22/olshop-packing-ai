//! `categories.toml` as a file: loading and validating it, the condition positions its errors
//! name, the catch-all entry of a folder without one, and the pick an order falls in.

use std::collections::BTreeSet;

use regex::Regex;

use super::condition::{Category, Condition};
use super::error::RulesError;
use super::evaluate::evaluate;
use super::parser::parse_condition;

pub const RULES_FILE_NAME: &str = "categories.toml";
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

/// The one entry a data folder without `categories.toml` uses (07 › *Inputs*): no condition, so
/// it takes every order and a batch saves every order in one PDF set (03 › *Picks*; 08 ›
/// *5. Categories*, which shows it as the one catch-all category).
pub fn catch_all_category() -> Category {
    Category {
        code: "Z".to_string(),
        name: "Lainnya".to_string(),
        condition: None,
        when: None,
    }
}

/// Whether `categories` are the unconfigured picks: exactly one entry and it has no condition —
/// the [`catch_all_category`] a folder without `categories.toml` starts with (07 › *Inputs*).
/// The Categories and the Plan screen show the **No picks yet** line then (Q-UI-4).
pub fn unconfigured<'a>(categories: impl IntoIterator<Item = &'a Category>) -> bool {
    let mut entries = categories.into_iter();
    match (entries.next(), entries.next()) {
        (Some(only), None) => only.condition.is_none(),
        _ => false,
    }
}

/// Load the categories from the text of `categories.toml` (the app reads the file). Errors
/// carry the line/col inside the file, like the reference implementation.
pub fn load_rules(text: &str) -> Result<Vec<Category>, RulesError> {
    load_rules_named(text, RULES_FILE_NAME)
}

/// Load the categories from the text of a rules file whose name is `file` (04 › *Sets*): one set
/// each. [`load_rules`] uses the data folder's own `categories.toml`; a set's file is
/// `categories/<set name>.toml`, so its errors name that file. The rules are the same.
pub fn load_rules_named(text: &str, file: &str) -> Result<Vec<Category>, RulesError> {
    let table: toml::Table =
        toml::from_str(text).map_err(|e| RulesError::new(format!("{file}: {e}")))?;

    for key in table.keys() {
        if key != "category" {
            return Err(RulesError::new(format!("{file}: unknown key \"{key}\"")));
        }
    }

    let raw = match table.get("category") {
        None => return Err(RulesError::new(format!("{file}: no categories"))),
        Some(toml::Value::Array(entries)) => entries,
        Some(_) => {
            return Err(RulesError::new(format!(
                "{file}: \"category\" must be an array of tables"
            )));
        }
    };
    if raw.is_empty() {
        return Err(RulesError::new(format!("{file}: no categories")));
    }

    let mut categories: Vec<Category> = Vec::new();
    let mut seen: BTreeSet<String> = BTreeSet::new();
    let count = raw.len();
    for (index, entry) in raw.iter().enumerate() {
        let toml::Value::Table(entry) = entry else {
            return Err(RulesError::new(format!(
                "{file}: category #{}: must be a table",
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
                    "{file}: category {cid}: unknown key \"{key}\""
                )));
            }
        }

        let Some(code_value) = code_value else {
            return Err(RulesError::new(format!(
                "{file}: category {cid}: missing \"code\""
            )));
        };
        let toml::Value::String(code) = code_value else {
            return Err(RulesError::new(format!(
                "{file}: category {cid}: \"code\" must be a string"
            )));
        };
        if code.is_empty() {
            return Err(RulesError::new(format!(
                "{file}: category {cid}: \"code\" must not be empty"
            )));
        }
        if !valid_code(code) {
            return Err(RulesError::new(format!(
                "{file}: category {cid}: \"code\" must be 1-3 letters or digits"
            )));
        }
        if seen.contains(code) {
            return Err(RulesError::new(format!(
                "{file}: category {cid}: duplicate \"code\" \"{code}\""
            )));
        }
        seen.insert(code.clone());

        let Some(name_value) = entry.get("name") else {
            return Err(RulesError::new(format!(
                "{file}: category {cid}: missing \"name\""
            )));
        };
        let toml::Value::String(name) = name_value else {
            return Err(RulesError::new(format!(
                "{file}: category {cid}: \"name\" must be a string"
            )));
        };
        if name.is_empty() {
            return Err(RulesError::new(format!(
                "{file}: category {cid}: \"name\" must not be empty"
            )));
        }

        let when_text = match entry.get("when") {
            None => None,
            Some(toml::Value::String(s)) => Some(s.clone()),
            Some(_) => {
                return Err(RulesError::new(format!(
                    "{file}: category {cid}: \"when\" must be a string"
                )));
            }
        };

        let condition = match &when_text {
            None => {
                if index != count - 1 {
                    return Err(RulesError::new(format!(
                        "{file}: category {cid}: missing \"when\" on a category that is not last"
                    )));
                }
                None
            }
            Some(when) => Some(parse_when(when, text, file, index, &cid)?),
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
    file: &str,
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
                    "{file}, line {file_line}, col {file_col}: category {cid} (when): {rest}"
                )));
            }
            Err(RulesError::new(format!(
                "{file}: category {cid} (when): {}",
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
