use crate::solution::{Cost, Solution};

use super::solution_selector::SolutionSelector;

pub struct SelectBestSelector;

impl SolutionSelector for SelectBestSelector {
    fn accept_solution(solutions: &[Solution]) -> Option<&Solution> {
        let mut min_cost = Cost::MAX;
        let mut best_solution = None;

        for solution in solutions {
            if solution.get_cost() < min_cost {
                best_solution = Some(solution);
                min_cost = solution.get_cost();
            }
        }

        best_solution
    }
}
