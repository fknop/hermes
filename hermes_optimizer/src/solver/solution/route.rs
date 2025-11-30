use fxhash::FxHashSet;
use jiff::{SignedDuration, Timestamp};

use crate::{
    problem::{
        amount::{Amount, AmountExpression},
        capacity::Capacity,
        job::{Job, JobId},
        service::{ServiceId, ServiceType},
        vehicle::{Vehicle, VehicleId},
        vehicle_routing_problem::VehicleRoutingProblem,
    },
    solver::solution::{
        activity::WorkingSolutionRouteActivity,
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

    pub(super) fwd_load_pickups: Vec<Capacity>,
    pub(super) fwd_load_deliveries: Vec<Capacity>,
    pub(super) fwd_load_shipments: Vec<Capacity>,
    pub(super) bwd_load_pickups: Vec<Capacity>,
    pub(super) bwd_load_deliveries: Vec<Capacity>,

    // fwd_load_peaks[i] stores the peak load up to step i
    // step 0 is the start depot
    pub(super) fwd_load_peaks: Vec<Capacity>,

    // bwd_load_peaks[i] stores the peak load from step i to the end
    // step 0 is the start depot
    pub(super) bwd_load_peaks: Vec<Capacity>,

    // current_load[i] stores the current load at step i
    // step 0 is the start depot
    pub(super) current_load: Vec<Capacity>,

    bbox: BBox,

    updated_in_iteration: bool,
}

impl WorkingSolutionRoute {
    pub fn empty(problem: &VehicleRoutingProblem, vehicle_id: VehicleId) -> Self {
        let mut route = WorkingSolutionRoute {
            vehicle_id,
            services: FxHashSet::default(),
            activities: Vec::new(),
            bbox: BBox::default(),
            updated_in_iteration: false,
            fwd_load_peaks: Vec::new(),
            bwd_load_peaks: Vec::new(),
            current_load: Vec::new(),
            bwd_load_deliveries: Vec::new(),
            bwd_load_pickups: Vec::new(),
            fwd_load_deliveries: Vec::new(),
            fwd_load_pickups: Vec::new(),
            fwd_load_shipments: Vec::new(),
        };

        route.resize_data();

        route
    }

    pub fn len(&self) -> usize {
        self.activities.len()
    }

    pub fn load_at(&self, position: usize) -> &Capacity {
        &self.current_load[position + 1]
    }

    pub fn bwd_load_peak(&self, i: usize) -> &Capacity {
        &self.bwd_load_peaks[i]
    }

    pub fn fwd_load_peak(&self, i: usize) -> &Capacity {
        &self.fwd_load_peaks[i]
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

    pub fn job_id_at(&self, position: usize) -> JobId {
        self.activities[position].job_id
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
        &self.current_load[0]
    }

    pub fn current_loads(&self) -> &[Capacity] {
        &self.current_load
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
        let vehicle = problem.vehicle(self.vehicle_id);
        let mut max_load = 0.0_f64;

        let vehicle_capacity = vehicle.capacity();

        for (index, demand) in self.fwd_load_peaks[self.len()].iter().enumerate() {
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
        self.update_data(problem);
        self.update_bbox(problem);
    }

    pub(super) fn resync(&mut self, problem: &VehicleRoutingProblem) {
        if !self.updated_in_iteration {
            return;
        }

        self.update_data(problem);
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

    fn resize_data(&mut self) {
        self.fwd_load_pickups
            .resize_with(self.len(), Capacity::empty);
        self.fwd_load_deliveries
            .resize_with(self.len(), Capacity::empty);
        self.bwd_load_deliveries
            .resize_with(self.len(), Capacity::empty);
        self.bwd_load_pickups
            .resize_with(self.len(), Capacity::empty);
        self.fwd_load_shipments
            .resize_with(self.len(), Capacity::empty);

        let steps = self.len() + 2;
        self.fwd_load_peaks.resize_with(steps, Capacity::empty);
        self.bwd_load_peaks.resize_with(steps, Capacity::empty);
        self.current_load.resize_with(steps, Capacity::empty);

        if self.is_empty() {
            self.fwd_load_peaks.fill_with(Capacity::empty);
            self.bwd_load_peaks.fill_with(Capacity::empty);
            self.current_load.fill_with(Capacity::empty);
        }
    }

    fn update_data(&mut self, problem: &VehicleRoutingProblem) {
        self.resize_data();

        if self.is_empty() {
            return;
        }

        let len = self.len();

        let mut current_load_pickups = Capacity::empty();
        let mut current_load_deliveries = Capacity::empty();
        let mut current_load_shipments = Capacity::empty();

        for i in 0..len {
            let (first, second) = self.activities.split_at_mut(i);
            let previous_activity = first.last();
            let current_activity = &mut second[0];
            let job_id = current_activity.job_id();
            let job = problem.job(job_id);

            match job_id {
                JobId::Service(_) => {
                    if let Job::Service(service) = job {
                        match service.service_type() {
                            ServiceType::Pickup => {
                                current_load_pickups += job.demand();
                            }
                            ServiceType::Delivery => {
                                current_load_deliveries += job.demand();
                            }
                        }
                    }
                }
                JobId::ShipmentPickup(_) => {
                    current_load_shipments += job.demand();
                }
                JobId::ShipmentDelivery(_) => {
                    current_load_shipments -= job.demand();
                }
            }

            self.fwd_load_pickups[i].update(&current_load_pickups);
            self.fwd_load_deliveries[i].update(&current_load_deliveries);
            self.fwd_load_shipments[i].update(&current_load_shipments);

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
                }
            }
        }

        assert!(self.fwd_load_shipments[self.len() - 1].is_empty());
        self.current_load[len + 1].update(&self.fwd_load_pickups[len - 1]);

        // Reset for the reverse pass
        current_load_deliveries.reset();
        current_load_pickups.reset();

        for i in (0..len).rev() {
            let job_id = self.activities[i].job_id;
            let job = problem.job(job_id);

            self.bwd_load_deliveries[i].update(&current_load_deliveries);
            self.bwd_load_pickups[i].update(&current_load_pickups);

            self.current_load[i + 1].update_expr(
                // Load from pickups + remaining load from shipments + remaining deliveries
                &self.fwd_load_pickups[i] + &self.fwd_load_shipments[i] + &current_load_deliveries,
            );

            if let Job::Service(service) = job {
                match service.service_type() {
                    ServiceType::Pickup => {
                        current_load_pickups += job.demand();
                    }
                    ServiceType::Delivery => {
                        current_load_deliveries += job.demand();
                    }
                }
            }
        }

        // The load at start is the load of all deliveries
        self.current_load[0].update(&current_load_deliveries);

        self.fwd_load_peaks[0].update(&self.current_load[0]);

        let mut peak = self.current_load[0].clone();
        self.fwd_load_peaks[0].update(&peak);
        for i in 1..self.fwd_load_peaks.len() {
            peak.update_max(&self.current_load[i]);

            self.fwd_load_peaks[i].update(&peak);
        }

        peak.update(&self.current_load[len + 1]);
        self.bwd_load_peaks[len + 1].update(&peak);
        for i in (0..self.bwd_load_peaks.len()).rev() {
            peak.update_max(&self.current_load[i]);
            self.bwd_load_peaks[i].update(&peak);
        }
    }

    pub fn random_activity<R>(&self, rng: &mut R) -> usize
    where
        R: rand::Rng,
    {
        rng.random_range(0..self.activities.len())
    }

    /// Returns an iterator over the job IDs in the route between the given [start, end) indices
    pub fn job_ids_iter(
        &self,
        start: usize,
        end: usize,
    ) -> impl DoubleEndedIterator<Item = JobId> + '_ {
        self.activities[start..end]
            .iter()
            .map(|activity| activity.job_id)
    }

    /// Checks whether inserting the given job IDs between the given [start, end) indices is valid
    pub fn is_valid_tw_change(
        &self,
        problem: &VehicleRoutingProblem,
        job_ids: impl Iterator<Item = JobId>,
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

        let succeeding_activities = if end < self.activities.len() {
            &self.activities[end..]
        } else {
            &[]
        };

        for job_id in job_ids.chain(
            succeeding_activities
                .iter()
                .map(|activity| activity.job_id()),
        ) {
            let service_id = job_id.into(); // TODO: handle shipments
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

        true
    }

    pub fn is_valid_capacity_change(
        &self,
        problem: &VehicleRoutingProblem,
        job_ids: impl Iterator<Item = JobId>,
        start: usize,
        end: usize,
    ) -> bool {
        if !problem.has_capacity() {
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

        let succeeding_activities = if end < self.activities.len() {
            &self.activities[end..]
        } else {
            &[]
        };

        for job_id in job_ids.chain(
            succeeding_activities
                .iter()
                .map(|activity| activity.job_id()),
        ) {
            let service_id = job_id.into(); // TODO: handle shipments
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

        true
    }
}

#[cfg(test)]
mod tests {

    use jiff::SignedDuration;

    use crate::{
        problem::{
            capacity::Capacity,
            service::{ServiceBuilder, ServiceType},
            time_window::TimeWindow,
            travel_cost_matrix::TravelCostMatrix,
            vehicle::VehicleBuilder,
            vehicle_routing_problem::VehicleRoutingProblemBuilder,
        },
        solver::solution::route::WorkingSolutionRoute,
        test_utils,
    };

    #[test]
    fn test_route_data_correctness() {
        // 10 locations from (0, 0) to (9, 0)
        let locations = test_utils::create_location_grid(1, 10);

        let mut vehicle_builder = VehicleBuilder::default();
        vehicle_builder.set_depot_location_id(0);
        vehicle_builder.set_capacity(Capacity::from_vec(vec![40.0]));
        vehicle_builder.set_vehicle_id(String::from("vehicle"));
        let vehicle = vehicle_builder.build();
        let vehicles = vec![vehicle];

        let mut service_builder = ServiceBuilder::default();
        service_builder.set_demand(Capacity::from_vec(vec![10.0]));
        service_builder.set_external_id(String::from("service_1"));
        service_builder.set_service_duration(SignedDuration::from_mins(10));
        service_builder.set_time_window(TimeWindow::from_iso(
            Some("2025-11-30T10:00:00+02:00"),
            Some("2025-11-30T12:00:00+02:00"),
        ));
        service_builder.set_location_id(1);
        let service_1 = service_builder.build();

        let mut service_builder = ServiceBuilder::default();
        service_builder.set_demand(Capacity::from_vec(vec![20.0]));
        service_builder.set_external_id(String::from("service_2"));
        service_builder.set_service_duration(SignedDuration::from_mins(10));
        service_builder.set_time_window(TimeWindow::from_iso(
            Some("2025-11-30T10:00:00+02:00"),
            Some("2025-11-30T12:00:00+02:00"),
        ));
        service_builder.set_location_id(2);
        let service_2 = service_builder.build();

        let mut service_builder = ServiceBuilder::default();
        service_builder.set_demand(Capacity::from_vec(vec![10.0]));
        service_builder.set_external_id(String::from("service_3"));
        service_builder.set_service_duration(SignedDuration::from_mins(10));
        service_builder.set_time_window(TimeWindow::from_iso(
            Some("2025-11-30T10:00:00+02:00"),
            Some("2025-11-30T12:00:00+02:00"),
        ));
        service_builder.set_location_id(3);
        service_builder.set_service_type(ServiceType::Pickup);
        let service_3 = service_builder.build();

        let services = vec![service_1, service_2, service_3];

        let mut builder = VehicleRoutingProblemBuilder::default();
        builder.set_travel_costs(TravelCostMatrix::from_constant(
            &locations,
            SignedDuration::from_mins(30).as_secs(),
            100.0,
            SignedDuration::from_mins(30).as_secs_f64(),
        ));
        builder.set_locations(locations);
        builder.set_vehicles(vehicles);
        builder.set_services(services);

        let problem = builder.build();

        let mut route = WorkingSolutionRoute::empty(&problem, 0);

        route.insert_service(&problem, 0, 0);
        route.insert_service(&problem, 1, 2);
        route.insert_service(&problem, 2, 1);

        assert_eq!(route.len(), 3);
        assert_eq!(route.current_load.len(), 5);
        assert_eq!(route.fwd_load_peaks.len(), 5);
        assert_eq!(route.bwd_load_peaks.len(), 5);
        assert_eq!(route.fwd_load_pickups.len(), 3);
        assert_eq!(route.fwd_load_deliveries.len(), 3);
        assert_eq!(route.fwd_load_shipments.len(), 3);
        assert_eq!(route.bwd_load_pickups.len(), 3);
        assert_eq!(route.bwd_load_deliveries.len(), 3);

        // Check fwd_load_deliveries
        assert_eq!(route.fwd_load_deliveries[0], Capacity::from_vec(vec![10.0]));
        assert_eq!(route.fwd_load_deliveries[1], Capacity::from_vec(vec![10.0]));
        assert_eq!(route.fwd_load_deliveries[2], Capacity::from_vec(vec![30.0]));

        // Check fwd_load_pickups
        assert_eq!(route.fwd_load_pickups[0], Capacity::empty());
        assert_eq!(route.fwd_load_pickups[1], Capacity::from_vec(vec![10.0]));
        assert_eq!(route.fwd_load_pickups[2], Capacity::from_vec(vec![10.0]));

        // Check bwd_load_deliveries
        assert_eq!(route.bwd_load_deliveries[0], Capacity::from_vec(vec![20.0]));
        assert_eq!(route.bwd_load_deliveries[1], Capacity::from_vec(vec![20.0]));
        assert_eq!(route.bwd_load_deliveries[2], Capacity::empty());

        // Check bwd_load_pickups
        assert_eq!(route.bwd_load_pickups[0], Capacity::from_vec(vec![10.0]));
        assert_eq!(route.bwd_load_pickups[1], Capacity::empty());
        assert_eq!(route.bwd_load_pickups[2], Capacity::empty());

        // Check fwd_load_shipments
        assert_eq!(route.fwd_load_shipments[0], Capacity::empty());
        assert_eq!(route.fwd_load_shipments[1], Capacity::empty());
        assert_eq!(route.fwd_load_shipments[2], Capacity::empty());

        // Check fwd_load_peaks
        assert_eq!(route.fwd_load_peaks[0], Capacity::from_vec(vec![30.0]));
        assert_eq!(route.fwd_load_peaks[1], Capacity::from_vec(vec![30.0]));
        assert_eq!(route.fwd_load_peaks[2], Capacity::from_vec(vec![30.0]));
        assert_eq!(route.fwd_load_peaks[3], Capacity::from_vec(vec![30.0]));
        assert_eq!(route.fwd_load_peaks[4], Capacity::from_vec(vec![30.0]));

        // Check bwd_load_peaks
        assert_eq!(route.bwd_load_peaks[0], Capacity::from_vec(vec![30.0]));
        assert_eq!(route.bwd_load_peaks[1], Capacity::from_vec(vec![30.0])); // Drop 10
        assert_eq!(route.bwd_load_peaks[2], Capacity::from_vec(vec![30.0])); // Pickup 10
        assert_eq!(route.bwd_load_peaks[3], Capacity::from_vec(vec![10.0])); // Drop 20
        assert_eq!(route.bwd_load_peaks[4], Capacity::from_vec(vec![10.0]));

        // Check current loads
        assert_eq!(route.current_load[0], Capacity::from_vec(vec![30.0])); // Start depot
        assert_eq!(route.current_load[1], Capacity::from_vec(vec![20.0])); // After service 1, drop 10
        assert_eq!(route.current_load[2], Capacity::from_vec(vec![30.0])); // After service 3, pickup 10
        assert_eq!(route.current_load[3], Capacity::from_vec(vec![10.0])); // After service 2, drop 20
        assert_eq!(route.current_load[4], Capacity::from_vec(vec![10.0])); // End depot

        // Check arrival times
        assert_eq!(
            route.activities[0].arrival_time,
            "2025-11-30T10:00:00+02:00".parse().unwrap()
        );
        assert_eq!(
            route.activities[1].arrival_time,
            "2025-11-30T10:40:00+02:00".parse().unwrap()
        );
        assert_eq!(
            route.activities[2].arrival_time,
            "2025-11-30T11:20:00+02:00".parse().unwrap()
        );

        // Check waiting durations
        assert_eq!(route.activities[0].waiting_duration, SignedDuration::ZERO);
        assert_eq!(route.activities[1].waiting_duration, SignedDuration::ZERO);
        assert_eq!(route.activities[2].waiting_duration, SignedDuration::ZERO);

        // Check departure times
        assert_eq!(
            route.activities[0].departure_time,
            "2025-11-30T10:10:00+02:00".parse().unwrap()
        );
        assert_eq!(
            route.activities[1].departure_time,
            "2025-11-30T10:50:00+02:00".parse().unwrap()
        );
        assert_eq!(
            route.activities[2].departure_time,
            "2025-11-30T11:30:00+02:00".parse().unwrap()
        )
    }
}
