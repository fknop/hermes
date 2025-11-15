use std::{
    ops::{Add, Sub},
    sync::Arc,
};

use fxhash::FxHashSet;
use jiff::{SignedDuration, Timestamp};
use rand::{Rng, rngs::SmallRng};
use serde::Serialize;

use crate::problem::{
    capacity::Capacity,
    service::{Service, ServiceId, ServiceType},
    travel_cost_matrix::Cost,
    vehicle::{Vehicle, VehicleId},
    vehicle_routing_problem::VehicleRoutingProblem,
};

use super::{
    insertion::Insertion,
    insertion_context::{ActivityInsertionContext, InsertionContext},
};

#[derive(Clone, Serialize)]
pub struct WorkingSolution {
    #[serde(skip_serializing)]
    problem: Arc<VehicleRoutingProblem>,
    routes: Vec<WorkingSolutionRoute>,
    unassigned_services: FxHashSet<ServiceId>,
}

impl WorkingSolution {
    pub fn new(problem: Arc<VehicleRoutingProblem>) -> Self {
        let routes = Vec::new();
        let unassigned_services = (0..problem.services().len()).collect();

        WorkingSolution {
            problem,
            routes,
            unassigned_services,
        }
    }

    pub fn is_unassigned(&self, service_id: ServiceId) -> bool {
        self.unassigned_services.contains(&service_id)
    }

    pub fn total_transport_costs(&self) -> f64 {
        self.routes
            .iter()
            .map(|route| route.transport_costs(&self.problem))
            .sum()
    }

    /// To check if two working solutions are identical, we compare:
    /// 1) the number of routes
    /// 2) the vehicle IDs of each route
    /// 3) the service IDs of each activity in the routes
    ///
    /// Not perfect as routes that are not in the same order may not match properly
    pub fn is_identical(&self, other: &WorkingSolution) -> bool {
        if self.routes.len() != other.routes.len() {
            return false;
        }

        for (route, other_route) in self.routes.iter().zip(&other.routes) {
            if route.vehicle_id != other_route.vehicle_id {
                return false;
            }

            if route.activities.len() != other_route.activities.len() {
                return false;
            }

            if !route
                .activities
                .iter()
                .map(|activity| activity.service_id)
                .eq(other_route
                    .activities
                    .iter()
                    .map(|activity| activity.service_id))
            {
                return false;
            }
        }

        true
    }

    pub fn num_available_vehicles(&self) -> usize {
        self.problem.vehicles().len() - self.routes.len()
    }

    pub fn has_available_vehicle(&self) -> bool {
        self.problem.vehicles().len() > self.routes.len()
    }

