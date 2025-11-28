use fxhash::FxHashSet;
use jiff::{SignedDuration, Timestamp};
use serde::Serialize;

use crate::{
    problem::{
        amount::AmountExpression,
        capacity::Capacity,
        job::{Job, JobId},
        service::{ServiceId, ServiceType},
        vehicle::{Vehicle, VehicleId},
        vehicle_routing_problem::VehicleRoutingProblem,
    },
    solver::solution::{
        activity::WorkingSolutionRouteActivity,
        route_job_id_iterator::RouteJobIdIterator,
        utils::{
            compute_activity_arrival_time, compute_activity_cumulative_load,
            compute_departure_time, compute_first_activity_arrival_time, compute_vehicle_end,
            compute_vehicle_start, compute_waiting_duration,
        },
    },
    utils::bbox::BBox,
};

#[derive(Clone)]
pub struct WorkingSolutionRoute {
    pub(super) vehicle_id: VehicleId,
    pub(super) services: FxHashSet<ServiceId>,
    pub(super) activities: Vec<WorkingSolutionRouteActivity>,

    // Current total demand of the route
    pub(super) total_initial_load: Capacity,

    bbox: BBox,

    updated_in_iteration: bool,
}

impl WorkingSolutionRoute {
    pub fn empty(vehicle_id: VehicleId) -> Self {
        WorkingSolutionRoute {
            vehicle_id,
            services: FxHashSet::default(),
            activities: Vec::new(),
            total_initial_load: Capacity::EMPTY,
            bbox: BBox::default(),
            updated_in_iteration: false,
        }
    }

    pub fn contains_service(&self, service_id: ServiceId) -> bool {
        self.services.contains(&service_id)
    }

    pub fn service_position(&self, service_id: ServiceId) -> Option<usize> {
        self.activities
            .iter()
            .position(|activity| activity.job_id == JobId::Service(service_id))
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
            let capacity = vehicle_capacity.get(index);
            if capacity == 0.0 && demand > 0.0 {
                max_load = 1.0;
            } else {
                let load = demand / capacity;
                max_load = max_load.max(load);
            }
        }

