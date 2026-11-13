//! The two files of the data folder (rules-file/README.md): `parser.toml`, the couriers, the clues
//! that name them and the images that are not couriers, and `config.toml`, the accounts and the
//! sort rules. Both are the owner's; a bundled copy is written when one is missing.

mod accounts;
mod config_file;
mod edit;
mod load;
mod parser_file;
mod remember;

use std::fmt;

use regex::Regex;

use crate::accounts::Account;
use crate::ids::CourierCode;
use crate::label::Form;
use crate::record::{LabelGroupRule, LabelSortRule};
use crate::rules::Condition;

pub use self::accounts::write_accounts;
pub use self::edit::{
    EditError, GroupEdit, NewRule, add_rule, delete_rule, rename_rule, set_rule_enabled,
    write_groups,
};
pub use self::load::load_workflows;
pub use self::remember::{LogoIs, remember_logo};

pub const PARSER_FILE: &str = "parser.toml";
pub const CONFIG_FILE: &str = "config.toml";

/// The parser the program carries: the eight couriers and the channel strips that are not couriers.
pub const BUNDLED_PARSER: &str = include_str!("../../parser.toml");
/// The config the program carries: no account and one sort rule `default` with one group.
pub const BUNDLED_CONFIG: &str = include_str!("../../config.toml");

/// The bundled files, loaded.
pub fn load_bundled() -> Workflows {
    load_workflows(BUNDLED_PARSER, BUNDLED_CONFIG).expect("the bundled files load")
}

#[derive(Clone, Debug)]
pub struct Workflows {
    pub couriers: Vec<Courier>,
    pub classifiers: Vec<Classifier>,
    /// Header images answered *Not a courier*, by their byte hash.
    pub not_courier_logos: Vec<String>,
    /// In the order they were made.
    pub accounts: Vec<Account>,
    pub sort_rules: Vec<SortRule>,
}

impl Workflows {
    pub fn courier(&self, code: &CourierCode) -> Option<&Courier> {
        self.couriers.iter().find(|courier| &courier.code == code)
    }

    pub fn sort_rule(&self, name: &str) -> Option<&SortRule> {
        self.sort_rules.iter().find(|rule| rule.name == name)
    }

    /// Whether the rules already know a header image by its byte hash: a courier's logo or an
    /// image answered *Not a courier*. Only an image they do not know needs its appearance read.
    pub fn knows_logo(&self, hash: &str) -> bool {
        self.not_courier_logos.iter().any(|known| known == hash)
            || self
                .classifiers
                .iter()
                .any(|classifier| classifier.logos.iter().any(|known| known == hash))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Courier {
    pub code: CourierCode,
    pub name: String,
}

/// The name of the courier with `code`, or the code when the rules no longer name it.
pub fn courier_name<'a>(couriers: &'a [Courier], code: &'a str) -> &'a str {
    couriers
        .iter()
        .find(|courier| courier.code.as_str() == code)
        .map_or(code, |courier| courier.name.as_str())
}

/// The clues that name one courier (courier.md).
#[derive(Clone, Debug)]
pub struct Classifier {
    pub courier: CourierCode,
    /// XXH3-64 of a logo's encoded bytes.
    pub logos: Vec<String>,
    /// XXH3-64 of a masked logo's 64 × 64 appearance.
    pub shapes: Vec<String>,
    pub tracking_id: Option<TrackingForm>,
    /// Printed text that stands in for the logo where no header image can be read.
    pub text: Vec<String>,
    /// The service tags of an instant-service label.
    pub instant_tags: Vec<String>,
}

/// A tracking-ID form: a token must match it fully.
#[derive(Clone, Debug)]
pub struct TrackingForm {
    pub source: String,
    pub full: Regex,
    /// For the whitespace-collapsed text of a spaced-out page.
    pub unanchored: Regex,
}

impl TrackingForm {
    pub fn matches(&self, token: &str) -> bool {
        self.full.is_match(token)
    }

    pub fn form(&self) -> Form<'_> {
        Form {
            source: &self.source,
            full: &self.full,
            unanchored: &self.unanchored,
        }
    }
}

/// A named list of label group rules; a batch is sorted by one.
#[derive(Clone, Debug)]
pub struct SortRule {
    pub name: String,
    /// Offered on New batch. A rule that is off is kept, and an amend still reads with it.
    pub enabled: bool,
    pub groups: Vec<GroupRule>,
}

impl SortRule {
    /// The rule as a saved batch keeps it.
    pub fn snapshot(&self) -> LabelSortRule {
        LabelSortRule {
            name: self.name.clone(),
            label_groups: self
                .groups
                .iter()
                .map(|group| LabelGroupRule {
                    code: group.code.clone(),
                    name: group.name.clone(),
                    filter: group.filter.as_ref().map(|filter| filter.text.clone()),
                })
                .collect(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct GroupRule {
    pub code: String,
    pub name: String,
    /// None: takes every order left; only the last group.
    pub filter: Option<Filter>,
}

/// A filter as written and as parsed.
#[derive(Clone, Debug)]
pub struct Filter {
    pub text: String,
    pub condition: Condition,
}

/// A check of `parser.toml` or `config.toml` that failed, at its line and column in the file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkflowsError {
    pub file: &'static str,
    pub line: usize,
    pub col: usize,
    pub message: String,
}

impl fmt::Display for WorkflowsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}, line {}, col {}: {}",
            self.file, self.line, self.col, self.message
        )
    }
}

impl std::error::Error for WorkflowsError {}

#[cfg(test)]
mod tests;
