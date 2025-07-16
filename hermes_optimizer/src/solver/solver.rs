use std::{cell::Cell, ops::DerefMut};

use parking_lot::{MappedRwLockReadGuard, RwLock, RwLockWriteGuard};

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

#[derive(Copy, Clone, Debug)]
pub enum SolverStatus {
    Pending,
    Running,
    Completed,
}

pub struct Solver {
    params: SolverParams,
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

        let mut final_params = params.clone();
        final_params.prepare();

        let search = Search::new(final_params, problem, constraints);

        Solver {
            search,
            params,
            status: RwLock::new(SolverStatus::Pending),
        }
    }

    pub fn solve(&self) {
        *self.status.write() = SolverStatus::Running;
        self.search.run();
        *self.status.write() = SolverStatus::Completed;
    }

    pub fn status(&self) -> SolverStatus {
        println!("STATUS? {:?}", self.status.read().clone());
        self.status.read().clone()
    }

    pub fn current_best_solution(&self) -> Option<MappedRwLockReadGuard<'_, AcceptedSolution>> {
        self.search.best_solution()
    }
}

#[cfg(test)]
mod tests {}
