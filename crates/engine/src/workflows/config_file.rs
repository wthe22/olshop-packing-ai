//! Reading `config.toml` (rules-file/config.md): the accounts and the sort rules. A sort rule's
//! courier filters are checked against the couriers of `parser.toml`.

use std::ops::Range;

use serde::Deserialize;
use toml::Spanned;

use super::load::{File, position};
use super::{Filter, GroupRule, SortRule, WorkflowsError};
use crate::accounts::{Account, check_account_name, check_shop_name};
use crate::ids::CourierCode;
use crate::rules::{Condition, TextOp, parse_filter};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawConfig {
    schema: Spanned<String>,
    #[serde(default)]
    accounts: Vec<RawAccount>,
    #[serde(default)]
    label_sort_rules: Vec<RawSortRule>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawAccount {
    name: Spanned<String>,
    #[serde(default)]
    shops: Vec<Spanned<String>>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawSortRule {
    name: Spanned<String>,
    #[serde(default = "on")]
    enabled: bool,
    label_groups: Vec<RawGroup>,
}

fn on() -> bool {
    true
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawGroup {
    code: Spanned<String>,
    name: Spanned<String>,
    filter: Option<Spanned<String>>,
}

/// What `config.toml` holds, every check passed.
pub(super) struct Config {
    pub(super) accounts: Vec<Account>,
    pub(super) sort_rules: Vec<SortRule>,
}

pub(super) fn read(file: &File, codes: &[&CourierCode]) -> Result<Config, WorkflowsError> {
    let raw: RawConfig = file.parse()?;
    file.check_schema(&raw.schema)?;
    Ok(Config {
        accounts: file.accounts(&raw.accounts)?,
        sort_rules: file.sort_rules(&raw.label_sort_rules, codes)?,
    })
}

impl File<'_> {
    fn accounts(&self, raw: &[RawAccount]) -> Result<Vec<Account>, WorkflowsError> {
        let mut accounts: Vec<Account> = Vec::new();
        for entry in raw {
            let name = entry.name.get_ref();
            check_account_name(name, &accounts, None).map_err(|error| {
                self.error(&entry.name.span(), format!("account {name:?}: {error}"))
            })?;
            let mut account = Account {
                name: name.clone(),
                shops: Vec::new(),
            };
            for shop in &entry.shops {
                let mut known = accounts.clone();
                known.push(account.clone());
                check_shop_name(shop.get_ref(), &known).map_err(|error| {
                    self.error(
                        &shop.span(),
                        format!("shop {:?} of account {name:?}: {error}", shop.get_ref()),
                    )
                })?;
                account.shops.push(shop.get_ref().trim().to_string());
            }
            accounts.push(account);
        }
        Ok(accounts)
    }

    fn sort_rules(
        &self,
        raw: &[RawSortRule],
        codes: &[&CourierCode],
    ) -> Result<Vec<SortRule>, WorkflowsError> {
        if raw.is_empty() {
            return Ok(vec![SortRule::built_in()]);
        }
        let mut rules: Vec<SortRule> = Vec::new();
        for entry in raw {
            let name = entry.name.get_ref();
            if name.trim().is_empty() || rules.iter().any(|rule| &rule.name == name) {
                return Err(self.error(
                    &entry.name.span(),
                    format!("sort rule {name:?}: a name of its own"),
                ));
            }
            if entry.label_groups.is_empty() {
                return Err(self.error(
                    &entry.name.span(),
                    format!("sort rule {name:?} has no label group"),
                ));
            }
            let mut groups = Vec::new();
            for (index, group) in entry.label_groups.iter().enumerate() {
                let last = index + 1 == entry.label_groups.len();
                groups.push(self.group(group, &groups, last, codes)?);
            }
            rules.push(SortRule {
                name: name.clone(),
                enabled: entry.enabled,
                groups,
            });
        }
        if !rules.iter().any(|rule| rule.enabled) {
            return Err(self.error(
                &raw[0].name.span(),
                "every sort rule is off: a new batch needs one that is on".to_string(),
            ));
        }
        Ok(rules)
    }

    fn group(
        &self,
        raw: &RawGroup,
        before: &[GroupRule],
        last: bool,
        codes: &[&CourierCode],
    ) -> Result<GroupRule, WorkflowsError> {
        let code = raw.code.get_ref();
        let chars = code.chars().count();
        if !(1..=3).contains(&chars) || !code.chars().all(|c| c.is_ascii_alphanumeric()) {
            return Err(self.error(
                &raw.code.span(),
                format!("group code {code:?}: 1 to 3 letters or digits"),
            ));
        }
        if before
            .iter()
            .any(|group| group.code.eq_ignore_ascii_case(code))
        {
            return Err(self.error(&raw.code.span(), format!("group code {code:?} twice")));
        }
        let name = raw.name.get_ref();
        if name.trim().is_empty()
            || before
                .iter()
                .any(|group| group.name.eq_ignore_ascii_case(name))
        {
            return Err(self.error(
                &raw.name.span(),
                format!("group name {name:?}: a name of its own in the rule"),
            ));
        }
        let filter = match &raw.filter {
            Some(filter) => Some(self.filter(filter, codes)?),
            None if last => None,
            None => {
                return Err(self.error(
                    &raw.code.span(),
                    format!("group {code} has no filter but is not the last"),
                ));
            }
        };
        Ok(GroupRule {
            code: code.clone(),
            name: name.clone(),
            filter,
        })
    }

    fn filter(
        &self,
        raw: &Spanned<String>,
        codes: &[&CourierCode],
    ) -> Result<Filter, WorkflowsError> {
        let text = raw.get_ref();
        let start = content_start(self.text, raw.span());
        let condition = parse_filter(text).map_err(|error| {
            let (line, col, message) = split_position(&error.message);
            let (base_line, base_col) = position(self.text, start);
            let (line, col) = if line == 1 {
                (base_line, base_col + col - 1)
            } else {
                (base_line + line - 1, col)
            };
            WorkflowsError {
                file: self.name,
                line,
                col,
                message: message.to_string(),
            }
        })?;
        if let Some(value) = unknown_courier(&condition, codes) {
            return Err(self.error_at(start, format!("courier {value:?} names no courier")));
        }
        Ok(Filter {
            text: text.clone(),
            condition,
        })
    }
}

/// A `courier equals` value that is no courier's code.
fn unknown_courier<'a>(condition: &'a Condition, codes: &[&CourierCode]) -> Option<&'a str> {
    match condition {
        Condition::Text {
            field,
            op: TextOp::Equals,
            value,
        } if field == "courier" => (!codes
            .iter()
            .any(|code| code.as_str().eq_ignore_ascii_case(value.trim())))
        .then_some(value.as_str()),
        Condition::Not(inner) => unknown_courier(inner, codes),
        Condition::And(all) | Condition::Or(all) => {
            all.iter().find_map(|inner| unknown_courier(inner, codes))
        }
        _ => None,
    }
}

/// Where a string's content starts: after its opening quote, or after `'''` and the line break
/// a multi-line string may open with.
fn content_start(text: &str, span: Range<usize>) -> usize {
    let token = &text[span.clone()];
    if token.starts_with("'''") || token.starts_with("\"\"\"") {
        let after = span.start + 3;
        match &text[after..] {
            rest if rest.starts_with("\r\n") => after + 2,
            rest if rest.starts_with('\n') => after + 1,
            _ => after,
        }
    } else {
        span.start + 1
    }
}

/// `line L, col C: message` of a filter error, as its three parts.
fn split_position(message: &str) -> (usize, usize, &str) {
    let parsed = message.strip_prefix("line ").and_then(|rest| {
        let (line, rest) = rest.split_once(", col ")?;
        let (col, text) = rest.split_once(": ")?;
        Some((line.parse().ok()?, col.parse().ok()?, text))
    });
    parsed.unwrap_or((1, 1, message))
}
