use std::sync::Arc;

use fxhash::{FxHashMap, FxHashSet};
use rand::seq::IteratorRandom;

use crate::{
    problem::{
        job::{ActivityId, Job, JobIdx},
        meters::Meters,
        vehicle::{Vehicle, VehicleIdx},
        vehicle_routing_problem::VehicleRoutingProblem,
    },
    solver::{
        insertion::Insertion,
        solution::{route::WorkingSolutionRoute, route_id::RouteIdx},
    },
    utils::enumerate_idx::EnumerateIdx,
};

#[derive(Clone)]
pub struct WorkingSolution {
    problem: Arc<VehicleRoutingProblem>,
    routes: Vec<WorkingSolutionRoute>,
    vehicle_route_map: FxHashMap<VehicleIdx, FxHashSet<RouteIdx>>,
    unassigned_jobs: FxHashSet<JobIdx>,
}

impl WorkingSolution {
    pub fn new(problem: Arc<VehicleRoutingProblem>) -> Self {
        let routes = problem
            .vehicles()
            .iter()
            .enumerate_idx()
            .map(|(vehicle_id, _)| WorkingSolutionRoute::empty(&problem, vehicle_id))
            .collect::<Vec<_>>();
        let unassigned_jobs = (0..problem.jobs().len()).map(JobIdx::new).collect();

        let vehicle_route_map = problem
            .vehicles()
            .iter()
            .enumerate_idx()
            .map(|(vehicle_id, _): (VehicleIdx, &Vehicle)| {
                let mut set = FxHashSet::default();
                set.insert(RouteIdx::new(vehicle_id.get()));
                (vehicle_id, set)
            })
            .collect::<FxHashMap<VehicleIdx, FxHashSet<RouteIdx>>>();

        WorkingSolution {
            problem,
            routes,
            unassigned_jobs,
            vehicle_route_map,
        }
    }

