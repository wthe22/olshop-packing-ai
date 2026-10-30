//! Reading `packing-workflows.toml` and every check of the design, each naming its line and
//! column in the file.

use std::ops::Range;

use regex::Regex;
use serde::Deserialize;
use toml::Spanned;

use super::{
    Classifier, Courier, Filter, GroupRule, SortRule, TrackingForm, Workflows, WorkflowsError,
};
use crate::ids::CourierCode;
use crate::record::SCHEMA;
use crate::rules::{Condition, TextOp, parse_filter};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawRoot {
    schema: Spanned<String>,
    #[serde(default)]
    couriers: Vec<RawCourier>,
    #[serde(default)]
    courier_classifier: Vec<RawClassifier>,
    #[serde(default)]
    not_courier: RawNotCourier,
    #[serde(default)]
    label_sort_rules: Vec<RawSortRule>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawCourier {
    code: Spanned<String>,
    name: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawClassifier {
    courier_code: Spanned<String>,
    #[serde(default)]
    logos: Vec<Spanned<String>>,
    #[serde(default)]
    shapes: Vec<Spanned<String>>,
    tracking_id: Option<Spanned<String>>,
    #[serde(default)]
    text: Vec<String>,
    #[serde(default)]
    instant_tags: Vec<String>,
}

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawNotCourier {
    #[serde(default)]
    logos: Vec<Spanned<String>>,
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

/// Read the file's text; the first failing check is the error.
pub fn load_workflows(text: &str) -> Result<Workflows, WorkflowsError> {
    let raw: RawRoot = toml::from_str(text).map_err(|error| {
        let at = error.span().map_or(0, |span| span.start);
        error_at(text, at, error.message().to_string())
    })?;
    let file = File { text };
    if raw.schema.get_ref() != SCHEMA {
        return Err(file.error(
            &raw.schema.span(),
            format!(
                "schema {} is not {SCHEMA}, the one this version reads",
                raw.schema.get_ref()
            ),
        ));
    }
    let couriers = file.couriers(&raw.couriers)?;
    let codes: Vec<&CourierCode> = couriers.iter().map(|courier| &courier.code).collect();
    let classifiers = file.classifiers(&raw.courier_classifier, &codes)?;
    let not_courier_logos = raw
        .not_courier
        .logos
        .iter()
        .map(|hash| file.hash(hash))
        .collect::<Result<_, _>>()?;
    let sort_rules = file.sort_rules(&raw.label_sort_rules, &codes)?;
    Ok(Workflows {
        couriers,
        classifiers,
        not_courier_logos,
        sort_rules,
    })
}

struct File<'a> {
    text: &'a str,
}

impl File<'_> {
    fn error(&self, span: &Range<usize>, message: String) -> WorkflowsError {
        error_at(self.text, span.start, message)
    }

    fn couriers(&self, raw: &[RawCourier]) -> Result<Vec<Courier>, WorkflowsError> {
        let mut couriers: Vec<Courier> = Vec::new();
        for entry in raw {
            let code = entry.code.get_ref();
            let valid = !code.is_empty()
                && code
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
            if !valid {
                return Err(self.error(
                    &entry.code.span(),
                    format!("courier code {code:?}: lower-case letters, digits and - only"),
                ));
            }
            if couriers.iter().any(|known| known.code.as_str() == code) {
                return Err(self.error(&entry.code.span(), format!("courier code {code:?} twice")));
            }
            couriers.push(Courier {
                code: CourierCode::new(code),
                name: entry.name.clone(),
            });
        }
        Ok(couriers)
    }

    fn classifiers(
        &self,
        raw: &[RawClassifier],
        codes: &[&CourierCode],
    ) -> Result<Vec<Classifier>, WorkflowsError> {
        let mut classifiers: Vec<Classifier> = Vec::new();
        for entry in raw {
            let code = entry.courier_code.get_ref();
            if !codes.iter().any(|known| known.as_str() == code) {
                return Err(self.error(
                    &entry.courier_code.span(),
                    format!("courier_code {code:?} names no courier"),
                ));
            }
            if classifiers
                .iter()
                .any(|known| known.courier.as_str() == code)
            {
                return Err(self.error(
                    &entry.courier_code.span(),
                    format!("a second courier_classifier for {code:?}"),
                ));
            }
            let tracking_id = match &entry.tracking_id {
                Some(form) => Some(self.tracking_form(form)?),
                None => None,
            };
            classifiers.push(Classifier {
                courier: CourierCode::new(code),
                logos: entry
                    .logos
                    .iter()
                    .map(|h| self.hash(h))
                    .collect::<Result<_, _>>()?,
                shapes: entry
                    .shapes
                    .iter()
                    .map(|h| self.hash(h))
                    .collect::<Result<_, _>>()?,
                tracking_id,
                text: entry.text.clone(),
                instant_tags: entry.instant_tags.clone(),
            });
        }
        Ok(classifiers)
    }

    fn hash(&self, hash: &Spanned<String>) -> Result<String, WorkflowsError> {
        let value = hash.get_ref();
        let valid = value.len() == 16
            && value
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b));
        if valid {
            Ok(value.clone())
        } else {
            Err(self.error(
                &hash.span(),
                format!("{value:?} is not a hash of 16 lower-case hex characters"),
            ))
        }
    }

    fn tracking_form(&self, form: &Spanned<String>) -> Result<TrackingForm, WorkflowsError> {
        let source = form.get_ref();
        let full = Regex::new(&format!("^(?:{source})$")).map_err(|error| {
            self.error(
                &form.span(),
                format!("tracking_id {source:?} is not a regex: {error}"),
            )
        })?;
        let unanchored =
            Regex::new(source).expect("a pattern that compiles anchored compiles bare");
        Ok(TrackingForm {
            source: source.clone(),
            full,
            unanchored,
        })
    }

    fn sort_rules(
        &self,
        raw: &[RawSortRule],
        codes: &[&CourierCode],
    ) -> Result<Vec<SortRule>, WorkflowsError> {
        if raw.is_empty() {
            return Err(error_at(self.text, 0, "no sort rule".to_string()));
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
                line,
                col,
                message: message.to_string(),
            }
        })?;
        if let Some(value) = unknown_courier(&condition, codes) {
            return Err(error_at(
                self.text,
                start,
                format!("courier {value:?} names no courier"),
            ));
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

fn error_at(text: &str, offset: usize, message: String) -> WorkflowsError {
    let (line, col) = position(text, offset);
    WorkflowsError { line, col, message }
}

/// The 1-based line and column of a byte offset.
fn position(text: &str, offset: usize) -> (usize, usize) {
    let before = &text[..offset.min(text.len())];
    let line = before.matches('\n').count() + 1;
    let line_start = before.rfind('\n').map_or(0, |at| at + 1);
    (line, before[line_start..].chars().count() + 1)
}
