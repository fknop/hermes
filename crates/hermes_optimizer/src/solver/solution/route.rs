use fxhash::FxHashMap;
use jiff::{SignedDuration, Timestamp};

use crate::{
    problem::{
        amount::AmountExpression,
        capacity::{Capacity, is_capacity_satisfied},
        job::{ActivityId, Job, JobTask},
        service::{Service, ServiceId, ServiceType},
        vehicle::{Vehicle, VehicleId},
        vehicle_routing_problem::VehicleRoutingProblem,
    },
    solver::{
        insertion::{Insertion, ServiceInsertion, ShipmentInsertion},
        solution::{
            route_update_iterator::RouteUpdateIterator,
            utils::{
                compute_activity_arrival_time, compute_departure_time,
                compute_first_activity_arrival_time, compute_time_slack, compute_vehicle_end,
                compute_vehicle_start, compute_waiting_duration,
            },
        },
    },
    utils::bbox::BBox,
};

#[derive(Clone)]
pub struct WorkingSolutionRoute {
    pub(super) vehicle_id: VehicleId,
    // Map of JobId to index in activities vector
    pub(super) jobs: FxHashMap<ActivityId, usize>,

    /// List of activity job IDs in the route order
    pub(super) activity_ids: Vec<ActivityId>,

    /// List of arrival times at each activity
    pub(super) arrival_times: Vec<Timestamp>,

    /// List of departure times at each activity
    pub(super) departure_times: Vec<Timestamp>,

    /// List of waiting durations at each activity
    pub(super) waiting_durations: Vec<SignedDuration>,

    /// Forward load for pickups at each activity
    pub(super) fwd_load_pickups: Vec<Capacity>,

    /// Forward load for deliveries at each activity
    pub(super) fwd_load_deliveries: Vec<Capacity>,

    /// Forward load for shipments at each activity
    pub(super) fwd_load_shipments: Vec<Capacity>,

    /// Backward load for pickups at each activity
    pub(super) bwd_load_pickups: Vec<Capacity>,

    /// Backward load for deliveries at each activity
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

    /// Total available capacity for the route
    pub(super) delivery_load_slack: Capacity,

    /// Total pickup capacity available for the route
    pub(super) pickup_load_slack: Capacity,

    // time_slacks[i] stores the maximum time delay that can be absorbed at activity i
    // before violating time windows of subsequent activities
    // computed backward from end to start
    pub(super) time_slacks: Vec<SignedDuration>,

    bbox: BBox,

    updated_in_iteration: bool,
}

impl WorkingSolutionRoute {
    pub fn empty(problem: &VehicleRoutingProblem, vehicle_id: VehicleId) -> Self {
        let mut route = WorkingSolutionRoute {
            vehicle_id,
            jobs: FxHashMap::default(),
            bbox: BBox::default(),
            updated_in_iteration: false,

            activity_ids: Vec::new(),
            arrival_times: Vec::new(),
            departure_times: Vec::new(),
            waiting_durations: Vec::new(),
            fwd_load_peaks: Vec::new(),
            bwd_load_peaks: Vec::new(),
            current_load: Vec::new(),
            bwd_load_deliveries: Vec::new(),
            bwd_load_pickups: Vec::new(),
            fwd_load_deliveries: Vec::new(),
            fwd_load_pickups: Vec::new(),
            fwd_load_shipments: Vec::new(),
            time_slacks: Vec::new(),
            delivery_load_slack: problem.vehicle(vehicle_id).capacity().clone(),
            pickup_load_slack: problem.vehicle(vehicle_id).capacity().clone(),
        };

        route.resize_data(problem);

        route
    }

    pub fn len(&self) -> usize {
        self.activity_ids.len()
    }

    pub fn is_empty(&self) -> bool {
        self.activity_ids.is_empty()
    }

    pub fn reset(&mut self, problem: &VehicleRoutingProblem) {
        self.jobs.clear();
        self.activity_ids.clear();
        self.bbox = BBox::default();

        self.update_activity_data(problem);
    }

    pub fn bbox_intersects(&self, other: &WorkingSolutionRoute) -> bool {
        if self.is_empty() || other.is_empty() {
            return false; // TODO: build this into bbox properly
        }

        self.bbox.intersects(&other.bbox)
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

    pub fn delivery_load_slack(&self) -> &Capacity {
        &self.delivery_load_slack
    }

    pub fn pickup_load_slack(&self) -> &Capacity {
        &self.pickup_load_slack
    }

    pub fn contains_job(&self, job_id: ActivityId) -> bool {
        self.jobs.contains_key(&job_id)
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

        for &job_id in &self.activity_ids {
            location_ids.push(problem.job_task(job_id).location_id());
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
            first.job_id(),
            self.arrival_times[0],
        )
    }

    pub fn end(&self, problem: &VehicleRoutingProblem) -> Timestamp {
        let last = self.last();
        compute_vehicle_end(
            problem,
            self.vehicle_id,
            last.job_id(),
            self.departure_times[self.len() - 1],
        )
    }

    pub fn job_id(&self, position: usize) -> ActivityId {
        self.activity_ids[position]
    }

    pub fn job_position(&self, job_id: ActivityId) -> Option<usize> {
        self.jobs.get(&job_id).copied()
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
                    vehicle,
                    depot_location_id,
                    self.first().job_task(problem).location_id(),
                );
            }

