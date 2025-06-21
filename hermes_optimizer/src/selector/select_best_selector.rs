use crate::solver::{score::Score, solution::Solution};

use super::solution_selector::SolutionSelector;

pub struct SelectBestSelector;

impl SolutionSelector for SelectBestSelector {
    fn select_solution<'a>(&self, solutions: &'a [Solution]) -> Option<&'a Solution> {
        let mut max_score = Score::MAX;
        let mut best_solution = None;

        for solution in solutions {
            if solution.score < max_score {
                best_solution = Some(solution);
                max_score = solution.score;
            }
        }

        best_solution
    }
}
