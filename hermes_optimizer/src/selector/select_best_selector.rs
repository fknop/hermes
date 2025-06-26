use crate::solver::{accepted_solution::AcceptedSolution, score::Score};

use super::select_solution::SelectSolution;

pub struct SelectBestSelector;

impl SelectSolution for SelectBestSelector {
    fn select_solution<'r, 'a>(
        &self,
        solutions: &'r [AcceptedSolution<'a>],
    ) -> Option<&'r AcceptedSolution<'a>> {
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
