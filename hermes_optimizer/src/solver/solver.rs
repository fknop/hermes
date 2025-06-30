use rand::{Rng, rngs::ThreadRng};
use tracing::info;

use crate::problem::vehicle_routing_problem::VehicleRoutingProblem;

use super::{
    constraints::{
        activity_constraint::ActivityConstraintType, capacity_constraint::CapacityConstraint,
        constraint::Constraint, global_constraint::GlobalConstraintType,
        route_constraint::RouteConstraintType, shift_constraint::ShiftConstraint,
        time_window_constraint::TimeWindowConstraint,
        transport_cost_constraint::TransportCostConstraint,
        vehicle_cost_constraint::VehicleCostConstraint,
        waiting_duration_constraint::WaitingDurationConstraint,
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
                Constraint::Route(RouteConstraintType::Shift(ShiftConstraint)),
                Constraint::Route(RouteConstraintType::WaitingDuration(
                    WaitingDurationConstraint,
                )),
                Constraint::Route(RouteConstraintType::VehicleCost(VehicleCostConstraint)),
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

        search.on_best_solution(|accepted_solution| {
            info!("Score: {:?}", accepted_solution.score_analysis);
            info!("Vehicles {:?}", accepted_solution.solution.routes().len());

            // for route in accepted_solution.solution.routes() {
            //     info!(
            //         "Activities: {:?}",
            //         route
            //             .activities()
            //             .iter()
            //             .map(|a| a.service_id())
            //             .collect::<Vec<_>>()
            //     );
            // }
        });

        search.run();
    }
}

#[cfg(test)]
mod tests {}
