use rand::seq::IndexedRandom;

use crate::solver::{accepted_solution::AcceptedSolution, solution::population::Population};

use super::select_solution::SelectSolution;

pub struct SelectRandomSelector;

impl SelectSolution for SelectRandomSelector {
    fn select_solution<'a>(
        &self,
        population: &'a Population,
        rng: &mut impl rand::Rng,
    ) -> Option<&'a AcceptedSolution> {
        population.solutions().choose(rng)
    }
}
