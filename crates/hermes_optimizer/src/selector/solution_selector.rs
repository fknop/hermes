use crate::{
    selector::select_binary_tournament::BinaryTournamentSelector,
    solver::{accepted_solution::AcceptedSolution, solution::population::Population},
};

use super::{
    select_best_selector::SelectBestSelector, select_random_selector::SelectRandomSelector,
    select_solution::SelectSolution, select_weighted::SelectWeightedSelector,
};

pub enum SolutionSelector {
    SelectBest(SelectBestSelector),
    SelectRandom(SelectRandomSelector),
    SelectWeighted(SelectWeightedSelector),
    BinaryTournament(BinaryTournamentSelector),
}

impl SelectSolution for SolutionSelector {
    fn select_solution<'a>(
        &self,
        population: &'a Population,
        rng: &mut impl rand::Rng,
    ) -> Option<&'a AcceptedSolution> {
        match self {
            SolutionSelector::SelectBest(selector) => selector.select_solution(population, rng),
            SolutionSelector::SelectRandom(selector) => selector.select_solution(population, rng),
            SolutionSelector::SelectWeighted(selector) => selector.select_solution(population, rng),
            SolutionSelector::BinaryTournament(selector) => {
                selector.select_solution(population, rng)
            }
        }
    }
}
