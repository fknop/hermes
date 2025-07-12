use std::sync::MutexGuard;

use crate::problem::vehicle_routing_problem::VehicleRoutingProblem;

use super::{
    accepted_solution::AcceptedSolution,
    constraints::{
        activity_constraint::ActivityConstraintType, capacity_constraint::CapacityConstraint,
        constraint::Constraint, global_constraint::GlobalConstraintType,
        maximum_working_duration_constraint::MaximumWorkingDurationConstraint,
        route_constraint::RouteConstraintType, shift_constraint::ShiftConstraint,
        time_window_constraint::TimeWindowConstraint,
        transport_cost_constraint::TransportCostConstraint,
        vehicle_cost_constraint::VehicleCostConstraint,
        waiting_duration_constraint::WaitingDurationConstraint,
    },
    search::Search,
    solver_params::SolverParams,
};

pub struct Solver<'a> {
    params: SolverParams,
    search: Search<'a>,
}

impl<'a> Solver<'a> {
    pub fn new(problem: VehicleRoutingProblem, params: SolverParams) -> Self {
        let constraints = vec![
            Constraint::Global(GlobalConstraintType::TransportCost(TransportCostConstraint)),
            Constraint::Activity(ActivityConstraintType::TimeWindow(TimeWindowConstraint)),
            Constraint::Route(RouteConstraintType::Capacity(CapacityConstraint)),
            Constraint::Route(RouteConstraintType::Shift(ShiftConstraint)),
            Constraint::Route(RouteConstraintType::WaitingDuration(
                WaitingDurationConstraint,
            )),
            Constraint::Route(RouteConstraintType::VehicleCost(VehicleCostConstraint)),
            Constraint::Route(RouteConstraintType::MaximumWorkingDuration(
                MaximumWorkingDurationConstraint,
            )),
        ];

        let mut final_params = params.clone();
        final_params.prepare();

        let search = Search::new(final_params, problem, constraints);

        Solver { search, params }
    }

    pub fn solve(&'a self) {
        self.search.run();
    }

    pub fn best_solutions(&self) -> MutexGuard<'_, Vec<AcceptedSolution<'a>>> {
        self.search.best_solutions()
    }
}

#[cfg(test)]
mod tests {}
