use fxhash::FxHashSet;
use jiff::{SignedDuration, Timestamp};

use crate::problem::{
    capacity::Capacity,
    service::{Service, ServiceId},
    travel_cost_matrix::Cost,
    vehicle::{Vehicle, VehicleId},
    vehicle_routing_problem::VehicleRoutingProblem,
};

use super::{
    insertion::Insertion,
    insertion_context::{ActivityInsertionContext, InsertionContext},
};

#[derive(Clone)]
pub struct WorkingSolution<'a> {
    problem: &'a VehicleRoutingProblem,
    routes: Vec<WorkingSolutionRoute<'a>>,
    unassigned_services: FxHashSet<ServiceId>,
}

impl<'a> WorkingSolution<'a> {
    pub fn new(problem: &'a VehicleRoutingProblem) -> Self {
        let routes = Vec::new();
        let unassigned_services = (0..problem.services().len()).collect();

        WorkingSolution {
            problem,
            routes,
            unassigned_services,
        }
    }

    pub fn has_available_vehicle(&self) -> bool {
        self.problem.vehicles().len() > self.routes.len()
    }

    pub fn available_vehicle(&self) -> Option<VehicleId> {
        // Find the first vehicle that has no routes assigned
        self.problem
            .vehicles()
            .iter()
            .enumerate()
            .map(|(vehicle_id, _)| vehicle_id)
            .find(|&vehicle_id| {
                !self
                    .routes
                    .iter()
                    .any(|route| route.vehicle_id == vehicle_id)
            })
    }

    pub fn unassigned_services(&self) -> &FxHashSet<ServiceId> {
        &self.unassigned_services
    }

    pub fn unassigned_services_mut(&mut self) -> &mut FxHashSet<ServiceId> {
        &mut self.unassigned_services
    }

    pub fn problem(&self) -> &VehicleRoutingProblem {
        self.problem
    }

    pub fn routes(&self) -> &[WorkingSolutionRoute] {
        &self.routes
    }

    pub fn route(&self, route_id: usize) -> &WorkingSolutionRoute {
        &self.routes[route_id]
    }

    pub fn insert_service(&mut self, insertion: &Insertion) {
        match insertion {
            Insertion::ExistingRoute(context) => {
                let route = &mut self.routes[context.route_id];
                route.insert_service(context.position, context.service_id);
                self.unassigned_services.remove(&context.service_id);
            }
            Insertion::NewRoute(context) => {
                let mut new_route = WorkingSolutionRoute::empty(self.problem, context.vehicle_id);
                new_route.insert_service(0, context.service_id);
                self.routes.push(new_route);
                self.unassigned_services.remove(&context.service_id);
            }
        }
    }

    pub fn remove_activity(&mut self, route_id: usize, activity_id: usize) {
        if route_id >= self.routes.len() {
            return; // Invalid route ID
        }

        let route = &mut self.routes[route_id];
        if let Some(service_id) = route.remove_activity(activity_id) {
            self.unassigned_services.insert(service_id);
        }

        if route.is_empty() {
            self.routes.remove(route_id);
        }
    }
}

#[derive(Clone)]
pub struct WorkingSolutionRoute<'a> {
    problem: &'a VehicleRoutingProblem,
    vehicle_id: VehicleId,
    services: FxHashSet<ServiceId>,
    activities: Vec<WorkingSolutionRouteActivity<'a>>,

    // Current total demand of the route
    total_demand: Capacity,

    // Current total cost of the route
    total_cost: Cost,

    // Current total waiting time of the route
    waiting_duration: SignedDuration,
}

impl<'a> WorkingSolutionRoute<'a> {
    pub fn empty(problem: &'a VehicleRoutingProblem, vehicle_id: VehicleId) -> Self {
        WorkingSolutionRoute {
            problem,
            vehicle_id,
            services: FxHashSet::default(),
            activities: Vec::new(),
            total_demand: Capacity::ZERO,
            total_cost: 0,
            waiting_duration: SignedDuration::ZERO,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.activities.is_empty()
    }

    pub fn problem(&self) -> &VehicleRoutingProblem {
        self.problem
    }

    pub fn start(&self) -> &WorkingSolutionRouteActivity<'_> {
        // Empty routes should not be allowed
        &self.activities[0]
    }

