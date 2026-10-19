//! The error a rules file or a condition text fails with.

use std::fmt;

/// Invalid rules file or condition; the message names the place (`categories.toml`, line, col).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RulesError {
    pub message: String,
}

impl RulesError {
    pub(super) fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for RulesError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for RulesError {}
