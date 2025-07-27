use rand::{rngs::SmallRng, seq::IndexedRandom};

use crate::solver::accepted_solution::AcceptedSolution;

use super::select_solution::SelectSolution;

pub struct SelectRandomSelector;

impl SelectSolution for SelectRandomSelector {
    fn select_solution<'a>(
        &self,
        solutions: &'a [AcceptedSolution],
        rng: &mut SmallRng,
    ) -> Option<&'a AcceptedSolution> {
        solutions.choose(rng)
    }
}
