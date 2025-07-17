use crate::solver::{
    accepted_solution::AcceptedSolution, score::Score, working_solution::WorkingSolution,
};

use super::accept_solution::{AcceptSolution, AcceptSolutionContext};

pub struct GreedySolutionAcceptor;

impl AcceptSolution for GreedySolutionAcceptor {
    fn accept(
        &self,
        current_solutions: &[AcceptedSolution],
        _: &WorkingSolution,
        score: &Score,
        context: AcceptSolutionContext,
    ) -> bool {
        if current_solutions.len() < context.max_solutions {
            return true; // Accept the first solution
        }

        // Check if the new solution has a better score than the worst current solution
        let worst_current_solution = current_solutions.iter().max_by_key(|s| s.score);
        if let Some(worst) = worst_current_solution {
            return *score < worst.score;
        }

        false
    }
}
