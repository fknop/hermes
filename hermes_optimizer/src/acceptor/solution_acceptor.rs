use crate::solver::{score::Score, solution::Solution, working_solution::WorkingSolution};

use super::{accept_solution::AcceptSolution, greedy_solution_acceptor::GreedySolutionAcceptor};

pub enum SolutionAcceptor {
    Greedy(GreedySolutionAcceptor),
}

impl AcceptSolution for SolutionAcceptor {
    fn accept(
        &self,
        current_solutions: &[Solution],
        solution: &WorkingSolution,
        score: &Score,
    ) -> bool {
        match self {
            SolutionAcceptor::Greedy(acceptor) => {
                acceptor.accept(current_solutions, solution, score)
            }
        }
    }
}
