use fxhash::FxHashSet;
use jiff::{SignedDuration, Timestamp};

use crate::problem::{
    capacity::Capacity,
    service::{Service, ServiceId},
    travel_cost_matrix::Cost,
    vehicle::{Vehicle, VehicleId},
    vehicle_routing_problem::VehicleRoutingProblem,
};

use super::{insertion_context::InsertionContext, score::Score, solution::Solution};

pub struct WorkingSolution<'a> {
    problem: &'a VehicleRoutingProblem,
    routes: Vec<WorkingSolutionRoute<'a>>,
    unassigned_services: FxHashSet<ServiceId>,
}

impl<'a> WorkingSolution<'a> {
    pub fn from_solution(problem: &'a VehicleRoutingProblem, solution: &Solution) -> Self {
        WorkingSolution {
            problem,
            routes: solution
                .routes
                .iter()
                .map(|route| {
                    let activities: Vec<WorkingSolutionRouteActivity> = route
                        .activities
                        .iter()
                        .map(|activity| {
                            WorkingSolutionRouteActivity::new(
                                problem,
                                activity.service_id,
                                activity.arrival_time,
                            )
                        })
                        .collect();

                    let waiting_duration = activities
                        .iter()
                        .map(|activity| activity.waiting_duration())
                        .sum();
                    WorkingSolutionRoute {
                        problem,
                        vehicle_id: route.vehicle_id,
                        total_demand: route.total_demand.clone(),
                        activities,
                        services: route
                            .activities
                            .iter()
                            .map(|activity| activity.service_id)
                            .collect(),
                        total_cost: route.total_cost,
                        waiting_duration,
                    }
                })
                .collect(),
            unassigned_services: FxHashSet::default(),
        }
    }

    pub fn new(problem: &'a VehicleRoutingProblem) -> Self {
        let routes = Vec::new();
        let unassigned_services = (0..problem.services().len()).collect();

        WorkingSolution {
            problem,
            routes,
            unassigned_services,
        }
    }

    pub fn problem(&self) -> &VehicleRoutingProblem {
        self.problem
    }

    pub fn routes(&self) -> &[WorkingSolutionRoute] {
        &self.routes
    }

    fn insert_service(&mut self, insertion_context: &InsertionContext) {
        match insertion_context {
            InsertionContext::ExistingRoute(context) => {
                let route = &mut self.routes[context.route_id];
                route.insert_service(context.position, context.service_id);
            }
            InsertionContext::NewRoute(context) => {
                let mut new_route = WorkingSolutionRoute {
                    problem: self.problem,
                    vehicle_id: context.vehicle_id,
                    services: FxHashSet::default(),
                    activities: Vec::new(),
                    total_demand: Capacity::ZERO,
                    total_cost: 0,
                    waiting_duration: SignedDuration::ZERO,
                };
                new_route.insert_service(0, context.service_id);
                self.routes.push(new_route);
            }
        }
    }
}

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

impl WorkingSolutionRoute<'_> {
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

    fn remove_service(&mut self, service_id: ServiceId) {
        if !self.services.contains(&service_id) {
            return;
        }

        let activity = self
            .activities
            .iter()
            .find(|activity| activity.service_id == service_id);

        if let Some(activity) = activity {
            self.waiting_duration -= activity.waiting_duration()
        }

        self.activities
            .retain(|activity| activity.service_id != service_id);
        self.services.remove(&service_id);

        self.total_demand
            .sub_mut(self.problem.service(service_id).demand());
    }

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
    let earliest_start_time = vehicle.earliest_start_time();
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

fn compute_insertion_waiting_duration_delta(
    problem: &VehicleRoutingProblem,
    solution: &WorkingSolution,
    insertion_context: &InsertionContext,
) -> SignedDuration {
    match insertion_context {
        InsertionContext::ExistingRoute(context) => {
            let route = solution.routes.get(context.route_id).unwrap();

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

            let mut delta = waiting_duration;

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

                last_service_id = service_id;

                delta += waiting_duration;
                delta -= route.activities[i].waiting_duration;
            }

            delta
        }
        InsertionContext::NewRoute(context) => {
            let arrival_time = compute_first_activity_arrival_time(
                problem,
                context.vehicle_id,
                context.service_id,
            );

            compute_waiting_duration(problem, arrival_time, context.service_id)
        }
    }
}

pub fn compute_insertion_score(
    solution: &WorkingSolution,
    insertion_context: &InsertionContext,
) -> Score {
    let route = match insertion_context {
        InsertionContext::ExistingRoute(context) => solution.routes.get(context.route_id),
        InsertionContext::NewRoute(_) => None,
    };

    let vehicle_id = match insertion_context {
        InsertionContext::ExistingRoute(context) => solution.routes()[context.route_id].vehicle_id,
        InsertionContext::NewRoute(context) => context.vehicle_id,
    };

    let problem = solution.problem();
    let service_id = insertion_context.service_id();
    let position = insertion_context.position();
    let service_to_insert = problem.service(service_id);
    let vehicle = problem.vehicle(vehicle_id);

    // 1. Check Capacity Constraint
    if vehicle.capacity().satisfies_demand(
        &(route
            .map(|route| &route.total_demand)
            .unwrap_or(&Capacity::ZERO)
            + service_to_insert.demand()),
    ) {
        return Score::hard(1); // TODO
    }

    // let depot_idx = 0; // Assuming depot is always at index 0
    let depot_location = problem.vehicle_depot_location(vehicle_id);
    let service_location_id = service_to_insert.location_id();

    let mut previous_location_id = None;
    let mut next_location_id = None;

    if route.is_none() || route.unwrap().is_empty() {
        if let Some(depot) = depot_location {
            previous_location_id = Some(depot.id());

            if vehicle.should_return_to_depot() {
                next_location_id = Some(depot.id());
            }
        }
    } else if position == 0 {
        if let Some(location) = depot_location {
            previous_location_id = Some(location.id());
        }

        let activities = route.unwrap().activities();
        next_location_id = Some(activities[1].service().location_id());
    } else if position >= route.unwrap().activities().len() {
        // Inserting at the end
        let activities = route.unwrap().activities();
        previous_location_id = Some(activities[activities.len() - 1].service().location_id());

        if let Some(depot) = depot_location {
            if vehicle.should_return_to_depot() {
                next_location_id = Some(depot.id());
            }
        }
    } else {
        let activities = route.unwrap().activities();
        previous_location_id = Some(activities[position - 1].service().location_id());
        next_location_id = Some(activities[position].service().location_id());
    }

    let old_cost = if let (Some(previous), Some(next)) = (previous_location_id, next_location_id) {
        problem.travel_cost(previous, next)
    } else {
        0
    };

    let mut new_cost = 0;

    if let Some(previous) = previous_location_id {
        new_cost += problem.travel_cost(previous, service_location_id);
    }

    if let Some(next) = next_location_id {
        new_cost += problem.travel_cost(service_location_id, next);
    }

    let travel_cost_delta = new_cost - old_cost;

    let waiting_duration_delta =
        compute_insertion_waiting_duration_delta(problem, solution, insertion_context);

    // (Optional) Add Time Window checks here. This is more complex as it
    // requires re-calculating arrival times from `position` onwards.

    Score::soft(travel_cost_delta + problem.waiting_cost(waiting_duration_delta))
}
