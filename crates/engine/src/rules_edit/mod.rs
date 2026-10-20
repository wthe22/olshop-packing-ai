//! Categories editor support: condition text <-> boxes tree, the live counts of the open batch,
//! and saving `categories.toml` keeping its comments and unchanged condition texts (08 › 5.
//! Categories).
//!
//! The boxes view (08 › 5) is a group of rows. A group is *All of these* (`and`) or *Any of
//! these* (`or`) and holds rows and other groups; a row is one comparison `field · operator ·
//! value`. A `not` flag sits on rows and groups. The tree is built with the engine's own parser
//! and turned back into text by its printer (04 › *How the app edits conditions*), so the window
//! has no second parser.
//!
//! `RulesDocument` edits the file through `toml_edit`, so comments and the original text of the
//! entries that were not changed survive a save (08 › 5: "comments kept", "a category not
//! changed keeps its text exactly as in the file, line breaks included").

use crate::rules::RulesError;

mod counts;
mod document;
mod tree;

pub use self::counts::counts;
pub use self::document::{Entry, RulesDocument};
pub use self::tree::{
    Group, GroupKind, Node, Row, RowValue, Tree, condition_to_tree, tree_to_condition,
};

/// A small `RulesError` (the field is public, the constructor in `rules` is not).
fn error(message: impl Into<String>) -> RulesError {
    RulesError {
        message: message.into(),
    }
}
