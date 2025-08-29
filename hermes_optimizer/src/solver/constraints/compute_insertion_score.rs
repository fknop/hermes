use crate::solver::{insertion_context::InsertionContext, noise::NoiseGenerator};

use super::constraint::Constraint;

pub fn compute_insertion_score(
    constraints: &[Constraint],
    context: &InsertionContext,
) -> crate::solver::score::Score {
    constraints
        .iter()
        .map(|constraint| constraint.compute_insertion_score(context))
        .sum()
}
