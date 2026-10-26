//! `packing-workflows.toml` (rules-file/README.md): the couriers, the clues that name them, the
//! images that are not couriers, and the sort rules. One file beside the program, the owner's;
//! the bundled copy is written when it is missing.

mod edit;
mod load;
mod remember;

use std::fmt;

use regex::Regex;

use crate::ids::CourierCode;
use crate::label::Form;
use crate::record::{LabelGroupRule, LabelSortRule};
use crate::rules::Condition;

pub use self::edit::{
    EditError, GroupEdit, NewRule, add_rule, delete_rule, rename_rule, write_groups,
};
pub use self::load::load_workflows;
pub use self::remember::{LogoIs, remember_logo};

pub const WORKFLOWS_FILE: &str = "packing-workflows.toml";

/// The copy the program carries: the six couriers and one rule `default` with one group.
pub const BUNDLED_WORKFLOWS: &str = include_str!("../../packing-workflows.toml");

#[derive(Clone, Debug)]
pub struct Workflows {
    pub couriers: Vec<Courier>,
    pub classifiers: Vec<Classifier>,
    /// Header images answered *Not a courier*, by their byte hash.
    pub not_courier_logos: Vec<String>,
    pub sort_rules: Vec<SortRule>,
}

impl Workflows {
    pub fn courier(&self, code: &CourierCode) -> Option<&Courier> {
        self.couriers.iter().find(|courier| &courier.code == code)
    }

    pub fn sort_rule(&self, name: &str) -> Option<&SortRule> {
        self.sort_rules.iter().find(|rule| rule.name == name)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Courier {
    pub code: CourierCode,
    pub name: String,
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

/// A check of `packing-workflows.toml` that failed, at its line and column in the file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkflowsError {
    pub line: usize,
    pub col: usize,
    pub message: String,
}

impl fmt::Display for WorkflowsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{WORKFLOWS_FILE}, line {}, col {}: {}",
            self.line, self.col, self.message
        )
    }
}

impl std::error::Error for WorkflowsError {}

#[cfg(test)]
mod tests;