    pub fn end(&self) -> &WorkingSolutionRouteActivity<'_> {
        // Empty routes should not be allowed
        &self.activities[self.activities().len() - 1]
    }

    pub fn activities(&self) -> &[WorkingSolutionRouteActivity] {
        &self.activities
    }

    pub fn total_demand(&self) -> &Capacity {
        &self.total_demand
    }

    pub fn total_cost(&self) -> Cost {
        self.total_cost
    }

    pub fn vehicle(&self) -> &Vehicle {
        self.problem.vehicle(self.vehicle_id)
    }

    fn remove_activity(&mut self, activity_id: usize) -> Option<ServiceId> {
        if activity_id >= self.activities.len() {
            return None;
        }

        let activity = &self.activities[activity_id];
        let service_id = activity.service_id();

        if !self.services.contains(&service_id) {
            return None;
        }

        self.services.remove(&activity.service_id);
        self.waiting_duration -= activity.waiting_duration();
        self.total_demand
            .sub_mut(self.problem.service(activity.service_id).demand());

        self.activities.remove(activity_id);

        Some(service_id)
    }

    // fn remove_services(&mut self, service_ids: &FxHashSet<ServiceId>) {
    //     self.activities
    //         .iter()
    //         .filter(|activity| service_ids.contains(&activity.service_id))
    //         .for_each(|activity| self.waiting_duration -= activity.waiting_duration());

    //     for service_id in service_ids {
    //         self.services.remove(service_id);
    //         self.total_demand
    //             .sub_mut(self.problem.service(*service_id).demand());
    //     }

    //     self.activities
    //         .retain(|activity| !service_ids.contains(&activity.service_id));
    // }

    fn insert_service(&mut self, position: usize, service_id: ServiceId) {
        if self.services.contains(&service_id) {
            return;
        }

        self.services.insert(service_id);
        let activity = WorkingSolutionRouteActivity::new(
            self.problem,
            service_id,
            if self.activities.is_empty() || position == 0 {
                compute_first_activity_arrival_time(self.problem, self.vehicle_id, service_id)
            } else {
                let last_activity = &self.activities[position - 1];
                compute_activity_arrival_time(
                    self.problem,
                    last_activity.service_id(),
                    last_activity.departure_time(),
                    service_id,
                )
            },
        );

        self.waiting_duration += activity.waiting_duration();
        self.activities.insert(position, activity);

        // Update the arrival times and departure times of subsequent activities
        for i in position + 1..self.activities().len() {
            let previous_service_id = self.activities[i - 1].service_id;
            let previous_departure_time = self.activities[i - 1].departure_time;

            let activity = &mut self.activities[i];

            self.waiting_duration -= activity.waiting_duration();

            activity.update_arrival_time(compute_activity_arrival_time(
                self.problem,
                previous_service_id,
                previous_departure_time,
                service_id,
            ));

            self.waiting_duration += activity.waiting_duration()
        }

        self.total_demand
            .add_mut(self.problem.service(service_id).demand());
    }
}

#[derive(Clone)]
pub struct WorkingSolutionRouteActivity<'a> {
    problem: &'a VehicleRoutingProblem,
    service_id: ServiceId,
    arrival_time: Timestamp,
    departure_time: Timestamp,
    waiting_duration: SignedDuration,
}

impl<'a> WorkingSolutionRouteActivity<'a> {
    pub fn new(
        problem: &'a VehicleRoutingProblem,
        service_id: ServiceId,
        arrival_time: Timestamp,
    ) -> Self {
        let waiting_duration = compute_waiting_duration(problem, arrival_time, service_id);
        WorkingSolutionRouteActivity {
            problem,
            service_id,
            arrival_time,
            waiting_duration,
            departure_time: compute_departure_time(
                problem,
                arrival_time,
                waiting_duration,
                service_id,
            ),
        }
    }

    pub fn service(&self) -> &'a Service {
        self.problem.service(self.service_id)
    }

    pub fn service_id(&self) -> ServiceId {
        self.service_id
    }

    pub fn arrival_time(&self) -> Timestamp {
        self.arrival_time
    }

    pub fn departure_time(&self) -> Timestamp {
        self.departure_time
    }

    pub fn waiting_duration(&self) -> SignedDuration {
        self.waiting_duration
    }

    pub fn update_arrival_time(&mut self, arrival_time: Timestamp) {
        self.arrival_time = arrival_time;
        self.waiting_duration =
            compute_waiting_duration(self.problem, arrival_time, self.service_id);
        self.departure_time = compute_departure_time(
            self.problem,
            self.arrival_time,
            self.waiting_duration,
            self.service_id,
        );
    }
}

