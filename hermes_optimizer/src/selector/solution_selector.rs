use crate::solver::solution::Solution;

use super::{
    select_best_selector::SelectBestSelector, select_random_selector::SelectRandomSelector,
    select_solution::SelectSolution,
};

pub enum SolutionSelector {
    SelectBest(SelectBestSelector),
    SelectRandom(SelectRandomSelector),
}

impl SelectSolution for SolutionSelector {
    fn select_solution<'a>(&self, solutions: &'a [Solution]) -> Option<&'a Solution> {
        match self {
            SolutionSelector::SelectBest(selector) => selector.select_solution(solutions),
            SolutionSelector::SelectRandom(selector) => selector.select_solution(solutions),
        }
    }
}