            if self.has_end(problem) {
                transport_duration += problem.travel_time(
                    vehicle,
                    self.last().job_task(problem).location_id(),
                    depot_location_id,
                );
            }
        }

        for (index, &job_id) in self.activity_ids.iter().enumerate() {
            if index == 0 {
                // Skip the first activity, as it is already counted with the depot
                continue;
            }

            transport_duration += problem.travel_time(
                vehicle,
                problem.job_task(self.activity_ids[index - 1]).location_id(),
                problem.job_task(job_id).location_id(),
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
                    vehicle,
                    depot_location_id,
                    self.first().job_task(problem).location_id(),
                );
            }

            if self.has_end(problem) {
                costs += problem.travel_cost(
                    vehicle,
                    self.last().job_task(problem).location_id(),
                    depot_location_id,
                );
            }
        }

        for (index, &activity) in self.activity_ids.iter().enumerate() {
            if index == 0 {
                // Skip the first activity, as it is already counted with the depot
                continue;
            }

            costs += problem.travel_cost(
                vehicle,
                problem.job_task(self.activity_ids[index - 1]).location_id(),
                problem.job_task(activity).location_id(),
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
                    vehicle,
                    depot_location_id,
                    self.first().job_task(problem).location_id(),
                );
            }

            if self.has_end(problem) {
                distance += problem.travel_distance(
                    vehicle,
                    self.last().job_task(problem).location_id(),
                    depot_location_id,
                );
            }
        }

        for (index, &job_id) in self.activity_ids.iter().enumerate() {
            if index == 0 {
                // Skip the first activity, as it is already counted with the depot
                continue;
            }

            distance += problem.travel_distance(
                vehicle,
                problem.job_task(self.activity_ids[index - 1]).location_id(),
                problem.job_task(job_id).location_id(),
            );
        }

        distance
    }

    pub fn first(&self) -> RouteActivityInfo {
        assert!(
            !self.is_empty(),
            "cannot call WorkingSolutionRoute::first() on empty route"
        );
        self.activity(0)
    }

    pub fn last(&self) -> RouteActivityInfo {
        assert!(
            !self.is_empty(),
            "cannot call WorkingSolutionRoute::last() on empty route"
        );
        self.activity(self.len() - 1)
    }

    pub fn activity_ids(&self) -> &[ActivityId] {
        &self.activity_ids
    }

    pub fn activity_id(&self, index: usize) -> ActivityId {
        self.activity_ids[index]
    }

    pub fn activities_iter(&self) -> impl Iterator<Item = RouteActivityInfo> {
        self.activity_ids
            .iter()
            .enumerate()
            .map(move |(index, _)| self.activity(index))
    }

    pub fn activity(&self, index: usize) -> RouteActivityInfo {
        assert!(
            !self.is_empty(),
            "cannot call WorkingSolutionRoute::activity() on empty route"
        );

        RouteActivityInfo {
            job_id: self.activity_ids[index],
            arrival_time: self.arrival_times[index],
            departure_time: self.departure_times[index],
            waiting_duration: self.waiting_durations[index],
        }
    }

    pub fn arrival_time(&self, index: usize) -> Timestamp {
        self.arrival_times[index]
    }

    pub fn departure_time(&self, index: usize) -> Timestamp {
        self.departure_times[index]
    }

    pub fn waiting_duration(&self, index: usize) -> SignedDuration {
        self.waiting_durations[index]
    }

    pub fn total_initial_load(&self) -> &Capacity {
        &self.current_load[0]
    }

    pub fn current_loads(&self) -> &[Capacity] {
        &self.current_load
    }

    pub fn total_waiting_duration(&self) -> SignedDuration {
        self.waiting_durations.iter().sum()
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
        self.activity_ids
            .get(position)
            .map(|&job_id| problem.job_task(job_id).location_id())
    }

    pub fn previous_location_id(
        &self,
        problem: &VehicleRoutingProblem,
        position: usize,
    ) -> Option<usize> {
        if position == 0 {
            let vehicle = self.vehicle(problem);
            vehicle.depot_location_id()
        } else if position <= self.activity_ids.len() {
            Some(
                problem
                    .job_task(self.activity_ids[position - 1])
                    .location_id(),
            )
        } else {
            None
        }
    }

    pub fn next_location_id(
        &self,
        problem: &VehicleRoutingProblem,
        position: usize,
    ) -> Option<usize> {
        let next_job_id = self.activity_ids.get(position + 1);

        match next_job_id {
            Some(&job_id) => Some(problem.job_task(job_id).location_id()),
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

    fn remove(&mut self, position: usize) -> Option<ActivityId> {
        if position >= self.activity_ids.len() {
            return None;
        }

        let job_id = self.activity_ids[position];
        self.activity_ids.remove(position);
        Some(job_id)
    }

    pub fn remove_activity(
        &mut self,
        problem: &VehicleRoutingProblem,
        position: usize,
    ) -> Option<ActivityId> {
        if let Some(job_id) = self.remove(position) {
            self.jobs.remove(&job_id);
            for (index, &job_id) in self.activity_ids.iter().skip(position).enumerate() {
                self.jobs.insert(job_id, index + position);
            }

            self.updated_in_iteration = true;

            Some(job_id)
        } else {
            None
        }
    }

    pub fn remove_job(&mut self, problem: &VehicleRoutingProblem, job_id: ActivityId) -> bool {
        if !self.contains_job(job_id) {
            return false; // Service is not in the route
        }

        if let Some(&activity_id) = self.jobs.get(&job_id) {
            match job_id {
                ActivityId::Service(_) => self.remove_activity(problem, activity_id).is_some(),
                ActivityId::ShipmentPickup(id) => {
                    let delivery = self.jobs.get(&ActivityId::ShipmentDelivery(id));
                    self.remove_activity(problem, *delivery.unwrap());
                    self.remove_activity(problem, activity_id).is_some()
                }
                ActivityId::ShipmentDelivery(id) => {
                    self.remove_activity(problem, activity_id);
                    let pickup = self.jobs.get(&ActivityId::ShipmentPickup(id));
                    self.remove_activity(problem, *pickup.unwrap()).is_some()
                }
            }
        } else {
            false
        }
    }

    pub fn insert(&mut self, problem: &VehicleRoutingProblem, insertion: &Insertion) {
        match insertion {
            Insertion::Service(ServiceInsertion {
                position,
                job_index,
                ..
            }) => {
                self.insert_service(problem, *position, *job_index);
            }
            Insertion::Shipment(ShipmentInsertion {
                route_id,
                job_index,
                delivery_position,
                ..
            }) => {
                unimplemented!()
            }
        }
    }

    fn insert_service(
        &mut self,
        problem: &VehicleRoutingProblem,
        position: usize,
        service_id: ServiceId,
    ) {
        if self.jobs.contains_key(&ActivityId::Service(service_id)) {
            return;
        }

        self.activity_ids
            .insert(position, ActivityId::Service(service_id));
        self.updated_in_iteration = true;

        // Update the arrival times and departure times of subsequent activities
        self.update_activity_data(problem);
    }

    pub fn replace_activities(
        &mut self,
        problem: &VehicleRoutingProblem,
        job_ids: &[ActivityId],
        start: usize,
        end: usize,
    ) {
        self.activity_ids
            .splice(start..end, job_ids.iter().copied());

        // Update the arrival times and departure times of subsequent activities
        self.update_activity_data(problem);
    }

    pub fn move_activity(&mut self, problem: &VehicleRoutingProblem, from: usize, to: usize) {
        if from > self.activity_ids.len() || to > self.activity_ids.len() || from == to {
            return;
        }

        let activity = self.activity_ids.remove(from);
        self.activity_ids
            .insert(if to > from { to - 1 } else { to }, activity);

        self.update_activity_data(problem);
    }

    pub fn swap_activities(&mut self, problem: &VehicleRoutingProblem, i: usize, j: usize) {
        self.activity_ids.swap(i, j);

        self.update_activity_data(problem);
    }

    fn update_activity_data(&mut self, problem: &VehicleRoutingProblem) {
        self.jobs.clear();
        self.jobs.extend(
            self.activity_ids
                .iter()
                .enumerate()
                .map(|(index, &job_id)| (job_id, index)),
        );
        self.update_data(problem);
        self.update_bbox(problem);
    }

    pub(super) fn resync(&mut self, problem: &VehicleRoutingProblem) {
        if !self.updated_in_iteration {
            return;
        }

        self.update_activity_data(problem);
    }

    fn update_bbox(&mut self, problem: &VehicleRoutingProblem) {
        let mut bbox = BBox::default();

        for &job_id in &self.activity_ids {
            let location_id = problem.job_task(job_id).location_id();
            let location = problem.location(location_id);
            bbox.extend(location);
        }

        self.bbox = bbox;
    }

    fn resize_data(&mut self, problem: &VehicleRoutingProblem) {
        self.arrival_times.resize(self.len(), Timestamp::MAX);
        self.departure_times.resize(self.len(), Timestamp::MAX);
        self.waiting_durations
            .resize(self.len(), SignedDuration::ZERO);
        self.time_slacks.resize(self.len(), SignedDuration::MAX);

        self.fwd_load_pickups.resize_with(self.len(), || {
            Capacity::with_dimensions(problem.capacity_dimensions())
        });
        self.fwd_load_deliveries.resize_with(self.len(), || {
            Capacity::with_dimensions(problem.capacity_dimensions())
        });
        self.bwd_load_deliveries.resize_with(self.len(), || {
            Capacity::with_dimensions(problem.capacity_dimensions())
        });
        self.bwd_load_pickups.resize_with(self.len(), || {
            Capacity::with_dimensions(problem.capacity_dimensions())
        });
        self.fwd_load_shipments.resize_with(self.len(), || {
            Capacity::with_dimensions(problem.capacity_dimensions())
        });

        let steps = self.len() + 2;
        self.fwd_load_peaks.resize_with(steps, || {
            Capacity::with_dimensions(problem.capacity_dimensions())
        });
        self.bwd_load_peaks.resize_with(steps, || {
            Capacity::with_dimensions(problem.capacity_dimensions())
        });
        self.current_load.resize_with(steps, || {
            Capacity::with_dimensions(problem.capacity_dimensions())
        });

        if self.is_empty() {
            self.fwd_load_peaks
                .fill_with(|| Capacity::with_dimensions(problem.capacity_dimensions()));
            self.bwd_load_peaks
                .fill_with(|| Capacity::with_dimensions(problem.capacity_dimensions()));
            self.current_load
                .fill_with(|| Capacity::with_dimensions(problem.capacity_dimensions()));
        }
    }

    fn update_data(&mut self, problem: &VehicleRoutingProblem) {
        self.resize_data(problem);

        if self.is_empty() {
            self.delivery_load_slack
                .update(self.vehicle(problem).capacity());
            self.pickup_load_slack
                .update(self.vehicle(problem).capacity());
            return;
        }

        let len = self.len();

        let mut current_load_pickups = Capacity::with_dimensions(problem.capacity_dimensions());
        let mut current_load_deliveries = Capacity::with_dimensions(problem.capacity_dimensions());
        let mut current_load_shipments = Capacity::with_dimensions(problem.capacity_dimensions());

        for i in 0..len {
            let job_id = self.activity_ids[i];
            let job = problem.job(job_id);

            match job_id {
                ActivityId::Service(_) => {
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
                ActivityId::ShipmentPickup(_) => {
                    current_load_shipments += job.demand();
                }
                ActivityId::ShipmentDelivery(_) => {
                    current_load_shipments -= job.demand();
                }
            }

            self.fwd_load_pickups[i].update(&current_load_pickups);
            self.fwd_load_deliveries[i].update(&current_load_deliveries);
            self.fwd_load_shipments[i].update(&current_load_shipments);

            self.arrival_times[i] = if i == 0 {
                compute_first_activity_arrival_time(problem, self.vehicle_id, job_id)
            } else {
                compute_activity_arrival_time(
                    problem,
                    self.vehicle_id,
                    self.activity_ids[i - 1],
                    self.departure_times[i - 1],
                    job_id,
                )
            };

            self.waiting_durations[i] =
                compute_waiting_duration(problem, job_id, self.arrival_times[i]);
            self.departure_times[i] = compute_departure_time(
                problem,
                self.arrival_times[i],
                self.waiting_durations[i],
                job_id,
            );
        }

        assert!(self.fwd_load_shipments[self.len() - 1].is_empty());
        self.current_load[len + 1].update(&self.fwd_load_pickups[len - 1]);

        // Reset for the reverse pass
        current_load_deliveries.reset();
        current_load_pickups.reset();

        for i in (0..len).rev() {
            let job_id = self.activity_ids[i];
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

        let vehicle_capacity = self.vehicle(problem).capacity();

        self.delivery_load_slack
            .update_expr(vehicle_capacity - &self.current_load[0]);
        self.pickup_load_slack
            .update_expr(vehicle_capacity - &self.current_load[self.len()]);

        if problem.has_time_windows() {
            let mut next_activity_time_slack = SignedDuration::MAX;
            for (index, &activity_job_id) in self.activity_ids.iter().enumerate().rev() {
                assert_ne!(self.arrival_times[index], Timestamp::MAX);
                let current_time_slack =
                    compute_time_slack(problem, activity_job_id, self.arrival_times[index]);

                self.time_slacks[index] = current_time_slack.min(next_activity_time_slack);

                next_activity_time_slack = self.time_slacks[index];
            }
        } else {
            self.time_slacks.fill(SignedDuration::MAX);
        }
    }

    pub fn random_activity<R>(&self, rng: &mut R) -> usize
    where
        R: rand::Rng,
    {
        rng.random_range(0..self.activity_ids.len())
    }

    pub fn job_ids(&self, start: usize, end: usize) -> &[ActivityId] {
        &self.activity_ids[start..end]
    }

    /// Returns an iterator over the job IDs in the route between the given [start, end) indices
    pub fn job_ids_iter(
        &self,
        start: usize,
        end: usize,
    ) -> impl DoubleEndedIterator<Item = ActivityId> + Clone + '_ {
        if end > self.activity_ids.len() || start > self.activity_ids.len() || start > end {
            println!("{} -> {}", start, end)
        }

        self.activity_ids[start..end].iter().copied()
    }

    pub fn updated_activities_iter<'a, I>(
        &'a self,
        problem: &'a VehicleRoutingProblem,
        jobs_iter: I,
        start: usize,
        end: usize,
    ) -> RouteUpdateIterator<'a, I>
    where
        I: Iterator<Item = ActivityId>,
    {
        RouteUpdateIterator::new(problem, self, jobs_iter, start, end)
    }

    pub fn is_valid_change(
        &self,
        problem: &VehicleRoutingProblem,
        job_ids: impl Iterator<Item = ActivityId> + Clone,
        start: usize,
        end: usize,
    ) -> bool {
        let is_valid_tw_change = self.is_valid_tw_change(problem, job_ids.clone(), start, end);
        let is_valid_capacity_change = self.is_valid_capacity_change(problem, job_ids, start, end);

        is_valid_tw_change && is_valid_capacity_change
    }

    /// Checks whether inserting the given job IDs between the given [start, end) indices is valid
    pub fn is_valid_tw_change(
        &self,
        problem: &VehicleRoutingProblem,
        job_ids: impl Iterator<Item = ActivityId>,
        start: usize,
        end: usize,
    ) -> bool {
        if !problem.has_time_windows() {
            return true;
        }

        let mut previous_job_id = if start == 0 {
            None
        } else {
            Some(self.activity_ids[start - 1])
        };

        let mut previous_departure_time = if start == 0 {
            None
        } else {
            Some(self.departure_times[start - 1])
        };

        let succeeding_activities = if end < self.activity_ids.len() {
            &self.activity_ids[end..]
        } else {
            &[]
        };

        for job_id in job_ids.chain(succeeding_activities.iter().copied()) {
            let arrival_time = if let Some(previous_job_id) = previous_job_id
                && let Some(previous_departure_time) = previous_departure_time
            {
                compute_activity_arrival_time(
                    problem,
                    self.vehicle_id,
                    previous_job_id,
                    previous_departure_time,
                    job_id,
                )
            } else {
                compute_first_activity_arrival_time(problem, self.vehicle_id, job_id)
            };

            let waiting_duration = compute_waiting_duration(problem, job_id, arrival_time);

            previous_job_id = Some(job_id);
            let new_departure_time =
                compute_departure_time(problem, arrival_time, waiting_duration, job_id);
            previous_departure_time = Some(new_departure_time);

            if let Some(&current_activity_index) = self.jobs.get(&job_id)
                && current_activity_index >= end
            {
                let current_time_slack = self.time_slacks[current_activity_index];
                let current_arrival_time = self.arrival_times[current_activity_index];

                if arrival_time
                    > current_arrival_time
                        .saturating_add(current_time_slack)
                        .unwrap()
                {
                    return false;
                }

                let current_departure_time = self.departure_times[current_activity_index];
                // Early termination, if the departure time is earlier or equal to the current one, we know the next one are valid as well
                if new_departure_time <= current_departure_time {
                    return true;
                }
            }

            if !problem
                .job_task(job_id)
                .time_windows_satisfied(arrival_time)
            {
                return false;
            }
        }

        true
    }

    pub fn is_valid_capacity_change(
        &self,
        problem: &VehicleRoutingProblem,
        job_ids: impl Iterator<Item = ActivityId>,
        start: usize,
        end: usize,
    ) -> bool {
        let mut added_delivery_load = Capacity::with_dimensions(problem.capacity_dimensions());
        let mut added_pickup_load = Capacity::with_dimensions(problem.capacity_dimensions());

        let mut delivery_load_delta = Capacity::with_dimensions(problem.capacity_dimensions());
        let mut pickup_load_delta = Capacity::with_dimensions(problem.capacity_dimensions());
        let mut shipment_load_delta = Capacity::with_dimensions(problem.capacity_dimensions());

        for job_id in job_ids {
            let job = problem.job(job_id);

            match job_id {
                ActivityId::Service(_) => {
                    if let Job::Service(service) = job {
                        match service.service_type() {
                            ServiceType::Pickup => {
                                pickup_load_delta += job.demand();
                                added_pickup_load += job.demand();
                            }
                            ServiceType::Delivery => {
                                delivery_load_delta += job.demand();
                                added_delivery_load += job.demand();
                            }
                        }
                    }
                }
                ActivityId::ShipmentPickup(_) => {
                    shipment_load_delta += job.demand();
                }
                ActivityId::ShipmentDelivery(_) => {
                    shipment_load_delta -= job.demand();
                }
            }
        }

        if start < end && end <= self.len() {
            if start > 0 {
                delivery_load_delta -=
                    &self.fwd_load_deliveries[end - 1] - &self.fwd_load_deliveries[start - 1];
                pickup_load_delta -=
                    &self.fwd_load_pickups[end - 1] - &self.fwd_load_pickups[start - 1];
            } else {
                delivery_load_delta -= &self.fwd_load_deliveries[end - 1];
                pickup_load_delta -= &self.fwd_load_pickups[end - 1];
            }
        }

        let new_initial_load = &self.current_load[0] + &delivery_load_delta;

        let vehicle = self.vehicle(problem);

        // Check the new initial load against vehicle capacity
        if !is_capacity_satisfied(vehicle.capacity(), &new_initial_load) {
            return false;
        }

        // Check 2: Peak load before insertion point [0, start]
        // The loads before start are increased by delivery_load_delta (more deliveries to carry)
        if start > 0 {
            let peak_load_before_insertion = &self.fwd_load_peaks[start] + &delivery_load_delta;
            if !is_capacity_satisfied(vehicle.capacity(), &peak_load_before_insertion) {
                return false;
            }
        }

        // let load_at_start = &self.current_load[start] + &delivery_load_delta;
        // if !is_capacity_satisfied(vehicle.capacity(), &load_at_start) {
        //     println!("load at start not satisfied");
        //     return false;
        // }

        // if !is_capacity_satisfied(vehicle.capacity(), &(load_at_start + &pickup_load_delta)) {
        //     println!("pickup load not satisfied");
        //     return false;
        // }

        let load_at_end = &self.current_load[start] + &delivery_load_delta + &pickup_load_delta
            - &added_delivery_load
            - &self.current_load[end];

        if !is_capacity_satisfied(
            vehicle.capacity(),
            &(load_at_end + &self.bwd_load_peaks[end]),
        ) {
            return false;
        }

        true
    }

    pub fn can_route_capacity_fit_in(&self, problem: &VehicleRoutingProblem, other: &Self) -> bool {
        if self.vehicle_id == other.vehicle_id {
            return false;
        }

        let other_vehicle_capacity = other.vehicle(problem).capacity();
        let self_delivery_peak = &self.current_load[0];
        let self_pickup_peak = &self.current_load[self.len()];

        if !is_capacity_satisfied(
            other_vehicle_capacity,
            &(other.delivery_load_slack() + self_delivery_peak),
        ) {
            return false;
        }

        if !is_capacity_satisfied(
            other_vehicle_capacity,
            &(other.pickup_load_slack() + self_pickup_peak),
        ) {
            return false;
        }

        true
    }
}

pub struct RouteActivityInfo {
    pub(super) job_id: ActivityId,
    pub(super) arrival_time: Timestamp,
    pub(super) departure_time: Timestamp,
    pub(super) waiting_duration: SignedDuration,
}

impl RouteActivityInfo {
    pub fn departure_time(&self) -> Timestamp {
        self.departure_time
    }

    pub fn arrival_time(&self) -> Timestamp {
        self.arrival_time
    }

    pub fn waiting_duration(&self) -> SignedDuration {
        self.waiting_duration
    }

    pub fn job_id(&self) -> ActivityId {
        self.job_id
    }

    #[deprecated]
    pub fn service<'a>(&self, problem: &'a VehicleRoutingProblem) -> &'a Service {
        problem.service(self.job_id.into())
    }

    pub fn job<'a>(&self, problem: &'a VehicleRoutingProblem) -> &'a Job {
        problem.job(self.job_id.index())
    }

    pub fn job_task<'a>(&self, problem: &'a VehicleRoutingProblem) -> JobTask<'a> {
        problem.job_task(self.job_id)
    }
}

