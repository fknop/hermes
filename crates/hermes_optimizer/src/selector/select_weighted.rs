use rand::seq::IndexedRandom;

use crate::solver::{accepted_solution::AcceptedSolution, solution::population::Population};

use super::select_solution::SelectSolution;

pub struct SelectWeightedSelector;

impl SelectSolution for SelectWeightedSelector {
    fn select_solution<'a>(
        &self,
        population: &'a Population,
        rng: &mut impl rand::Rng,
    ) -> Option<&'a AcceptedSolution> {
        if population.is_empty() {
            return None; // No solutions to select from
        }

        let solutions = population.solutions();

        let weights = (0..solutions.len()).collect::<Vec<_>>();
        let index = weights
            .choose_weighted(rng, |index| {
                2.0 - population.biased_fitness(&solutions[*index])
                // 1.3_f64.powf((solutions.len() - 1 - index) as f64)
            })
            .ok();

        if let Some(selected_index) = index {
            Some(&solutions[*selected_index])
        } else {
            None // In case of an error in weighted selection
        }
    }
}
