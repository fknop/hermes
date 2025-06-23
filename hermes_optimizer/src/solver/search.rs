use crate::{
    acceptor::solution_acceptor::SolutionAcceptor,
    problem::vehicle_routing_problem::VehicleRoutingProblem,
    selector::{
        select_best_selector::SelectBestSelector, select_solution::SelectSolution,
        solution_selector::SolutionSelector,
    },
};

use super::{
    constraints::constraint::Constraint, solution::Solution, working_solution::WorkingSolution,
};

pub struct Search<'a> {
    problem: &'a VehicleRoutingProblem,
    constraints: &'a Vec<Constraint>,
    best_solutions: Vec<Solution>,
    solution_selector: SolutionSelector,
    solution_acceptor: SolutionAcceptor,
}

impl<'a> Search<'a> {
    pub fn new(
        problem: &'a VehicleRoutingProblem,
        constraints: &'a Vec<Constraint>,
        solution_selector: SolutionSelector,
        solution_acceptor: SolutionAcceptor,
    ) -> Self {
        Search {
            problem,
            constraints,
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
