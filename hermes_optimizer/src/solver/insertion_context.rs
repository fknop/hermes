use jiff::{SignedDuration, Timestamp};

use crate::{
    problem::{
        capacity::Capacity,
        service::{ServiceId, ServiceType},
        vehicle_routing_problem::VehicleRoutingProblem,
    },
    solver::solution::utils::{
        compute_activity_arrival_time, compute_departure_time, compute_first_activity_arrival_time,
        compute_vehicle_end, compute_vehicle_start, compute_waiting_duration,
    },
};

use super::{insertion::Insertion, solution::working_solution::WorkingSolution};

pub struct ActivityInsertionContext {
    pub service_id: ServiceId,
    pub arrival_time: Timestamp,
    pub departure_time: Timestamp,
    // pub waiting_duration: SignedDuration,
    // pub cumulative_load: Capacity,
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
    pub initial_load: Capacity,
    pub activities: Vec<ActivityInsertionContext>,
    pub start: Timestamp,
    pub end: Timestamp,
    pub waiting_duration_delta: SignedDuration,
}

impl InsertionContext<'_> {
    pub fn inserted_activity(&self) -> &ActivityInsertionContext {
        &self.activities[self.insertion.position()]
    }

    pub fn problem(&self) -> &VehicleRoutingProblem {
        self.problem
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
            let mut activities = Vec::with_capacity(route.activities().len() + 1);

            activities.extend(
                route
                    .activities()
                    .iter()
                    .take(context.position)
                    .map(|activity| ActivityInsertionContext {
                        service_id: activity.service_id(),
                        arrival_time: activity.arrival_time(),
                        departure_time: activity.departure_time(),
                    }),
            );

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
            activities.push(ActivityInsertionContext {
                service_id: context.service_id,
                arrival_time,
                departure_time,
            });

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

                activities.push(ActivityInsertionContext {
                    service_id,
                    arrival_time,
                    departure_time,
                });

                last_service_id = service_id;
            }

            let service = problem.service(context.service_id);
            let current_initial_load = route.total_initial_load();
            let new_initial_load = if service.service_type() == ServiceType::Delivery {
                current_initial_load + service.demand()
            } else {
                current_initial_load.clone()
            };

            InsertionContext {
                problem,
                start: compute_vehicle_start(
                    problem,
                    route.vehicle_id(),
                    activities[0].service_id,
                    activities[0].arrival_time,
                ),
                end: compute_vehicle_end(
                    problem,
                    route.vehicle_id(),
                    activities[activities.len() - 1].service_id,
                    activities[activities.len() - 1].departure_time,
                ),
                solution,
                initial_load: new_initial_load,
                waiting_duration_delta,
                activities,
                insertion,
            }
        }
        Insertion::NewRoute(context) => {
            let arrival_time = compute_first_activity_arrival_time(
                problem,
                context.vehicle_id,
                context.service_id,
            );

            let departure_time = compute_departure_time(
                problem,
                arrival_time,
                compute_waiting_duration(problem.service(context.service_id), arrival_time),
                context.service_id,
            );

            let waiting_duration =
                compute_waiting_duration(problem.service(context.service_id), arrival_time);

            let activities = vec![ActivityInsertionContext {
                service_id: context.service_id,
                arrival_time,
                departure_time,
            }];

            let service = problem.service(context.service_id);

            InsertionContext {
                problem,
                waiting_duration_delta: waiting_duration,
                start: compute_vehicle_start(
                    problem,
                    context.vehicle_id,
                    activities[0].service_id,
                    activities[0].arrival_time,
                ),
                end: compute_vehicle_end(
                    problem,
                    context.vehicle_id,
                    activities[activities.len() - 1].service_id,
                    activities[activities.len() - 1].departure_time,
                ),
                solution,
                initial_load: if service.service_type() == ServiceType::Delivery {
                    service.demand().clone()
                } else {
                    Capacity::zero()
                },
                activities,
                insertion,
            }
        }
    }
}
