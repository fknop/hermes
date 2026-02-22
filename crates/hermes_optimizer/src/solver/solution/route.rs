use fxhash::FxHashMap;
use jiff::{SignedDuration, Timestamp};

use crate::{
    problem::{
        amount::AmountExpression,
        capacity::{Capacity, is_capacity_satisfied},
        job::{ActivityId, Job, JobActivity, JobIdx},
        location::LocationIdx,
        meters::Meters,
        service::ServiceType,
        vehicle::{Vehicle, VehicleIdx},
        vehicle_routing_problem::VehicleRoutingProblem,
    },
    solver::{
        insertion::{Insertion, ServiceInsertion, ShipmentInsertion},
        solution::{
            route_update_iterator::RouteUpdateIterator,
            utils::{
                compute_activity_arrival_time, compute_departure_time,
                compute_first_activity_arrival_time, compute_time_slack, compute_vehicle_end,
                compute_vehicle_start, compute_waiting_duration, compute_waiting_time_slack,
            },
        },
    },
    utils::{bbox::BBox, bitset::BitSet, sparse_table::SparseTable},
};

#[derive(Clone)]
pub struct WorkingSolutionRoute {
    pub(super) version: usize,

    pub(super) vehicle_id: VehicleIdx,
    // Map of ActivityId to index in activities vector
    pub(super) jobs: FxHashMap<ActivityId, usize>,

    total_transport_cost: f64,

    /// fwd_transport_cost[profile_id][i] is the transport cost from activity 0 to activity i for vehicle profile 'profile_id'
    pub(super) fwd_transport_cost: Vec<Vec<f64>>,

    /// bwd_transport_cost[profile_id][i] is the transport cost from activity i to activity 0 for vehicle profile 'profile_id', useful for quickly computing segment reversals
    pub(super) bwd_transport_cost: Vec<Vec<f64>>,

    /// List of activity job IDs in the route order
    pub(super) activity_ids: Vec<ActivityId>,

    /// List of arrival times at each activity
    pub(super) arrival_times: Vec<Timestamp>,

    /// List of departure times at each activity
    pub(super) departure_times: Vec<Timestamp>,

    /// List of waiting durations at each activity
    pub(super) waiting_durations: Vec<SignedDuration>,

    /// List of cumulative waiting duration computed forward
    /// fwd_cumulative_waiting_durations[i] is the total waiting duration from step 0 until step i
    /// step 0 is the depot, step[len + 1] is the last step
    pub(super) fwd_cumulative_waiting_durations: Vec<SignedDuration>,

    /// List of cumulative waiting duration computed backward
    /// bwd_cumulative_waiting_durations[i] is the total waiting duration from step i to the end of the route
    /// step 0 is the depot, step[len + 1] is the last step
    pub(super) bwd_cumulative_waiting_durations: Vec<SignedDuration>,

    /// waiting_time_slacks[i] stored the maximum time a task can be moved "backward" in time without causing waiting time, or more waiting time if some already exists.
    pub(super) waiting_time_slacks: Vec<SignedDuration>,

    // fwd_time_slacks[i] stores the maximum time delay that can be absorbed (arrival time is moved forward) at activity i
    // before violating time windows of subsequent activities
    // computed backward from end to start
    pub(super) fwd_time_slacks: Vec<SignedDuration>,

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

    /// Store the total set of required skills for the services from the start to step i
    pub(super) skills_sparse_table: SparseTable,

    /// pending_shipments[i] stores the set of pending shipments after visiting activity i
    pub(super) pending_shipments: Vec<BitSet>,

    /// num_shipments[i] counts the total number of shipments activities from start to activity i
    pub(super) num_shipments: Vec<usize>,

    bbox: BBox,

    updated_in_iteration: bool,
}

impl WorkingSolutionRoute {
    pub fn empty(problem: &VehicleRoutingProblem, vehicle_id: VehicleIdx) -> Self {
        let mut route = WorkingSolutionRoute {
            version: 0, // Will be set in udpate_data
            vehicle_id,
            jobs: FxHashMap::default(),
            bbox: BBox::default(),
            updated_in_iteration: false,
            total_transport_cost: 0.0,
            fwd_transport_cost: vec![vec![]; problem.vehicle_profiles().len()],
            bwd_transport_cost: vec![vec![]; problem.vehicle_profiles().len()],
            activity_ids: Vec::new(),
            arrival_times: Vec::new(),
            departure_times: Vec::new(),
            waiting_durations: Vec::new(),
            fwd_cumulative_waiting_durations: Vec::new(),
            bwd_cumulative_waiting_durations: Vec::new(),
            waiting_time_slacks: Vec::new(),
            fwd_load_peaks: Vec::new(),
            bwd_load_peaks: Vec::new(),
            current_load: Vec::new(),
            bwd_load_deliveries: Vec::new(),
            bwd_load_pickups: Vec::new(),
            fwd_load_deliveries: Vec::new(),
            fwd_load_pickups: Vec::new(),
            fwd_load_shipments: Vec::new(),
            fwd_time_slacks: Vec::new(),
            pending_shipments: Vec::new(),
            num_shipments: Vec::new(),
            skills_sparse_table: SparseTable::empty(),
            delivery_load_slack: problem.vehicle(vehicle_id).capacity().clone(),
            pickup_load_slack: problem.vehicle(vehicle_id).capacity().clone(),
        };

        route.update_data(problem);

        route
    }

    pub fn len(&self) -> usize {
        self.activity_ids.len()
    }

    pub fn is_empty(&self) -> bool {
        self.activity_ids.is_empty()
    }

    pub fn version(&self) -> usize {
        self.version
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

    pub fn contains_activity(&self, job_id: ActivityId) -> bool {
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

    pub fn has_maximum_activities(&self, problem: &VehicleRoutingProblem) -> bool {
        let vehicle = problem.vehicle(self.vehicle_id);
        // Assuming the vehicle has a method `maximum_activities` that returns the maximum allowed activities
        if let Some(max_activities) = vehicle.maximum_activities() {
            self.activity_ids.len() >= max_activities
        } else {
            false
        }
    }

    pub fn compute_location_ids(&self, problem: &VehicleRoutingProblem) -> Vec<LocationIdx> {
        let mut location_ids = vec![];

        let vehicle = self.vehicle(problem);
        if self.has_start(problem)
            && let Some(depot_location_id) = vehicle.depot_location_id()
        {
            location_ids.push(depot_location_id);
        }

        for &job_id in &self.activity_ids {
            location_ids.push(problem.job_activity(job_id).location_id());
        }

        if self.has_end(problem)
            && let Some(depot_location_id) = vehicle.depot_location_id()
        {
            location_ids.push(depot_location_id);
        }

        location_ids
    }

    pub fn start(&self, problem: &VehicleRoutingProblem) -> Timestamp {
        let first = self.first();
        compute_vehicle_start(
            problem,
            self.vehicle_id,
            first.activity_id(),
            first.arrival_time(),
        )
    }

    pub fn end(&self, problem: &VehicleRoutingProblem) -> Timestamp {
        let last = self.last();
        compute_vehicle_end(
            problem,
            self.vehicle_id,
            last.activity_id(),
            last.departure_time(),
        )
    }

    pub fn activity_id(&self, position: usize) -> ActivityId {
        self.activity_ids[position]
    }

    pub fn job_position(&self, job_id: ActivityId) -> Option<usize> {
        self.jobs.get(&job_id).copied()
    }

    pub fn matching_shipment_position(&self, id: ActivityId) -> usize {
        match id {
            ActivityId::ShipmentPickup(job_id) => self
                .jobs
                .get(&ActivityId::ShipmentDelivery(job_id))
                .copied()
                .expect("ShipmentDelivery should be present if ShipmentPickup is in the route"),
            ActivityId::ShipmentDelivery(job_id) => self
                .jobs
                .get(&ActivityId::ShipmentPickup(job_id))
                .copied()
                .expect("ShipmentPickup should be present if ShipmentDelivery is in the route"),
            ActivityId::Service(_) => panic!("Calling matching_shipment_position with a service"),
        }
    }

    pub fn duration(&self, problem: &VehicleRoutingProblem) -> SignedDuration {
        if self.is_empty() {
            return SignedDuration::ZERO;
        }

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
                    self.first().job_activity(problem).location_id(),
                );
            }

            if self.has_end(problem) {
                transport_duration += problem.travel_time(
                    vehicle,
                    self.last().job_activity(problem).location_id(),
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
                problem
                    .job_activity(self.activity_ids[index - 1])
                    .location_id(),
                problem.job_activity(job_id).location_id(),
            );
        }

        transport_duration
    }

    pub fn transport_costs(&self, _problem: &VehicleRoutingProblem) -> f64 {
        assert!(
            (self.is_empty() && self.total_transport_cost == 0.0)
                || (!self.is_empty() && self.total_transport_cost > 0.0)
        );

        self.total_transport_cost
    }