        max_load
    }

    pub fn location_id(&self, problem: &VehicleRoutingProblem, position: usize) -> Option<usize> {
        self.activities
            .get(position)
            .map(|activity| activity.service(problem).location_id())
    }

    pub fn previous_location_id(
        &self,
        problem: &VehicleRoutingProblem,
        position: usize,
    ) -> Option<usize> {
        if position == 0 {
            let vehicle = self.vehicle(problem);
            vehicle.depot_location_id()
        } else if position <= self.activities.len() {
            let previous_activity = &self.activities[position - 1];
            Some(previous_activity.service(problem).location_id())
        } else {
            None
        }
    }

    pub fn next_location_id(
        &self,
        problem: &VehicleRoutingProblem,
        position: usize,
    ) -> Option<usize> {
        let next_activity = self.activities.get(position + 1);

        match next_activity {
            Some(activity) => Some(activity.service(problem).location_id()),
            None => {
                let vehicle = self.vehicle(problem);
                if vehicle.should_return_to_depot() {
                    vehicle.depot_location_id()
                } else {
                    None
                }
            }
        }
    }

    pub fn remove_activity(
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

        self.services.remove(&activity.job_id.into());

        if activity.service(problem).service_type() == ServiceType::Delivery {
            self.total_initial_load -= activity.service(problem).demand();
        }

        self.activities.remove(activity_id);

        // self.update_next_activities(problem, activity_id);

        self.updated_in_iteration = true;

        Some(service_id)
    }

    pub fn remove_service(
        &mut self,
        problem: &VehicleRoutingProblem,
        service_id: ServiceId,
    ) -> bool {
        if !self.contains_service(service_id) {
            return false; // Service is not in the route
        }

        let activity_id = self
            .activities
            .iter()
            .position(|activity| activity.job_id == JobId::Service(service_id))
            .unwrap();

        self.remove_activity(problem, activity_id).is_some()
    }

    pub fn insert_service(
        &mut self,
        problem: &VehicleRoutingProblem,
        position: usize,
        service_id: ServiceId,
    ) {
        if self.services.contains(&service_id) {
            return;
        }

        self.services.insert(service_id);
        // let activity = WorkingSolutionRouteActivity::new(
        //     problem,
        //     service_id,
        //     if self.activities.is_empty() || position == 0 {
        //         compute_first_activity_arrival_time(problem, self.vehicle_id, service_id)
        //     } else {
        //         let previous_activity = &self.activities[position - 1];
        //         compute_activity_arrival_time(
        //             problem,
        //             previous_activity.service_id(),
        //             previous_activity.departure_time(),
        //             service_id,
        //         )
        //     },
        //     if self.activities().is_empty() || position == 0 {
        //         compute_activity_cumulative_load(problem.service(service_id), &Capacity::EMPTY)
        //     } else {
        //         let previous_activity = &self.activities[position - 1];
        //         compute_activity_cumulative_load(
        //             problem.service(service_id),
        //             &previous_activity.cumulative_load,
        //         )
        //     },
        // );

        self.activities
            .insert(position, WorkingSolutionRouteActivity::invalid(service_id));
        self.updated_in_iteration = true;

        let job = problem.job(service_id);

        if let Job::Service(service) = job
            && service.service_type() == ServiceType::Delivery
        {
            self.total_initial_load += problem.job(service_id).demand();
        }

        // Update the arrival times and departure times of subsequent activities
        self.update_activity_data(problem, position);
    }

    pub fn replace_activities(
        &mut self,
        problem: &VehicleRoutingProblem,
        job_ids: &[JobId],
        start: usize,
    ) {
        for (i, &job_id) in job_ids.iter().enumerate() {
            self.activities[start + i].job_id = job_id;
        }

        // Update the arrival times and departure times of subsequent activities
        self.update_activity_data(problem, start);
    }

    pub fn move_activity(&mut self, problem: &VehicleRoutingProblem, from: usize, to: usize) {
        if from >= self.activities.len() || to >= self.activities.len() || from == to {
            return;
        }

        let activity = self.activities.remove(from);
        self.activities
            .insert(if to > from { to - 1 } else { to }, activity);

        let start = from.min(to);
        self.update_activity_data(problem, start);
    }

    pub fn swap_activities(&mut self, problem: &VehicleRoutingProblem, i: usize, j: usize) {
        self.activities.swap(i, j);
        let start = i.min(j);

        self.update_activity_data(problem, start);
    }

    fn update_activity_data(&mut self, problem: &VehicleRoutingProblem, start: usize) {
        self.forward_update_pass(problem, start);
        self.backward_update_pass(problem);
        self.update_bbox(problem);
    }

    pub(super) fn resync(&mut self, problem: &VehicleRoutingProblem) {
        if !self.updated_in_iteration {
            return;
        }

        let mut total_initial_load = Capacity::EMPTY;

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
        self.update_bbox(problem);
    }

    fn update_bbox(&mut self, problem: &VehicleRoutingProblem) {
        let mut bbox = BBox::default();

        for activity in &self.activities {
            let location = problem.service_location(activity.service_id());
            bbox.extend(location);
        }

        self.bbox = bbox;
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
                            previous_activity.job_id.into(),
                            previous_activity.departure_time,
                            current_activity.job_id.into(),
                        ),
                    );

                    current_activity.cumulative_load = compute_activity_cumulative_load(
                        problem.service(current_activity.job_id.into()),
                        &previous_activity.cumulative_load,
                    );
                }
                None => {
                    current_activity.update_arrival_time(
                        problem,
                        compute_first_activity_arrival_time(
                            problem,
                            self.vehicle_id,
                            current_activity.job_id.into(),
                        ),
                    );

                    current_activity.cumulative_load = compute_activity_cumulative_load(
                        problem.service(current_activity.job_id.into()),
                        &Capacity::EMPTY,
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
                        current_activity.max_load_until_end = load.into()
                    }
                    _ => {
                        current_activity.max_load_until_end =
                            next_activity.max_load_until_end.clone();
                    }
                }
            }
            None => {
                current_activity.max_load_until_end =
                    (total_initial_load + &current_activity.cumulative_load).into();
            }
        }
    }

    pub fn random_activity<R>(&self, rng: &mut R) -> usize
    where
        R: rand::Rng,
    {
        rng.random_range(0..self.activities.len())
    }

    pub fn job_ids_iter(&self, start: usize, end: usize) -> impl Iterator<Item = JobId> + '_ {
        self.activities[start..end]
            .iter()
            .map(|activity| activity.job_id)

        // RouteJobIdIterator::new(self, start, end)
    }

    /// Checks whether inserting the given job IDs between the given [start, end) indices is valid
    pub fn is_valid_tw_change(
        &self,
        problem: &VehicleRoutingProblem,
        job_ids: impl Iterator<Item = usize>,
        start: usize,
        end: usize,
    ) -> bool {
        if !problem.has_time_windows() {
            return true;
        }

        let previous_activity = if start == 0 {
            None
        } else {
            Some(&self.activities[start - 1])
        };

        let mut previous_service_id = previous_activity.map(|activity| activity.service_id());
        let mut previous_departure_time =
            previous_activity.map(|activity| activity.departure_time());

        for service_id in job_ids {
            let arrival_time = if let Some(previous_service_id) = previous_service_id
                && let Some(previous_departure_time) = previous_departure_time
            {
                compute_activity_arrival_time(
                    problem,
                    previous_service_id,
                    previous_departure_time,
                    service_id,
                )
            } else {
                compute_first_activity_arrival_time(problem, self.vehicle_id, service_id)
            };

            let waiting_duration =
                compute_waiting_duration(problem.service(service_id), arrival_time);

            previous_service_id = Some(service_id);
            previous_departure_time = Some(compute_departure_time(
                problem,
                arrival_time,
                waiting_duration,
                service_id,
            ));

            let service = problem.service(service_id);

            if !service.time_windows_satisfied(arrival_time) {
                return false;
            }
        }

        for i in end..self.activities.len() {
            let next_service_id = self.activities[i].service_id();

            let arrival_time = if let Some(previous_service_id) = previous_service_id
                && let Some(previous_departure_time) = previous_departure_time
            {
                compute_activity_arrival_time(
                    problem,
                    previous_service_id,
                    previous_departure_time,
                    next_service_id,
                )
            } else {
                compute_first_activity_arrival_time(problem, self.vehicle_id, next_service_id)
            };

            let waiting_duration =
                compute_waiting_duration(problem.service(next_service_id), arrival_time);

            previous_service_id = Some(next_service_id);
            previous_departure_time = Some(compute_departure_time(
                problem,
                arrival_time,
                waiting_duration,
                next_service_id,
            ));

            let service = problem.service(next_service_id);

            if !service.time_windows_satisfied(arrival_time) {
                return false;
            }
        }

        true
    }
}
