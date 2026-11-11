//! Packing engine: the reading and sorting rules. Takes text runs and returns decisions, so
//! every rule is tested without PDF files.

pub mod account_questions;
pub mod accounts;
pub mod batch_record;
pub mod copies;
pub mod ids;
pub mod label;
pub mod orders;
pub mod record;
pub mod repeats;
pub mod rules;

pub mod slip;
pub mod sorting;
pub mod text;
pub mod workflows;

#[cfg(test)]
mod tests {
    #[test]
    fn placeholder() {
        assert_eq!(1 + 1, 2);
    }
}
