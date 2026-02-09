use rand::rngs::SmallRng;

use crate::{
    selector::select_binary_tournament::BinaryTournamentSelector,
    solver::accepted_solution::AcceptedSolution,
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
        solutions: &'a [AcceptedSolution],
        rng: &mut impl rand::Rng,
    ) -> Option<&'a AcceptedSolution> {
        match self {
            SolutionSelector::SelectBest(selector) => selector.select_solution(solutions, rng),
            SolutionSelector::SelectRandom(selector) => selector.select_solution(solutions, rng),
            SolutionSelector::SelectWeighted(selector) => selector.select_solution(solutions, rng),
            SolutionSelector::BinaryTournament(selector) => {
                selector.select_solution(solutions, rng)
            }
        }
    }
}
