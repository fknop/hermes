use rand::rngs::SmallRng;

use crate::solver::{insertion_context::InsertionContext, noise::NoiseGenerator};

use super::constraint::Constraint;

pub fn compute_insertion_score(
    constraints: &[Constraint],
    context: &InsertionContext,
    noise_generator: &NoiseGenerator,
    rng: &mut SmallRng,
) -> crate::solver::score::Score {
    constraints
        .iter()
        .map(|constraint| constraint.compute_insertion_score(context, noise_generator, rng))
        .sum()
}
