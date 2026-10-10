//! Packing engine: the reading and sorting rules. Takes text runs and returns decisions, so
//! every rule is tested without PDF files.

pub mod day;
pub mod label;
pub mod names;
pub mod orders;
pub mod plan;
pub mod rules;
pub mod slip;
pub mod text;

#[cfg(test)]
mod tests {
    #[test]
    fn placeholder() {
        assert_eq!(1 + 1, 2);
    }
}
