use std::sync::Arc;

use fxhash::FxHashSet;
use jiff::{SignedDuration, Timestamp};

use crate::problem::{
    capacity::Capacity,
    service::{Service, ServiceId},
    vehicle::VehicleId,
    vehicle_routing_problem::VehicleRoutingProblem,
};

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
}

pub struct WorkingSolutionRoute<'a> {
    problem: &'a VehicleRoutingProblem,
    vehicle_id: VehicleId,
    services: FxHashSet<ServiceId>,
    activities: Vec<WorkingSolutionRouteActivity<'a>>,
    total_demand: Capacity,
}

impl WorkingSolutionRoute<'_> {
    fn remove_service(&mut self, service_id: ServiceId) {
        if !self.services.contains(&service_id) {
            return;
        }

        self.activities
            .retain(|activity| activity.service_id != service_id);
        self.services.remove(&service_id);

        self.total_demand
            .sub(self.problem.service(service_id).demand());
    }

    fn insert_service(&mut self, index: usize, service_id: ServiceId) {
        if self.services.contains(&service_id) {
            return;
        }

        self.services.insert(service_id);
        self.activities.insert(
            index,
            WorkingSolutionRouteActivity {
                problem: self.problem,
                service_id,
                arrival_time: if self.activities.is_empty() {
                    compute_first_activity_arrival_time(self.problem, self.vehicle_id, service_id)
                } else {
                    compute_next_activity_arrival_time(
                        self.problem,
                        self.activities.last().unwrap(),
                        service_id,
                    )
                },
            },
        );

        self.total_demand
            .add(self.problem.service(service_id).demand());
    }
}

pub struct WorkingSolutionRouteActivity<'a> {
    problem: &'a VehicleRoutingProblem,
    service_id: ServiceId,
    arrival_time: Timestamp,
}

impl<'a> WorkingSolutionRouteActivity<'a> {
    pub fn service(&self) -> &'a Service {
        self.problem.service(self.service_id)
    }

    pub fn service_id(&self) -> ServiceId {
        self.service_id
    }

    pub fn arrival_time(&self) -> Timestamp {
        self.arrival_time
    }

    pub fn waiting_time(&self) -> SignedDuration {
        let service = self.service();
        let time_window = service.time_window();

        match time_window {
            Some(window) => {
                if let Some(time_window_start) = window.start() {
                    if self.arrival_time < time_window_start {
                        return SignedDuration::from_secs(
                            time_window_start.as_second() - self.arrival_time.as_second(),
                        );
                    }
                }

                SignedDuration::ZERO
            }
            None => SignedDuration::ZERO,
        }
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
        Some(depot_location) => {
            problem.travel_time(depot_location, problem.service_location(service_id))
        }
        None => SignedDuration::ZERO,
    };

    match time_window_start {
        Some(start) => (earliest_start_time + travel_time).max(start),
        None => earliest_start_time + travel_time,
    }
}

fn compute_next_activity_arrival_time(
    problem: &VehicleRoutingProblem,
    previous_activity: &WorkingSolutionRouteActivity,
    service_id: ServiceId,
) -> Timestamp {
    let previous_arrival_time = previous_activity.arrival_time();
    let travel_time = problem.travel_time(
        problem.service_location(previous_activity.service_id),
        problem.service_location(service_id),
    );

    previous_arrival_time + travel_time
}
