use std::sync::Arc;

use fxhash::FxHashSet;
use serde::Serialize;

use crate::{
    problem::{service::ServiceId, vehicle_routing_problem::VehicleRoutingProblem},
    solver::{insertion::Insertion, solution::route::WorkingSolutionRoute},
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

    pub fn random_route<R>(&self, rng: &mut R) -> usize
    where
        R: rand::Rng,
    {
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
