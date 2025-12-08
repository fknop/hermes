use std::sync::Arc;

use fxhash::FxHashSet;
use rand::seq::IteratorRandom;

use crate::{
    problem::{
        job::ActivityId, service::ServiceId, vehicle_routing_problem::VehicleRoutingProblem,
    },
    solver::{insertion::Insertion, solution::route::WorkingSolutionRoute},
};

#[derive(Clone)]
pub struct WorkingSolution {
    problem: Arc<VehicleRoutingProblem>,
    routes: Vec<WorkingSolutionRoute>,
    unassigned_jobs: FxHashSet<ServiceId>,
}

impl WorkingSolution {
    pub fn new(problem: Arc<VehicleRoutingProblem>) -> Self {
        let routes = problem
            .vehicles()
            .iter()
            .enumerate()
            .map(|(vehicle_id, _)| WorkingSolutionRoute::empty(&problem, vehicle_id))
            .collect();
        let unassigned_services = (0..problem.jobs().len()).collect();

        WorkingSolution {
            problem,
            routes,
            unassigned_jobs: unassigned_services,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.unassigned_jobs.len() == self.problem.jobs().len()
    }

    pub fn is_unassigned(&self, service_id: ServiceId) -> bool {
        self.unassigned_jobs.contains(&service_id)
    }

    pub fn total_transport_costs(&self) -> f64 {
        self.non_empty_routes_iter()
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

            if route.activity_ids.len() != other_route.activity_ids.len() {
                return false;
            }

            if !route.activity_ids.eq(&other_route.activity_ids) {
                return false;
            }
        }

        true
    }

    // pub fn num_available_vehicles(&self) -> usize {
    //     self.problem.vehicles().len() - self.routes.len()
    // }

    // pub fn has_available_vehicle(&self) -> bool {
    //     self.problem.vehicles().len() > self.routes.len()
    // }

    pub fn available_vehicles_iter(&self) -> impl std::iter::Iterator<Item = usize> {
        // Find the first vehicle that has no routes assigned
        self.problem
            .vehicles()
            .iter()
            .enumerate()
            .map(|(vehicle_id, _)| vehicle_id)
            .filter(|&vehicle_id| {
                self.routes
                    .iter()
                    .any(|route| route.is_empty() && route.vehicle_id == vehicle_id)
            })
    }

    pub fn unassigned_jobs(&self) -> &FxHashSet<ServiceId> {
        &self.unassigned_jobs
    }

    pub fn problem(&self) -> &VehicleRoutingProblem {
        self.problem.as_ref()
    }

    pub fn non_empty_routes_iter(&self) -> impl Iterator<Item = &WorkingSolutionRoute> {
        self.routes.iter().filter(|route| !route.is_empty())
    }

    pub fn non_empty_routes_count(&self) -> usize {
        self.routes.iter().filter(|route| !route.is_empty()).count()
    }

    pub fn routes(&self) -> &[WorkingSolutionRoute] {
        &self.routes
    }

    pub fn route(&self, route_id: usize) -> &WorkingSolutionRoute {
        &self.routes[route_id]
    }

    pub fn route_mut(&mut self, route_id: usize) -> &mut WorkingSolutionRoute {
        &mut self.routes[route_id]
    }

    pub fn random_non_empty_route<R>(&self, rng: &mut R) -> Option<usize>
    where
        R: rand::Rng,
    {
        self.routes
            .iter()
            .enumerate()
            .filter(|(_, route)| !route.is_empty())
            .choose(rng)
            .map(|(index, _)| index)
    }

    pub fn route_of_service(&self, service_id: ServiceId) -> Option<usize> {
        self.routes
            .iter()
            .enumerate()
            .find(|(_, route)| route.contains_job(ActivityId::Service(service_id)))
            .map(|(index, _)| index)
    }

    pub fn route_of_job(&self, activity_id: ActivityId) -> Option<usize> {
        self.routes
            .iter()
            .enumerate()
            .find(|(_, route)| route.contains_job(activity_id))
            .map(|(index, _)| index)
    }

    pub fn insert(&mut self, insertion: &Insertion) {
        match insertion {
            Insertion::Service(context) => {
                let route = &mut self.routes[context.route_id];
                route.insert(&self.problem, insertion);
                self.unassigned_jobs.remove(&context.job_index);
            }
            Insertion::Shipment(context) => {
                unimplemented!()
            }
        }
    }

    pub fn remove_activity(&mut self, route_id: usize, activity_id: usize) {
        if route_id >= self.routes.len() {
            return; // Invalid route ID
        }

        let route = &mut self.routes[route_id];
        if let Some(job_id) = route.remove_activity(&self.problem, activity_id) {
            self.unassigned_jobs.insert(job_id.index());
        }
    }

    pub fn remove_job(&mut self, job_id: ActivityId) -> bool {
        let mut removed = false;
        for route in self.routes.iter_mut() {
            removed = route.remove_job(&self.problem, job_id);

            if removed {
                self.unassigned_jobs.insert(job_id.index());
                break;
            }
        }

        removed
    }

    pub fn remove_service(&mut self, service_id: ServiceId) -> bool {
        self.remove_job(ActivityId::Service(service_id))
    }

    pub fn remove_service_from_route(&mut self, route_id: usize, service_id: ServiceId) -> bool {
        let mut removed = false;
        let route = &mut self.routes[route_id];
        if route.contains_job(ActivityId::Service(service_id)) {
            removed = route.remove_job(&self.problem, ActivityId::Service(service_id));

            if removed {
                self.unassigned_jobs.insert(service_id);
            }
        }

        removed
    }

    pub fn resync(&mut self) {
        for route in &mut self.routes {
            route.resync(&self.problem);
        }
    }

    pub fn remove_route(&mut self, route_id: usize) -> usize {
        let removed = self.routes[route_id].activity_ids.len();
        for job_id in self.routes[route_id].activity_ids.iter() {
            self.unassigned_jobs.insert(job_id.index());
        }

        // TODO: reset to avoid reallocations
        self.routes[route_id] = WorkingSolutionRoute::empty(&self.problem, route_id);

        removed
    }

    pub fn distance(&self) -> f64 {
        self.routes
            .iter()
            .map(|route| route.distance(&self.problem))
            .sum()
    }
}
