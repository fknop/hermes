
use crate::solver::{
    accepted_solution::AcceptedSolution, solution::population::Population,
};

use super::select_solution::SelectSolution;

pub struct SelectBestSelector;

impl SelectSolution for SelectBestSelector {
    fn select_solution<'a>(
        &self,
        population: &'a Population,
        _: &mut impl rand::Rng,
    ) -> Option<&'a AcceptedSolution> {
        // Assumption that it's sorted
        population.solutions().first()
    }
}
