use crate::solver::{
    accepted_solution::AcceptedSolution, score::Score, working_solution::WorkingSolution,
};

use super::{
    accept_solution::{AcceptSolution, AcceptSolutionContext},
    greedy_solution_acceptor::GreedySolutionAcceptor,
    shrimpf_acceptor::ShrimpfAcceptor,
};

pub enum SolutionAcceptor {
    Greedy(GreedySolutionAcceptor),
    Shrimpf(ShrimpfAcceptor),
}

impl AcceptSolution for SolutionAcceptor {
    fn accept(
        &self,
        current_solutions: &[AcceptedSolution],
        solution: &WorkingSolution,
        score: &Score,
        context: AcceptSolutionContext,
    ) -> bool {
        match self {
            SolutionAcceptor::Greedy(acceptor) => {
                acceptor.accept(current_solutions, solution, score, context)
            }
            SolutionAcceptor::Shrimpf(acceptor) => {
                acceptor.accept(current_solutions, solution, score, context)
            }
        }
    }
}
