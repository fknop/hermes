use crate::solver::{score::Score, solution::Solution, working_solution::WorkingSolution};

use super::accept_solution::AcceptSolution;

pub struct GreedySolutionAcceptor;

impl AcceptSolution for GreedySolutionAcceptor {
    fn accept(
        &self,
        current_solutions: &[Solution],
        solution: &WorkingSolution,
        score: &Score,
    ) -> bool {
        if current_solutions.is_empty() {
            return true; // Accept the first solution
        }

        // Check if the new solution has a better score than the worst current solution
        let worst_current_solution = current_solutions.iter().max_by_key(|s| s.score);
        if let Some(worst) = worst_current_solution {
            return *score < worst.score;
        }

        false // If no current solutions, do not accept
    }
}
