use std::sync::Arc;

use parking_lot::{MappedRwLockReadGuard, RwLock};

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
    statistics::SearchStatistics,
};

#[derive(Copy, Clone, Debug)]
pub enum SolverStatus {
    Pending,
    Running,
    Completed,
}

pub struct Solver {
    search: Search,
    status: RwLock<SolverStatus>,
}

impl Solver {
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

        let search = Search::new(params, problem, constraints);

        Solver {
            status: RwLock::new(SolverStatus::Pending),
            search,
        }
    }

    pub fn on_best_solution(&mut self, callback: fn(&AcceptedSolution)) {
        self.search.on_best_solution(callback);
    }

    pub fn solve(&self) {
        *self.status.write() = SolverStatus::Running;
        self.search.run();
        *self.status.write() = SolverStatus::Completed;
    }

    pub fn stop(&self) {
        self.search.stop();
        *self.status.write() = SolverStatus::Completed;
    }

    pub fn status(&self) -> SolverStatus {
        *self.status.read()
    }

    pub fn current_best_solution(&self) -> Option<MappedRwLockReadGuard<'_, AcceptedSolution>> {
        self.search.best_solution()
    }

    pub fn statistics(&self) -> Arc<SearchStatistics> {
        self.search.statistics()
    }
}

#[cfg(test)]
mod tests {}
