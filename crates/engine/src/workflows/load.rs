//! Reading `parser.toml` and `config.toml` and every check of the design, each naming its file,
//! line and column.

use std::ops::Range;

use serde::de::DeserializeOwned;
use toml::Spanned;

use super::{Workflows, WorkflowsError, config_file, parser_file};
use crate::record::SCHEMA;
use crate::schema::{Schema, SchemaFit};
use crate::unknown::UnknownKey;

/// One file's text and name, for the errors its checks raise.
pub(super) struct File<'a> {
    pub(super) name: &'static str,
    pub(super) text: &'a str,
}

impl File<'_> {
    pub(super) fn error(&self, span: &Range<usize>, message: String) -> WorkflowsError {
        self.error_at(span.start, message)
    }

    pub(super) fn error_at(&self, offset: usize, message: String) -> WorkflowsError {
        let (line, col) = position(self.text, offset);
        WorkflowsError {
            file: self.name,
            line,
            col,
            message,
        }
    }

    /// The text as TOML of the shape `T`.
    pub(super) fn parse<T: DeserializeOwned>(&self) -> Result<T, WorkflowsError> {
        toml::from_str(self.text).map_err(|error| {
            let at = error.span().map_or(0, |span| span.start);
            self.error_at(at, error.message().to_string())
        })
    }

    /// Every version of this copy's major reads: a key it does not know is passed over.
    pub(super) fn check_schema(&self, schema: &Spanned<String>) -> Result<(), WorkflowsError> {
        let found = schema.get_ref();
        let message = match Schema::parse(found).map(|version| SCHEMA.fit(version)) {
            Some(SchemaFit::Reads) => return Ok(()),
            Some(SchemaFit::Newer) => format!(
                "schema {found} is of a newer format: this copy reads {}. Update this copy",
                SCHEMA.family()
            ),
            Some(SchemaFit::Older) => format!(
                "schema {found} is of an older format: this copy reads {}",
                SCHEMA.family()
            ),
            None => format!("schema {found:?} is not a version (X.Y)"),
        };
        Err(self.error(&schema.span(), message))
    }
}

/// Read the two files' texts; the first failing check is the error.
pub fn load_workflows(parser: &str, config: &str) -> Result<Workflows, WorkflowsError> {
    let parser = parser_file::read(&File {
        name: super::PARSER_FILE,
        text: parser,
    })?;
    let codes: Vec<_> = parser
        .couriers
        .iter()
        .map(|courier| &courier.code)
        .collect();
    let config = config_file::read(
        &File {
            name: super::CONFIG_FILE,
            text: config,
        },
        &codes,
    )?;
    Ok(Workflows {
        couriers: parser.couriers,
        classifiers: parser.classifiers,
        not_courier_logos: parser.not_courier_logos,
        accounts: config.accounts,
        sort_rules: config.sort_rules,
    })
}

/// The keys of the two files this version does not read, each with its file: they are kept, and
/// named for the owner to look at. A file of a newer version than this copy writes is expected to
/// hold keys it does not know and names none.
pub fn unknown_rules_keys(parser: &str, config: &str) -> Vec<(&'static str, UnknownKey)> {
    let named = |name: &'static str, text: &str, keys: fn(&str) -> Vec<UnknownKey>| {
        let keys = if of_a_newer_version(text) {
            Vec::new()
        } else {
            keys(text)
        };
        keys.into_iter().map(move |key| (name, key))
    };
    named(super::PARSER_FILE, parser, parser_file::unknown_keys)
        .chain(named(super::CONFIG_FILE, config, config_file::unknown_keys))
        .collect()
}

/// Whether a TOML text names a `schema` newer than this copy writes.
pub fn of_a_newer_version(text: &str) -> bool {
    #[derive(serde::Deserialize)]
    struct Version {
        schema: String,
    }
    toml::from_str::<Version>(text)
        .ok()
        .and_then(|found| Schema::parse(&found.schema))
        .is_some_and(|found| found > SCHEMA)
}

/// The 1-based line and column of a byte offset.
pub(super) fn position(text: &str, offset: usize) -> (usize, usize) {
    let before = &text[..offset.min(text.len())];
    let line = before.matches('\n').count() + 1;
    let line_start = before.rfind('\n').map_or(0, |at| at + 1);
    (line, before[line_start..].chars().count() + 1)
}
