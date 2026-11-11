//! The condition language: tokenizer, parser, evaluator, printer, and the boxes the window shows.
//!
//! The PC app reads no orders CSV, so the CSV-only fields are known but always an error
//! (07-pc-app "Fields the conditions can use"); the app scan-filter fields stay errors too.

mod condition;
mod error;
mod evaluate;
mod parser;
mod printer;
mod tokenizer;
mod tree;

pub use condition::{Condition, IS_EMPTY, NumberOp, TextOp};
pub use error::RulesError;
pub use evaluate::{FilterFields, FilterLine, evaluate, evaluate_filter, fields_used};
pub use parser::{parse_condition, parse_filter};
pub use printer::format_condition;
pub use tree::{Group, GroupKind, Node, Row, RowValue, Tree, filter_to_tree, tree_to_filter};
