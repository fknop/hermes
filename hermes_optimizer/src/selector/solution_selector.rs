use crate::solver::accepted_solution::AcceptedSolution;

use super::{
    select_best_selector::SelectBestSelector, select_random_selector::SelectRandomSelector,
    select_solution::SelectSolution,
};

pub enum SolutionSelector {
    SelectBest(SelectBestSelector),
    SelectRandom(SelectRandomSelector),
}

impl SelectSolution for SolutionSelector {
    fn select_solution<'r, 'a>(
        &self,
        solutions: &'r [AcceptedSolution<'a>],
    ) -> Option<&'r AcceptedSolution<'a>> {
        match self {
            SolutionSelector::SelectBest(selector) => selector.select_solution(solutions),
            SolutionSelector::SelectRandom(selector) => selector.select_solution(solutions),
        }
    }
}
