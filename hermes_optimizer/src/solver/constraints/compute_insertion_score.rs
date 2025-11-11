use crate::solver::{insertion_context::InsertionContext, score::Score};

use super::constraint::Constraint;

pub fn compute_insertion_score(constraints: &[Constraint], context: &InsertionContext) -> Score {
    constraints
        .iter()
        .map(|constraint| constraint.compute_insertion_score(context))
        .sum()
}
