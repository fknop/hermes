use crate::solver::solution::Solution;

use super::{accept_solution::AcceptSolution, greedy_solution_acceptor::GreedySolutionAcceptor};

pub enum SolutionAcceptor {
    Greedy(GreedySolutionAcceptor),
}

impl AcceptSolution for SolutionAcceptor {
    fn accept_solution(&self, current_solutions: &[Solution], solution: &Solution) -> bool {
        match self {
            SolutionAcceptor::Greedy(acceptor) => {
                acceptor.accept_solution(current_solutions, solution)
            }
        }
    }
}
