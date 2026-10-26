use std::fmt;

use jiff::civil::Date;
use packing_engine::copies::CopiesStop;
use packing_engine::ids::OrderId;
use packing_engine::repeats::ChoiceError;
use packing_engine::rules::RulesError;
use packing_engine::sorting::FileNameError;
use packing_pdf::PdfError;
use packing_store::StoreError;

/// Why a batch step stopped. Nothing of the date changed, except where a variant says so.
#[derive(Debug)]
pub enum BatchError {
    Pdf(PdfError),
    Store(StoreError),
    /// A page that cannot be placed.
    Stop(CopiesStop),
    Choice(ChoiceError),
    AllSavedEarlier {
        orders: usize,
    },
    /// A chosen file's bytes are not the ones that were read.
    FileChanged {
        file: String,
    },
    FileName(FileNameError),
    /// The date has no live batch to revert or amend.
    NoBatch,
    /// The draft amends a batch that is no longer the newest live one.
    NotNewest,
    /// A filter of the sort rule an amend takes from the batch's record does not parse.
    Rule(RulesError),
    /// A reprint names an order no live batch of that date holds.
    NotSaved {
        order_id: OrderId,
        date: Date,
    },
    /// An input a reprint needs is missing or not the bytes the journal names.
    InputUnusable {
        file: String,
        date: Date,
    },
    /// A restore names a file that is no output of a live batch.
    NoOutput {
        file: String,
    },
    TooManyReprints,
}

impl fmt::Display for BatchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BatchError::Pdf(error) => error.fmt(f),
            BatchError::Store(error) => error.fmt(f),
            BatchError::Stop(stop) => stop.fmt(f),
            BatchError::Choice(error) => error.fmt(f),
            BatchError::AllSavedEarlier { orders } => {
                write!(f, "All {orders} orders were already saved today.")
            }
            BatchError::FileChanged { file } => write!(
                f,
                "{file} changed after it was read. Read the labels again."
            ),
            BatchError::FileName(error) => error.fmt(f),
            BatchError::NoBatch => f.write_str("This date has no saved batch."),
            BatchError::NotNewest => {
                f.write_str("The batch being amended is no longer the newest one.")
            }
            BatchError::Rule(error) => write!(f, "the batch's sort rule: {error}"),
            BatchError::NotSaved { order_id, date } => {
                write!(f, "Order {order_id} is not saved on {date}.")
            }
            BatchError::InputUnusable { file, date } => write!(
                f,
                "{file}, an input of {date}, is missing or has changed. It cannot be printed again."
            ),
            BatchError::NoOutput { file } => {
                write!(f, "{file} is not a file of a saved batch of this date.")
            }
            BatchError::TooManyReprints => f.write_str("This date has too many reprints."),
        }
    }
}

impl std::error::Error for BatchError {}

impl From<PdfError> for BatchError {
    fn from(error: PdfError) -> Self {
        BatchError::Pdf(error)
    }
}

impl From<StoreError> for BatchError {
    fn from(error: StoreError) -> Self {
        BatchError::Store(error)
    }
}

impl From<CopiesStop> for BatchError {
    fn from(stop: CopiesStop) -> Self {
        BatchError::Stop(stop)
    }
}

impl From<ChoiceError> for BatchError {
    fn from(error: ChoiceError) -> Self {
        BatchError::Choice(error)
    }
}