#[cfg(test)]
mod tests {

    use jiff::SignedDuration;

    use crate::{
        problem::{
            capacity::Capacity,
            job::ActivityId,
            service::{ServiceBuilder, ServiceType},
            time_window::TimeWindow,
            travel_cost_matrix::TravelMatrices,
            vehicle::VehicleBuilder,
            vehicle_profile::VehicleProfile,
            vehicle_routing_problem::{VehicleRoutingProblem, VehicleRoutingProblemBuilder},
        },
        solver::solution::{
            route::WorkingSolutionRoute, route_update_iterator::RouteUpdateActivityData,
        },
        test_utils,
    };

    fn create_problem() -> VehicleRoutingProblem {
        // 10 locations from (0, 0) to (9, 0)
        let locations = test_utils::create_location_grid(1, 10);

        let mut vehicle_builder = VehicleBuilder::default();
        vehicle_builder.set_depot_location_id(0);
        vehicle_builder.set_capacity(Capacity::from_vec(vec![40.0]));
        vehicle_builder.set_vehicle_id(String::from("vehicle"));
        vehicle_builder.set_profile_id(0);
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

        builder.set_vehicle_profiles(vec![VehicleProfile::new(
            "test_profile".to_owned(),
            TravelMatrices::from_constant(
                &locations,
                SignedDuration::from_mins(30).as_secs_f64(),
                100.0,
                SignedDuration::from_mins(30).as_secs_f64(),
            ),
        )]);

        builder.set_locations(locations);
        builder.set_vehicles(vehicles);
        builder.set_services(services);

        builder.build()
    }

