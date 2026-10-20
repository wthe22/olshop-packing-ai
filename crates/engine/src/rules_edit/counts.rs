//! The live counts of the open batch: how many orders each category takes, for the Categories
//! screen's *In batch `<n>`*.

use crate::orders::Order;
use crate::rules::{Category, evaluate};

/// The orders each category takes in the open batch, in file order from what is left.
/// The same first-match, take-from-what-is-left rule as `plan.rs`: a category
/// without a condition takes every order still left (the rest).
pub fn counts(categories: &[Category], orders: &[Order]) -> Vec<usize> {
    let mut remaining: Vec<&Order> = orders.iter().collect();
    let mut result = Vec::with_capacity(categories.len());
    for category in categories {
        let mut left = Vec::with_capacity(remaining.len());
        let mut taken = 0usize;
        for order in &remaining {
            let matches = match &category.condition {
                None => true,
                Some(condition) => evaluate(condition, order),
            };
            if matches {
                taken += 1;
            } else {
                left.push(*order);
            }
        }
        result.push(taken);
        remaining = left;
    }
    result
}