    pub fn distance(&self, problem: &VehicleRoutingProblem) -> Meters {
        if self.is_empty() {
            return Meters::ZERO;
        }

        let vehicle = self.vehicle(problem);
        let mut distance = Meters::ZERO;

        if let Some(depot_location_id) = vehicle.depot_location_id() {
            if self.has_start(problem) {
                distance += problem.travel_distance(
                    vehicle,
                    depot_location_id,
                    self.first().job_activity(problem).location_id(),
                );
            }

            if self.has_end(problem) {
                distance += problem.travel_distance(
                    vehicle,
                    self.last().job_activity(problem).location_id(),
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
                problem
                    .job_activity(self.activity_ids[index - 1])
                    .location_id(),
                problem.job_activity(job_id).location_id(),
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

    pub fn activities_iter(&self) -> impl Iterator<Item = RouteActivityInfo> {
        self.activity_ids
            .iter()
            .enumerate()
            .map(move |(index, _)| self.activity(index))
    }

    pub fn get(&self, index: usize) -> Option<ActivityId> {
        self.activity_ids.get(index).copied()
    }

    pub fn activity(&self, index: usize) -> RouteActivityInfo {
        assert!(
            !self.is_empty(),
            "cannot call WorkingSolutionRoute::activity() on empty route"
        );

        RouteActivityInfo {
            activity_id: self.activity_ids[index],
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

    pub fn vehicle_id(&self) -> VehicleIdx {
        self.vehicle_id
    }

    pub fn vehicle<'a>(&self, problem: &'a VehicleRoutingProblem) -> &'a Vehicle {
        problem.vehicle(self.vehicle_id)
    }

    pub fn will_break_maximum_activities(
        &self,
        problem: &VehicleRoutingProblem,
        added: usize,
    ) -> bool {
        if let Some(max) = self.vehicle(problem).maximum_activities() {
            self.len() + added > max
        } else {
            false
        }
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

    pub fn location_id(
        &self,
        problem: &VehicleRoutingProblem,
        position: usize,
    ) -> Option<LocationIdx> {
        self.activity_ids
            .get(position)
            .map(|&activity_id| problem.job_activity(activity_id).location_id())
    }

    pub fn previous_location_id(
        &self,
        problem: &VehicleRoutingProblem,
        position: usize,
    ) -> Option<LocationIdx> {
        if position == 0 {
            let vehicle = self.vehicle(problem);
            vehicle.depot_location_id()
        } else if position <= self.activity_ids.len() {
            Some(
                problem
                    .job_activity(self.activity_ids[position - 1])
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
    ) -> Option<LocationIdx> {
        let next_job_id = self.activity_ids.get(position + 1);

        match next_job_id {
            Some(&job_id) => Some(problem.job_activity(job_id).location_id()),
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

    pub fn end_location(&self, problem: &VehicleRoutingProblem) -> Option<LocationIdx> {
        let vehicle = self.vehicle(problem);
        if vehicle.should_return_to_depot() {
            vehicle.depot_location_id()
        } else {
            None
        }
    }

    /// Check if an activity id is in the neighborhood for insertion at a given position
    pub fn in_insertion_neighborhood(
        &self,
        problem: &VehicleRoutingProblem,
        activity_id: ActivityId,
        position: usize,
    ) -> bool {
        let previous = self.activity_ids.get(position - 1).copied();
        let next = self.activity_ids.get(position).copied();

        match (previous, next) {
            (Some(previous), Some(next)) => {
                problem.in_nearest_neighborhood_of(previous, activity_id)
                    || problem.in_nearest_neighborhood_of(next, activity_id)
            }
            (Some(previous), None) => problem.in_nearest_neighborhood_of(previous, activity_id),
            (None, Some(next)) => problem.in_nearest_neighborhood_of(next, activity_id),
            (None, None) => true, // Route is empty
        }
    }

    pub fn in_segment_insertion_neighborhood(
        &self,
        problem: &VehicleRoutingProblem,
        from_activity_id: ActivityId,
        to_activity_id: ActivityId,
        position: usize,
    ) -> bool {
        // position - 1 => from_activity_id ... to_activity_id ... position

        let previous = self.activity_ids.get(position - 1).copied();
        let next = self.activity_ids.get(position).copied();

        match (previous, next) {
            (Some(previous), Some(next)) => {
                problem.in_nearest_neighborhood_of(previous, from_activity_id)
                    && problem.in_nearest_neighborhood_of(next, to_activity_id)
            }
            (Some(previous), None) => {
                problem.in_nearest_neighborhood_of(previous, from_activity_id)
            }
            (None, Some(next)) => problem.in_nearest_neighborhood_of(next, to_activity_id),
            (None, None) => true, // Route is empty
        }
    }

    pub fn in_swap_neighborhood(
        &self,
        problem: &VehicleRoutingProblem,
        self_pos_start: usize,
        self_pos_end: usize, // Exclusive
        other: &WorkingSolutionRoute,
        other_pos_start: usize,
        other_pos_end: usize, // Exclusive
    ) -> bool {
        let self_previous = self.get(self_pos_start - 1);
        let self_next = self.get(self_pos_end);

        let other_activity_start = other.activity_id(other_pos_start);
        let other_activity_end = other.activity_id(other_pos_end - 1);

        match (self_previous, self_next) {
            (Some(previous), Some(next)) => {
                problem.in_nearest_neighborhood_of(previous, other_activity_start)
                    && problem.in_nearest_neighborhood_of(next, other_activity_end)
            }
            (Some(previous), None) => {
                problem.in_nearest_neighborhood_of(previous, other_activity_start)
            }
            (None, Some(next)) => problem.in_nearest_neighborhood_of(next, other_activity_end),
            (None, None) => true, // Route is empty
        }
    }

    fn increment_version(&mut self, problem: &VehicleRoutingProblem) {
        self.updated_in_iteration = true;
        self.version = problem.next_route_version();
    }

    pub fn reset(&mut self, problem: &VehicleRoutingProblem) {
        self.jobs.clear();
        self.activity_ids.clear();
        self.bbox = BBox::default();

        self.update_data(problem);
    }

    pub fn remove(
        &mut self,
        problem: &VehicleRoutingProblem,
        position: usize,
    ) -> Option<ActivityId> {
        if position >= self.activity_ids.len() {
            return None;
        }

        let activity_id = self.activity_ids.remove(position);

        self.jobs.remove(&activity_id);
        for (index, &activity_id) in self.activity_ids.iter().skip(position).enumerate() {
            self.jobs.insert(activity_id, index + position);
        }

        self.increment_version(problem);

        Some(activity_id)
    }

    pub fn remove_activity(
        &mut self,
        problem: &VehicleRoutingProblem,
        activity_id: ActivityId,
    ) -> bool {
        if !self.contains_activity(activity_id) {
            return false; // Service is not in the route
        }

        if let Some(&position) = self.jobs.get(&activity_id) {
            match activity_id {
                ActivityId::Service(_) => self.remove(problem, position).is_some(),
                ActivityId::ShipmentPickup(id) => {
                    let delivery = self.jobs.get(&ActivityId::ShipmentDelivery(id));
                    self.remove(problem, *delivery.unwrap());
                    self.remove(problem, position).is_some()
                }
                ActivityId::ShipmentDelivery(id) => {
                    self.remove(problem, position);
                    let pickup = self.jobs.get(&ActivityId::ShipmentPickup(id));
                    self.remove(problem, *pickup.unwrap()).is_some()
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
                job_index,
                pickup_position,
                delivery_position,
                ..
            }) => {
                self.insert_shipment(problem, *pickup_position, *delivery_position, *job_index);
            }
        }
    }

    fn insert_service(
        &mut self,
        problem: &VehicleRoutingProblem,
        position: usize,
        service_id: JobIdx,
    ) {
        assert!(position <= self.len());
        if self.jobs.contains_key(&ActivityId::Service(service_id)) {
            return;
        }

        self.activity_ids
            .insert(position, ActivityId::Service(service_id));

        // Update the arrival times and departure times of subsequent activities
        self.update_data(problem);
    }

    /// Inserts a shipment into the route at the specified positions.
    /// If delivery_position is equal to pickup position, it means the delivery will be inserted directly after the pickup
    fn insert_shipment(
        &mut self,
        problem: &VehicleRoutingProblem,
        pickup_position: usize,
        delivery_position: usize,
        shipment_id: JobIdx,
    ) {
        assert!(pickup_position <= self.len());
        assert!(delivery_position <= self.len());
        assert!(delivery_position >= pickup_position);

        if self
            .jobs
            .contains_key(&ActivityId::ShipmentPickup(shipment_id))
            || self
                .jobs
                .contains_key(&ActivityId::ShipmentDelivery(shipment_id))
        {
            return;
        }

        self.activity_ids
            .insert(pickup_position, ActivityId::ShipmentPickup(shipment_id));

        self.activity_ids.insert(
            delivery_position + 1,
            ActivityId::ShipmentDelivery(shipment_id),
        );

        // Update the arrival times and departure times of subsequent activities
        self.update_data(problem);
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
        self.update_data(problem);
    }

    pub(crate) fn resync(&mut self, problem: &VehicleRoutingProblem) {
        if !self.updated_in_iteration {
            return;
        }

        self.update_data(problem);
        self.updated_in_iteration = false;
    }

    fn update_bbox(&mut self, problem: &VehicleRoutingProblem) {
        let mut bbox = BBox::default();

        for &job_id in &self.activity_ids {
            let location_id = problem.job_activity(job_id).location_id();
            let location = problem.location(location_id);
            bbox.extend(location);
        }

        self.bbox = bbox;
    }

    fn resize_data(&mut self, problem: &VehicleRoutingProblem) {
        let len = self.len();

        self.fwd_transport_cost
            .iter_mut()
            .for_each(|ftc| ftc.resize(len, 0.0));
        self.bwd_transport_cost
            .iter_mut()
            .for_each(|ftc| ftc.resize(len, 0.0));

        self.arrival_times.resize(len, Timestamp::MAX);
        self.departure_times.resize(len, Timestamp::MAX);
        self.waiting_durations.resize(len, SignedDuration::ZERO);

        self.waiting_time_slacks.resize(len, SignedDuration::MAX);

        self.fwd_load_pickups.resize_with(len, || {
            Capacity::with_dimensions(problem.capacity_dimensions())
        });
        self.fwd_load_deliveries.resize_with(len, || {
            Capacity::with_dimensions(problem.capacity_dimensions())
        });
        self.bwd_load_deliveries.resize_with(len, || {
            Capacity::with_dimensions(problem.capacity_dimensions())
        });
        self.bwd_load_pickups.resize_with(len, || {
            Capacity::with_dimensions(problem.capacity_dimensions())
        });
        self.fwd_load_shipments.resize_with(len, || {
            Capacity::with_dimensions(problem.capacity_dimensions())
        });

        self.pending_shipments
            .resize_with(len, || BitSet::with_capacity(problem.jobs().len()));
        self.pending_shipments
            .iter_mut()
            .for_each(|set| set.clear());
        self.num_shipments.resize(len, 0);
        self.num_shipments.fill(0);

        let steps = len + 2;
        self.bwd_cumulative_waiting_durations
            .resize(steps, SignedDuration::ZERO);
        self.fwd_cumulative_waiting_durations
            .resize(steps, SignedDuration::ZERO);

        self.fwd_time_slacks.resize(steps, SignedDuration::MAX);

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
        self.increment_version(problem);
        self.jobs.clear();
        self.jobs.extend(
            self.activity_ids
                .iter()
                .enumerate()
                .map(|(index, &job_id)| (job_id, index)),
        );

        self.update_bbox(problem);
        self.resize_data(problem);

        let vehicle = self.vehicle(problem);

        self.total_transport_cost = 0.0;

        if self.is_empty() {
            self.delivery_load_slack.update(vehicle.capacity());
            self.pickup_load_slack.update(vehicle.capacity());
            return;
        }

        let len = self.len();

        if problem.has_skills() {
            let bitsets = self
                .activity_ids
                .iter()
                .map(|&activity_id| {
                    let job = problem.job(activity_id.job_id());
                    job.skills_bitset()
                })
                .cloned()
                .collect();
            self.skills_sparse_table = SparseTable::build(bitsets)
        }

        let mut current_load_pickups = Capacity::with_dimensions(problem.capacity_dimensions());
        let mut current_load_deliveries = Capacity::with_dimensions(problem.capacity_dimensions());
        let mut current_load_shipments = Capacity::with_dimensions(problem.capacity_dimensions());

        self.fwd_cumulative_waiting_durations[0] = SignedDuration::ZERO;

        for i in 0..len {
            let activity_id = self.activity_ids[i];

            if problem.has_shipments() {
                match activity_id {
                    ActivityId::ShipmentPickup(id) => {
                        let delivery_pos = self.jobs[&ActivityId::ShipmentDelivery(id)];
                        assert!(
                            delivery_pos > i,
                            "Activity {} does not have its delivery after it, {:?}",
                            activity_id,
                            self.activity_ids
                        );

                        if i == 0 {
                            self.pending_shipments[i].set(id.get(), true);
                            self.num_shipments[i] = 1;
                        } else {
                            let (left, right) = self.pending_shipments.split_at_mut(i);
                            right[0].clone_from(&left[i - 1]);
                            right[0].set(id.get(), true);

                            self.num_shipments[i] = self.num_shipments[i - 1] + 1;
                        }
                    }
                    ActivityId::ShipmentDelivery(id) => {
                        let pickup_pos = self.jobs[&ActivityId::ShipmentPickup(id)];
                        assert!(
                            pickup_pos < i,
                            "Activity {} does not have its pickup before it, {:?}",
                            activity_id,
                            self.activity_ids
                        );

                        let (left, right) = self.pending_shipments.split_at_mut(i);
                        right[0].clone_from(&left[i - 1]);
                        right[0].set(id.get(), false);
                        self.num_shipments[i] = self.num_shipments[i - 1] + 1;
                    }
                    ActivityId::Service(_) => {
                        if i > 0 {
                            let (left, right) = self.pending_shipments.split_at_mut(i);
                            right[0].clone_from(&left[i - 1]);
                            self.num_shipments[i] = self.num_shipments[i - 1];
                        }
                    }
                }
            }

            let job = problem.job(activity_id.job_id());

            match activity_id {
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
                compute_first_activity_arrival_time(problem, self.vehicle_id, activity_id)
            } else {
                compute_activity_arrival_time(
                    problem,
                    self.vehicle_id,
                    self.activity_ids[i - 1],
                    self.departure_times[i - 1],
                    activity_id,
                )
            };

            self.waiting_durations[i] =
                compute_waiting_duration(problem, activity_id, self.arrival_times[i]);

            self.departure_times[i] = compute_departure_time(
                problem,
                self.arrival_times[i],
                self.waiting_durations[i],
                activity_id,
            );

            self.fwd_cumulative_waiting_durations[i + 1] =
                self.waiting_durations[i] + self.fwd_cumulative_waiting_durations[i];

            for (profile_id, profile) in problem.vehicle_profiles().iter().enumerate() {
                if i == 0 {
                    self.fwd_transport_cost[profile_id][i] = 0.0;
                    self.bwd_transport_cost[profile_id][i] = 0.0;
                } else {
                    self.fwd_transport_cost[profile_id][i] = self.fwd_transport_cost[profile_id]
                        [i - 1]
                        + profile.travel_cost(
                            problem.job_activity(self.activity_ids[i - 1]).location_id(),
                            problem.job_activity(activity_id).location_id(),
                        );
                    self.bwd_transport_cost[profile_id][i] = self.bwd_transport_cost[profile_id]
                        [i - 1]
                        + profile.travel_cost(
                            problem.job_activity(activity_id).location_id(),
                            problem.job_activity(self.activity_ids[i - 1]).location_id(),
                        );
                }
            }

            if let Some(previous_location_id) = self.previous_location_id(problem, i) {
                self.total_transport_cost += problem.travel_cost(
                    vehicle,
                    previous_location_id,
                    problem.job_activity(activity_id).location_id(),
                );
            }

            if i == len - 1
                && let Some(end_location) = self.end_location(problem)
            {
                self.total_transport_cost += problem.travel_cost(
                    vehicle,
                    problem.job_activity(activity_id).location_id(),
                    end_location,
                );
            }
        }

        self.fwd_cumulative_waiting_durations[len + 1] = self.fwd_cumulative_waiting_durations[len];

        assert!(
            self.fwd_load_shipments[self.len() - 1].is_empty(),
            "{:?}",
            self.fwd_load_shipments[self.len() - 1]
        );
        self.current_load[len + 1].update(&self.fwd_load_pickups[len - 1]);

        // Reset for the reverse pass
        current_load_deliveries.reset();
        current_load_pickups.reset();

        for i in (0..len).rev() {
            let activity_id = self.activity_ids[i];
            let job = problem.job(activity_id.job_id());

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

        if problem.has_time_windows()
            // || vehicle.maximum_working_duration().is_some()
            || vehicle.latest_end_time().is_some()
        {
            // TODO: remove max working duration from time slacks calculations, it's not correct
            // It's not corect because arriving later does not mean we would break the duration, we could also remove a segment and start later
            // let latest_end = match (
            //     vehicle.latest_end_time(),
            //     vehicle.maximum_working_duration(),
            // ) {
            //     (Some(latest_end), Some(maximum_working_duration)) => {
            //         latest_end.min(self.start(problem) + maximum_working_duration)
            //     }
            //     (Some(latest_end), None) => latest_end,
            //     (None, Some(maximum_working_duration)) => {
            //         self.start(problem) + maximum_working_duration
            //     }
            //     (None, None) => Timestamp::MAX,
            // };

            let latest_end = vehicle.latest_end_time().unwrap_or(Timestamp::MAX);

            self.fwd_time_slacks[len + 1] = latest_end.duration_since(self.end(problem));
            self.bwd_cumulative_waiting_durations[len + 1] = SignedDuration::ZERO;

            for (index, &activity_job_id) in self.activity_ids.iter().enumerate().rev() {
                assert_ne!(self.arrival_times[index], Timestamp::MAX);
                let current_time_slack =
                    compute_time_slack(problem, activity_job_id, self.arrival_times[index]);

                self.bwd_cumulative_waiting_durations[index + 1] = self.waiting_durations[index]
                    + self
                        .bwd_cumulative_waiting_durations
                        .get(index + 2)
                        .copied()
                        .unwrap_or(SignedDuration::ZERO);

                let waiting_time_slack = compute_waiting_time_slack(
                    problem.job_activity(activity_job_id).time_windows(),
                    self.arrival_times[index],
                );
                self.waiting_time_slacks[index] = waiting_time_slack.min(
                    self.waiting_time_slacks
                        .get(index + 1)
                        .copied()
                        .unwrap_or(SignedDuration::MAX),
                );

                self.fwd_time_slacks[index + 1] = if let Some(next_activity_time_slack) =
                    self.fwd_time_slacks.get(index + 2)
                {
                    current_time_slack
                        // The waiting time will absorb the shift before the next activity does
                        .min(next_activity_time_slack.saturating_add(self.waiting_durations[index]))
                } else {
                    current_time_slack
                };
            }

            self.bwd_cumulative_waiting_durations[0] = self.bwd_cumulative_waiting_durations[1];

            if let Some(latest_start) = vehicle.latest_start_time() {
                self.fwd_time_slacks[0] =
                    (latest_start.duration_since(self.start(problem))).min(self.fwd_time_slacks[1]);
            } else {
                self.fwd_time_slacks[0] = self.fwd_time_slacks[1];
            }
        } else {
            self.fwd_time_slacks.fill(SignedDuration::MAX);
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
    pub fn activity_ids_iter(
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

    /// Checks if there are any pending shipments in the range [start, end)
    pub fn contains_pending_shipment(&self, start: usize, end: usize) -> bool {
        assert!(start < self.len());
        assert!(end <= self.len());

        if start == 0 {
            !self.pending_shipments[end - 1].is_all_zeroes()
        } else {
            self.pending_shipments[start - 1] != self.pending_shipments[end - 1]
        }
    }

    /// Checks if there are any shipments in the range [start, end)
    pub fn contains_shipments(&self, start: usize, end: usize) -> bool {
        assert!(start < self.len());
        assert!(end <= self.len());

        if start == 0 {
            self.num_shipments[end - 1] > 0
        } else {
            self.num_shipments[end - 1] - self.num_shipments[start - 1] > 0
        }
    }

    /// Check if a vehicle can deliver a segment from another route between [start, end)
    pub fn can_vehicle_deliver_segment(
        &self,
        problem: &VehicleRoutingProblem,
        other: &Self,
        start: usize,
        end: usize,
    ) -> bool {
        assert!(start < end, "{} < {}", start, end);

        if self.vehicle_id == other.vehicle_id {
            return true;
        }

        if problem.has_skills() {
            let vehicle = self.vehicle(problem);
            if !other
                .skills_sparse_table
                .range_covered_by(start, end - 1, vehicle.skills_bitset())
            {
                return false;
            }
        }

        true
    }

    pub fn can_vehicle_deliver_job(&self, problem: &VehicleRoutingProblem, job_id: JobIdx) -> bool {
        let job = problem.job(job_id);
        let vehicle = self.vehicle(problem);

        job.skills_satisfied_by_vehicle(vehicle)
    }

    pub fn is_valid_change(
        &self,
        problem: &VehicleRoutingProblem,
        activity_ids: impl Iterator<Item = ActivityId> + Clone,
        start: usize,
        end: usize,
    ) -> bool {
        self.is_valid_time_change(problem, activity_ids.clone(), start, end)
            && self.is_valid_capacity_change(problem, activity_ids, start, end)
    }

    /// Return the transport cost delta of inserting [r2_start, r2_end) of r2 into [r1_start, r1_end) of r1
    /// It does not return any delta related to r2 itself
    pub fn transport_cost_delta_update(
        &self,
        problem: &VehicleRoutingProblem,
        r1_start: usize,
        r1_end: usize,
        r2: &Self,
        r2_start: usize,
        r2_end: usize,
    ) -> (f64, f64) {
        assert!(r1_start <= r1_end, "{r1_start} <= {r1_end}");
        assert!(r2_start < r2_end, "{r2_start} < {r2_end}");

        let r1 = self;

        let v1 = r1.vehicle(problem);
        let p1 = v1.profile_id().get();

        // Removed cost of r1
        let r1_removed_cost = if r1_end > r1_start {
            let mut removed_cost =
                r1.fwd_transport_cost[p1][r1_end - 1] - r1.fwd_transport_cost[p1][r1_start];
            removed_cost += problem.travel_cost_or_zero(
                v1,
                r1.previous_location_id(problem, r1_start),
                r1.location_id(problem, r1_start),
            );
            removed_cost += problem.travel_cost_or_zero(
                v1,
                r1.location_id(problem, r1_end - 1),
                r1.next_location_id(problem, r1_end - 1),
            );
            removed_cost
        } else {
            0.0
        };

        // Removed segment of r2
        let mut fwd_cost = 0.0;

        fwd_cost += r2.fwd_transport_cost[p1][r2_end - 1] - r2.fwd_transport_cost[p1][r2_start];

        let mut bwd_cost = 0.0;

        bwd_cost += r2.bwd_transport_cost[p1][r2_end - 1] - r2.bwd_transport_cost[p1][r2_start];

        // Compute the cost from the previous location in r1 to the start of the segment in r2
        if let Some(r1_start_previous) = r1.previous_location_id(problem, r1_start) {
            fwd_cost += problem.travel_cost(
                v1,
                r1_start_previous,
                problem
                    .job_activity(r2.activity_ids[r2_start])
                    .location_id(),
            );

            bwd_cost += problem.travel_cost(
                v1,
                r1_start_previous,
                problem
                    .job_activity(r2.activity_ids[r2_end - 1])
                    .location_id(),
            );
        }

        let r1_next = if r1_end > r1_start {
            r1.next_location_id(problem, r1_end - 1)
        } else {
            r1.location_id(problem, r1_start)
        };

        // Compute the cost from the end of the segment in r2 to the next location in r1
        if let Some(r1_end_next) = r1_next {
            fwd_cost += problem.travel_cost(
                v1,
                problem
                    .job_activity(r2.activity_ids[r2_end - 1])
                    .location_id(),
                r1_end_next,
            );
            bwd_cost += problem.travel_cost(
                v1,
                problem
                    .job_activity(r2.activity_ids[r2_start])
                    .location_id(),
                r1_end_next,
            );
        }

        (fwd_cost - r1_removed_cost, bwd_cost - r1_removed_cost)
    }

    // TODO: tests
    pub fn waiting_duration_change_delta(
        &self,
        problem: &VehicleRoutingProblem,
        activity_ids: impl Iterator<Item = ActivityId>,
        start: usize,
        end: usize,
    ) -> SignedDuration {
        if !problem.has_time_windows() || !problem.has_waiting_duration_cost() {
            return SignedDuration::ZERO;
        }

        let mut delta = SignedDuration::ZERO;

        // Compute waiting duration from [start, end)
        let old_duration_in_range = self.fwd_cumulative_waiting_durations[end]
            - self.fwd_cumulative_waiting_durations[start];
        delta -= old_duration_in_range;

        let mut previous_activity_id = if start == 0 {
            None
        } else {
            Some(self.activity_ids[start - 1])
        };

        let mut previous_departure_time = if start == 0 {
            None
        } else {
            Some(self.departure_times[start - 1])
        };

        for activity_id in activity_ids {
            let arrival_time = if let Some(previous_activity_id) = previous_activity_id
                && let Some(previous_departure_time) = previous_departure_time
            {
                compute_activity_arrival_time(
                    problem,
                    self.vehicle_id,
                    previous_activity_id,
                    previous_departure_time,
                    activity_id,
                )
            } else {
                compute_first_activity_arrival_time(problem, self.vehicle_id, activity_id)
            };
            let waiting_duration = compute_waiting_duration(problem, activity_id, arrival_time);
            let departure_time =
                compute_departure_time(problem, arrival_time, waiting_duration, activity_id);
            previous_departure_time = Some(departure_time);
            previous_activity_id = Some(activity_id);

            delta += waiting_duration;
        }

        if end < self.len() {
            let activity_id = self.activity_ids[end];
            let arrival_time = if let Some(previous_activity_id) = previous_activity_id
                && let Some(previous_departure_time) = previous_departure_time
            {
                compute_activity_arrival_time(
                    problem,
                    self.vehicle_id,
                    previous_activity_id,
                    previous_departure_time,
                    activity_id,
                )
            } else {
                compute_first_activity_arrival_time(problem, self.vehicle_id, activity_id)
            };

            let shift = arrival_time.duration_since(self.arrival_times[end]);
            if shift.is_positive() || shift.is_zero() {
                // Later or equal arrival
                // This is the waiting that can absorb the shift, recucing total waiting time
                let bwd_cumulative_waiting_at_end = self.bwd_cumulative_waiting_durations[end + 1];

                delta -= shift.min(bwd_cumulative_waiting_at_end)
            } else {
                // Earlier arrival
                // This is the maximum time that can be added without adding waiting time, excess will become waiting time
                let slack = self.waiting_time_slacks[end];

                // e.g. you arrive 10 minutes early, you can absorb 20 minutes max
                // 10 - 20 = -10 added -> zero
                // e.g. you arrive 30 minutes early, you can absorb 20 minutes max
                // 30 - 20 = 10 added

                let new_waiting_duration = (-shift - slack).max(SignedDuration::ZERO);

                delta += new_waiting_duration
            }
        }

        delta
    }

    /// Checks whether inserting the given job IDs between the given [start, end) indices is valid
    /// Checks the time windows, maximum working duration, and latest end time constraints.
    pub fn is_valid_time_change(
        &self,
        problem: &VehicleRoutingProblem,
        activity_ids: impl Iterator<Item = ActivityId>,
        start: usize,
        end: usize,
    ) -> bool {
        if !problem.has_time_windows()
            && self.vehicle(problem).maximum_working_duration().is_none()
            && self.vehicle(problem).latest_end_time().is_none()
        {
            return true;
        }

        let mut previous_activity_id = if start == 0 {
            None
        } else {
            Some(self.activity_ids[start - 1])
        };

        let mut previous_departure_time = if start == 0 {
            None
        } else {
            Some(self.departure_times[start - 1])
        };

        let mut vehicle_start: Option<Timestamp> = if self.is_empty() {
            None
        } else {
            Some(self.start(problem))
        };

        let current_vehicle_start = vehicle_start;

        let mut vehicle_end: Option<Timestamp> = if self.is_empty() {
            None
        } else {
            Some(self.end(problem))
        };

        for activity_id in activity_ids {
            let arrival_time = if let Some(previous_activity_id) = previous_activity_id
                && let Some(previous_departure_time) = previous_departure_time
            {
                compute_activity_arrival_time(
                    problem,
                    self.vehicle_id,
                    previous_activity_id,
                    previous_departure_time,
                    activity_id,
                )
            } else {
                let first_arrival_time =
                    compute_first_activity_arrival_time(problem, self.vehicle_id, activity_id);

                vehicle_start = Some(compute_vehicle_start(
                    problem,
                    self.vehicle_id,
                    activity_id,
                    first_arrival_time,
                ));

                first_arrival_time
            };

            let waiting_duration = compute_waiting_duration(problem, activity_id, arrival_time);

            let new_departure_time =
                compute_departure_time(problem, arrival_time, waiting_duration, activity_id);
            vehicle_end = Some(compute_vehicle_end(
                problem,
                self.vehicle_id,
                activity_id,
                new_departure_time,
            ));

            if !problem
                .job_activity(activity_id)
                .time_windows()
                .is_satisfied(arrival_time)
            {
                return false;
            }

            previous_activity_id = Some(activity_id);
            previous_departure_time = Some(new_departure_time);
        }

        let mut next_delta = SignedDuration::ZERO;
        if let Some(&next_activity_id) = self.activity_ids.get(end)
            && let Some(previous_departure_time) = previous_departure_time
            && let Some(previous_activity_id) = previous_activity_id
        {
            let current_time_slack = self.fwd_time_slacks[end + 1];
            let current_arrival_time = self.arrival_times[end];

            let arrival_time = compute_activity_arrival_time(
                problem,
                self.vehicle_id,
                previous_activity_id,
                previous_departure_time,
                next_activity_id,
            );

            next_delta = arrival_time.duration_since(current_arrival_time);

            if next_delta > current_time_slack {
                return false;
            }
        } else if end >= self.len()
            && !self.is_empty()
            && let Some(previous_departure_time) = previous_departure_time
            && let Some(previous_activity_id) = previous_activity_id
        {
            let vehicle_end = compute_vehicle_end(
                problem,
                self.vehicle_id,
                previous_activity_id,
                previous_departure_time,
            );

            let current_vehicle_end = self.end(problem);
            let delta = vehicle_end.duration_since(current_vehicle_end);
            if delta > self.fwd_time_slacks[self.len() + 1] {
                return false;
            }
        }

        let start_delta = if let Some(vehicle_start) = vehicle_start
            && let Some(current_vehicle_start) = current_vehicle_start
            && current_vehicle_start != vehicle_start
        {
            current_vehicle_start.duration_since(vehicle_start)
        } else {
            SignedDuration::ZERO
        };

        if let Some(max_working_duration) = self.vehicle(problem).maximum_working_duration() {
            if let Some(vehicle_start) = vehicle_start
                && let Some(vehicle_end) = vehicle_end
                && (self.is_empty() || end >= self.len())
                && vehicle_end.duration_since(vehicle_start) > max_working_duration
            {
                return false;
            }

            // If we arrive earlier (next_delta < 0), we may need to add duration because we end up adding waiting time
            // If we arrive later (next_delta > 0), we need to remove duration because we end up removing waiting time from the next activities
            let delta = if next_delta < SignedDuration::ZERO {
                let waiting_slack = self.waiting_time_slacks[end];

                // next_delta: the amount of work that we actually remove
                // -next_delta - waiting_slack: if we end up adding waiting time, we need to add that time to the total duration of the route
                // TODO: better tests for this
                let work_delta =
                    next_delta + (-next_delta - waiting_slack).max(SignedDuration::ZERO);

                start_delta + work_delta
            } else {
                // When arriving later, we remove waiting time from the next activities
                (start_delta + next_delta - self.bwd_cumulative_waiting_durations[end + 1])
                    .max(SignedDuration::ZERO)
            };

            if self.duration(problem) + delta > max_working_duration {
                return false;
            }
        }

        if let Some(latest_end) = self.vehicle(problem).latest_end_time()
            && let Some(vehicle_end) = vehicle_end
            // Other use cases are already handled by the fwd_time_slacks
            // In these two use cases, the vehicle_start and vehicle_end actually represent the start and end of the route
            && (self.is_empty() || end >= self.len()) && vehicle_end > latest_end
        {
            return false;
        }

        true
    }

    pub fn is_valid_capacity_change(
        &self,
        problem: &VehicleRoutingProblem,
        activity_ids: impl Iterator<Item = ActivityId>,
        start: usize,
        end: usize,
    ) -> bool {
        assert!(start <= end);
        assert!(end <= self.len() + 1);

        let vehicle = self.vehicle(problem);

        // Added delivery load from the new activities
        let mut added_delivery_load = Capacity::with_dimensions(problem.capacity_dimensions());

        // Load from added deliveries minus load from removed deliveries
        let mut delivery_load_delta = Capacity::with_dimensions(problem.capacity_dimensions());

        // Load from added pickups minus load from removed pickups
        let mut pickup_load_delta = Capacity::with_dimensions(problem.capacity_dimensions());

        let mut peak_load_delta = Capacity::with_dimensions(problem.capacity_dimensions());
        let mut load_delta = Capacity::with_dimensions(problem.capacity_dimensions());

        for activity_id in activity_ids {
            let activity = problem.job_activity(activity_id);

            match activity {
                JobActivity::Service(service) => match service.service_type() {
                    ServiceType::Pickup => {
                        pickup_load_delta += service.demand();
                        load_delta += service.demand();
                    }
                    ServiceType::Delivery => {
                        delivery_load_delta += service.demand();
                        added_delivery_load += service.demand();
                        load_delta -= service.demand();
                    }
                },
                JobActivity::ShipmentPickup(shipment) => {
                    load_delta += shipment.demand();
                }
                JobActivity::ShipmentDelivery(shipment) => {
                    load_delta -= shipment.demand();
                }
            }

            if load_delta > peak_load_delta {
                peak_load_delta.update(&load_delta);
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

        // The new initial load is the initial load with the additional or removed deliveries
        let new_initial_load = &self.current_load[0] + &delivery_load_delta;

        // Check 1: check the new initial load against vehicle capacity
        if !is_capacity_satisfied(vehicle.capacity(), &new_initial_load) {
            return false;
        }

        // Check 2: check the peak load in the insertion range
        // current_load[start] is the load before insertion, updated by delivery_load_delta
        let peak_during_insertion =
            &self.current_load[start] + &delivery_load_delta + &peak_load_delta;
        if !is_capacity_satisfied(vehicle.capacity(), &peak_during_insertion) {
            return false;
        }

        // Check 3: check the load at end
        // self.current_load[start] + delivery_load_delta is the new initial load
        // - added_delivery_load because we already delivered them
        // + pickup_load_delta because we may have added or removed pickups
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

    pub fn dump(&self, problem: &VehicleRoutingProblem) {
        println!("Route for vehicle_id: {}", self.vehicle_id);
        if self.is_empty() {
            println!("  (empty route)");
            return;
        }

        println!(
            "Maximum duration {:?}",
            self.vehicle(problem).maximum_working_duration()
        );
        println!(
            "Route duration {:?}",
            self.end(problem).duration_since(self.start(problem))
        );
        println!("Route start {}", self.start(problem));

        for (i, &activity_id) in self.activity_ids.iter().enumerate() {
            println!(
                "  Activity {}: {:?}, Arrival: {}, Departure: {}, Waiting: {} {:?}",
                i,
                activity_id,
                self.arrival_times[i],
                self.departure_times[i],
                self.waiting_durations[i],
                problem.job_activity(activity_id).time_windows()
            );
        }
        println!("Route end {}", self.end(problem));
    }
}

pub struct RouteActivityInfo {
    pub(super) activity_id: ActivityId,
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

    pub fn activity_id(&self) -> ActivityId {
        self.activity_id
    }

    pub fn job<'a>(&self, problem: &'a VehicleRoutingProblem) -> &'a Job {
        problem.job(self.activity_id.job_id())
    }

    pub fn job_activity<'a>(&self, problem: &'a VehicleRoutingProblem) -> JobActivity<'a> {
        problem.job_activity(self.activity_id)
    }
}

#[cfg(test)]
mod tests {

    use std::sync::Arc;

    use jiff::{SignedDuration, Timestamp};

    use crate::{
        problem::{
            capacity::Capacity,
            fleet::Fleet,
            job::{ActivityId, JobIdx},
            service::{ServiceBuilder, ServiceType},
            time_window::TimeWindow,
            travel_cost_matrix::TravelMatrices,
            vehicle::{VehicleBuilder, VehicleIdx},
            vehicle_profile::VehicleProfile,
            vehicle_routing_problem::{VehicleRoutingProblem, VehicleRoutingProblemBuilder},
        },
        solver::{
            insertion::{Insertion, ServiceInsertion, ShipmentInsertion},
            solution::{
                route::WorkingSolutionRoute, route_id::RouteIdx,
                route_update_iterator::RouteUpdateActivityData,
            },
        },
        test_utils::{
            self, TestProblemOptions, TestService, TestShipment, create_mixed_problem,
            create_problem_for_tw_change,
        },
        timestamp,
    };

    fn create_problem() -> VehicleRoutingProblem {
        // 10 locations from (0, 0) to (9, 0)
        let locations = test_utils::create_location_grid(1, 10);

        let mut vehicle_builder = VehicleBuilder::default();
        vehicle_builder.set_depot_location_id(0);
        vehicle_builder.set_capacity(Capacity::from_vec(vec![40.0]));
        vehicle_builder.set_vehicle_id(String::from("vehicle"));
        vehicle_builder.set_profile_id(0);
        vehicle_builder.set_depot_duration(SignedDuration::from_mins(10));
        vehicle_builder.set_return(true);
        vehicle_builder.set_end_depot_duration(SignedDuration::from_mins(5));
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
        builder.set_fleet(Fleet::Finite(vehicles));
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
        builder.set_fleet(Fleet::Finite(vehicles));
        builder.set_services(services);

        builder.build()
    }

    #[test]
    fn test_route_insert() {
        let problem = create_mixed_problem(
            vec![TestService::default(), TestService::default()],
            vec![TestShipment::default(), TestShipment::default()],
            TestProblemOptions::default(),
        );
        let mut route = WorkingSolutionRoute::empty(&problem, VehicleIdx::new(0));

        route.insert(
            &problem,
            &Insertion::Service(ServiceInsertion {
                job_index: JobIdx::new(0),
                position: 0,
                route_id: RouteIdx::new(0),
            }),
        );

        assert_eq!(route.activity_ids, vec![ActivityId::service(0)]);

        route.insert(
            &problem,
            &Insertion::Shipment(ShipmentInsertion {
                pickup_position: 0,
                delivery_position: 1,
                job_index: JobIdx::new(2),
                route_id: RouteIdx::new(0),
            }),
        );

        assert_eq!(
            route.activity_ids,
            vec![
                ActivityId::shipment_pickup(2),
                ActivityId::service(0),
                ActivityId::shipment_delivery(2)
            ]
        );

        route.insert(
            &problem,
            &Insertion::Shipment(ShipmentInsertion {
                pickup_position: 1,
                delivery_position: 1,
                job_index: JobIdx::new(3),
                route_id: RouteIdx::new(0),
            }),
        );

        assert_eq!(
            route.activity_ids,
            vec![
                ActivityId::shipment_pickup(2),
                ActivityId::shipment_pickup(3),
                ActivityId::shipment_delivery(3),
                ActivityId::service(0),
                ActivityId::shipment_delivery(2)
            ]
        );
    }

    #[test]
    fn test_remove_shipment() {
        let problem = create_mixed_problem(
            vec![TestService::default(), TestService::default()],
            vec![TestShipment::default(), TestShipment::default()],
            TestProblemOptions::default(),
        );
        let mut route = WorkingSolutionRoute::empty(&problem, VehicleIdx::new(0));

        route.insert(
            &problem,
            &Insertion::Service(ServiceInsertion {
                job_index: JobIdx::new(0),
                position: 0,
                route_id: RouteIdx::new(0),
            }),
        );

        route.insert(
            &problem,
            &Insertion::Shipment(ShipmentInsertion {
                pickup_position: 0,
                delivery_position: 1,
                job_index: JobIdx::new(2),
                route_id: RouteIdx::new(0),
            }),
        );

        route.insert(
            &problem,
            &Insertion::Shipment(ShipmentInsertion {
                pickup_position: 1,
                delivery_position: 1,
                job_index: JobIdx::new(3),
                route_id: RouteIdx::new(0),
            }),
        );

        assert_eq!(
            route.activity_ids,
            vec![
                ActivityId::shipment_pickup(2),
                ActivityId::shipment_pickup(3),
                ActivityId::shipment_delivery(3),
                ActivityId::service(0),
                ActivityId::shipment_delivery(2)
            ]
        );

        route.remove_activity(&problem, ActivityId::shipment_pickup(2));

        assert_eq!(
            route.activity_ids,
            vec![
                ActivityId::shipment_pickup(3),
                ActivityId::shipment_delivery(3),
                ActivityId::service(0),
            ]
        );

        route.remove_activity(&problem, ActivityId::shipment_delivery(3));
        assert_eq!(route.activity_ids, vec![ActivityId::service(0),]);
    }

    #[test]
    fn test_pending_shipments() {
        let problem = create_mixed_problem(
            vec![TestService::default(), TestService::default()],
            vec![TestShipment::default(), TestShipment::default()],
            TestProblemOptions::default(),
        );
        let mut route = WorkingSolutionRoute::empty(&problem, VehicleIdx::new(0));

        route.insert(
            &problem,
            &Insertion::Service(ServiceInsertion {
                job_index: JobIdx::new(0),
                position: 0,
                route_id: RouteIdx::new(0),
            }),
        );

        route.insert(
            &problem,
            &Insertion::Shipment(ShipmentInsertion {
                pickup_position: 0,
                delivery_position: 1,
                job_index: JobIdx::new(2),
                route_id: RouteIdx::new(0),
            }),
        );

        route.insert(
            &problem,
            &Insertion::Shipment(ShipmentInsertion {
                pickup_position: 1,
                delivery_position: 1,
                job_index: JobIdx::new(3),
                route_id: RouteIdx::new(0),
            }),
        );

        assert_eq!(
            route.activity_ids,
            vec![
                ActivityId::shipment_pickup(2),
                ActivityId::shipment_pickup(3),
                ActivityId::shipment_delivery(3),
                ActivityId::service(0),
                ActivityId::shipment_delivery(2)
            ]
        );

        assert_eq!(route.pending_shipments[0].ones(), vec![2]);
        assert_eq!(route.pending_shipments[1].ones(), vec![2, 3]);
        assert_eq!(route.pending_shipments[2].ones(), vec![2]);
        assert_eq!(route.pending_shipments[3].ones(), vec![2]);
        assert_eq!(route.pending_shipments[4].ones(), Vec::<usize>::new());
    }

    #[test]
    fn test_num_shipments() {
        let problem = create_mixed_problem(
            vec![TestService::default(), TestService::default()],
            vec![TestShipment::default(), TestShipment::default()],
            TestProblemOptions::default(),
        );
        let mut route = WorkingSolutionRoute::empty(&problem, VehicleIdx::new(0));

        route.insert(
            &problem,
            &Insertion::Service(ServiceInsertion {
                job_index: JobIdx::new(0),
                position: 0,
                route_id: RouteIdx::new(0),
            }),
        );

        route.insert(
            &problem,
            &Insertion::Shipment(ShipmentInsertion {
                pickup_position: 0,
                delivery_position: 1,
                job_index: JobIdx::new(2),
                route_id: RouteIdx::new(0),
            }),
        );

        route.insert(
            &problem,
            &Insertion::Shipment(ShipmentInsertion {
                pickup_position: 1,
                delivery_position: 1,
                job_index: JobIdx::new(3),
                route_id: RouteIdx::new(0),
            }),
        );

        assert_eq!(
            route.activity_ids,
            vec![
                ActivityId::shipment_pickup(2),
                ActivityId::shipment_pickup(3),
                ActivityId::shipment_delivery(3),
                ActivityId::service(0),
                ActivityId::shipment_delivery(2)
            ]
        );

        assert_eq!(route.num_shipments[0], 1);
        assert_eq!(route.num_shipments[1], 2);
        assert_eq!(route.num_shipments[2], 3);
        assert_eq!(route.num_shipments[3], 3);
        assert_eq!(route.num_shipments[4], 4);
    }

    #[test]
    fn test_contains_pending_shipment() {
        let problem = create_mixed_problem(
            vec![TestService::default(), TestService::default()],
            vec![TestShipment::default(), TestShipment::default()],
            TestProblemOptions::default(),
        );
        let mut route = WorkingSolutionRoute::empty(&problem, VehicleIdx::new(0));

        route.insert(
            &problem,
            &Insertion::Service(ServiceInsertion {
                job_index: JobIdx::new(0),
                position: 0,
                route_id: RouteIdx::new(0),
            }),
        );

        route.insert(
            &problem,
            &Insertion::Shipment(ShipmentInsertion {
                pickup_position: 0,
                delivery_position: 1,
                job_index: JobIdx::new(2),
                route_id: RouteIdx::new(0),
            }),
        );

        route.insert(
            &problem,
            &Insertion::Shipment(ShipmentInsertion {
                pickup_position: 1,
                delivery_position: 1,
                job_index: JobIdx::new(3),
                route_id: RouteIdx::new(0),
            }),
        );

        assert_eq!(
            route.activity_ids,
            vec![
                ActivityId::shipment_pickup(2),
                ActivityId::shipment_pickup(3),
                ActivityId::shipment_delivery(3),
                ActivityId::service(0),
                ActivityId::shipment_delivery(2)
            ]
        );

        assert!(route.contains_pending_shipment(0, 1));
        assert!(route.contains_pending_shipment(0, 2));
        assert!(route.contains_pending_shipment(0, 3));

        assert!(!route.contains_pending_shipment(1, 3));
        assert!(!route.contains_pending_shipment(1, 4));
        assert!(route.contains_pending_shipment(1, 5));
        assert!(!route.contains_pending_shipment(0, 5));
    }

    #[test]
    fn test_contains_shipments() {
        let problem = create_mixed_problem(
            vec![TestService::default(), TestService::default()],
            vec![TestShipment::default(), TestShipment::default()],
            TestProblemOptions::default(),
        );
        let mut route = WorkingSolutionRoute::empty(&problem, VehicleIdx::new(0));

        route.insert(
            &problem,
            &Insertion::Service(ServiceInsertion {
                job_index: JobIdx::new(0),
                position: 0,
                route_id: RouteIdx::new(0),
            }),
        );

        route.insert(
            &problem,
            &Insertion::Shipment(ShipmentInsertion {
                pickup_position: 0,
                delivery_position: 1,
                job_index: JobIdx::new(2),
                route_id: RouteIdx::new(0),
            }),
        );

        route.insert(
            &problem,
            &Insertion::Shipment(ShipmentInsertion {
                pickup_position: 1,
                delivery_position: 1,
                job_index: JobIdx::new(3),
                route_id: RouteIdx::new(0),
            }),
        );

        assert_eq!(
            route.activity_ids,
            vec![
                ActivityId::shipment_pickup(2),
                ActivityId::shipment_pickup(3),
                ActivityId::shipment_delivery(3),
                ActivityId::service(0),
                ActivityId::shipment_delivery(2)
            ]
        );

        assert!(route.contains_shipments(0, 1));
        assert!(route.contains_shipments(0, 2));
        assert!(route.contains_shipments(0, 3));

        assert!(route.contains_shipments(1, 3));
        assert!(route.contains_shipments(1, 4));
        assert!(route.contains_shipments(1, 5));
        assert!(route.contains_shipments(0, 5));

        assert!(!route.contains_shipments(3, 4));
    }

    #[test]
    fn test_route_data_correctness() {
        let problem = create_problem();

        let mut route = WorkingSolutionRoute::empty(&problem, VehicleIdx::new(0));

        route.insert_service(&problem, 0, JobIdx::new(0));
        route.insert_service(&problem, 1, JobIdx::new(2));
        route.insert_service(&problem, 2, JobIdx::new(1));

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

        assert_eq!(
            route.start(&problem),
            "2025-11-30T09:20:00+02:00".parse().unwrap()
        );

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

        assert_eq!(
            route.end(&problem),
            "2025-11-30T12:05:00+02:00".parse().unwrap()
        );

        // Check waiting durations
        assert_eq!(route.waiting_durations[0], SignedDuration::ZERO);
        assert_eq!(route.waiting_durations[1], SignedDuration::ZERO);
        assert_eq!(route.waiting_durations[2], SignedDuration::ZERO);

        assert_eq!(
            route.fwd_cumulative_waiting_durations[0],
            SignedDuration::ZERO
        );
        assert_eq!(
            route.fwd_cumulative_waiting_durations[1],
            SignedDuration::ZERO
        );
        assert_eq!(
            route.fwd_cumulative_waiting_durations[2],
            SignedDuration::ZERO
        );

        assert_eq!(
            route.bwd_cumulative_waiting_durations[0],
            SignedDuration::ZERO
        );
        assert_eq!(
            route.bwd_cumulative_waiting_durations[1],
            SignedDuration::ZERO
        );
        assert_eq!(
            route.bwd_cumulative_waiting_durations[2],
            SignedDuration::ZERO
        );
        assert_eq!(
            route.bwd_cumulative_waiting_durations[3],
            SignedDuration::ZERO
        );
        assert_eq!(
            route.bwd_cumulative_waiting_durations[4],
            SignedDuration::ZERO
        );

        assert_eq!(route.waiting_time_slacks[0], SignedDuration::ZERO);
        assert_eq!(route.waiting_time_slacks[1], SignedDuration::from_mins(40));
        assert_eq!(route.waiting_time_slacks[2], SignedDuration::from_mins(80));

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
        assert_eq!(route.fwd_time_slacks[0], SignedDuration::from_mins(40));
        assert_eq!(route.fwd_time_slacks[1], SignedDuration::from_mins(40));
        assert_eq!(route.fwd_time_slacks[2], SignedDuration::from_mins(40));
        assert_eq!(route.fwd_time_slacks[3], SignedDuration::from_mins(40));
        assert_eq!(
            route.fwd_time_slacks[4],
            Timestamp::MAX.duration_since(route.end(&problem))
        );
    }

    #[test]
    fn test_jobs_iter() {
        let problem = create_problem();

        let mut route = WorkingSolutionRoute::empty(&problem, VehicleIdx::new(0));

        route.insert_service(&problem, 0, JobIdx::new(0));
        route.insert_service(&problem, 1, JobIdx::new(2));
        route.insert_service(&problem, 2, JobIdx::new(1));

        let job_ids: Vec<ActivityId> = route.activity_ids_iter(0, 3).collect();

        assert_eq!(
            job_ids,
            vec![
                ActivityId::Service(0.into()),
                ActivityId::Service(2.into()),
                ActivityId::Service(1.into())
            ]
        );

        // Same value returns empty iterator
        let job_ids: Vec<ActivityId> = route.activity_ids_iter(0, 0).collect();
        assert_eq!(job_ids, vec![]);
    }

    #[test]
    fn test_route_update_iter() {
        let problem = create_problem();

        let mut route = WorkingSolutionRoute::empty(&problem, VehicleIdx::new(0));

        route.insert_service(&problem, 0, JobIdx::new(0));
        route.insert_service(&problem, 1, JobIdx::new(2));
        route.insert_service(&problem, 2, JobIdx::new(1));

        let mut iterator =
            route.updated_activities_iter(&problem, route.activity_ids_iter(1, 3).rev(), 1, 3);

        assert_eq!(
            iterator.next(),
            Some(RouteUpdateActivityData {
                arrival_time: "2025-11-30T10:40:00+02:00".parse().unwrap(),
                waiting_duration: SignedDuration::ZERO,
                departure_time: "2025-11-30T10:50:00+02:00".parse().unwrap(),
                job_id: ActivityId::service(1),
                current_position: Some(2)
            })
        );

        assert_eq!(
            iterator.next(),
            Some(RouteUpdateActivityData {
                arrival_time: "2025-11-30T11:20:00+02:00".parse().unwrap(),
                waiting_duration: SignedDuration::ZERO,
                departure_time: "2025-11-30T11:30:00+02:00".parse().unwrap(),
                job_id: ActivityId::service(2),
                current_position: Some(1)
            })
        );

        assert_eq!(iterator.next(), None);
    }

    #[test]
    fn test_replace_activities() {
        let problem = create_problem();

        let mut route = WorkingSolutionRoute::empty(&problem, VehicleIdx::new(0));

        route.insert_service(&problem, 0, JobIdx::new(0));
        route.insert_service(&problem, 1, JobIdx::new(2));
        route.insert_service(&problem, 2, JobIdx::new(1));

        route.replace_activities(
            &problem,
            &[
                ActivityId::Service(1.into()),
                ActivityId::Service(2.into()),
                ActivityId::Service(0.into()),
            ],
            0,
            3,
        );

        assert_eq!(route.len(), 3);
        assert_eq!(
            route.activity_ids.to_vec(),
            vec![
                ActivityId::service(1),
                ActivityId::service(2),
                ActivityId::service(0),
            ]
        );

        route.replace_activities(
            &problem,
            &[ActivityId::Service(0.into()), ActivityId::Service(2.into())],
            1,
            3,
        );

        assert_eq!(
            route.activity_ids.to_vec(),
            vec![
                ActivityId::Service(1.into()),
                ActivityId::Service(0.into()),
                ActivityId::Service(2.into())
            ]
        );
    }

    #[test]
    fn test_replace_activities_remove() {
        let problem = create_problem();

        let mut route = WorkingSolutionRoute::empty(&problem, VehicleIdx::new(0));

        route.insert_service(&problem, 0, JobIdx::new(0));
        route.insert_service(&problem, 1, JobIdx::new(2));
        route.insert_service(&problem, 2, JobIdx::new(1));

        route.replace_activities(&problem, &[], 1, 2);

        assert_eq!(route.len(), 2);
        assert_eq!(
            route.activity_ids.to_vec(),
            vec![ActivityId::service(0), ActivityId::service(1)]
        );

        route.replace_activities(
            &problem,
            &[ActivityId::service(2), ActivityId::service(1)],
            1,
            2,
        );

        assert_eq!(route.len(), 3);
        assert_eq!(
            route.activity_ids.to_vec(),
            vec![
                ActivityId::service(0),
                ActivityId::service(2),
                ActivityId::service(1)
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

        let mut route = WorkingSolutionRoute::empty(&problem, VehicleIdx::new(0));
        route.insert_service(&problem, 0, JobIdx::new(0));
        route.insert_service(&problem, 1, JobIdx::new(1));
        route.insert_service(&problem, 2, JobIdx::new(2));

        let is_valid =
            route.is_valid_capacity_change(&problem, std::iter::once(ActivityId::service(4)), 1, 2);
        assert!(is_valid);

        let is_valid =
            route.is_valid_capacity_change(&problem, std::iter::once(ActivityId::service(3)), 1, 2);
        assert!(is_valid);

        let is_valid = route.is_valid_capacity_change(
            &problem,
            [ActivityId::service(1), ActivityId::service(3)].into_iter(),
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
            [ActivityId::service(5), ActivityId::service(6)].into_iter(),
            1,
            3,
        );

        assert!(is_valid);

        // Replace 0 by 4
        let is_valid =
            route.is_valid_capacity_change(&problem, [ActivityId::service(4)].into_iter(), 0, 1);

        assert!(!is_valid);

        // Add 4 before 0
        let is_valid =
            route.is_valid_capacity_change(&problem, [ActivityId::service(4)].into_iter(), 0, 0);

        assert!(!is_valid);

        // Add 4 after 0
        let is_valid =
            route.is_valid_capacity_change(&problem, [ActivityId::service(4)].into_iter(), 1, 1);

        assert!(!is_valid);

        // Replace 1 and 2 by 5 and 4
        let is_valid = route.is_valid_capacity_change(
            &problem,
            [ActivityId::service(5), ActivityId::service(4)].into_iter(),
            1,
            3,
        );

        assert!(!is_valid);

        // Add 6 at end
        let is_valid =
            route.is_valid_capacity_change(&problem, [ActivityId::service(6)].into_iter(), 3, 3);

        assert!(!is_valid);
    }

    /// Build a problem with configurable vehicle capacity, services (type + demand), and
    /// shipments (demand). Services are indexed first (0..n_services), then shipments
    /// (n_services..n_services+n_shipments), matching the convention used in
    /// `create_problem_for_capacity_change`.
    fn create_problem_for_capacity_change_with_shipments(
        vehicle_capacity: Capacity,
        services: Vec<(ServiceType, Capacity)>,
        shipment_demands: Vec<Capacity>,
    ) -> VehicleRoutingProblem {
        use crate::problem::shipment::ShipmentBuilder;

        let n_locations = services.len() + shipment_demands.len() * 2 + 1;
        let locations = test_utils::create_location_grid(1, n_locations);

        let mut vehicle_builder = VehicleBuilder::default();
        vehicle_builder.set_depot_location_id(0);
        vehicle_builder.set_capacity(vehicle_capacity);
        vehicle_builder.set_vehicle_id(String::from("vehicle"));
        vehicle_builder.set_profile_id(0);
        let vehicle = vehicle_builder.build();

        let services = services
            .into_iter()
            .enumerate()
            .map(|(i, (service_type, demand))| {
                let mut b = ServiceBuilder::default();
                b.set_demand(demand);
                b.set_external_id(format!("service_{}", i + 1));
                b.set_service_duration(SignedDuration::from_mins(10));
                b.set_location_id(i + 1);
                b.set_service_type(service_type);
                b.build()
            })
            .collect::<Vec<_>>();

        let n_services = services.len();
        let shipments = shipment_demands
            .into_iter()
            .enumerate()
            .map(|(i, demand)| {
                let mut b = crate::problem::shipment::ShipmentBuilder::default();
                b.set_demand(demand);
                b.set_external_id(format!("shipment_{}", i + 1));
                b.set_pickup_location_id(n_services + i * 2 + 1);
                b.set_delivery_location_id(n_services + i * 2 + 2);
                b.build()
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
        builder.set_fleet(Fleet::Finite(vec![vehicle]));
        builder.set_services(services);
        builder.set_shipments(shipments);

        builder.build()
    }

    // -------------------------------------------------------------------------
    // Tests: is_valid_capacity_change with shipments
    // -------------------------------------------------------------------------
    //
    // Job index layout produced by `create_problem_for_capacity_change_with_shipments`:
    //   services 0..n_services  (delivery or pickup)
    //   shipments n_services..  (each occupying one JobIdx)
    //
    // For a route that only contains shipments (no services), n_services == 0 so
    // the shipments start at JobIdx 0.
    //
    // Capacity semantics recap
    // ------------------------
    // `is_valid_capacity_change(problem, new_activity_ids, start, end)` simulates
    // replacing the half-open segment [start, end) of the current route with the
    // supplied activities and checks that the resulting load profile fits inside
    // the vehicle capacity.
    //
    //   start == end  → pure insertion (nothing removed)
    //   activity_ids empty, start < end → pure removal
    //
    // Shipment pickup  : adds demand to the running load (like a pickup service).
    // Shipment delivery: subtracts demand from the running load.

    /// Shipments only – inserting a single shipment (pickup then delivery) into an
    /// empty route must always succeed when demand ≤ capacity.
    #[test]
    fn test_is_valid_capacity_change_shipments_only_insert_fits() {
        // Vehicle capacity: 30, one shipment with demand 20.
        // n_services = 0 → shipment 0 has JobIdx 0.
        let problem = create_problem_for_capacity_change_with_shipments(
            Capacity::from_vec(vec![30.0]),
            vec![],
            vec![Capacity::from_vec(vec![20.0])],
        );

        let mut route = WorkingSolutionRoute::empty(&problem, VehicleIdx::new(0));

        // Insert pickup then delivery into an empty route.
        let is_valid = route.is_valid_capacity_change(
            &problem,
            [
                ActivityId::shipment_pickup(0),
                ActivityId::shipment_delivery(0),
            ]
            .into_iter(),
            0,
            0,
        );
        assert!(is_valid);
    }

    /// Shipments only – inserting a shipment whose demand exceeds the vehicle
    /// capacity must be rejected.
    #[test]
    fn test_is_valid_capacity_change_shipments_only_insert_exceeds_capacity() {
        // Vehicle capacity: 10, shipment demand 20 → exceeds capacity at pickup.
        let problem = create_problem_for_capacity_change_with_shipments(
            Capacity::from_vec(vec![10.0]),
            vec![],
            vec![Capacity::from_vec(vec![20.0])],
        );

        let mut route = WorkingSolutionRoute::empty(&problem, VehicleIdx::new(0));

        let is_valid = route.is_valid_capacity_change(
            &problem,
            [
                ActivityId::shipment_pickup(0),
                ActivityId::shipment_delivery(0),
            ]
            .into_iter(),
            0,
            0,
        );
        assert!(!is_valid);
    }

    /// Shipments only – inserting a second shipment between the first shipment's
    /// pickup and delivery.  The combined load at the inner segment must fit.
    ///
    /// Route after first insert: [P0, D0]
    /// Swap: insert P1 at position 1, D1 at position 1 (nested: P0 P1 D1 D0).
    /// Peak load = demand_0 + demand_1; must be ≤ capacity.
    #[test]
    fn test_is_valid_capacity_change_shipments_only_nested_fits() {
        // Two shipments each with demand 15; vehicle capacity 30.
        // Nested layout peak = 30 which is exactly at capacity → valid.
        let problem = create_problem_for_capacity_change_with_shipments(
            Capacity::from_vec(vec![30.0]),
            vec![],
            vec![
                Capacity::from_vec(vec![15.0]), // shipment 0
                Capacity::from_vec(vec![15.0]), // shipment 1
            ],
        );

        let mut route = WorkingSolutionRoute::empty(&problem, VehicleIdx::new(0));
        // Build route: [P0, D0]
        route.insert(
            &problem,
            &Insertion::Shipment(ShipmentInsertion {
                pickup_position: 0,
                delivery_position: 0,
                job_index: JobIdx::new(0),
                route_id: RouteIdx::new(0),
            }),
        );
        // route is now [P0, D0]

        // Insert P1+D1 between P0 and D0 (position 1, 1 → nested).
        let is_valid = route.is_valid_capacity_change(
            &problem,
            [
                ActivityId::shipment_pickup(1),
                ActivityId::shipment_delivery(1),
            ]
            .into_iter(),
            1,
            1,
        );
        assert!(is_valid);
    }

    /// Shipments only – nested layout where combined demand exceeds capacity.
    #[test]
    fn test_is_valid_capacity_change_shipments_only_nested_exceeds_capacity() {
        // Two shipments with demand 20 each; vehicle capacity 30.
        // Nested peak = 40 > 30 → invalid.
        let problem = create_problem_for_capacity_change_with_shipments(
            Capacity::from_vec(vec![30.0]),
            vec![],
            vec![
                Capacity::from_vec(vec![20.0]), // shipment 0
                Capacity::from_vec(vec![20.0]), // shipment 1
            ],
        );

        let mut route = WorkingSolutionRoute::empty(&problem, VehicleIdx::new(0));
        route.insert(
            &problem,
            &Insertion::Shipment(ShipmentInsertion {
                pickup_position: 0,
                delivery_position: 0,
                job_index: JobIdx::new(0),
                route_id: RouteIdx::new(0),
            }),
        );

        let is_valid = route.is_valid_capacity_change(
            &problem,
            [
                ActivityId::shipment_pickup(1),
                ActivityId::shipment_delivery(1),
            ]
            .into_iter(),
            1,
            1,
        );
        assert!(!is_valid);
    }

    /// Shipments only – replacing one shipment with another of smaller demand is valid.
    #[test]
    fn test_is_valid_capacity_change_shipments_only_replace_fits() {
        // Route: [P0, D0] with demand 25.  Replace with shipment 1 demand 10.
        // Vehicle capacity 30. After replacement peak = 10 ≤ 30 → valid.
        let problem = create_problem_for_capacity_change_with_shipments(
            Capacity::from_vec(vec![30.0]),
            vec![],
            vec![
                Capacity::from_vec(vec![25.0]), // shipment 0
                Capacity::from_vec(vec![10.0]), // shipment 1
            ],
        );

        let mut route = WorkingSolutionRoute::empty(&problem, VehicleIdx::new(0));
        route.insert(
            &problem,
            &Insertion::Shipment(ShipmentInsertion {
                pickup_position: 0,
                delivery_position: 0,
                job_index: JobIdx::new(0),
                route_id: RouteIdx::new(0),
            }),
        );
        // route: [P0, D0]

        // Replace both activities [0, 2) with shipment 1.
        let is_valid = route.is_valid_capacity_change(
            &problem,
            [
                ActivityId::shipment_pickup(1),
                ActivityId::shipment_delivery(1),
            ]
            .into_iter(),
            0,
            2,
        );
        assert!(is_valid);
    }

    /// Shipments only – replacing one shipment with another of larger demand that
    /// would exceed capacity is invalid.
    #[test]
    fn test_is_valid_capacity_change_shipments_only_replace_exceeds_capacity() {
        // Route: [P0, D0] with demand 10.  Replace with shipment 1 demand 40.
        // Vehicle capacity 30. After replacement peak = 40 > 30 → invalid.
        let problem = create_problem_for_capacity_change_with_shipments(
            Capacity::from_vec(vec![30.0]),
            vec![],
            vec![
                Capacity::from_vec(vec![10.0]), // shipment 0
                Capacity::from_vec(vec![40.0]), // shipment 1
            ],
        );

        let mut route = WorkingSolutionRoute::empty(&problem, VehicleIdx::new(0));
        route.insert(
            &problem,
            &Insertion::Shipment(ShipmentInsertion {
                pickup_position: 0,
                delivery_position: 0,
                job_index: JobIdx::new(0),
                route_id: RouteIdx::new(0),
            }),
        );

        let is_valid = route.is_valid_capacity_change(
            &problem,
            [
                ActivityId::shipment_pickup(1),
                ActivityId::shipment_delivery(1),
            ]
            .into_iter(),
            0,
            2,
        );
        assert!(!is_valid);
    }

    /// Shipments only – purely removing a shipment (empty activity list, segment
    /// covers both pickup and delivery) must always succeed.
    #[test]
    fn test_is_valid_capacity_change_shipments_only_removal() {
        let problem = create_problem_for_capacity_change_with_shipments(
            Capacity::from_vec(vec![30.0]),
            vec![],
            vec![Capacity::from_vec(vec![20.0])],
        );

        let mut route = WorkingSolutionRoute::empty(&problem, VehicleIdx::new(0));
        route.insert(
            &problem,
            &Insertion::Shipment(ShipmentInsertion {
                pickup_position: 0,
                delivery_position: 0,
                job_index: JobIdx::new(0),
                route_id: RouteIdx::new(0),
            }),
        );
        // route: [P0, D0]

        // Remove both → empty iter, segment [0, 2)
        let is_valid = route.is_valid_capacity_change(&problem, [].into_iter(), 0, 2);
        assert!(is_valid);
    }

    // -----------------------------------------------------------------------
    // Shipments mixed with delivery services
    // -----------------------------------------------------------------------
    //
    // Job index layout (n_services = 2):
    //   0 → delivery service_1 (demand 20)
    //   1 → delivery service_2 (demand 20)
    //   2 → shipment_1         (demand 15)
    //   3 → shipment_2         (demand 15)
    //
    // Vehicle capacity: 50.

    /// Mixed: inserting a shipment into a route that already has delivery services
    /// – combined load must fit.
    #[test]
    fn test_is_valid_capacity_change_shipments_mixed_delivery_services_fits() {
        // Route: [D_svc0(20), D_svc1(20)]
        // Initial load = 40 (deliveries are pre-loaded).
        // Insert P2+D2 (shipment demand 5) at end → peak at pickup point = 40+5 = 45 ≤ 50 → valid.
        let problem = create_problem_for_capacity_change_with_shipments(
            Capacity::from_vec(vec![50.0]),
            vec![
                (ServiceType::Delivery, Capacity::from_vec(vec![20.0])),
                (ServiceType::Delivery, Capacity::from_vec(vec![20.0])),
            ],
            vec![
                Capacity::from_vec(vec![5.0]),  // shipment 0 (JobIdx 2)
                Capacity::from_vec(vec![15.0]), // shipment 1 (JobIdx 3)
            ],
        );

        let mut route = WorkingSolutionRoute::empty(&problem, VehicleIdx::new(0));
        route.insert_service(&problem, 0, JobIdx::new(0));
        route.insert_service(&problem, 1, JobIdx::new(1));
        // route: [D_svc0, D_svc1]

        // Insert shipment 2 (demand 5) at the end – after both deliveries have
        // been made, load is 0 at that point → shipment pickup adds 5 → fine.
        let is_valid = route.is_valid_capacity_change(
            &problem,
            [
                ActivityId::shipment_pickup(2),
                ActivityId::shipment_delivery(2),
            ]
            .into_iter(),
            2,
            2,
        );
        assert!(is_valid);
    }

    /// Mixed: inserting a shipment *before* the delivery services so the combined
    /// peak (initial delivery load + shipment demand) exceeds capacity.
    #[test]
    fn test_is_valid_capacity_change_shipments_mixed_delivery_services_exceeds_capacity() {
        // Route: [D_svc0(20), D_svc1(20)], vehicle capacity 30.
        // Inserting shipment with demand 15 at the front raises peak at pickup
        // to 40+15 = 55 > 30 → invalid.
        let problem = create_problem_for_capacity_change_with_shipments(
            Capacity::from_vec(vec![30.0]),
            vec![
                (ServiceType::Delivery, Capacity::from_vec(vec![10.0])),
                (ServiceType::Delivery, Capacity::from_vec(vec![10.0])),
            ],
            vec![Capacity::from_vec(vec![15.0])], // shipment 0 (JobIdx 2)
        );

        let mut route = WorkingSolutionRoute::empty(&problem, VehicleIdx::new(0));
        route.insert_service(&problem, 0, JobIdx::new(0));
        route.insert_service(&problem, 1, JobIdx::new(1));
        // route: [D_svc0, D_svc1], initial load = 20

        // Insert shipment pickup before first delivery → peak = 20 + 15 = 35 > 30.
        let is_valid = route.is_valid_capacity_change(
            &problem,
            [
                ActivityId::shipment_pickup(2),
                ActivityId::shipment_delivery(2),
            ]
            .into_iter(),
            0,
            0,
        );
        assert!(!is_valid);
    }

    /// Mixed: replacing a heavy delivery service with a lighter shipment that frees
    /// enough capacity for the shipment to fit.
    #[test]
    fn test_is_valid_capacity_change_shipments_mixed_replace_delivery_with_shipment_fits() {
        // Services: svc0(30), svc1(10).  Shipment: demand 5.
        // Vehicle capacity: 35.
        // Route: [D_svc0(30), D_svc1(10)], initial load = 40 which would fail by
        // itself – but capacity_change validation starts from the current route.
        //
        // Let's use a cleaner setup: svc0(20), svc1(20), capacity 30.
        // Route: [D_svc0, D_svc1] → initial 40 already invalid... use capacity 40.
        //
        // Route: [D_svc0(20), D_svc1(10)], capacity 30.  Initial load = 30.
        // Replace D_svc1 (position 1) with shipment demand 5:
        //   new initial load = 20+5 = 25 ≤ 30 (shipment load tracked separately).
        //   Actually shipment demand does NOT affect initial load, only pickup adds.
        //   new initial load stays 20, peak = 20 + 5 = 25 ≤ 30 → valid.
        let problem = create_problem_for_capacity_change_with_shipments(
            Capacity::from_vec(vec![30.0]),
            vec![
                (ServiceType::Delivery, Capacity::from_vec(vec![20.0])),
                (ServiceType::Delivery, Capacity::from_vec(vec![10.0])),
            ],
            vec![Capacity::from_vec(vec![5.0])], // shipment 0 (JobIdx 2)
        );

        let mut route = WorkingSolutionRoute::empty(&problem, VehicleIdx::new(0));
        route.insert_service(&problem, 0, JobIdx::new(0));
        route.insert_service(&problem, 1, JobIdx::new(1));
        // route: [D_svc0(20), D_svc1(10)]

        // Replace D_svc1 [position 1, 2) with a shipment pickup+delivery.
        let is_valid = route.is_valid_capacity_change(
            &problem,
            [
                ActivityId::shipment_pickup(2),
                ActivityId::shipment_delivery(2),
            ]
            .into_iter(),
            1,
            2,
        );
        assert!(is_valid);
    }

    // -----------------------------------------------------------------------
    // Shipments mixed with pickup services
    // -----------------------------------------------------------------------
    //
    // Pickup services accumulate load as the route progresses (they are picked up
    // from customers), so peak load occurs toward the end of the route rather than
    // at the start.
    //
    // Job index layout (n_services = 2):
    //   0 → pickup service_1
    //   1 → pickup service_2
    //   2 → shipment_1
    //   3 → shipment_2
    //
    // Vehicle capacity: 30.

    /// Pickup-mode services + shipment: inserting a shipment into a route of pickup
    /// services that still has remaining capacity is valid.
    #[test]
    fn test_is_valid_capacity_change_shipments_mixed_pickup_services_fits() {
        // Pickup services each demand 10, vehicle capacity 30.
        // Route: [P_svc0(10), P_svc1(10)]
        // Cumulative pickup load grows: 10 after svc0, 20 after svc1.
        // Insert shipment (demand 5) at the end → extra load during P2 = 20+5=25 ≤ 30 → valid.
        let problem = create_problem_for_capacity_change_with_shipments(
            Capacity::from_vec(vec![30.0]),
            vec![
                (ServiceType::Pickup, Capacity::from_vec(vec![10.0])),
                (ServiceType::Pickup, Capacity::from_vec(vec![10.0])),
            ],
            vec![
                Capacity::from_vec(vec![5.0]), // shipment 0 (JobIdx 2)
                Capacity::from_vec(vec![5.0]), // shipment 1 (JobIdx 3)
            ],
        );

        let mut route = WorkingSolutionRoute::empty(&problem, VehicleIdx::new(0));
        route.insert_service(&problem, 0, JobIdx::new(0));
        route.insert_service(&problem, 1, JobIdx::new(1));
        // route: [P_svc0, P_svc1]

        let is_valid = route.is_valid_capacity_change(
            &problem,
            [
                ActivityId::shipment_pickup(2),
                ActivityId::shipment_delivery(2),
            ]
            .into_iter(),
            2,
            2,
        );
        assert!(is_valid);
    }

    /// Pickup-mode services + shipment: cumulative load after pickup services plus
    /// shipment demand exceeds capacity → invalid.
    #[test]
    fn test_is_valid_capacity_change_shipments_mixed_pickup_services_exceeds_capacity() {
        // Pickup services each demand 10, vehicle capacity 25.
        // Route: [P_svc0(10), P_svc1(10)] → cumulative 20 at end.
        // Insert shipment demand 10 → peak = 20+10 = 30 > 25 → invalid.
        let problem = create_problem_for_capacity_change_with_shipments(
            Capacity::from_vec(vec![25.0]),
            vec![
                (ServiceType::Pickup, Capacity::from_vec(vec![10.0])),
                (ServiceType::Pickup, Capacity::from_vec(vec![10.0])),
            ],
            vec![Capacity::from_vec(vec![10.0])], // shipment 0 (JobIdx 2)
        );

        let mut route = WorkingSolutionRoute::empty(&problem, VehicleIdx::new(0));
        route.insert_service(&problem, 0, JobIdx::new(0));
        route.insert_service(&problem, 1, JobIdx::new(1));

        let is_valid = route.is_valid_capacity_change(
            &problem,
            [
                ActivityId::shipment_pickup(2),
                ActivityId::shipment_delivery(2),
            ]
            .into_iter(),
            2,
            2,
        );
        assert!(!is_valid);
    }

    #[test]
    fn test_is_valid_capacity_change_shipments_mixed_pickup_services_fits_capacity_at_start() {
        let problem = create_problem_for_capacity_change_with_shipments(
            Capacity::from_vec(vec![25.0]),
            vec![
                (ServiceType::Pickup, Capacity::from_vec(vec![10.0])),
                (ServiceType::Pickup, Capacity::from_vec(vec![10.0])),
            ],
            vec![Capacity::from_vec(vec![10.0])], // shipment 0 (JobIdx 2)
        );

        let mut route = WorkingSolutionRoute::empty(&problem, VehicleIdx::new(0));
        route.insert_service(&problem, 0, JobIdx::new(0));
        route.insert_service(&problem, 1, JobIdx::new(1));

        let is_valid = route.is_valid_capacity_change(
            &problem,
            [
                ActivityId::shipment_pickup(2),
                ActivityId::shipment_delivery(2),
            ]
            .into_iter(),
            0,
            0,
        );
        assert!(is_valid);
    }

    #[test]
    fn test_is_valid_capacity_change_shipments_mixed_pickup_services_replace_all() {
        let problem = create_problem_for_capacity_change_with_shipments(
            Capacity::from_vec(vec![25.0]),
            vec![
                (ServiceType::Pickup, Capacity::from_vec(vec![10.0])),
                (ServiceType::Pickup, Capacity::from_vec(vec![10.0])),
            ],
            vec![Capacity::from_vec(vec![10.0])], // shipment 0 (JobIdx 2)
        );

        let mut route = WorkingSolutionRoute::empty(&problem, VehicleIdx::new(0));
        route.insert_service(&problem, 0, JobIdx::new(0));
        route.insert_service(&problem, 1, JobIdx::new(1));

        let is_valid = route.is_valid_capacity_change(
            &problem,
            [
                ActivityId::shipment_pickup(2),
                ActivityId::shipment_delivery(2),
                ActivityId::service(0),
                ActivityId::service(1),
            ]
            .into_iter(),
            0,
            3,
        );
        assert!(is_valid);
    }

    /// Pickup-mode services + shipment: replacing a heavy pickup service with a
    /// shipment whose load is lower results in a valid change.
    #[test]
    fn test_is_valid_capacity_change_shipments_mixed_replace_pickup_with_shipment_fits() {
        // Pickup services: svc0(20), svc1(10). Vehicle capacity 25.
        // Route: [P_svc0(20), P_svc1(10)] → peaks at 30 (already violates on its
        // own, but we are validating a proposed *change*, not the existing route).
        //
        // Replace P_svc0 with shipment (demand 5).
        // New effective pickup flow: shipment pickup (5) then svc1 pickup (10)
        // → peak at end = 15 ≤ 25 → valid.
        let problem = create_problem_for_capacity_change_with_shipments(
            Capacity::from_vec(vec![25.0]),
            vec![
                (ServiceType::Pickup, Capacity::from_vec(vec![20.0])),
                (ServiceType::Pickup, Capacity::from_vec(vec![10.0])),
            ],
            vec![Capacity::from_vec(vec![5.0])], // shipment 0 (JobIdx 2)
        );

        let mut route = WorkingSolutionRoute::empty(&problem, VehicleIdx::new(0));
        route.insert_service(&problem, 0, JobIdx::new(0));
        route.insert_service(&problem, 1, JobIdx::new(1));
        // route: [P_svc0(20), P_svc1(10)]

        // Replace P_svc0 [position 0, 1) with shipment pickup+delivery.
        let is_valid = route.is_valid_capacity_change(
            &problem,
            [
                ActivityId::shipment_pickup(2),
                ActivityId::shipment_delivery(2),
            ]
            .into_iter(),
            0,
            1,
        );
        assert!(is_valid);
    }

    /// Adds a shipment and a pickup one after the other with fitting capacities
    #[test]
    fn test_is_valid_capacity_change_shipments_mixed_pickup_with_shipment_fits() {
        let problem = create_problem_for_capacity_change_with_shipments(
            Capacity::from_vec(vec![40.0]),
            vec![
                (ServiceType::Delivery, Capacity::from_vec(vec![20.0])),
                (ServiceType::Delivery, Capacity::from_vec(vec![10.0])),
                (ServiceType::Pickup, Capacity::from_vec(vec![10.0])),
            ],
            vec![Capacity::from_vec(vec![10.0])], // shipment 0 (JobIdx 2)
        );

        let mut route = WorkingSolutionRoute::empty(&problem, VehicleIdx::new(0));
        route.insert_service(&problem, 0, JobIdx::new(0));
        route.insert_service(&problem, 1, JobIdx::new(1));

        let is_valid = route.is_valid_capacity_change(
            &problem,
            [
                ActivityId::shipment_pickup(3),
                ActivityId::shipment_delivery(3),
                ActivityId::service(2),
            ]
            .into_iter(),
            0,
            0,
        );
        assert!(is_valid);
    }

    /// Adds a shipment pickup and a pickup and the shipment delivery after it, exceeding capacity
    #[test]
    fn test_is_valid_capacity_change_shipments_mixed_pickup_with_shipment_exceeds_capacity() {
        let problem = create_problem_for_capacity_change_with_shipments(
            Capacity::from_vec(vec![40.0]),
            vec![
                (ServiceType::Delivery, Capacity::from_vec(vec![20.0])),
                (ServiceType::Delivery, Capacity::from_vec(vec![10.0])),
                (ServiceType::Pickup, Capacity::from_vec(vec![10.0])),
            ],
            vec![Capacity::from_vec(vec![10.0])],
        );

        let mut route = WorkingSolutionRoute::empty(&problem, VehicleIdx::new(0));
        route.insert_service(&problem, 0, JobIdx::new(0));
        route.insert_service(&problem, 1, JobIdx::new(1));

        let is_valid = route.is_valid_capacity_change(
            &problem,
            [
                ActivityId::shipment_pickup(3),
                ActivityId::service(2),
                ActivityId::shipment_delivery(3),
            ]
            .into_iter(),
            0,
            0,
        );
        assert!(!is_valid);
    }

    /// Adds a shipment pickup and a pickup and the shipment delivery after it, exceeding capacity
    #[test]
    fn test_is_valid_capacity_change_shipments_mixed_move_shipment() {
        let problem = create_problem_for_capacity_change_with_shipments(
            Capacity::from_vec(vec![40.0]),
            vec![
                (ServiceType::Delivery, Capacity::from_vec(vec![10.0])),
                (ServiceType::Delivery, Capacity::from_vec(vec![10.0])),
                (ServiceType::Delivery, Capacity::from_vec(vec![10.0])),
                (ServiceType::Delivery, Capacity::from_vec(vec![10.0])),
                (ServiceType::Pickup, Capacity::from_vec(vec![40.0])),
            ],
            vec![Capacity::from_vec(vec![10.0])], // shipment 0 (JobIdx 2)
        );

        let mut route = WorkingSolutionRoute::empty(&problem, VehicleIdx::new(0));
        route.insert_service(&problem, 0, JobIdx::new(0));
        route.insert_service(&problem, 1, JobIdx::new(1));
        route.insert_shipment(&problem, 2, 2, JobIdx::new(5));
        route.insert_service(&problem, 3, JobIdx::new(2));
        route.insert_service(&problem, 4, JobIdx::new(3));
        route.insert_service(&problem, 6, JobIdx::new(4));

        assert_eq!(
            route.activity_ids,
            vec![
                // Capacity at start: 40
                ActivityId::service(0),           // -10 -> 30
                ActivityId::service(1),           // -10 -> 20
                ActivityId::shipment_pickup(5),   // +10 -> 30
                ActivityId::service(2),           // -10 -> 20
                ActivityId::service(3),           // -10 -> 10
                ActivityId::shipment_delivery(5), // -10 -> 0
                ActivityId::service(4),           // +40 -> 40
            ]
        );

        assert_eq!(route.current_load[0], Capacity::from_vec(vec![40.0]));
        assert_eq!(route.max_load(&problem), 1.0);

        // Moving the shipment pickup at the start exceeds the capacity
        let is_valid = route.is_valid_capacity_change(
            &problem,
            [
                ActivityId::shipment_pickup(5),
                ActivityId::service(0),
                ActivityId::service(1),
            ]
            .into_iter(),
            0,
            3,
        );
        assert!(!is_valid);

        // Moving the shipment pickup at position 1 fits
        let is_valid = route.is_valid_capacity_change(
            &problem,
            [ActivityId::shipment_pickup(5), ActivityId::service(1)].into_iter(),
            1,
            3,
        );
        assert!(is_valid);

        // Moving the shipment delivery after the pickup exceeds the capacity
        let is_valid = route.is_valid_capacity_change(
            &problem,
            [ActivityId::service(4), ActivityId::shipment_delivery(5)].into_iter(),
            5,
            7,
        );
        assert!(!is_valid);
    }

    /// Check 3: inserting a pickup before the suffix must account for the suffix peak.
    ///
    /// Route: [P_svc0(10), P_svc1(10)]  capacity = 20
    ///   current_load: [0, 10, 20]
    ///   bwd_load_peaks: peak from each position to end
    ///
    /// Insert P_svc2(5) at position 1 (start=1, end=1), keeping svc1 in the suffix.
    ///   Check 2 (peak during insertion): current_load[1] + 5 = 15 ≤ 20  → passes
    ///   Check 3 (suffix peak):           15 + 10 (from svc1) = 25 > 20  → must reject
    #[test]
    fn test_is_valid_capacity_change_pickup_suffix_overflow() {
        let problem = create_problem_for_capacity_change(
            Capacity::from_vec(vec![20.0]),
            vec![
                (ServiceType::Pickup, Capacity::from_vec(vec![10.0])), // svc0  JobIdx 0
                (ServiceType::Pickup, Capacity::from_vec(vec![10.0])), // svc1  JobIdx 1
                (ServiceType::Pickup, Capacity::from_vec(vec![5.0])),  // svc2  JobIdx 2
            ],
        );

        let mut route = WorkingSolutionRoute::empty(&problem, VehicleIdx::new(0));
        route.insert_service(&problem, 0, JobIdx::new(0));
        route.insert_service(&problem, 1, JobIdx::new(1));
        // route: [P_svc0(10), P_svc1(10)]
        // loads:  10            20
        // Both pickups together exactly fill capacity.

        // Insert svc2(5) between svc0 and svc1: Check 2 sees only 10+5=15 ≤ 20,
        // but the suffix (svc1, +10) would push the total to 25 > 20 → invalid.
        let is_valid =
            route.is_valid_capacity_change(&problem, [ActivityId::service(2)].into_iter(), 1, 1);
        assert!(!is_valid);

        // Sanity: inserting svc2 *after* svc1 (at the very end) is also invalid because
        // the load at that point is already 20 and adding 5 more gives 25 > 20.
        let is_valid =
            route.is_valid_capacity_change(&problem, [ActivityId::service(2)].into_iter(), 2, 2);
        assert!(!is_valid);

        // Sanity: with more capacity (25) the mid-insertion is valid.
        let problem_larger = create_problem_for_capacity_change(
            Capacity::from_vec(vec![25.0]),
            vec![
                (ServiceType::Pickup, Capacity::from_vec(vec![10.0])),
                (ServiceType::Pickup, Capacity::from_vec(vec![10.0])),
                (ServiceType::Pickup, Capacity::from_vec(vec![5.0])),
            ],
        );
        let mut route_larger = WorkingSolutionRoute::empty(&problem_larger, VehicleIdx::new(0));
        route_larger.insert_service(&problem_larger, 0, JobIdx::new(0));
        route_larger.insert_service(&problem_larger, 1, JobIdx::new(1));

        let is_valid = route_larger.is_valid_capacity_change(
            &problem_larger,
            [ActivityId::service(2)].into_iter(),
            1,
            1,
        );
        assert!(is_valid);
    }

    #[test]
    fn test_is_valid_capacity_change_pickup_suffix_overflow_with_shipments() {
        let problem = create_problem_for_capacity_change_with_shipments(
            Capacity::from_vec(vec![30.0]),
            vec![
                (ServiceType::Pickup, Capacity::from_vec(vec![10.0])), // svc0  JobIdx 0
                (ServiceType::Pickup, Capacity::from_vec(vec![10.0])), // svc1  JobIdx 1
                (ServiceType::Pickup, Capacity::from_vec(vec![10.0])), // svc2  JobIdx 2
            ],
            vec![Capacity::from_vec(vec![10.0])],
        );

        let mut route = WorkingSolutionRoute::empty(&problem, VehicleIdx::new(0));
        route.insert_service(&problem, 0, JobIdx::new(0));
        route.insert_service(&problem, 1, JobIdx::new(1));
        route.insert_shipment(&problem, 2, 2, JobIdx::new(3));

        let is_valid =
            route.is_valid_capacity_change(&problem, [ActivityId::service(2)].into_iter(), 4, 4);
        assert!(is_valid);

        let is_valid =
            route.is_valid_capacity_change(&problem, [ActivityId::service(2)].into_iter(), 3, 3);
        assert!(!is_valid);

        let is_valid = route.is_valid_capacity_change(
            &problem,
            [ActivityId::service(2), ActivityId::shipment_delivery(3)].into_iter(),
            3,
            4,
        );
        assert!(!is_valid);

        let mut route = WorkingSolutionRoute::empty(&problem, VehicleIdx::new(0));
        route.insert_service(&problem, 0, JobIdx::new(0));
        route.insert_service(&problem, 1, JobIdx::new(1));
        route.insert_service(&problem, 2, JobIdx::new(2));

        let is_valid = route.is_valid_capacity_change(
            &problem,
            [
                ActivityId::shipment_pickup(3),
                ActivityId::shipment_delivery(3),
            ]
            .into_iter(),
            0,
            0,
        );
        assert!(is_valid);
    }

    #[test]
    fn test_is_valid_tw_change_delivery_only() {
        // Vehicle starts at depot (location 0) at 08:00
        // Travel time between locations is 30 mins, service duration is 10 mins
        // So arrival at location 1 is 08:30, departure is 08:40
        // Arrival at location 2 is 09:10, departure is 09:20
        // etc.
        let problem = create_problem_for_tw_change(
            vec![
                TestService::with_time_window(TimeWindow::new(
                    timestamp!("2025-11-30T08:00:00+02:00"),
                    timestamp!("2025-11-30T09:00:00+02:00"),
                )),
                TestService::with_time_window(TimeWindow::new(
                    timestamp!("2025-11-30T08:00:00+02:00"),
                    timestamp!("2025-11-30T10:00:00+02:00"),
                )),
                TestService::with_time_window(TimeWindow::new(
                    timestamp!("2025-11-30T08:00:00+02:00"),
                    timestamp!("2025-11-30T11:00:00+02:00"),
                )),
                TestService::with_time_window(TimeWindow::new(
                    timestamp!("2025-11-30T08:00:00+02:00"),
                    timestamp!("2025-11-30T12:00:00+02:00"),
                )),
                TestService::with_time_window(TimeWindow::new(
                    timestamp!("2025-11-30T08:00:00+02:00"),
                    timestamp!("2025-11-30T08:35:00+02:00"),
                )),
                TestService::with_time_window(TimeWindow::new(
                    timestamp!("2025-11-30T08:00:00+02:00"),
                    timestamp!("2025-11-30T14:00:00+02:00"),
                )),
            ],
            TestProblemOptions::default(),
        );

        let mut route = WorkingSolutionRoute::empty(&problem, VehicleIdx::new(0));
        // Route: 0 -> 1 -> 2 (arrival times: 08:30, 09:10, 09:50)
        route.insert_service(&problem, 0, JobIdx::new(0));
        route.insert_service(&problem, 1, JobIdx::new(1));
        route.insert_service(&problem, 2, JobIdx::new(2));

        // Test 1: Replace service 1 with service 3 (same position, later TW)
        // This should be valid since service 3 has a later end time
        let is_valid =
            route.is_valid_time_change(&problem, std::iter::once(ActivityId::service(3)), 1, 2);
        assert!(is_valid);

        // Test 2: Replace service 1 with service 5 (relaxed TW)
        let is_valid =
            route.is_valid_time_change(&problem, std::iter::once(ActivityId::service(5)), 1, 2);
        assert!(is_valid);

        // Test 3: Insert service 4 (tight TW ending at 08:35) before service 0
        // Service 0 arrives at 08:30, so inserting 4 before would push 0 later
        // Service 4 arrives at 08:30 which is before its TW end of 08:35 - valid
        let is_valid =
            route.is_valid_time_change(&problem, std::iter::once(ActivityId::service(4)), 0, 0);
        assert!(is_valid);

        // Test 4: Insert service 4 (tight TW) after service 0
        // After serving 0 (depart 08:40), arrive at 4's location at 09:10
        // But service 4's TW ends at 08:35, so this should be invalid
        let is_valid =
            route.is_valid_time_change(&problem, std::iter::once(ActivityId::service(4)), 1, 1);
        assert!(!is_valid);

        // Test 5: Replace service 0 with service 4
        // Service 4 would arrive at 08:30 (within TW ending 08:35) - valid
        let is_valid =
            route.is_valid_time_change(&problem, std::iter::once(ActivityId::service(4)), 0, 1);
        assert!(is_valid);

        // Test 6: Insert service 5 at end
        // After route 0->1->2, departing at 10:00, arrive at 5 at 10:30
        // Service 5's TW ends at 14:00, so this is valid
        let is_valid =
            route.is_valid_time_change(&problem, std::iter::once(ActivityId::service(5)), 3, 3);
        assert!(is_valid);

        // Test 7: Remove service 0 (replace with nothing)
        // This should be valid as it only makes subsequent services earlier
        let is_valid = route.is_valid_time_change(&problem, [].into_iter(), 0, 1);
        assert!(is_valid);

        // Test 8: Remove service 2 (replace with nothing)
        // This should be valid
        let is_valid = route.is_valid_time_change(&problem, [].into_iter(), 2, 3);
        assert!(is_valid);

        // Test 9: Insert 3 and 5, replace 1 by 3
        let is_valid = route.is_valid_time_change(
            &problem,
            [ActivityId::service(3), ActivityId::service(5)].into_iter(),
            1,
            2,
        );
        assert!(is_valid);

        // Test 9: Insert 4 and 5, replace 1 by 4
        let is_valid = route.is_valid_time_change(
            &problem,
            [ActivityId::service(4), ActivityId::service(5)].into_iter(),
            1,
            2,
        );
        assert!(!is_valid);
    }

    #[test]
    fn test_is_valid_tw_after_insertion() {
        let problem = create_problem_for_tw_change(
            vec![
                TestService::with_time_window(TimeWindow::from_iso(
                    Some("2025-11-30T08:00:00+02:00"),
                    Some("2025-11-30T09:00:00+02:00"),
                )),
                TestService::with_time_window(TimeWindow::from_iso(
                    Some("2025-11-30T08:00:00+02:00"),
                    Some("2025-11-30T10:00:00+02:00"),
                )),
                TestService::with_time_window(TimeWindow::from_iso(
                    Some("2025-11-30T08:00:00+02:00"),
                    Some("2025-11-30T11:00:00+02:00"),
                )),
                TestService::with_time_window(TimeWindow::from_iso(
                    Some("2025-11-30T08:00:00+02:00"),
                    Some("2025-11-30T12:00:00+02:00"),
                )),
                TestService::with_time_window(TimeWindow::from_iso(
                    Some("2025-11-30T08:00:00+02:00"),
                    Some("2025-11-30T13:00:00+02:00"),
                )),
                TestService::with_time_window(TimeWindow::from_iso(
                    Some("2025-11-30T08:00:00+02:00"),
                    Some("2025-11-30T14:00:00+02:00"),
                )),
            ],
            TestProblemOptions::default(),
        );

        let mut route = WorkingSolutionRoute::empty(&problem, VehicleIdx::new(0));
        // Route: 0 -> 1 -> 2 (arrival times: 08:30, 09:10, 09:50)
        route.insert_service(&problem, 0, JobIdx::new(1));
        route.insert_service(&problem, 1, JobIdx::new(2));
        route.insert_service(&problem, 2, JobIdx::new(3));

        assert_eq!(
            route.start(&problem),
            "2025-11-30T07:30:00+02:00".parse().unwrap()
        );

        assert_eq!(
            route.arrival_times[0],
            "2025-11-30T08:00:00+02:00".parse().unwrap()
        );

        assert_eq!(
            route.arrival_times[1],
            "2025-11-30T08:40:00+02:00".parse().unwrap()
        );

        assert_eq!(
            route.arrival_times[2],
            "2025-11-30T09:20:00+02:00".parse().unwrap()
        );

        // Insert job 0 at position 3
        let is_valid =
            route.is_valid_time_change(&problem, [ActivityId::service(0)].into_iter(), 3, 3);
        assert!(!is_valid);

        let mut route = WorkingSolutionRoute::empty(&problem, VehicleIdx::new(0));
        // Route: 0 -> 1 -> 2 (arrival times: 08:30, 09:10, 09:50)
        route.insert_service(&problem, 0, JobIdx::new(1));
        route.insert_service(&problem, 1, JobIdx::new(0));
        route.insert_service(&problem, 2, JobIdx::new(2));

        assert_eq!(
            route.arrival_times[0],
            "2025-11-30T08:00:00+02:00".parse().unwrap()
        );

        assert_eq!(
            route.arrival_times[1],
            "2025-11-30T08:40:00+02:00".parse().unwrap()
        );

        assert_eq!(
            route.arrival_times[2],
            "2025-11-30T09:20:00+02:00".parse().unwrap()
        );

        // Insert job 3 at position 1
        let is_valid =
            route.is_valid_time_change(&problem, [ActivityId::service(3)].into_iter(), 1, 1);
        assert!(!is_valid);
    }

    #[test]
    fn test_is_valid_tw_change_maximum_working_duration_scenario_1() {
        let problem = create_problem_for_tw_change(
            vec![
                TestService::with_time_window(TimeWindow::from_iso(
                    Some("2025-11-30T08:00:00+02:00"),
                    Some("2025-11-30T09:00:00+02:00"),
                )),
                TestService::with_time_window(TimeWindow::from_iso(
                    Some("2025-11-30T08:00:00+02:00"),
                    Some("2025-11-30T10:00:00+02:00"),
                )),
                TestService::with_time_window(TimeWindow::from_iso(
                    Some("2025-11-30T08:00:00+02:00"),
                    Some("2025-11-30T11:00:00+02:00"),
                )),
                TestService::with_time_window(TimeWindow::from_iso(
                    Some("2025-11-30T08:00:00+02:00"),
                    Some("2025-11-30T12:00:00+02:00"),
                )),
                TestService::with_time_window(TimeWindow::from_iso(
                    Some("2025-11-30T10:00:00+02:00"),
                    Some("2025-11-30T11:00:00+02:00"),
                )),
                TestService::with_time_window(TimeWindow::from_iso(
                    Some("2025-11-30T10:00:00+02:00"),
                    Some("2025-11-30T11:00:00+02:00"),
                )),
                TestService::with_time_window(TimeWindow::from_iso(
                    Some("2025-11-30T09:00:00+02:00"),
                    Some("2025-11-30T11:00:00+02:00"),
                )),
            ],
            TestProblemOptions {
                earliest_start: timestamp!("2025-11-30T06:00:00+02:00"),
                latest_start: timestamp!("2025-11-30T08:30:00+02:00"),
                maximum_working_duration: Some(SignedDuration::from_hours(2)),
                ..TestProblemOptions::default()
            },
        );

        let mut route = WorkingSolutionRoute::empty(&problem, VehicleIdx::new(0));
        route.insert_service(&problem, 0, JobIdx::new(0));
        route.insert_service(&problem, 1, JobIdx::new(1));
        route.insert_service(&problem, 2, JobIdx::new(2));

        assert_eq!(
            route.start(&problem),
            timestamp!("2025-11-30T07:30:00+02:00")
        );

        assert_eq!(
            route.arrival_times[0],
            timestamp!("2025-11-30T08:00:00+02:00")
        );

        assert_eq!(
            route.arrival_times[1],
            timestamp!("2025-11-30T08:40:00+02:00")
        );

        assert_eq!(
            route.arrival_times[2],
            timestamp!("2025-11-30T09:20:00+02:00")
        );

        // 1h50 of work total right now, adding a new service would break that
        let is_valid =
            route.is_valid_time_change(&problem, [ActivityId::service(3)].into_iter(), 3, 3);

        assert!(!is_valid);

        // Test when route is empty
        let route = WorkingSolutionRoute::empty(&problem, VehicleIdx::new(0));
        let is_valid = route.is_valid_time_change(
            &problem,
            [
                ActivityId::service(0),
                ActivityId::service(1),
                ActivityId::service(2),
                ActivityId::service(3),
            ]
            .into_iter(),
            0,
            0,
        );
        assert!(!is_valid);

        let mut route = WorkingSolutionRoute::empty(&problem, VehicleIdx::new(0));
        route.insert_service(&problem, 0, JobIdx::new(4));

        assert_eq!(route.duration(&problem), SignedDuration::from_mins(100));

        let is_valid =
            route.is_valid_time_change(&problem, [ActivityId::service(0)].into_iter(), 0, 0);

        assert!(!is_valid);

        let mut route = WorkingSolutionRoute::empty(&problem, VehicleIdx::new(0));
        route.insert_service(&problem, 0, JobIdx::new(0));
        assert_eq!(route.duration(&problem), SignedDuration::from_mins(40));
        let is_valid =
            route.is_valid_time_change(&problem, [ActivityId::service(1)].into_iter(), 1, 1);
        assert!(is_valid);
        route.insert_service(&problem, 1, JobIdx::new(1));
        assert_eq!(route.duration(&problem), SignedDuration::from_mins(80));
        let is_valid =
            route.is_valid_time_change(&problem, [ActivityId::service(2)].into_iter(), 2, 2);
        assert!(is_valid);
        let is_valid =
            route.is_valid_time_change(&problem, [ActivityId::service(2)].into_iter(), 1, 1);
        assert!(is_valid);

        route.insert_service(&problem, 2, JobIdx::new(2));

        // Maximum duration
        assert_eq!(route.duration(&problem), SignedDuration::from_mins(120));

        // Reverse segment
        let is_valid = route.is_valid_time_change(
            &problem,
            [ActivityId::service(2), ActivityId::service(1)].into_iter(),
            1,
            3,
        );
        assert!(is_valid);

        let mut route = WorkingSolutionRoute::empty(&problem, VehicleIdx::new(0));
        route.insert_service(&problem, 0, JobIdx::new(0));
        route.insert_service(&problem, 1, JobIdx::new(6));

        assert_eq!(route.duration(&problem), SignedDuration::from_mins(100));

        // Valid because we add 40minutes for the service 1, but remove 20 minutes from the waiting duration of service 6
        let is_valid =
            route.is_valid_time_change(&problem, [ActivityId::service(1)].into_iter(), 1, 1);
        assert!(is_valid);

        let is_valid = route.is_valid_time_change(
            &problem,
            [ActivityId::service(1), ActivityId::service(2)].into_iter(),
            1,
            1,
        );
        assert!(!is_valid);

        let is_valid = route.is_valid_time_change(
            &problem,
            [
                ActivityId::service(1),
                ActivityId::service(2),
                ActivityId::service(3),
            ]
            .into_iter(),
            1,
            3,
        );
        assert!(!is_valid);
    }

    #[test]
    fn test_is_valid_tw_change_maximum_working_duration_scenario_2() {
        let problem = create_problem_for_tw_change(
            vec![
                TestService::with_time_window(TimeWindow::from_iso(
                    Some("2025-11-30T09:30:00+02:00"),
                    Some("2025-11-30T20:00:00+02:00"),
                )),
                TestService::with_time_window(TimeWindow::from_iso(
                    Some("2025-11-30T10:30:00+02:00"),
                    Some("2025-11-30T20:00:00+02:00"),
                )),
                TestService::with_time_window(TimeWindow::from_iso(
                    Some("2025-11-30T11:30:00+02:00"),
                    Some("2025-11-30T20:00:00+02:00"),
                )),
                TestService::with_time_window(TimeWindow::from_iso(
                    Some("2025-11-30T08:30:00+02:00"),
                    Some("2025-11-30T20:00:00+02:00"),
                )),
                TestService {
                    time_windows: Some(vec![TimeWindow::from_iso(
                        Some("2025-11-30T08:30:00+02:00"),
                        Some("2025-11-30T20:00:00+02:00"),
                    )]),
                    service_duration: Some(SignedDuration::from_mins(40)),
                },
            ],
            TestProblemOptions {
                earliest_start: timestamp!("2025-11-30T06:00:00+02:00"),
                latest_start: timestamp!("2025-11-30T08:30:00+02:00"),
                maximum_working_duration: Some(SignedDuration::from_mins(190)),
                ..TestProblemOptions::default()
            },
        );

        let mut route = WorkingSolutionRoute::empty(&problem, VehicleIdx::new(0));
        route.insert_service(&problem, 0, JobIdx::new(0));
        route.insert_service(&problem, 1, JobIdx::new(1));
        route.insert_service(&problem, 2, JobIdx::new(2));

        assert_eq!(
            route.start(&problem),
            timestamp!("2025-11-30T08:30:00+02:00")
        );

        assert_eq!(
            route.arrival_times[0],
            timestamp!("2025-11-30T09:00:00+02:00")
        );

        assert_eq!(
            route.arrival_times[1],
            timestamp!("2025-11-30T10:10:00+02:00")
        );

        assert_eq!(
            route.arrival_times[2],
            timestamp!("2025-11-30T11:10:00+02:00")
        );

        // 08:30 -> 11:40
        assert_eq!(route.duration(&problem), SignedDuration::from_mins(190));
        assert_eq!(
            route.bwd_cumulative_waiting_durations[0],
            SignedDuration::from_mins(70)
        );
        assert_eq!(
            route.bwd_cumulative_waiting_durations[1],
            SignedDuration::from_mins(70)
        );
        assert_eq!(
            route.bwd_cumulative_waiting_durations[2],
            SignedDuration::from_mins(40)
        );
        assert_eq!(
            route.bwd_cumulative_waiting_durations[3],
            SignedDuration::from_mins(20)
        );
        assert_eq!(
            route.bwd_cumulative_waiting_durations[4],
            SignedDuration::ZERO
        );

        assert!(!route.is_valid_time_change(&problem, [ActivityId::service(3)].into_iter(), 3, 3));
        assert!(!route.is_valid_time_change(&problem, [ActivityId::service(3)].into_iter(), 2, 2));
        assert!(route.is_valid_time_change(&problem, [ActivityId::service(3)].into_iter(), 1, 1));
        assert!(route.is_valid_time_change(&problem, [ActivityId::service(3)].into_iter(), 0, 0));

        // 20 minutes less waiting time
        // 30 minutes more service time -> NOT OK
        assert!(!route.is_valid_time_change(&problem, [ActivityId::service(4)].into_iter(), 2, 3));

        // // Arrival at 10h20
        // // Service time until 10h50
        // // Arrival at next at 11h20
        assert!(route.is_valid_time_change(&problem, [ActivityId::service(4)].into_iter(), 1, 2));
        assert!(!route.is_valid_time_change(&problem, [ActivityId::service(4)].into_iter(), 0, 1));
    }

    #[test]
    fn test_is_valid_tw_change_maximum_working_duration_scenario_3() {
        let problem = create_problem_for_tw_change(
            vec![
                TestService::with_time_window(TimeWindow::from_iso(
                    Some("2025-11-30T09:30:00+02:00"),
                    Some("2025-11-30T20:00:00+02:00"),
                )),
                TestService::with_time_window(TimeWindow::from_iso(
                    Some("2025-11-30T10:30:00+02:00"),
                    Some("2025-11-30T20:00:00+02:00"),
                )),
                TestService::with_time_window(TimeWindow::from_iso(
                    Some("2025-11-30T11:30:00+02:00"),
                    Some("2025-11-30T20:00:00+02:00"),
                )),
                TestService::with_time_window(TimeWindow::from_iso(
                    Some("2025-11-30T08:30:00+02:00"),
                    Some("2025-11-30T20:00:00+02:00"),
                )),
                TestService::with_time_window(TimeWindow::from_iso(
                    Some("2025-11-30T08:30:00+02:00"),
                    Some("2025-11-30T20:00:00+02:00"),
                )),
                TestService {
                    time_windows: Some(vec![TimeWindow::from_iso(
                        Some("2025-11-30T08:30:00+02:00"),
                        Some("2025-11-30T20:00:00+02:00"),
                    )]),
                    service_duration: Some(SignedDuration::from_mins(10)),
                },
            ],
            TestProblemOptions {
                earliest_start: timestamp!("2025-11-30T06:00:00+02:00"),
                latest_start: timestamp!("2025-11-30T08:30:00+02:00"),
                maximum_working_duration: Some(SignedDuration::from_mins(250)),
                ..TestProblemOptions::default()
            },
        );

        let mut route = WorkingSolutionRoute::empty(&problem, VehicleIdx::new(0));
        route.insert_service(&problem, 0, JobIdx::new(0));
        route.insert_service(&problem, 1, JobIdx::new(1));
        route.insert_service(&problem, 2, JobIdx::new(3));
        route.insert_service(&problem, 3, JobIdx::new(4));

        assert_eq!(
            route.start(&problem),
            timestamp!("2025-11-30T08:30:00+02:00")
        );

        assert_eq!(
            route.arrival_times[0],
            timestamp!("2025-11-30T09:00:00+02:00")
        );

        assert_eq!(
            route.arrival_times[1],
            timestamp!("2025-11-30T10:10:00+02:00")
        );

        assert_eq!(
            route.arrival_times[2],
            timestamp!("2025-11-30T11:10:00+02:00")
        );

        assert_eq!(
            route.arrival_times[3],
            timestamp!("2025-11-30T11:50:00+02:00")
        );

        assert_eq!(route.duration(&problem), SignedDuration::from_mins(210));
        assert_eq!(
            route.bwd_cumulative_waiting_durations[0],
            SignedDuration::from_mins(50)
        );
        assert_eq!(
            route.bwd_cumulative_waiting_durations[1],
            SignedDuration::from_mins(50)
        );
        assert_eq!(
            route.bwd_cumulative_waiting_durations[2],
            SignedDuration::from_mins(20)
        );
        assert_eq!(
            route.bwd_cumulative_waiting_durations[3],
            SignedDuration::ZERO
        );
        assert_eq!(
            route.bwd_cumulative_waiting_durations[4],
            SignedDuration::ZERO
        );
        assert_eq!(
            route.bwd_cumulative_waiting_durations[5],
            SignedDuration::ZERO
        );

        assert!(route.is_valid_time_change(&problem, [ActivityId::service(5)].into_iter(), 2, 2));

        route.insert_service(&problem, 2, JobIdx::new(5));
        assert_eq!(route.duration(&problem), SignedDuration::from_mins(250));
    }

    #[test]
    fn test_is_valid_tw_change_maximum_working_duration_vehicle_start_later() {
        let problem = create_problem_for_tw_change(
            vec![
                TestService::with_time_window(TimeWindow::from_iso(
                    Some("2025-11-30T11:00:00+02:00"),
                    Some("2025-11-30T20:00:00+02:00"),
                )),
                TestService::with_time_window(TimeWindow::from_iso(
                    Some("2025-11-30T11:00:00+02:00"),
                    Some("2025-11-30T20:00:00+02:00"),
                )),
                TestService::with_time_window(TimeWindow::from_iso(
                    Some("2025-11-30T11:00:00+02:00"),
                    Some("2025-11-30T20:00:00+02:00"),
                )),
                TestService::with_time_window(TimeWindow::from_iso(
                    Some("2025-11-30T11:00:00+02:00"),
                    Some("2025-11-30T20:00:00+02:00"),
                )),
                TestService::with_time_window(TimeWindow::from_iso(
                    Some("2025-11-30T11:00:00+02:00"),
                    Some("2025-11-30T20:00:00+02:00"),
                )),
                TestService::with_time_window(TimeWindow::from_iso(
                    Some("2025-11-30T14:00:00+02:00"),
                    Some("2025-11-30T20:00:00+02:00"),
                )),
                TestService::with_time_window(TimeWindow::from_iso(
                    Some("2025-11-30T16:00:00+02:00"),
                    Some("2025-11-30T20:00:00+02:00"),
                )),
            ],
            TestProblemOptions {
                maximum_working_duration: Some(SignedDuration::from_mins(200)),
                ..TestProblemOptions::default()
            },
        );

        let mut route = WorkingSolutionRoute::empty(&problem, VehicleIdx::new(0));
        route.insert_service(&problem, 0, JobIdx::new(0));
        route.insert_service(&problem, 1, JobIdx::new(1));
        route.insert_service(&problem, 2, JobIdx::new(2));
        route.insert_service(&problem, 3, JobIdx::new(3));
        route.insert_service(&problem, 4, JobIdx::new(4));

        assert_eq!(route.duration(&problem), SignedDuration::from_mins(200));

        assert!(route.is_valid_time_change(&problem, [ActivityId::service(5)].into_iter(), 0, 1));
        assert!(!route.is_valid_time_change(&problem, [ActivityId::service(5)].into_iter(), 0, 0));
        assert!(route.is_valid_time_change(&problem, [ActivityId::service(6)].into_iter(), 0, 2));
        assert!(route.is_valid_time_change(&problem, [ActivityId::service(6)].into_iter(), 0, 1));

        route.replace_activities(&problem, &[ActivityId::service(5)], 0, 1);

        assert_eq!(route.duration(&problem), SignedDuration::from_mins(200));
    }

    #[test]
    fn test_is_valid_tw_change_maximum_working_duration_removal() {
        let problem = create_problem_for_tw_change(
            vec![
                TestService::with_time_window(TimeWindow::from_iso(
                    Some("2025-11-30T11:00:00+02:00"),
                    Some("2025-11-30T20:00:00+02:00"),
                )),
                TestService::with_time_window(TimeWindow::from_iso(
                    Some("2025-11-30T12:00:00+02:00"),
                    Some("2025-11-30T20:00:00+02:00"),
                )),
                TestService::with_time_window(TimeWindow::from_iso(
                    Some("2025-11-30T13:00:00+02:00"),
                    Some("2025-11-30T20:00:00+02:00"),
                )),
                TestService::with_time_window(TimeWindow::from_iso(
                    Some("2025-11-30T14:00:00+02:00"),
                    Some("2025-11-30T20:00:00+02:00"),
                )),
                TestService::with_time_window(TimeWindow::from_iso(
                    Some("2025-11-30T10:00:00+02:00"),
                    Some("2025-11-30T20:00:00+02:00"),
                )),
            ],
            TestProblemOptions {
                maximum_working_duration: Some(SignedDuration::from_mins(60 * 3 + 40)),
                ..TestProblemOptions::default()
            },
        );

        let mut route = WorkingSolutionRoute::empty(&problem, VehicleIdx::new(0));
        route.insert_service(&problem, 0, JobIdx::new(0));
        route.insert_service(&problem, 1, JobIdx::new(1));
        route.insert_service(&problem, 2, JobIdx::new(2));
        route.insert_service(&problem, 3, JobIdx::new(3));

        assert_eq!(
            route.duration(&problem),
            SignedDuration::from_mins(60 * 3 + 40)
        );

        assert!(route.is_valid_time_change(&problem, [].into_iter(), 0, 1));
        assert!(route.is_valid_time_change(&problem, [].into_iter(), 1, 3));
        assert!(!route.is_valid_time_change(&problem, [ActivityId::service(4)].into_iter(), 0, 2));
    }

    #[test]
    fn test_is_valid_tw_change_maximum_working_duration_vehicle_start_change() {
        let problem = create_problem_for_tw_change(
            vec![
                TestService::with_time_window(TimeWindow::from_iso(
                    Some("2025-11-30T10:00:00+02:00"),
                    Some("2025-11-30T20:00:00+02:00"),
                )),
                TestService::with_time_window(TimeWindow::from_iso(
                    Some("2025-11-30T11:00:00+02:00"),
                    Some("2025-11-30T20:00:00+02:00"),
                )),
                TestService::with_time_window(TimeWindow::from_iso(
                    Some("2025-11-30T11:00:00+02:00"),
                    Some("2025-11-30T20:00:00+02:00"),
                )),
                TestService::with_time_window(TimeWindow::from_iso(
                    Some("2025-11-30T11:00:00+02:00"),
                    Some("2025-11-30T20:00:00+02:00"),
                )),
            ],
            TestProblemOptions {
                earliest_start: timestamp!("2025-11-30T06:00:00+02:00"),
                maximum_working_duration: Some(SignedDuration::from_mins(140)),
                ..TestProblemOptions::default()
            },
        );

        let mut route = WorkingSolutionRoute::empty(&problem, VehicleIdx::new(0));
        route.insert_service(&problem, 0, JobIdx::new(1));
        route.insert_service(&problem, 1, JobIdx::new(2));
        route.insert_service(&problem, 2, JobIdx::new(3));

        assert_eq!(route.duration(&problem), SignedDuration::from_mins(120));

        assert!(!route.is_valid_time_change(&problem, [ActivityId::service(0)].into_iter(), 0, 0));
    }

    #[test]
    fn test_waiting_duration_data() {
        let problem = create_problem_for_tw_change(
            vec![
                TestService::with_time_window(TimeWindow::from_iso(
                    Some("2025-11-30T08:00:00+02:00"),
                    Some("2025-11-30T09:00:00+02:00"),
                )),
                TestService::with_time_window(TimeWindow::from_iso(
                    Some("2025-11-30T10:00:00+02:00"),
                    Some("2025-11-30T11:00:00+02:00"),
                )),
                TestService::with_time_window(TimeWindow::from_iso(
                    Some("2025-11-30T10:00:00+02:00"),
                    Some("2025-11-30T12:00:00+02:00"),
                )),
            ],
            TestProblemOptions::default(),
        );

        let mut route = WorkingSolutionRoute::empty(&problem, VehicleIdx::new(0));
        route.insert_service(&problem, 0, JobIdx::new(0));

        assert_eq!(
            route.arrival_times[0],
            timestamp!("2025-11-30T08:00:00+02:00")
        );
        assert_eq!(route.waiting_durations[0], SignedDuration::ZERO);
        for i in 0..route.len() + 2 {
            assert_eq!(
                route.fwd_cumulative_waiting_durations[i],
                SignedDuration::ZERO
            );
            assert_eq!(
                route.bwd_cumulative_waiting_durations[i],
                SignedDuration::ZERO
            );
        }

        route.insert_service(&problem, 1, JobIdx::new(1));
        assert_eq!(
            route.arrival_times[1],
            timestamp!("2025-11-30T08:40:00+02:00")
        );
        assert_eq!(route.waiting_durations[0], SignedDuration::ZERO);
        assert_eq!(route.waiting_durations[1], SignedDuration::from_mins(80));
        assert_eq!(route.waiting_time_slacks[0], SignedDuration::ZERO);
        assert_eq!(route.waiting_time_slacks[1], SignedDuration::ZERO);

        assert_eq!(
            route.fwd_cumulative_waiting_durations[0],
            SignedDuration::ZERO
        );
        assert_eq!(
            route.fwd_cumulative_waiting_durations[1],
            SignedDuration::ZERO
        );
        assert_eq!(
            route.fwd_cumulative_waiting_durations[2],
            SignedDuration::from_mins(80)
        );
        assert_eq!(
            route.fwd_cumulative_waiting_durations[3],
            SignedDuration::from_mins(80)
        );

        assert_eq!(
            route.bwd_cumulative_waiting_durations[0],
            SignedDuration::from_mins(80)
        );
        assert_eq!(
            route.bwd_cumulative_waiting_durations[1],
            SignedDuration::from_mins(80)
        );
        assert_eq!(
            route.bwd_cumulative_waiting_durations[2],
            SignedDuration::from_mins(80)
        );
        assert_eq!(
            route.bwd_cumulative_waiting_durations[3],
            SignedDuration::ZERO
        );

        route.insert_service(&problem, 2, JobIdx::new(2));
        assert_eq!(
            route.arrival_times[2],
            timestamp!("2025-11-30T10:40:00+02:00")
        );
        assert_eq!(route.waiting_durations[0], SignedDuration::ZERO);
        assert_eq!(route.waiting_durations[1], SignedDuration::from_mins(80));
        assert_eq!(route.waiting_durations[2], SignedDuration::ZERO);
        assert_eq!(route.waiting_time_slacks[0], SignedDuration::ZERO);
        assert_eq!(route.waiting_time_slacks[1], SignedDuration::ZERO);
        // Can be shifted in time 40 minutes until waiting time occurs
        assert_eq!(route.waiting_time_slacks[2], SignedDuration::from_mins(40));

        assert_eq!(
            route.fwd_cumulative_waiting_durations[0],
            SignedDuration::ZERO
        );
        assert_eq!(
            route.fwd_cumulative_waiting_durations[1],
            SignedDuration::ZERO
        );
        assert_eq!(
            route.fwd_cumulative_waiting_durations[2],
            SignedDuration::from_mins(80)
        );
        assert_eq!(
            route.fwd_cumulative_waiting_durations[3],
            SignedDuration::from_mins(80)
        );
        assert_eq!(
            route.fwd_cumulative_waiting_durations[4],
            SignedDuration::from_mins(80)
        );

        assert_eq!(
            route.bwd_cumulative_waiting_durations[0],
            SignedDuration::from_mins(80)
        );
        assert_eq!(
            route.bwd_cumulative_waiting_durations[1],
            SignedDuration::from_mins(80)
        );
        assert_eq!(
            route.bwd_cumulative_waiting_durations[2],
            SignedDuration::from_mins(80)
        );
        assert_eq!(
            route.bwd_cumulative_waiting_durations[3],
            SignedDuration::ZERO
        );
        assert_eq!(
            route.bwd_cumulative_waiting_durations[4],
            SignedDuration::ZERO
        );
    }

    #[test]
    fn test_waiting_duration_delta() {
        let problem = create_problem_for_tw_change(
            vec![
                TestService::with_time_window(TimeWindow::from_iso(
                    Some("2025-11-30T08:00:00+02:00"),
                    Some("2025-11-30T09:00:00+02:00"),
                )),
                TestService::with_time_window(TimeWindow::from_iso(
                    Some("2025-11-30T10:00:00+02:00"),
                    Some("2025-11-30T11:00:00+02:00"),
                )),
                TestService::with_time_window(TimeWindow::from_iso(
                    Some("2025-11-30T10:00:00+02:00"),
                    Some("2025-11-30T12:00:00+02:00"),
                )),
                TestService::with_time_window(TimeWindow::from_iso(
                    Some("2025-11-30T08:00:00+02:00"),
                    Some("2025-11-30T12:00:00+02:00"),
                )),
            ],
            TestProblemOptions {
                earliest_start: timestamp!("2025-11-30T07:00:00+02:00"),
                latest_start: timestamp!("2025-11-30T09:00:00+02:00"),
                ..TestProblemOptions::default()
            },
        );

        let mut route = WorkingSolutionRoute::empty(&problem, VehicleIdx::new(0));

        let delta = route.waiting_duration_change_delta(
            &problem,
            [ActivityId::service(0)].into_iter(),
            0,
            0,
        );

        assert_eq!(delta, SignedDuration::ZERO);

        route.insert_service(&problem, 0, JobIdx::new(0));

        let delta = route.waiting_duration_change_delta(
            &problem,
            [ActivityId::service(1)].into_iter(),
            1,
            1,
        );

        assert_eq!(delta, SignedDuration::from_mins(80));

        route.insert_service(&problem, 1, JobIdx::new(1));

        let delta = route.waiting_duration_change_delta(
            &problem,
            [ActivityId::service(3)].into_iter(),
            1,
            1,
        );

        assert_eq!(delta, SignedDuration::from_mins(-40));

        route.insert_service(&problem, 1, JobIdx::new(3));

        // Reverse segment [3, 1] to [1, 3]
        let delta = route.waiting_duration_change_delta(
            &problem,
            [ActivityId::service(1), ActivityId::service(3)].into_iter(),
            1,
            3,
        );

        assert_eq!(delta, SignedDuration::from_mins(40));

        // Removal of service 1
        let delta = route.waiting_duration_change_delta(&problem, [].into_iter(), 2, 3);
        assert_eq!(delta, SignedDuration::from_mins(-40));

        // Add service 2 at the start and remove the first one
        let delta = route.waiting_duration_change_delta(
            &problem,
            [ActivityId::service(2)].into_iter(),
            0,
            1,
        );

        // Add 30m waiting time, but remove 40m later
        assert_eq!(delta, SignedDuration::from_mins(-10));

        let delta = route.waiting_duration_change_delta(
            &problem,
            [ActivityId::service(2)].into_iter(),
            0,
            0,
        );

        // Add 30m waiting time, but remove 40m later, but time windows break here
        assert_eq!(delta, SignedDuration::from_mins(-10));
        assert!(!route.is_valid_time_change(&problem, [ActivityId::service(2)].into_iter(), 0, 0))
    }

    fn create_problem_with_n_services(services: usize) -> VehicleRoutingProblem {
        // 10 locations from (0, 0) to (9, 0)
        let locations = test_utils::create_location_grid(1, services + 1);

        let mut vehicle_builder = VehicleBuilder::default();
        vehicle_builder.set_depot_location_id(0);
        vehicle_builder.set_capacity(Capacity::from_vec(vec![100.0]));
        vehicle_builder.set_vehicle_id(String::from("vehicle"));
        vehicle_builder.set_profile_id(0);
        let vehicle = vehicle_builder.build();
        let vehicles = vec![vehicle];

        let services = (0..services)
            .map(|i| {
                let mut service_builder = ServiceBuilder::default();
                service_builder.set_demand(Capacity::from_vec(vec![10.0]));
                service_builder.set_external_id(format!("service_{}", i + 1));
                service_builder.set_service_duration(SignedDuration::from_mins(10));
                service_builder.set_location_id(i + 1);
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
        builder.set_fleet(Fleet::Finite(vehicles));
        builder.set_services(services);

        builder.build()
    }

    #[test]
    fn test_remove() {
        let problem = create_problem_with_n_services(10);

        let mut route = WorkingSolutionRoute::empty(&problem, VehicleIdx::new(0));
        route.insert_service(&problem, 0, JobIdx::new(0));
        route.insert_service(&problem, 1, JobIdx::new(1));
        route.insert_service(&problem, 2, JobIdx::new(2));
        route.insert_service(&problem, 3, JobIdx::new(3));
        route.insert_service(&problem, 4, JobIdx::new(4));
        route.insert_service(&problem, 5, JobIdx::new(5));
        route.insert_service(&problem, 6, JobIdx::new(6));
        route.insert_service(&problem, 7, JobIdx::new(7));

        let activity_id = route.remove(&problem, 3);
        assert_eq!(activity_id, Some(ActivityId::Service(JobIdx::new(3))));

        let mapping = vec![(0, 0), (1, 1), (2, 2), (4, 3), (5, 4), (6, 5), (7, 6)];
        for (id, position) in mapping.into_iter() {
            assert_eq!(
                route.jobs.get(&ActivityId::Service(JobIdx::new(id))),
                Some(&position)
            );
        }
    }

    #[test]
    fn test_remove_activity() {
        let problem = create_problem_with_n_services(10);

        let mut route = WorkingSolutionRoute::empty(&problem, VehicleIdx::new(0));
        route.insert_service(&problem, 0, JobIdx::new(0));
        route.insert_service(&problem, 1, JobIdx::new(1));
        route.insert_service(&problem, 2, JobIdx::new(2));
        route.insert_service(&problem, 3, JobIdx::new(3));
        route.insert_service(&problem, 4, JobIdx::new(4));
        route.insert_service(&problem, 5, JobIdx::new(5));
        route.insert_service(&problem, 6, JobIdx::new(6));
        route.insert_service(&problem, 7, JobIdx::new(7));

        let removed = route.remove_activity(&problem, ActivityId::Service(JobIdx::new(3)));
        assert!(removed);

        let mapping = vec![(0, 0), (1, 1), (2, 2), (4, 3), (5, 4), (6, 5), (7, 6)];
        for (id, position) in mapping.into_iter() {
            assert_eq!(
                route.jobs.get(&ActivityId::Service(JobIdx::new(id))),
                Some(&position)
            );
        }
    }

    #[test]
    fn test_global_version() {
        let problem = Arc::new(create_problem());

        assert_eq!(problem.next_route_version(), 0);
        assert_eq!(problem.next_route_version(), 1);
        assert_eq!(problem.next_route_version(), 2);

        std::thread::scope(|s| {
            let problem1 = problem.clone();
            let h1 = s.spawn(move || problem1.next_route_version());
            let problem2 = problem.clone();
            let h2 = s.spawn(move || problem2.next_route_version());

            let v1 = h1.join().unwrap();
            let v2 = h2.join().unwrap();
            assert_ne!(v1, v2);
        });

        assert_eq!(problem.next_route_version(), 5);
    }

    #[test]
    fn test_routes_versions() {
        let problem = create_problem_for_tw_change(
            vec![
                TestService::with_time_window(TimeWindow::from_iso(
                    Some("2025-11-30T08:00:00+02:00"),
                    Some("2025-11-30T09:00:00+02:00"),
                )),
                TestService::with_time_window(TimeWindow::from_iso(
                    Some("2025-11-30T08:00:00+02:00"),
                    Some("2025-11-30T10:00:00+02:00"),
                )),
                TestService::with_time_window(TimeWindow::from_iso(
                    Some("2025-11-30T08:00:00+02:00"),
                    Some("2025-11-30T11:00:00+02:00"),
                )),
                TestService::with_time_window(TimeWindow::from_iso(
                    Some("2025-11-30T08:00:00+02:00"),
                    Some("2025-11-30T12:00:00+02:00"),
                )),
                TestService::with_time_window(TimeWindow::from_iso(
                    Some("2025-11-30T08:00:00+02:00"),
                    Some("2025-11-30T13:00:00+02:00"),
                )),
                TestService::with_time_window(TimeWindow::from_iso(
                    Some("2025-11-30T08:00:00+02:00"),
                    Some("2025-11-30T13:00:00+02:00"),
                )),
            ],
            TestProblemOptions::default(),
        );

        let mut route = WorkingSolutionRoute::empty(&problem, VehicleIdx::new(0));
        assert_eq!(route.version(), 0);
        // Route: 0 -> 1 -> 2 (arrival times: 08:30, 09:10, 09:50)
        route.insert_service(&problem, 0, JobIdx::new(0));

        assert_eq!(route.version(), 1);

        route.insert_service(&problem, 1, JobIdx::new(1));

        assert_eq!(route.version(), 2);

        route.insert_service(&problem, 2, JobIdx::new(2));

        assert_eq!(route.version(), 3);

        let route2 = WorkingSolutionRoute::empty(&problem, VehicleIdx::new(0));
        assert_eq!(route2.version(), 4);
    }
}