    fn create_problem_for_capacity_change(
        vehicle_capacity: Capacity,
        services: Vec<(ServiceType, Capacity)>,
    ) -> VehicleRoutingProblem {
        // 10 locations from (0, 0) to (9, 0)
        let locations = test_utils::create_location_grid(1, 10);

        let mut vehicle_builder = VehicleBuilder::default();
        vehicle_builder.set_depot_location_id(0);
        vehicle_builder.set_capacity(vehicle_capacity);
        vehicle_builder.set_vehicle_id(String::from("vehicle"));
        vehicle_builder.set_profile_id(0);
        let vehicle = vehicle_builder.build();
        let vehicles = vec![vehicle];

        let services = services
            .into_iter()
            .enumerate()
            .map(|(i, (service_type, demand))| {
                let mut service_builder = ServiceBuilder::default();
                service_builder.set_demand(demand);
                service_builder.set_external_id(format!("service_{}", i + 1));
                service_builder.set_service_duration(SignedDuration::from_mins(10));
                service_builder.set_location_id(i + 1);
                service_builder.set_service_type(service_type);
                service_builder.build()
            })
            .collect::<Vec<_>>();

        let mut builder = VehicleRoutingProblemBuilder::default();
        builder.set_vehicle_profiles(vec![VehicleProfile::new(
            "test_profile".to_owned(),
            TravelMatrices::from_constant(
                &locations,
                SignedDuration::from_mins(30).as_secs_f64(),
                100.0,
                SignedDuration::from_mins(30).as_secs_f64(),
            ),
        )]);
        builder.set_locations(locations);
        builder.set_vehicles(vehicles);
        builder.set_services(services);

        builder.build()
    }