fn compute_first_activity_arrival_time(
    problem: &VehicleRoutingProblem,
    vehicle_id: VehicleId,
    service_id: ServiceId,
) -> Timestamp {
    let service = problem.service(service_id);

    let vehicle_depot_location = problem.vehicle_depot_location(vehicle_id);

    let vehicle = problem.vehicle(vehicle_id);
    let earliest_start_time = vehicle
        .earliest_start_time()
        .unwrap_or_else(|| Timestamp::from_second(0).unwrap());
    let time_window_start = service
        .time_window()
        .and_then(|time_window| time_window.start());

    let travel_time = match vehicle_depot_location {
        Some(depot_location) => problem.travel_time(
            depot_location.id(),
            problem.service(service_id).location_id(),
        ),
        None => SignedDuration::ZERO,
    };

    match time_window_start {
        Some(start) => (earliest_start_time + travel_time).max(start),
        None => earliest_start_time + travel_time,
    }
}

fn compute_activity_arrival_time(
    problem: &VehicleRoutingProblem,
    previous_service_id: ServiceId,
    previous_activity_departure_time: Timestamp,
    service_id: ServiceId,
) -> Timestamp {
    let travel_time = problem.travel_time(
        problem.service(previous_service_id).location_id(),
        problem.service(service_id).location_id(),
    );

    previous_activity_departure_time + travel_time
}

pub fn compute_waiting_duration(
    problem: &VehicleRoutingProblem,
    arrival_time: Timestamp,
    service_id: ServiceId,
) -> SignedDuration {
    let service = problem.service(service_id);
    let time_window = service.time_window();

    match time_window {
        Some(window) => {
            if let Some(time_window_start) = window.start() {
                if arrival_time < time_window_start {
                    return SignedDuration::from_secs(
                        time_window_start.as_second() - arrival_time.as_second(),
                    );
                }
            }

            SignedDuration::ZERO
        }
        None => SignedDuration::ZERO,
    }
}

pub fn compute_departure_time(
    problem: &VehicleRoutingProblem,
    arrival_time: Timestamp,
    waiting_duration: SignedDuration,
    service_id: ServiceId,
) -> Timestamp {
    arrival_time + waiting_duration + problem.service(service_id).service_duration()
}

fn compute_insertion_context<'a>(
    problem: &'a VehicleRoutingProblem,
    solution: &'a WorkingSolution<'a>,
    insertion: &'a Insertion,
) -> InsertionContext<'a> {
    let mut activities = Vec::new();

    match insertion {
        Insertion::ExistingRoute(context) => {
            let route = solution.routes.get(context.route_id).unwrap();

            for i in 0..context.position {
                activities.push({
                    ActivityInsertionContext {
                        service_id: route.activities[i].service_id,
                        arrival_time: route.activities[i].arrival_time,
                        departure_time: route.activities[i].departure_time,
                        waiting_duration: route.activities[i].waiting_duration,
                    }
                })
            }

            let mut arrival_time = if route.is_empty() || context.position == 0 {
                compute_first_activity_arrival_time(problem, route.vehicle_id, context.service_id)
            } else {
                let last_activity = &route.activities[context.position - 1];
                compute_activity_arrival_time(
                    problem,
                    last_activity.service_id(),
                    last_activity.departure_time(),
                    context.service_id,
                )
            };
            let mut waiting_duration =
                compute_waiting_duration(problem, arrival_time, context.service_id);
            let mut departure_time =
                compute_departure_time(problem, arrival_time, waiting_duration, context.service_id);

            activities.push(ActivityInsertionContext {
                service_id: context.service_id,
                arrival_time,
                departure_time,
                waiting_duration,
            });

            let mut last_service_id = context.service_id;

            // We don't do +1 here because the list didn't change
            for i in context.position..route.activities.len() {
                let service_id = route.activities[i].service_id;
                arrival_time = compute_activity_arrival_time(
                    problem,
                    last_service_id,
                    departure_time,
                    service_id,
                );

                waiting_duration = compute_waiting_duration(problem, arrival_time, service_id);
                departure_time =
                    compute_departure_time(problem, arrival_time, waiting_duration, service_id);

                activities.push(ActivityInsertionContext {
                    service_id,
                    arrival_time,
                    departure_time,
                    waiting_duration,
                });

                last_service_id = service_id;
            }

            InsertionContext {
                solution,
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
                compute_waiting_duration(problem, arrival_time, context.service_id),
                context.service_id,
            );

            let waiting_duration =
                compute_waiting_duration(problem, arrival_time, context.service_id);

            activities.push(ActivityInsertionContext {
                service_id: context.service_id,
                arrival_time,
                departure_time,
                waiting_duration,
            });

            InsertionContext {
                solution,
                activities,
                insertion,
            }
        }
    }
}
