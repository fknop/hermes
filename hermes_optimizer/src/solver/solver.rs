use crate::{
    acceptor::{
        greedy_solution_acceptor::GreedySolutionAcceptor, solution_acceptor::SolutionAcceptor,
    },
    problem::vehicle_routing_problem::VehicleRoutingProblem,
    selector::{
        select_best_selector::SelectBestSelector, select_random_selector::SelectRandomSelector,
        solution_selector::SolutionSelector,
    },
};

use super::{
    constraints::{
        activity_constraint::ActivityConstraintType, constraint::Constraint,
        time_window_constraint::TimeWindowConstraint,
    },
    search::Search,
    solver_params::{SolverAcceptorType, SolverParams, SolverSelectorType},
};

pub struct Solver {
    problem: VehicleRoutingProblem,
    constraints: Vec<Constraint>,
    params: SolverParams,
}

impl Solver {
    pub fn new(problem: VehicleRoutingProblem, params: SolverParams) -> Self {
        Solver {
            problem,
            constraints: vec![Constraint::Activity(ActivityConstraintType::TimeWindow(
                TimeWindowConstraint,
            ))],
            params,
        }
    }

    pub fn solve(self) {
        let search = Search::new(
            &self.problem,
            &self.constraints,
            match self.params.solver_selector {
                SolverSelectorType::SelectBest => SolutionSelector::SelectBest(SelectBestSelector),
                SolverSelectorType::SelectRandom => {
                    SolutionSelector::SelectRandom(SelectRandomSelector)
                }
            },
            match self.params.solver_acceptor {
                SolverAcceptorType::Greedy => SolutionAcceptor::Greedy(GreedySolutionAcceptor),
            },
        );
    }
}