    #[test]
    fn test_route_data_correctness() {
        let problem = create_problem();

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

        assert_eq!(route.delivery_load_slack, Capacity::from_vec(vec![10.0]));
        assert_eq!(route.pickup_load_slack, Capacity::from_vec(vec![30.0]));

        // Check arrival times
        assert_eq!(
            route.arrival_times[0],
            "2025-11-30T10:00:00+02:00".parse().unwrap()
        );
        assert_eq!(
            route.arrival_times[1],
            "2025-11-30T10:40:00+02:00".parse().unwrap()
        );
        assert_eq!(
            route.arrival_times[2],
            "2025-11-30T11:20:00+02:00".parse().unwrap()
        );

        // Check waiting durations
        assert_eq!(route.waiting_durations[0], SignedDuration::ZERO);
        assert_eq!(route.waiting_durations[1], SignedDuration::ZERO);
        assert_eq!(route.waiting_durations[2], SignedDuration::ZERO);

        // Check departure times
        assert_eq!(
            route.departure_times[0],
            "2025-11-30T10:10:00+02:00".parse().unwrap()
        );
        assert_eq!(
            route.departure_times[1],
            "2025-11-30T10:50:00+02:00".parse().unwrap()
        );
        assert_eq!(
            route.departure_times[2],
            "2025-11-30T11:30:00+02:00".parse().unwrap()
        );

        // Check time slacks
        assert_eq!(route.time_slacks[0], SignedDuration::from_mins(40));
        assert_eq!(route.time_slacks[1], SignedDuration::from_mins(40));
        assert_eq!(route.time_slacks[2], SignedDuration::from_mins(40))
    }

