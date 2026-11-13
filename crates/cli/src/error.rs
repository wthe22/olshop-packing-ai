use std::fmt;

use packing_batch::BatchError;
use packing_pdf::PdfError;
use packing_store::StoreError;

/// A failure that ends the command with exit code 1 and one message.
#[derive(Debug)]
pub enum CliError {
    Usage(String),
    /// `packing-app.toml`, `parser.toml` or `config.toml` cannot be used.
    Program(String),
    /// Orders whose shop no account lists, and no `--account` to put them under.
    Accounts(String),
    /// The verifier found something and `--save-anyway` was not given.
    Findings(String),
    Batch(BatchError),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliError::Usage(message)
            | CliError::Program(message)
            | CliError::Accounts(message)
            | CliError::Findings(message) => f.write_str(message),
            CliError::Batch(error) => error.fmt(f),
        }
    }
}

impl From<BatchError> for CliError {
    fn from(error: BatchError) -> Self {
        CliError::Batch(error)
    }
}

impl From<StoreError> for CliError {
    fn from(error: StoreError) -> Self {
        CliError::Batch(error.into())
    }
}

impl From<PdfError> for CliError {
    fn from(error: PdfError) -> Self {
        CliError::Batch(error.into())
    }
}
