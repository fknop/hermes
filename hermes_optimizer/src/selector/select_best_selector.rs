use crate::solver::{score::Score, solution::Solution};

use super::select_solution::SelectSolution;

pub struct SelectBestSelector;

impl SelectSolution for SelectBestSelector {
    fn select_solution<'a>(&self, solutions: &'a [Solution]) -> Option<&'a Solution> {
        // TODO
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