    #[test]
    fn test_jobs_iter() {
        let problem = create_problem();

        let mut route = WorkingSolutionRoute::empty(&problem, 0);

        route.insert_service(&problem, 0, 0);
        route.insert_service(&problem, 1, 2);
        route.insert_service(&problem, 2, 1);

        let job_ids: Vec<ActivityId> = route.job_ids_iter(0, 3).collect();

        assert_eq!(
            job_ids,
            vec![
                ActivityId::Service(0),
                ActivityId::Service(2),
                ActivityId::Service(1)
            ]
        );

        // Same value returns empty iterator
        let job_ids: Vec<ActivityId> = route.job_ids_iter(0, 0).collect();
        assert_eq!(job_ids, vec![]);
    }

    #[test]
    fn test_route_update_iter() {
        let problem = create_problem();

        let mut route = WorkingSolutionRoute::empty(&problem, 0);

        route.insert_service(&problem, 0, 0);
        route.insert_service(&problem, 1, 2);
        route.insert_service(&problem, 2, 1);

        let mut iterator =
            route.updated_activities_iter(&problem, route.job_ids_iter(1, 3).rev(), 1, 3);

        assert_eq!(
            iterator.next(),
            Some(RouteUpdateActivityData {
                arrival_time: "2025-11-30T10:40:00+02:00".parse().unwrap(),
                waiting_duration: SignedDuration::ZERO,
                departure_time: "2025-11-30T10:50:00+02:00".parse().unwrap(),
                job_id: ActivityId::Service(1),
                current_position: Some(2)
            })
        );

        assert_eq!(
            iterator.next(),
            Some(RouteUpdateActivityData {
                arrival_time: "2025-11-30T11:20:00+02:00".parse().unwrap(),
                waiting_duration: SignedDuration::ZERO,
                departure_time: "2025-11-30T11:30:00+02:00".parse().unwrap(),
                job_id: ActivityId::Service(2),
                current_position: Some(1)
            })
        );

        assert_eq!(iterator.next(), None);
    }

