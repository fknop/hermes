use rand::{
    rngs::SmallRng,
    seq::{IndexedRandom, IteratorRandom},
};

use crate::solver::{accepted_solution::AcceptedSolution, solution::population::Population};

use super::select_solution::SelectSolution;

pub struct BinaryTournamentSelector;

impl SelectSolution for BinaryTournamentSelector {
    fn select_solution<'a>(
        &self,
        population: &'a Population,
        rng: &mut impl rand::Rng,
    ) -> Option<&'a AcceptedSolution> {
        if population.is_empty() {
            return None; // No solutions to select from
        }

        let solutions = population.solutions();
        if solutions.len() == 1 {
            return Some(&solutions[0]);
        }

        let solutions = solutions.iter().choose_multiple(rng, 2);
        let first = solutions[0];
        let second = solutions[1];

        if population.biased_fitness(first) < population.biased_fitness(second) {
            Some(first)
        } else {
            Some(second)
        }
    }
}