    pub fn available_vehicles_iter(&self) -> impl std::iter::Iterator<Item = usize> {
        // Find the first vehicle that has no routes assigned
        self.problem
            .vehicles()
            .iter()
            .enumerate()
            .map(|(vehicle_id, _)| vehicle_id)
            .filter(|&vehicle_id| {
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
        self.problem.as_ref()
    }

    pub fn routes(&self) -> &[WorkingSolutionRoute] {
        &self.routes
    }

    pub fn route(&self, route_id: usize) -> &WorkingSolutionRoute {
        &self.routes[route_id]
    }

    pub fn random_route(&self, rng: &mut SmallRng) -> usize {
        rng.random_range(0..self.routes.len())
    }

    pub fn route_of_service(&self, service_id: ServiceId) -> Option<usize> {
        self.routes
            .iter()
            .enumerate()
            .find(|(_, route)| route.contains_service(service_id))
            .map(|(index, _)| index)
    }

    pub fn insert_service(&mut self, insertion: &Insertion) {
        match insertion {
            Insertion::ExistingRoute(context) => {
                let route = &mut self.routes[context.route_id];
                route.insert_service(&self.problem, context.position, context.service_id);
                self.unassigned_services.remove(&context.service_id);
            }
            Insertion::NewRoute(context) => {
                let mut new_route = WorkingSolutionRoute::empty(context.vehicle_id);
                new_route.insert_service(&self.problem, 0, context.service_id);
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
        if let Some(service_id) = route.remove_activity(&self.problem, activity_id) {
            self.unassigned_services.insert(service_id);
        }

        if route.is_empty() {
            self.routes.remove(route_id);
        }
    }

    pub fn remove_service(&mut self, service_id: ServiceId) -> bool {
        let mut route_to_remove = None;
        let mut removed = false;
        for (route_id, route) in self.routes.iter_mut().enumerate() {
            removed = route.remove_service(&self.problem, service_id);

            if removed {
                self.unassigned_services.insert(service_id);

                if route.is_empty() {
                    route_to_remove = Some(route_id);
                }
                break;
            }
        }

        if let Some(route_id) = route_to_remove {
            self.routes.remove(route_id);
        }

        removed
    }

    pub fn remove_service_from_route(&mut self, route_id: usize, service_id: ServiceId) -> bool {
        let mut route_to_remove = None;
        let mut removed = false;
        let route = &mut self.routes[route_id];
        if route.contains_service(service_id) {
            removed = route.remove_service(&self.problem, service_id);

            if removed {
                self.unassigned_services.insert(service_id);

                if route.is_empty() {
                    route_to_remove = Some(route_id);
                }
            }
        }

        if let Some(route_id) = route_to_remove {
            self.routes.remove(route_id);
        }

        removed
    }

    pub fn resync(&mut self) {
        for route in &mut self.routes {
            route.resync(&self.problem);
        }
    }

    pub fn remove_route(&mut self, route_id: usize) -> usize {
        let removed = self.routes[route_id].activities.len();
        for activity in self.routes[route_id].activities.iter() {
            self.unassigned_services.insert(activity.service_id);
        }

        self.routes.remove(route_id);

        removed
    }

    pub fn distance(&self) -> f64 {
        self.routes
            .iter()
            .map(|route| route.distance(&self.problem))
            .sum()
    }
}

#[derive(Clone, Serialize)]
pub struct WorkingSolutionRoute {
    vehicle_id: VehicleId,
    services: FxHashSet<ServiceId>,
    activities: Vec<WorkingSolutionRouteActivity>,

    // Current total demand of the route
    total_initial_load: Capacity,

    // Current total cost of the route
    total_cost: Cost,
}

impl WorkingSolutionRoute {
    pub fn empty(vehicle_id: VehicleId) -> Self {
        WorkingSolutionRoute {
            vehicle_id,
            services: FxHashSet::default(),
            activities: Vec::new(),
            total_initial_load: Capacity::ZERO,
            total_cost: 0.0,
        }
    }

    pub fn contains_service(&self, service_id: ServiceId) -> bool {
        self.services.contains(&service_id)
    }

    pub fn service_position(&self, service_id: ServiceId) -> Option<usize> {
        self.activities
            .iter()
            .position(|activity| activity.service_id == service_id)
    }

    pub fn is_empty(&self) -> bool {
        self.activities.is_empty()
    }

    pub fn has_start(&self, problem: &VehicleRoutingProblem) -> bool {
        let vehicle = problem.vehicle(self.vehicle_id);
        vehicle.depot_location_id().is_some()
    }

    pub fn has_end(&self, problem: &VehicleRoutingProblem) -> bool {
        let vehicle = problem.vehicle(self.vehicle_id);
        vehicle.depot_location_id().is_some() && vehicle.should_return_to_depot()
    }

    pub fn compute_location_ids(&self, problem: &VehicleRoutingProblem) -> Vec<usize> {
        let mut location_ids = vec![];

        if self.has_start(problem)
            && let Some(depot_location) = problem.vehicle_depot_location(self.vehicle_id)
        {
            location_ids.push(depot_location.id());
        }

        for activity in &self.activities {
            location_ids.push(activity.service(problem).location_id());
        }

        if self.has_end(problem)
            && let Some(depot_location) = problem.vehicle_depot_location(self.vehicle_id)
        {
            location_ids.push(depot_location.id());
        }

        location_ids
    }

    pub fn start(&self, problem: &VehicleRoutingProblem) -> Timestamp {
        let first = self.first();
        compute_vehicle_start(
            problem,
            self.vehicle_id,
            first.service_id(),
            first.arrival_time(),
        )
    }

    pub fn end(&self, problem: &VehicleRoutingProblem) -> Timestamp {
        let last = self.last();
        compute_vehicle_end(
            problem,
            self.vehicle_id,
            last.service_id(),
            last.departure_time(),
        )
    }

    pub fn duration(&self, problem: &VehicleRoutingProblem) -> SignedDuration {
        let start = self.start(problem);
        let end = self.end(problem);
        end.duration_since(start)
    }

    pub fn transport_duration(&self, problem: &VehicleRoutingProblem) -> SignedDuration {
        let vehicle = self.vehicle(problem);
        let mut transport_duration = SignedDuration::ZERO;

        if let Some(depot_location_id) = vehicle.depot_location_id() {
            if self.has_start(problem) {
                transport_duration += problem.travel_time(
                    depot_location_id,
                    self.first().service(problem).location_id(),
                );
            }

            if self.has_end(problem) {
                transport_duration += problem.travel_time(
                    self.last().service(problem).location_id(),
                    depot_location_id,
                );
            }
        }

        for (index, activity) in self.activities.iter().enumerate() {
            if index == 0 {
                // Skip the first activity, as it is already counted with the depot
                continue;
            }

            transport_duration += problem.travel_time(
                self.activities[index - 1].service(problem).location_id(),
                activity.service(problem).location_id(),
            );
        }

        transport_duration
    }

    pub fn transport_costs(&self, problem: &VehicleRoutingProblem) -> f64 {
        let vehicle = self.vehicle(problem);
        let mut costs = 0.0;

        if let Some(depot_location_id) = vehicle.depot_location_id() {
            if self.has_start(problem) {
                costs += problem.travel_cost(
                    depot_location_id,
                    self.first().service(problem).location_id(),
                );
            }

            if self.has_end(problem) {
                costs += problem.travel_cost(
                    self.last().service(problem).location_id(),
                    depot_location_id,
                );
            }
        }

        for (index, activity) in self.activities.iter().enumerate() {
            if index == 0 {
                // Skip the first activity, as it is already counted with the depot
                continue;
            }

            costs += problem.travel_cost(
                self.activities[index - 1].service(problem).location_id(),
                activity.service(problem).location_id(),
            );
        }

        costs
    }

    pub fn distance(&self, problem: &VehicleRoutingProblem) -> f64 {
        let vehicle = self.vehicle(problem);
        let mut distance = 0.0;

        if let Some(depot_location_id) = vehicle.depot_location_id() {
            if self.has_start(problem) {
                distance += problem.travel_distance(
                    depot_location_id,
                    self.first().service(problem).location_id(),
                );
            }

            if self.has_end(problem) {
                distance += problem.travel_distance(
                    self.last().service(problem).location_id(),
                    depot_location_id,
                );
            }
        }

        for (index, activity) in self.activities.iter().enumerate() {
            if index == 0 {
                // Skip the first activity, as it is already counted with the depot
                continue;
            }

            distance += problem.travel_distance(
                self.activities[index - 1].service(problem).location_id(),
                activity.service(problem).location_id(),
            );
        }

        distance
    }

    pub fn first(&self) -> &WorkingSolutionRouteActivity {
        // Empty routes should not be allowed
        &self.activities[0]
    }

    pub fn last(&self) -> &WorkingSolutionRouteActivity {
        // Empty routes should not be allowed
        &self.activities[self.activities().len() - 1]
    }

    pub fn activities(&self) -> &[WorkingSolutionRouteActivity] {
        &self.activities
    }

    pub fn total_initial_load(&self) -> &Capacity {
        &self.total_initial_load
    }

    pub fn total_cost(&self) -> Cost {
        self.total_cost
    }

    pub fn total_waiting_duration(&self) -> SignedDuration {
        self.activities
            .iter()
            .map(|activity| activity.waiting_duration)
            .sum()
    }

    pub fn vehicle_id(&self) -> VehicleId {
        self.vehicle_id
    }

    pub fn vehicle<'a>(&self, problem: &'a VehicleRoutingProblem) -> &'a Vehicle {
        problem.vehicle(self.vehicle_id)
    }

    pub fn max_load(&self, problem: &VehicleRoutingProblem) -> f64 {
        // TODO: incldue cumulative load here

        let vehicle = problem.vehicle(self.vehicle_id);
        let mut max_load = 0.0_f64;

        let vehicle_capacity = vehicle.capacity();

        for (index, demand) in self.total_initial_load.iter().enumerate() {
            let capacity = vehicle_capacity.get(index).unwrap_or(0.0);
            if capacity == 0.0 && demand > 0.0 {
                max_load = 1.0;
            } else {
                let load = demand / capacity;
                max_load = max_load.max(load);
            }
        }

        max_load
    }

    fn remove_activity(
        &mut self,
        problem: &VehicleRoutingProblem,
        activity_id: usize,
    ) -> Option<ServiceId> {
        if activity_id >= self.activities.len() {
            return None;
        }

        let activity = &self.activities[activity_id];
        let service_id = activity.service_id();

        if !self.services.contains(&service_id) {
            return None;
        }

        self.services.remove(&activity.service_id);

        if activity.service(problem).service_type() == ServiceType::Delivery {
            self.total_initial_load -= activity.service(problem).demand();
        }

        self.activities.remove(activity_id);

        // self.update_next_activities(problem, activity_id);

        Some(service_id)
    }

    fn remove_service(&mut self, problem: &VehicleRoutingProblem, service_id: ServiceId) -> bool {
        if !self.contains_service(service_id) {
            return false; // Service is not in the route
        }

        let activity_id = self
            .activities
            .iter()
            .position(|activity| activity.service_id == service_id)
            .unwrap();

        self.remove_activity(problem, activity_id).is_some()
    }

    fn insert_service(
        &mut self,
        problem: &VehicleRoutingProblem,
        position: usize,
        service_id: ServiceId,
    ) {
        if self.services.contains(&service_id) {
            return;
        }

        self.services.insert(service_id);
        let activity = WorkingSolutionRouteActivity::new(
            problem,
            service_id,
            if self.activities.is_empty() || position == 0 {
                compute_first_activity_arrival_time(problem, self.vehicle_id, service_id)
            } else {
                let previous_activity = &self.activities[position - 1];
                compute_activity_arrival_time(
                    problem,
                    previous_activity.service_id(),
                    previous_activity.departure_time(),
                    service_id,
                )
            },
            if self.activities().is_empty() || position == 0 {
                compute_activity_cumulative_load(problem.service(service_id), &Capacity::ZERO)
            } else {
                let previous_activity = &self.activities[position - 1];
                compute_activity_cumulative_load(
                    problem.service(service_id),
                    &previous_activity.cumulative_load,
                )
            },
        );

        self.activities.insert(position, activity);

        if problem.service(service_id).service_type() == ServiceType::Delivery {
            self.total_initial_load += problem.service(service_id).demand();
        }

        // Update the arrival times and departure times of subsequent activities
        self.forward_update_pass(problem, position);
        self.backward_update_pass(problem);
    }

    fn resync(&mut self, problem: &VehicleRoutingProblem) {
        let mut total_initial_load = Capacity::ZERO;

        for activity in self
            .activities()
            .iter()
            .filter(|activity| activity.service(problem).service_type() == ServiceType::Delivery)
        {
            total_initial_load += activity.service(problem).demand();
        }

        self.total_initial_load = total_initial_load;
        self.forward_update_pass(problem, 0);
        self.backward_update_pass(problem);
    }

    fn forward_update_pass(&mut self, problem: &VehicleRoutingProblem, start: usize) {
        for i in start..self.activities().len() {
            let (first, second) = self.activities.split_at_mut(i);
            let previous_activity = first.last();
            let current_activity = &mut second[0];

            match previous_activity {
                Some(previous_activity) => {
                    current_activity.update_arrival_time(
                        problem,
                        compute_activity_arrival_time(
                            problem,
                            previous_activity.service_id,
                            previous_activity.departure_time,
                            current_activity.service_id,
                        ),
                    );

                    current_activity.cumulative_load = compute_activity_cumulative_load(
                        problem.service(current_activity.service_id),
                        &previous_activity.cumulative_load,
                    );
                }
                None => {
                    current_activity.update_arrival_time(
                        problem,
                        compute_first_activity_arrival_time(
                            problem,
                            self.vehicle_id,
                            current_activity.service_id,
                        ),
                    );

                    current_activity.cumulative_load = compute_activity_cumulative_load(
                        problem.service(current_activity.service_id),
                        &Capacity::ZERO,
                    );
                }
            }
        }
    }

    fn backward_update_pass(&mut self, problem: &VehicleRoutingProblem) {
        for i in (0..self.activities.len()).rev() {
            let (first, second) = self.activities.split_at_mut(i + 1);
            let current_activity = &mut first[i];
            let next_activity = second.first();

            WorkingSolutionRoute::update_max_load_until_end(
                &self.total_initial_load,
                current_activity,
                next_activity,
            );
        }
    }

    fn update_max_load_until_end(
        total_initial_load: &Capacity,
        current_activity: &mut WorkingSolutionRouteActivity,
        next_activity: Option<&WorkingSolutionRouteActivity>,
    ) {
        match next_activity {
            Some(next_activity) => {
                let load = total_initial_load + &current_activity.cumulative_load;
                match load.partial_cmp(&next_activity.max_load_until_end) {
                    Some(std::cmp::Ordering::Greater) => {
                        current_activity.max_load_until_end = load.clone();
                    }
                    _ => {
                        current_activity.max_load_until_end =
                            next_activity.max_load_until_end.clone();
                    }
                }
            }
            None => {
                current_activity.max_load_until_end =
                    total_initial_load + &current_activity.cumulative_load;
            }
        }
    }

    pub fn random_activity(&self, rng: &mut SmallRng) -> usize {
        rng.random_range(0..self.activities.len())
    }
}

#[derive(Clone, Serialize)]
pub struct WorkingSolutionRouteActivity {
    // problem: &'a VehicleRoutingProblem,
    service_id: ServiceId,
    arrival_time: Timestamp,
    departure_time: Timestamp,
    waiting_duration: SignedDuration,
    cumulative_load: Capacity,
    max_load_until_end: Capacity,
}

impl WorkingSolutionRouteActivity {
    pub fn new(
        problem: &VehicleRoutingProblem,
        service_id: ServiceId,
        arrival_time: Timestamp,
        cumulative_load: Capacity,
    ) -> Self {
        let waiting_duration = compute_waiting_duration(problem.service(service_id), arrival_time);
        WorkingSolutionRouteActivity {
            service_id,
            arrival_time,
            waiting_duration,
            departure_time: compute_departure_time(
                problem,
                arrival_time,
                waiting_duration,
                service_id,
            ),
            cumulative_load,
            max_load_until_end: Capacity::ZERO,
        }
    }

    pub fn service<'a>(&self, problem: &'a VehicleRoutingProblem) -> &'a Service {
        problem.service(self.service_id)
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

    pub fn cumulative_load(&self) -> &Capacity {
        &self.cumulative_load
    }

    pub fn max_load_until_end(&self) -> &Capacity {
        &self.max_load_until_end
    }

    fn update_arrival_time(&mut self, problem: &VehicleRoutingProblem, arrival_time: Timestamp) {
        self.arrival_time = arrival_time;
        self.waiting_duration =
            compute_waiting_duration(problem.service(self.service_id), arrival_time);
        self.departure_time = compute_departure_time(
            problem,
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

    let travel_time = match vehicle_depot_location {
        Some(depot_location) => problem.travel_time(
            depot_location.id(),
            problem.service(service_id).location_id(),
        ),
        None => SignedDuration::ZERO,
    };

    let depot_duration = vehicle.depot_duration();

    let time_window_start = service
        .time_windows()
        .iter()
        .filter(|tw| tw.is_satisfied(earliest_start_time + travel_time + depot_duration))
        .min_by_key(|tw| tw.start())
        .and_then(|tw| tw.start());

    match time_window_start {
        Some(start) => (earliest_start_time + travel_time + depot_duration).max(start),
        None => earliest_start_time + travel_time + depot_duration,
    }
}

fn compute_vehicle_start(
    problem: &VehicleRoutingProblem,
    vehicle_id: VehicleId,
    first_service_id: ServiceId,
    first_arrival_time: Timestamp,
) -> Timestamp {
    let vehicle = problem.vehicle(vehicle_id);
    let service = problem.service(first_service_id);

    if let Some(depot_location) = problem.vehicle_depot_location(vehicle_id) {
        let travel_time = problem.travel_time(depot_location.id(), service.location_id());

        first_arrival_time - travel_time - vehicle.depot_duration()
    } else {
        first_arrival_time
    }
}

fn compute_vehicle_end(
    problem: &VehicleRoutingProblem,
    vehicle_id: VehicleId,
    last_service_id: ServiceId,
    last_departure_time: Timestamp,
) -> Timestamp {
    let service = problem.service(last_service_id);
    let vehicle = problem.vehicle(vehicle_id);
    if let Some(depot_location_id) = vehicle.depot_location_id()
        && vehicle.should_return_to_depot()
    {
        let travel_time = problem.travel_time(service.location_id(), depot_location_id);

        last_departure_time + travel_time + vehicle.end_depot_duration()
    } else {
        last_departure_time
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

fn compute_activity_cumulative_load(
    service: &Service,
    current_cumulative_load: &Capacity,
) -> Capacity {
    match service.service_type() {
        ServiceType::Pickup => current_cumulative_load.add(service.demand()),
        ServiceType::Delivery => current_cumulative_load.sub(service.demand()),
    }
}

pub fn compute_waiting_duration(service: &Service, arrival_time: Timestamp) -> SignedDuration {
    SignedDuration::from_secs(
        service
            .time_windows()
            .iter()
            .filter(|tw| tw.is_satisfied(arrival_time))
            .filter_map(|tw| tw.start())
            .map(|start| (start.as_second() - arrival_time.as_second()).max(0))
            .min()
            .unwrap_or(0),
    )
}

pub fn compute_departure_time(
    problem: &VehicleRoutingProblem,
    arrival_time: Timestamp,
    waiting_duration: SignedDuration,
    service_id: ServiceId,
) -> Timestamp {
    arrival_time + waiting_duration + problem.service(service_id).service_duration()
}

pub fn compute_insertion_context<'a>(
    problem: &'a VehicleRoutingProblem,
    solution: &'a WorkingSolution,
    insertion: &'a Insertion,
) -> InsertionContext<'a> {
    match insertion {
        Insertion::ExistingRoute(context) => {
            let route = &solution.routes[context.route_id];
            let mut activities = Vec::with_capacity(route.activities.len() + 1);

            activities.extend(
                route
                    .activities
                    .iter()
                    .take(context.position)
                    .map(|activity| ActivityInsertionContext {
                        service_id: activity.service_id,
                        arrival_time: activity.arrival_time,
                        departure_time: activity.departure_time,
                        waiting_duration: activity.waiting_duration,
                        // cumulative_load: activity.cumulative_load.clone(),
                    }),
            );

            let mut arrival_time = if route.is_empty() || context.position == 0 {
                compute_first_activity_arrival_time(problem, route.vehicle_id, context.service_id)
            } else {
                let previous_activity = &route.activities[context.position - 1];
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
            // let cumulative_load = if route.activities.is_empty() || context.position == 0 {
            //     compute_activity_cumulative_load(
            //         problem.service(context.service_id),
            //         &Capacity::ZERO,
            //     )
            // } else {
            //     let previous_activity = &route.activities[context.position - 1];
            //     compute_activity_cumulative_load(
            //         problem.service(context.service_id),
            //         &previous_activity.cumulative_load,
            //     )
            // };

            activities.push(ActivityInsertionContext {
                service_id: context.service_id,
                arrival_time,
                departure_time,
                waiting_duration,
                // cumulative_load,
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

                waiting_duration =
                    compute_waiting_duration(problem.service(context.service_id), arrival_time);
                departure_time =
                    compute_departure_time(problem, arrival_time, waiting_duration, service_id);

                // let cumulative_load = compute_activity_cumulative_load(
                //     problem.service(service_id),
                //     &activities[activities.len() - 1].cumulative_load,
                // );

                activities.push(ActivityInsertionContext {
                    service_id,
                    arrival_time,
                    departure_time,
                    waiting_duration,
                    // cumulative_load,
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
                    route.vehicle_id,
                    activities[0].service_id,
                    activities[0].arrival_time,
                ),
                end: compute_vehicle_end(
                    problem,
                    route.vehicle_id,
                    activities[activities.len() - 1].service_id,
                    activities[activities.len() - 1].departure_time,
                ),
                solution,
                initial_load: new_initial_load,
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
                waiting_duration,
                // cumulative_load: compute_activity_cumulative_load(
                // problem.service(context.service_id),
                // &Capacity::ZERO,
                // ),
            }];

            let service = problem.service(context.service_id);

            InsertionContext {
                problem,
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

#[cfg(test)]
mod tests {

    use crate::problem::{service::ServiceBuilder, time_window::TimeWindowBuilder};

    use super::*;

    #[test]
    fn test_compute_waiting_duration() {
        let time_windows = vec![
            TimeWindowBuilder::default()
                .with_iso_start("2025-06-10T08:00:00+02:00")
                .with_iso_end("2025-06-10T10:00:00+02:00")
                .build(),
            TimeWindowBuilder::default()
                .with_iso_start("2025-06-10T14:00:00+02:00")
                .with_iso_end("2025-06-10T16:00:00+02:00")
                .build(),
        ];
        let mut builder = ServiceBuilder::default();

        builder
            .set_time_windows(time_windows)
            .set_external_id(String::from("0"))
            .set_location_id(0);

        let service = builder.build();

        let mut waiting_duration =
            compute_waiting_duration(&service, "2025-06-10T09:00:00+02:00".parse().unwrap());

        assert_eq!(waiting_duration.as_secs(), 0);

        waiting_duration =
            compute_waiting_duration(&service, "2025-06-10T07:00:00+02:00".parse().unwrap());
        assert_eq!(waiting_duration.as_secs(), 3600); // 1 hour waiting time

        waiting_duration =
            compute_waiting_duration(&service, "2025-06-10T11:00:00+02:00".parse().unwrap());
        assert_eq!(waiting_duration.as_secs(), 10800); // 3 hours waiting time

        waiting_duration =
            compute_waiting_duration(&service, "2025-06-10T15:00:00+02:00".parse().unwrap());
        assert_eq!(waiting_duration.as_secs(), 0);
    }
}