    #[test]
    fn test_replace_activities() {
        let problem = create_problem();

        let mut route = WorkingSolutionRoute::empty(&problem, 0);

        route.insert_service(&problem, 0, 0);
        route.insert_service(&problem, 1, 2);
        route.insert_service(&problem, 2, 1);

        route.replace_activities(
            &problem,
            &[
                ActivityId::Service(1),
                ActivityId::Service(2),
                ActivityId::Service(0),
            ],
            0,
            3,
        );

        assert_eq!(route.len(), 3);
        assert_eq!(
            route.activity_ids.iter().copied().collect::<Vec<_>>(),
            vec![
                ActivityId::Service(1),
                ActivityId::Service(2),
                ActivityId::Service(0)
            ]
        );

        route.replace_activities(
            &problem,
            &[ActivityId::Service(0), ActivityId::Service(2)],
            1,
            3,
        );

        assert_eq!(
            route.activity_ids.iter().copied().collect::<Vec<_>>(),
            vec![
                ActivityId::Service(1),
                ActivityId::Service(0),
                ActivityId::Service(2)
            ]
        );
    }

    #[test]
    fn test_replace_activities_remove() {
        let problem = create_problem();

        let mut route = WorkingSolutionRoute::empty(&problem, 0);

        route.insert_service(&problem, 0, 0);
        route.insert_service(&problem, 1, 2);
        route.insert_service(&problem, 2, 1);

        route.replace_activities(&problem, &[], 1, 2);

        assert_eq!(route.len(), 2);
        assert_eq!(
            route.activity_ids.iter().copied().collect::<Vec<_>>(),
            vec![ActivityId::Service(0), ActivityId::Service(1)]
        );

        route.replace_activities(
            &problem,
            &[ActivityId::Service(2), ActivityId::Service(1)],
            1,
            2,
        );

        assert_eq!(route.len(), 3);
        assert_eq!(
            route.activity_ids.iter().copied().collect::<Vec<_>>(),
            vec![
                ActivityId::Service(0),
                ActivityId::Service(2),
                ActivityId::Service(1)
            ]
        );
    }

    #[test]
    fn test_is_valid_capacity_change_delivery_only() {
        let problem = create_problem_for_capacity_change(
            Capacity::from_vec(vec![50.0]),
            vec![
                (ServiceType::Delivery, Capacity::from_vec(vec![10.0])), // 0
                (ServiceType::Delivery, Capacity::from_vec(vec![20.0])), // 1
                (ServiceType::Delivery, Capacity::from_vec(vec![20.0])), // 2
                (ServiceType::Delivery, Capacity::from_vec(vec![20.0])), // 3
                (ServiceType::Delivery, Capacity::from_vec(vec![15.0])), // 4
                (ServiceType::Delivery, Capacity::from_vec(vec![30.0])), // 5
                (ServiceType::Delivery, Capacity::from_vec(vec![10.0])), // 6
            ],
        );

        let mut route = WorkingSolutionRoute::empty(&problem, 0);
        route.insert_service(&problem, 0, 0);
        route.insert_service(&problem, 1, 1);
        route.insert_service(&problem, 2, 2);

        let is_valid =
            route.is_valid_capacity_change(&problem, std::iter::once(ActivityId::Service(4)), 1, 2);
        assert!(is_valid);

        let is_valid =
            route.is_valid_capacity_change(&problem, std::iter::once(ActivityId::Service(3)), 1, 2);
        assert!(is_valid);

        let is_valid = route.is_valid_capacity_change(
            &problem,
            [ActivityId::Service(1), ActivityId::Service(3)].into_iter(),
            1,
            3,
        );
        assert!(is_valid);

        // Remove 0
        let is_valid = route.is_valid_capacity_change(&problem, [].into_iter(), 0, 1);

        assert!(is_valid);

        // Replace 1 and 2 by 5 and 6
        let is_valid = route.is_valid_capacity_change(
            &problem,
            [ActivityId::Service(5), ActivityId::Service(6)].into_iter(),
            1,
            3,
        );

        assert!(is_valid);

        // Replace 0 by 4
        let is_valid =
            route.is_valid_capacity_change(&problem, [ActivityId::Service(4)].into_iter(), 0, 1);

        assert!(!is_valid);

        // Add 4 before 0
        let is_valid =
            route.is_valid_capacity_change(&problem, [ActivityId::Service(4)].into_iter(), 0, 0);

        assert!(!is_valid);

        // Add 4 after 0
        let is_valid =
            route.is_valid_capacity_change(&problem, [ActivityId::Service(4)].into_iter(), 1, 1);

        assert!(!is_valid);

        // Replace 1 and 2 by 5 and 4
        let is_valid = route.is_valid_capacity_change(
            &problem,
            [ActivityId::Service(5), ActivityId::Service(4)].into_iter(),
            1,
            3,
        );

        assert!(!is_valid);

        // Add 6 at end
        let is_valid =
            route.is_valid_capacity_change(&problem, [ActivityId::Service(6)].into_iter(), 3, 3);

        assert!(!is_valid);
    }

