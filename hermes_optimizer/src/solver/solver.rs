use rand::{Rng, rngs::ThreadRng};

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
        activity_constraint::ActivityConstraintType, capacity_constraint::CapacityConstraint,
        constraint::Constraint, route_constraint::RouteConstraintType,
        time_window_constraint::TimeWindowConstraint,
    },
    ruin::{ruin_random::RuinRandom, ruin_solution::RuinSolution, ruin_strategy::RuinStrategy},
    search::Search,
    solver_params::{SolverAcceptorType, SolverParams, SolverSelectorType},
    working_solution::WorkingSolution,
};

pub struct Solver {
    problem: VehicleRoutingProblem,
    constraints: Vec<Constraint>,
    params: SolverParams,
}

impl Solver {
    pub fn new(problem: VehicleRoutingProblem, params: SolverParams) -> Self {
        let mut solver = Solver {
            problem,
            constraints: vec![
                Constraint::Activity(ActivityConstraintType::TimeWindow(TimeWindowConstraint)),
                Constraint::Route(RouteConstraintType::Capacity(CapacityConstraint)),
            ],
            params,
        };

        solver
            .params
            .ruin
            .ruin_strategies
            .sort_by(|(_, w1), (_, w2)| w1.cmp(w2));

        solver
    }

    pub fn solve(&self) {
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

    fn ruin(&self, solution: &mut WorkingSolution) {
        let mut rng = rand::rng();

        let ruin_strategy = self.select_ruin_strategy(&mut rng);
        let ruin_maximum_ratio = self.params.ruin.ruin_maximum_ratio;
        let maximum_ruin_size =
            (ruin_maximum_ratio * self.problem.services().len() as f64).floor() as usize;
        let ruin_size = rng.random_range(0..maximum_ruin_size);

        match ruin_strategy {
            RuinStrategy::Random => {
                let strategy = RuinRandom;
                strategy.ruin_solution(solution, ruin_size);
            }
        }
    }

    fn select_ruin_strategy(&self, rng: &mut ThreadRng) -> RuinStrategy {
        let total_weight: u64 = self
            .params
            .ruin
            .ruin_strategies
            .iter()
            .map(|strategy| strategy.1)
            .sum();

        let random = rng.random_range(0..total_weight);
        for (strategy, weight) in &self.params.ruin.ruin_strategies {
            if random < *weight {
                return *strategy;
            }
        }

        panic!("No ruin strategy configured on solver");
    }
}
