use jiff::Timestamp;

use crate::{
    problem::{job::JobId, service::ServiceId, vehicle_routing_problem::VehicleRoutingProblem},
    solver::solution::{
        route::WorkingSolutionRoute,
        route_update_iterator::RouteUpdateIterator,
        utils::{compute_first_activity_arrival_time, compute_vehicle_end, compute_vehicle_start},
    },
};

use super::{insertion::Insertion, solution::working_solution::WorkingSolution};

pub struct ActivityInsertionContext {
    pub service_id: ServiceId,
    pub arrival_time: Timestamp,
    pub departure_time: Timestamp,
}

impl ActivityInsertionContext {
    pub fn departure_time(&self) -> Timestamp {
        self.departure_time
    }
}

pub struct InsertionContext<'a> {
    pub problem: &'a VehicleRoutingProblem,
    pub solution: &'a WorkingSolution,
    pub insertion: &'a Insertion,
}

impl<'a> InsertionContext<'a> {
    pub fn new(
        problem: &'a VehicleRoutingProblem,
        solution: &'a WorkingSolution,
        insertion: &'a Insertion,
    ) -> Self {
        InsertionContext {
            problem,
            solution,
            insertion,
        }
    }

    pub fn problem(&self) -> &VehicleRoutingProblem {
        self.problem
    }

    pub fn route(&self) -> &WorkingSolutionRoute {
        self.insertion.route(self.solution)
    }

    pub fn compute_vehicle_start(&self) -> Timestamp {
        let route = self.insertion.route(self.solution);

        if self.insertion.position() == 0 {
            let vehicle_id = route.vehicle_id();
            compute_vehicle_start(
                self.problem,
                vehicle_id,
                self.insertion.job_id(),
                compute_first_activity_arrival_time(
                    self.problem,
                    vehicle_id,
                    self.insertion.job_id(),
                ),
            )
        } else {
            route.start(self.problem)
        }
    }

    pub fn updated_activities_iter(
        &'a self,
    ) -> RouteUpdateIterator<'a, impl Iterator<Item = JobId>> {
        let route = self.insertion.route(self.solution);
        let job_id = self.insertion.job_id();
        route.updated_activities_iter(
            self.problem,
            std::iter::once(job_id)
                .chain(route.job_ids_iter(self.insertion.position(), route.len())),
            self.insertion.position(),
            route.len() + 1,
        )
    }

    pub fn compute_vehicle_end(&self) -> Timestamp {
        let route = self.insertion.route(self.solution);

        let last_service = self.updated_activities_iter().last().unwrap();

        compute_vehicle_end(
            self.problem,
            route.vehicle_id(),
            last_service.job_id.into(),
            last_service.departure_time,
        )
    }
}