    fn create_problem_for_tw_change(services: Vec<TimeWindow>) -> VehicleRoutingProblem {
        // 10 locations from (0, 0) to (9, 0)
        let locations = test_utils::create_location_grid(1, 10);

        let mut vehicle_builder = VehicleBuilder::default();
        vehicle_builder.set_depot_location_id(0);
        vehicle_builder.set_capacity(Capacity::from_vec(vec![100.0]));
        vehicle_builder.set_vehicle_id(String::from("vehicle"));
        vehicle_builder.set_profile_id(0);
        let vehicle = vehicle_builder.build();
        let vehicles = vec![vehicle];

        let services = services
            .into_iter()
            .enumerate()
            .map(|(i, time_window)| {
                let mut service_builder = ServiceBuilder::default();
                service_builder.set_demand(Capacity::from_vec(vec![10.0]));
                service_builder.set_external_id(format!("service_{}", i + 1));
                service_builder.set_service_duration(SignedDuration::from_mins(10));
                service_builder.set_location_id(i + 1);
                service_builder.set_time_window(time_window);
                service_builder.build()
            })
            .collect::<Vec<_>>();

        let mut builder = VehicleRoutingProblemBuilder::default();
        // Travel time of 30 mins between consecutive locations

        builder.set_vehicle_profiles(vec![VehicleProfile::new(
            "test_profile".to_owned(),
            TravelMatrices::from_constant(
                &locations,
                SignedDuration::from_mins(30).as_secs_f64(),
                100.0,
                SignedDuration::from_mins(30).as_secs_f64(),
            ),
        )]);
        builder.set_locations(locations);
        builder.set_vehicles(vehicles);
        builder.set_services(services);

        builder.build()
    }

    #[test]
    fn test_is_valid_tw_change_delivery_only() {
        // Vehicle starts at depot (location 0) at 08:00
        // Travel time between locations is 30 mins, service duration is 10 mins
        // So arrival at location 1 is 08:30, departure is 08:40
        // Arrival at location 2 is 09:10, departure is 09:20
        // etc.
        let problem = create_problem_for_tw_change(vec![
            TimeWindow::from_iso(
                Some("2025-11-30T08:00:00+02:00"),
                Some("2025-11-30T09:00:00+02:00"),
            ), // 0: TW ends at 09:00
            TimeWindow::from_iso(
                Some("2025-11-30T08:00:00+02:00"),
                Some("2025-11-30T10:00:00+02:00"),
            ), // 1: TW ends at 10:00
            TimeWindow::from_iso(
                Some("2025-11-30T08:00:00+02:00"),
                Some("2025-11-30T11:00:00+02:00"),
            ), // 2: TW ends at 11:00
            TimeWindow::from_iso(
                Some("2025-11-30T08:00:00+02:00"),
                Some("2025-11-30T12:00:00+02:00"),
            ), // 3: TW ends at 12:00
            TimeWindow::from_iso(
                Some("2025-11-30T08:00:00+02:00"),
                Some("2025-11-30T08:35:00+02:00"),
            ), // 4: TW ends at 08:35 (tight)
            TimeWindow::from_iso(
                Some("2025-11-30T08:00:00+02:00"),
                Some("2025-11-30T14:00:00+02:00"),
            ), // 5: TW ends at 14:00 (relaxed)
        ]);

        let mut route = WorkingSolutionRoute::empty(&problem, 0);
        // Route: 0 -> 1 -> 2 (arrival times: 08:30, 09:10, 09:50)
        route.insert_service(&problem, 0, 0);
        route.insert_service(&problem, 1, 1);
        route.insert_service(&problem, 2, 2);

        // Test 1: Replace service 1 with service 3 (same position, later TW)
        // This should be valid since service 3 has a later end time
        let is_valid =
            route.is_valid_tw_change(&problem, std::iter::once(ActivityId::Service(3)), 1, 2);
        assert!(is_valid);

        // Test 2: Replace service 1 with service 5 (relaxed TW)
        let is_valid =
            route.is_valid_tw_change(&problem, std::iter::once(ActivityId::Service(5)), 1, 2);
        assert!(is_valid);

        // Test 3: Insert service 4 (tight TW ending at 08:35) before service 0
        // Service 0 arrives at 08:30, so inserting 4 before would push 0 later
        // Service 4 arrives at 08:30 which is before its TW end of 08:35 - valid
        let is_valid =
            route.is_valid_tw_change(&problem, std::iter::once(ActivityId::Service(4)), 0, 0);
        assert!(is_valid);

        // Test 4: Insert service 4 (tight TW) after service 0
        // After serving 0 (depart 08:40), arrive at 4's location at 09:10
        // But service 4's TW ends at 08:35, so this should be invalid
        let is_valid =
            route.is_valid_tw_change(&problem, std::iter::once(ActivityId::Service(4)), 1, 1);
        assert!(!is_valid);

        // Test 5: Replace service 0 with service 4
        // Service 4 would arrive at 08:30 (within TW ending 08:35) - valid
        let is_valid =
            route.is_valid_tw_change(&problem, std::iter::once(ActivityId::Service(4)), 0, 1);
        assert!(is_valid);

        // Test 6: Insert service 5 at end
        // After route 0->1->2, departing at 10:00, arrive at 5 at 10:30
        // Service 5's TW ends at 14:00, so this is valid
        let is_valid =
            route.is_valid_tw_change(&problem, std::iter::once(ActivityId::Service(5)), 3, 3);
        assert!(is_valid);

        // Test 7: Remove service 0 (replace with nothing)
        // This should be valid as it only makes subsequent services earlier
        let is_valid = route.is_valid_tw_change(&problem, [].into_iter(), 0, 1);
        assert!(is_valid);

        // Test 8: Remove service 2 (replace with nothing)
        // This should be valid
        let is_valid = route.is_valid_tw_change(&problem, [].into_iter(), 2, 3);
        assert!(is_valid);

        // Test 9: Insert 3 and 5, replace 1 by 3
        let is_valid = route.is_valid_tw_change(
            &problem,
            [ActivityId::Service(3), ActivityId::Service(5)].into_iter(),
            1,
            2,
        );
        assert!(is_valid);

        // Test 9: Insert 4 and 5, replace 1 by 4
        let is_valid = route.is_valid_tw_change(
            &problem,
            [ActivityId::Service(4), ActivityId::Service(5)].into_iter(),
            1,
            2,
        );
        assert!(!is_valid);
    }
}
