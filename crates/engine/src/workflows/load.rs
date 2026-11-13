//! Reading `parser.toml` and `config.toml` and every check of the design, each naming its file,
//! line and column.

use std::ops::Range;

use serde::de::DeserializeOwned;
use toml::Spanned;

use super::{Workflows, WorkflowsError, config_file, parser_file};
use crate::record::SCHEMA;

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

    pub(super) fn check_schema(&self, schema: &Spanned<String>) -> Result<(), WorkflowsError> {
        if schema.get_ref() == SCHEMA {
            return Ok(());
        }
        Err(self.error(
            &schema.span(),
            format!(
                "schema {} is not {SCHEMA}, the one this version reads",
                schema.get_ref()
            ),
        ))
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

/// The 1-based line and column of a byte offset.
pub(super) fn position(text: &str, offset: usize) -> (usize, usize) {
    let before = &text[..offset.min(text.len())];
    let line = before.matches('\n').count() + 1;
    let line_start = before.rfind('\n').map_or(0, |at| at + 1);
    (line, before[line_start..].chars().count() + 1)
}
