use crate::solver::{
    accepted_solution::AcceptedSolution, score::Score, working_solution::WorkingSolution,
};

use super::{
    accept_solution::{AcceptSolution, AcceptSolutionContext},
    greedy_solution_acceptor::GreedySolutionAcceptor,
    schrimpf_acceptor::SchrimpfAcceptor,
};

pub enum SolutionAcceptor {
    Greedy(GreedySolutionAcceptor),
    Schrimpf(SchrimpfAcceptor),
    Any,
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
            SolutionAcceptor::Schrimpf(acceptor) => {
                acceptor.accept(current_solutions, solution, score, context)
            }
            SolutionAcceptor::Any => true,
        }
    }
}
