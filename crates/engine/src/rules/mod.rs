//! `categories.toml` and the condition language (04-rules-file): loading and validation, the
//! condition tokenizer/parser, the evaluator (item-level "any line" semantics) and the printer.
//!
//! The PC app reads no orders CSV, so the CSV-only fields are known but always an error
//! (07-pc-app "Fields the conditions can use"); the app scan-filter fields stay errors too.

mod condition;
mod error;
mod evaluate;
mod file;
mod parser;
mod printer;
mod tokenizer;

pub use condition::{Category, Condition, IS_EMPTY, NumberOp, TextOp};
pub use error::RulesError;
pub use evaluate::{FilterFields, FilterLine, evaluate, evaluate_filter, fields_used};
pub use file::{
    RULES_FILE_NAME, catch_all_category, categorize, load_rules, load_rules_named, unconfigured,
};
pub use parser::{parse_condition, parse_filter};
pub use printer::format_condition;
