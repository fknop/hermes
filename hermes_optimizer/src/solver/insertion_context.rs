use jiff::{SignedDuration, Timestamp};

use crate::{
    problem::{job::JobId, service::ServiceId, vehicle_routing_problem::VehicleRoutingProblem},
    solver::solution::{
        route_update_iterator::RouteUpdateIterator,
        utils::{
            compute_activity_arrival_time, compute_departure_time,
            compute_first_activity_arrival_time, compute_vehicle_end, compute_vehicle_start,
            compute_waiting_duration,
        },
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
    pub waiting_duration_delta: SignedDuration,
}

impl<'a> InsertionContext<'a> {
    pub fn problem(&self) -> &VehicleRoutingProblem {
        self.problem
    }

    pub fn compute_vehicle_start(&self) -> Timestamp {
        let route = self.insertion.route(self.solution);

        if self.insertion.position() == 0 {
            let vehicle_id = route.vehicle_id();
            compute_vehicle_start(
                self.problem,
                vehicle_id,
                self.insertion.service_id(),
                compute_first_activity_arrival_time(
                    self.problem,
                    vehicle_id,
                    self.insertion.service_id(),
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

pub fn compute_insertion_context<'a>(
    problem: &'a VehicleRoutingProblem,
    solution: &'a WorkingSolution,
    insertion: &'a Insertion,
) -> InsertionContext<'a> {
    match insertion {
        Insertion::ExistingRoute(context) => {
            let route = solution.route(context.route_id);

            let mut arrival_time = if route.is_empty() || context.position == 0 {
                compute_first_activity_arrival_time(problem, route.vehicle_id(), context.service_id)
            } else {
                let previous_activity = &route.activities()[context.position - 1];
                compute_activity_arrival_time(
                    problem,
                    previous_activity.service_id(),
                    previous_activity.departure_time(),
                    context.service_id,
                )
            };
            let mut waiting_duration =
                compute_waiting_duration(problem.service(context.service_id), arrival_time);
            let mut departure_time =
                compute_departure_time(problem, arrival_time, waiting_duration, context.service_id);

            let mut waiting_duration_delta =
                compute_waiting_duration(problem.service(context.service_id), arrival_time);

            let mut last_service_id = context.service_id;

            // We don't do +1 here because the list didn't change
            for i in context.position..route.activities().len() {
                let service_id = route.activities()[i].service_id();
                arrival_time = compute_activity_arrival_time(
                    problem,
                    last_service_id,
                    departure_time,
                    service_id,
                );

                waiting_duration_delta -= route.activities()[i].waiting_duration();

                waiting_duration =
                    compute_waiting_duration(problem.service(context.service_id), arrival_time);
                waiting_duration_delta += waiting_duration;

                departure_time =
                    compute_departure_time(problem, arrival_time, waiting_duration, service_id);

                last_service_id = service_id;
            }

            InsertionContext {
                problem,
                solution,
                waiting_duration_delta,
                insertion,
            }
        }
        Insertion::NewRoute(context) => {
            let arrival_time = compute_first_activity_arrival_time(
                problem,
                context.vehicle_id,
                context.service_id,
            );

            let waiting_duration =
                compute_waiting_duration(problem.service(context.service_id), arrival_time);

            InsertionContext {
                problem,
                waiting_duration_delta: waiting_duration,
                solution,
                insertion,
            }
        }
    }
}