    fn create_additional_route(&mut self, vehicle_id: VehicleIdx) {
        // Don't create an additional route if the fleet is not infinite
        if !self.problem.fleet().is_infinite() {
            return;
        }

        let route = WorkingSolutionRoute::empty(&self.problem, vehicle_id);
        let route_id = RouteIdx::new(route.len());
        self.routes.push(route);

        if let Some(route_ids) = self.vehicle_route_map.get_mut(&vehicle_id) {
            let has_empty_route = route_ids
                .iter()
                .all(|&route_id| self.routes[route_id].is_empty());

            if !has_empty_route {
                route_ids.insert(route_id);
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.unassigned_jobs.len() == self.problem.jobs().len()
    }

    pub fn has_unassigned(&self) -> bool {
        !self.unassigned_jobs.is_empty()
    }

    pub fn is_unassigned(&self, service_id: JobIdx) -> bool {
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

    pub fn has_available_vehicle(&self) -> bool {
        let fleet = self.problem.fleet();
        let is_infinite = fleet.is_infinite();
        if is_infinite {
            true
        } else {
            self.routes.len() < fleet.vehicles().len()
        }
    }

    pub fn available_vehicles_for_insertion(&self) -> impl std::iter::Iterator<Item = VehicleIdx> {
        let fleet = self.problem.fleet();
        let is_infinite = fleet.is_infinite();
        // Find the first vehicle that has no routes assigned
        fleet
            .vehicles()
            .iter()
            .enumerate()
            .map(|(vehicle_id, _)| VehicleIdx::new(vehicle_id))
            .filter(move |&vehicle_id| {
                if is_infinite {
                    return true;
                }

                self.routes
                    .iter()
                    .any(|route| route.is_empty() && route.vehicle_id == vehicle_id)
            })
    }

    pub fn unassigned_jobs(&self) -> &FxHashSet<JobIdx> {
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

    pub fn route(&self, route_id: RouteIdx) -> &WorkingSolutionRoute {
        &self.routes[route_id]
    }

    pub fn route_mut(&mut self, route_id: RouteIdx) -> &mut WorkingSolutionRoute {
        &mut self.routes[route_id]
    }

    pub fn random_assigned_job<R>(&self, rng: &mut R) -> Option<JobIdx>
    where
        R: rand::Rng,
    {
        if self.unassigned_jobs.len() == self.problem.jobs().len() {
            return None;
        }

        loop {
            let job_id = self.problem().random_job(rng);
            if !self.unassigned_jobs.contains(&job_id) {
                return Some(job_id);
            }
        }
    }

    pub fn random_non_empty_route<R>(&self, rng: &mut R) -> Option<RouteIdx>
    where
        R: rand::Rng,
    {
        self.routes
            .iter()
            .enumerate()
            .filter(|(_, route)| !route.is_empty())
            .choose(rng)
            .map(|(index, _)| RouteIdx::new(index))
    }

    pub fn route_of_job(&self, job_id: JobIdx) -> Option<RouteIdx> {
        let job = self.problem().job(job_id);
        self.routes
            .iter()
            .enumerate_idx()
            .find(|(_, route)| match job {
                Job::Service(_) => route.contains_activity(ActivityId::Service(job_id)),
                Job::Shipment(_) => {
                    route.contains_activity(ActivityId::ShipmentPickup(job_id))
                        || route.contains_activity(ActivityId::ShipmentDelivery(job_id))
                }
            })
            .map(|(index, _)| index)
    }

    pub fn route_of_activity(&self, activity_id: ActivityId) -> Option<RouteIdx> {
        self.routes
            .iter()
            .enumerate_idx()
            .find(|(_, route)| route.contains_activity(activity_id))
            .map(|(index, _)| index)
    }

    pub fn insert(&mut self, insertion: &Insertion) {
        match insertion {
            Insertion::Service(context) => {
                let route = &mut self.routes[context.route_id];
                let is_currently_empty = route.is_empty();
                let vehicle_id = route.vehicle_id;

                route.insert(&self.problem, insertion);
                self.unassigned_jobs.remove(&context.job_index);

                if is_currently_empty {
                    self.create_additional_route(vehicle_id);
                }
            }
            Insertion::Shipment(_context) => {
                unimplemented!()
            }
        }
    }

    pub fn remove_route_activity(&mut self, route_id: RouteIdx, activity_id: usize) {
        if route_id.get() >= self.routes.len() {
            return; // Invalid route ID
        }

        let route = &mut self.routes[route_id];
        if let Some(job_id) = route.remove(&self.problem, activity_id) {
            self.unassigned_jobs.insert(job_id.job_id());
        }
    }

    pub fn remove_activity(&mut self, activity_id: ActivityId) -> bool {
        let mut removed = false;
        for route in self.routes.iter_mut() {
            removed = route.remove_activity(&self.problem, activity_id);

            if removed {
                self.unassigned_jobs.insert(activity_id.job_id());
                break;
            }
        }

        removed
    }

    pub fn remove_service(&mut self, service_id: JobIdx) -> bool {
        self.remove_activity(ActivityId::Service(service_id))
    }

    pub fn remove_service_from_route(&mut self, route_id: usize, service_id: JobIdx) -> bool {
        let mut removed = false;
        let route = &mut self.routes[route_id];
        if route.contains_activity(ActivityId::Service(service_id)) {
            removed = route.remove_activity(&self.problem, ActivityId::Service(service_id));

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

        if !self.problem().fleet().is_infinite() {
            // Assert that all vehicle_id are different when not using an infinite fleet
            assert_eq!(
                self.routes
                    .iter()
                    .map(|route| route.vehicle_id())
                    .collect::<FxHashSet<_>>()
                    .len(),
                self.routes.len()
            );
        }
    }

    pub fn resync_route(&mut self, route_id: RouteIdx) {
        self.routes[route_id].resync(&self.problem);
    }

    pub fn remove_route(&mut self, route_id: RouteIdx) -> usize {
        let mut removed = 0;
        removed += self.routes[route_id].len();
        for job_id in self.routes[route_id].activity_ids.iter() {
            self.unassigned_jobs.insert(job_id.job_id());
        }

        self.routes[route_id].reset(&self.problem);

        removed
    }

    pub fn distance(&self) -> Meters {
        self.routes
            .iter()
            .map(|route| route.distance(&self.problem))
            .sum()
    }
}
