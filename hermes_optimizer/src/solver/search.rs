use crate::{
    acceptor::solution_acceptor::SolutionAcceptor,
    problem::vehicle_routing_problem::VehicleRoutingProblem,
    selector::{select_best_selector::SelectBestSelector, solution_selector::SolutionSelector},
};

use super::{solution::Solution, working_solution::WorkingSolution};

pub struct Search<'a, Selector, Acceptor>
where
    Selector: SolutionSelector,
    Acceptor: SolutionAcceptor,
{
    problem: &'a VehicleRoutingProblem,
    best_solutions: Vec<Solution>,
    solution_selector: Selector,
    solution_acceptor: Acceptor,
}

impl<'a, Selector, Acceptor> Search<'a, Selector, Acceptor>
where
    Selector: SolutionSelector,
    Acceptor: SolutionAcceptor,
{
    pub fn new(
        problem: &'a VehicleRoutingProblem,
        solution_selector: Selector,
        solution_acceptor: Acceptor,
    ) -> Self {
        Search {
            problem,
            best_solutions: Vec::new(),
            solution_selector,
            solution_acceptor,
        }
    }

    pub fn run(&mut self) {
        let current_solution = self.solution_selector.select_solution(&self.best_solutions);
        let mut working_solution = if let Some(solution) = current_solution {
            WorkingSolution::from_solution(self.problem, solution)
        } else {
            WorkingSolution::new(self.problem)
        };

        // Update working_solution
    }
}
