use rand::{Rng, rngs::ThreadRng};

use crate::problem::vehicle_routing_problem::VehicleRoutingProblem;

use super::{
    constraints::{
        activity_constraint::ActivityConstraintType, capacity_constraint::CapacityConstraint,
        constraint::Constraint, global_constraint::GlobalConstraintType,
        route_constraint::RouteConstraintType, time_window_constraint::TimeWindowConstraint,
        transport_cost_constraint::TransportCostConstraint,
    },
    search::Search,
    solver_params::SolverParams,
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
                Constraint::Global(GlobalConstraintType::TransportCost(TransportCostConstraint)),
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
            .params
            .recreate
            .recreate_strategies
            .sort_by(|(_, w1), (_, w2)| w1.cmp(w2));

        solver
    }

    pub fn solve(&self) {
        let mut search = Search::new(&self.params, &self.problem, &self.constraints);

        search.on_best_solution(|solution| println!("Score: {:?}", solution.score_analysis));

        search.run();
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        solomon::solomon_parser::SolomonParser, solver::solver_params::SolverAcceptorStrategy,
    };

    use super::*;

    #[test]
    fn test_solomon_parser() {
        let file = "datasets/c1/c101.txt";

        let vrp = SolomonParser::from_file(file).unwrap();

        let solver = Solver::new(
            vrp,
            SolverParams {
                max_iterations: 100,
                ..SolverParams::default()
            },
        );

        solver.solve();
    }
}
