use std::sync::atomic::AtomicUsize;

use fxhash::FxHashSet;
use jiff::SignedDuration;
use thiserror::Error;
use tokio::time::error;
use tracing::instrument;
use uuid::Uuid;

use crate::{
    problem::{
        amount::AmountExpression,
        capacity::Capacity,
        fleet::Fleet,
        job::{ActivityId, Job, JobActivity, JobIdx},
        meters::Meters,
        relation::Relation,
        service::Service,
        shipment::Shipment,
        skill::Skill,
        task_dependencies::{MalformedRelationError, TaskDependencies},
        vehicle_profile::{VehicleProfile, VehicleProfileIdx},
    },
    solver::constraints::transport_cost_constraint::TRANSPORT_COST_WEIGHT,
    utils::{
        enumerate_idx::EnumerateIdx, one_tree::alpha_nearest_neighbors, zip_longest::zip_longest,
    },
};

use super::{
    distance_method::DistanceMethod,
    location::{Location, LocationIdx},
    service_location_index::ServiceLocationIndex,
    travel_cost_matrix::Cost,
    vehicle::{Vehicle, VehicleIdx},
};

type PrecomputedAverageCostFromDepot = Vec<Cost>;
type PrecomputedNormalizedDemands = Vec<Capacity>;

pub struct VehicleRoutingProblem {
    id: String,
    locations: Vec<Location>,
    fleet: Fleet,
    vehicle_profiles: Vec<VehicleProfile>,
    jobs: Vec<Job>,
    service_location_index: ServiceLocationIndex,

    has_services: bool,
    has_shipments: bool,
    has_time_windows: bool,
    has_capacity: bool,
    has_task_dependencies: bool,

    neighborhoods: Vec<FxHashSet<ActivityId>>,

    task_dependencies: TaskDependencies,

    skill_registry: Vec<Skill>,
    precomputed_capacity_dimensions: usize,
    precomputed_normalized_demands: PrecomputedNormalizedDemands,
    precomputed_average_cost_from_depot: PrecomputedAverageCostFromDepot,

    /// Normalized weight for converting waiting duration into cost
    waiting_duration_weight: f64,

    version_counter: AtomicUsize,
}

#[derive(Error, Debug)]
pub enum VehicleRoutingProblemError {
    #[error("{0}")]
    MalformedRelation(#[from] MalformedRelationError),
    #[error("Invalid vehicle profile {vehicle_id} for {profile_id}")]
    InvalidVehicleProfile {
        vehicle_id: usize,
        profile_id: usize,
    },
    #[error("Location ID {0} out of bounds")]
    LocationIdOutOfBounds(usize),
    #[error("Missing jobs")]
    MissingJobs,
    #[error("Missing locations")]
    MissingLocations,
    #[error("Missing vehicle profiles")]
    MissingVehicleProfiles,
    #[error("Missing fleet")]
    MissingFleet,
    #[error("Empty fleet")]
    EmptyFleet,
}

struct VehicleRoutingProblemParams {
    id: String,
    locations: Vec<Location>,
    fleet: Fleet,
    vehicle_profiles: Vec<VehicleProfile>,
    jobs: Vec<Job>,
    distance_method: DistanceMethod,
    penalize_waiting_duration: bool,
    relations: Option<Vec<Relation>>,
}

impl VehicleRoutingProblem {
    fn try_from_params(
        params: VehicleRoutingProblemParams,
    ) -> Result<Self, VehicleRoutingProblemError> {
        if params.fleet.vehicles().is_empty() {
            return Err(VehicleRoutingProblemError::EmptyFleet);
        }

        for (vehicle_id, vehicle) in params.fleet.vehicles().iter().enumerate() {
            if vehicle.profile_id().get() >= params.vehicle_profiles.len() {
                return Err(VehicleRoutingProblemError::InvalidVehicleProfile {
                    vehicle_id,
                    profile_id: vehicle.profile_id().get(),
                });
            }
        }

        let service_location_index =
            ServiceLocationIndex::new(&params.locations, &params.jobs, params.distance_method);

        let precomputed_average_cost_from_depot =
            VehicleRoutingProblem::precompute_average_cost_from_depot(
                &params.locations,
                params.fleet.vehicles(),
                &params.vehicle_profiles,
            );

        let precomputed_normalized_demands =
            VehicleRoutingProblem::precompute_normalized_demands(&params.jobs);

        let precomputed_capacity_dimensions = params
            .jobs
            .iter()
            .map(|job| job.demand())
            .chain(
                params
                    .fleet
                    .vehicles()
                    .iter()
                    .map(|vehicle| vehicle.capacity()),
            )
            .map(|capacity| capacity.len())
            .max()
            .unwrap_or(0);

        let waiting_duration_weight = if params.penalize_waiting_duration {
            VehicleRoutingProblem::precompute_waiting_duration_weight(&params.vehicle_profiles)
        } else {
            0.0
        };

        let neighborhoods = VehicleRoutingProblem::precompute_neighborhoods(
            &params.locations,
            &params.jobs,
            &params.vehicle_profiles,
        );

        for neighborhood in &neighborhoods {
            // Should be fine, warning for now
            if neighborhood.is_empty() {
                tracing::warn!("Empty neighborhood");
            }
        }

        let skills = VehicleRoutingProblem::collect_skills(params.fleet.vehicles(), &params.jobs);

        let has_services = params.jobs.iter().any(|job| matches!(job, Job::Service(_)));
        let has_shipments = params
            .jobs
            .iter()
            .any(|job| matches!(job, Job::Shipment(_)));

        let has_task_dependencies = params
            .relations
            .as_ref()
            .map(|relations| !relations.is_empty())
            .unwrap_or(false);

        let task_dependencies = TaskDependencies::try_from_jobs_and_relations(
            &params.jobs,
            &params.relations.unwrap_or_default(),
        )
        .map_err(VehicleRoutingProblemError::MalformedRelation)?;

        let mut problem = Self {
            id: params.id,
            has_time_windows: params.jobs.iter().any(|job| job.has_time_windows()),
            has_capacity: params.jobs.iter().any(|job| !job.demand().is_empty()),
            has_task_dependencies,
            locations: params.locations,
            fleet: params.fleet,
            vehicle_profiles: params.vehicle_profiles,
            jobs: params.jobs,
            task_dependencies,
            neighborhoods,
            service_location_index,
            precomputed_average_cost_from_depot,
            precomputed_normalized_demands,
            precomputed_capacity_dimensions,
            waiting_duration_weight,
            has_services,
            has_shipments,
            skill_registry: skills,
            version_counter: AtomicUsize::new(0),
        };

        for vehicle in problem.fleet.vehicles_mut() {
            vehicle.build_skills_bitset(&problem.skill_registry);
        }

        for job in &mut problem.jobs {
            job.build_skills_bitset(&problem.skill_registry);
        }

        Ok(problem)
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub(crate) fn next_route_version(&self) -> usize {
        self.version_counter
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    }

    pub fn jobs(&self) -> &[Job] {
        &self.jobs
    }

    pub fn neighbors(&self, location_id: LocationIdx) -> &FxHashSet<ActivityId> {
        &self.neighborhoods[location_id.get()]
    }

    pub fn has_services(&self) -> bool {
        self.has_services
    }

    pub fn has_shipments(&self) -> bool {
        self.has_shipments
    }

    pub fn services_iter(&self) -> impl Iterator<Item = &Service> {
        self.jobs.iter().filter_map(|job| match job {
            Job::Service(service) => Some(service),
            _ => None,
        })
    }

    pub fn job(&self, job_id: JobIdx) -> &Job {
        &self.jobs[job_id]
    }

    pub fn service(&self, index: JobIdx) -> &Service {
        let job = &self.jobs[index];

        match job {
            Job::Service(service) => service,
            _ => panic!("Job {index} is not a service"),
        }
    }

    pub fn shipment(&self, index: usize) -> &Shipment {
        let job = &self.jobs[index];

        match job {
            Job::Shipment(shipment) => shipment,
            _ => panic!("Job {index} is not a shipment"),
        }
    }

    pub fn random_location<R>(&self, rng: &mut R) -> LocationIdx
    where
        R: rand::Rng,
    {
        rng.random_range(0..self.locations.len()).into()
    }

    pub fn random_job<R>(&self, rng: &mut R) -> JobIdx
    where
        R: rand::Rng,
    {
        rng.random_range(0..self.jobs.len()).into()
    }

    pub fn fleet(&self) -> &Fleet {
        &self.fleet
    }

    pub fn vehicle(&self, vehicle_id: VehicleIdx) -> &Vehicle {
        self.fleet.vehicle(vehicle_id)
    }

    pub fn vehicles(&self) -> &[Vehicle] {
        self.fleet.vehicles()
    }

    pub fn vehicle_profiles(&self) -> &[VehicleProfile] {
        &self.vehicle_profiles
    }

    pub fn vehicle_profile(&self, profile_id: VehicleProfileIdx) -> &VehicleProfile {
        &self.vehicle_profiles[profile_id]
    }

    pub fn locations(&self) -> &[Location] {
        &self.locations
    }

    pub fn location(&self, location_id: LocationIdx) -> &Location {
        &self.locations[location_id]
    }

    pub fn job_activity<'a>(&'a self, activity_id: ActivityId) -> JobActivity<'a> {
        let job_id = activity_id.job_id().get();
        assert!(job_id < self.jobs.len(), "Job ID out of bounds");

        // Can't use match here because if let bindings are experimental
        if let ActivityId::Service(service_id) = activity_id
            && let Job::Service(service) = &self.jobs[service_id]
        {
            JobActivity::Service(service)
        } else if let ActivityId::ShipmentPickup(shipment_id) = activity_id
            && let Job::Shipment(shipment) = &self.jobs[shipment_id]
        {
            JobActivity::ShipmentPickup(shipment)
        } else if let ActivityId::ShipmentDelivery(shipment_id) = activity_id
            && let Job::Shipment(shipment) = &self.jobs[shipment_id]
        {
            JobActivity::ShipmentDelivery(shipment)
        } else {
            unreachable!("Job {activity_id} is not valid");
        }
    }

    pub fn vehicle_depot_location(&self, vehicle_id: VehicleIdx) -> Option<&Location> {
        let vehicle = &self.fleet.vehicle(vehicle_id);
        vehicle
            .depot_location_id()
            .map(|location_id| &self.locations[location_id])
    }

    pub fn max_cost(&self) -> Cost {
        self.vehicle_profiles
            .iter()
            .map(|profile| profile.travel_costs().max_cost() * TRANSPORT_COST_WEIGHT)
            .fold(0.0_f64, |a, b| a.max(b))
    }

    #[inline(always)]
    pub fn travel_distance(&self, vehicle: &Vehicle, from: LocationIdx, to: LocationIdx) -> Meters {
        let profile_id = vehicle.profile_id();
        self.vehicle_profiles[profile_id].travel_distance(from, to)
    }

    #[inline(always)]
    pub fn travel_time(
        &self,
        vehicle: &Vehicle,
        from: LocationIdx,
        to: LocationIdx,
    ) -> jiff::SignedDuration {
        let profile_id = vehicle.profile_id();
        self.vehicle_profiles[profile_id].travel_time(from, to)
    }

    #[inline(always)]
    pub fn travel_cost(&self, vehicle: &Vehicle, from: LocationIdx, to: LocationIdx) -> Cost {
        let profile_id = vehicle.profile_id();
        self.vehicle_profiles[profile_id].travel_cost(from, to)
    }

    #[inline(always)]
    pub fn travel_cost_or_zero(
        &self,
        vehicle: &Vehicle,
        from: Option<LocationIdx>,
        to: Option<LocationIdx>,
    ) -> Cost {
        self.vehicle_profiles[vehicle.profile_id()].travel_cost_or_zero(from, to)
    }

    pub fn travel_distance_between_jobs(&self, a: JobIdx, b: JobIdx) -> Meters {
        match (self.job(a), self.job(b)) {
            (Job::Service(service_a), Job::Service(service_b)) => self.travel_distance(
                self.vehicle(0.into()),
                service_a.location_id(),
                service_b.location_id(),
            ),
            (Job::Shipment(shipment_a), Job::Shipment(shipment_b)) => {
                let pickup_a = shipment_a.pickup().location_id();
                let delivery_a = shipment_a.delivery().location_id();
                let pickup_b = shipment_b.pickup().location_id();
                let delivery_b = shipment_b.delivery().location_id();

                self.travel_distance(self.vehicle(0.into()), pickup_a, pickup_b)
                    + self.travel_distance(self.vehicle(0.into()), delivery_a, delivery_b)
            }
            (Job::Service(service_a), Job::Shipment(shipment_b)) => {
                let pickup_b = shipment_b.pickup().location_id();
                let delivery_b = shipment_b.delivery().location_id();

                (self.travel_distance(self.vehicle(0.into()), service_a.location_id(), pickup_b)
                    + self.travel_distance(
                        self.vehicle(0.into()),
                        service_a.location_id(),
                        delivery_b,
                    ))
                    / 2.0
            }
            (Job::Shipment(shipment_a), Job::Service(service_b)) => {
                let pickup_a = shipment_a.pickup().location_id();
                let delivery_a = shipment_a.delivery().location_id();

                (self.travel_distance(self.vehicle(0.into()), pickup_a, service_b.location_id())
                    + self.travel_distance(
                        self.vehicle(0.into()),
                        delivery_a,
                        service_b.location_id(),
                    ))
                    / 2.0
            }
        }
    }

    #[inline(always)]
    pub fn acceptable_service_waiting_duration(&self) -> SignedDuration {
        SignedDuration::ZERO
    }

    #[inline(always)]
    pub fn waiting_duration_weight(&self) -> f64 {
        self.waiting_duration_weight
    }

    pub fn has_waiting_duration_cost(&self) -> bool {
        self.waiting_duration_weight() > 0.0
    }

    pub fn waiting_duration_cost(&self, waiting_duration: SignedDuration) -> Cost {
        (waiting_duration - self.acceptable_service_waiting_duration()).as_secs_f64()
            * self.waiting_duration_weight()
    }

    pub fn fixed_vehicle_costs(&self) -> f64 {
        100000.0 //self.max_cost() // Placeholder for the static cost of a route
    }

    pub fn nearest_jobs_of_location(
        &self,
        location_id: LocationIdx,
    ) -> impl Iterator<Item = ActivityId> {
        let location = &self.locations[location_id];
        self.service_location_index.nearest_neighbor_iter(location)
    }

    pub fn nearest_jobs(&self, job_id: ActivityId) -> impl Iterator<Item = ActivityId> {
        let job_location_id = self.job_activity(job_id).location_id();
        self.nearest_jobs_of_location(job_location_id)
    }

    pub fn in_nearest_neighborhood_of(&self, of: ActivityId, activity_id: ActivityId) -> bool {
        let location_id = self.job_activity(activity_id).location_id();
        self.neighborhoods[location_id.get()].contains(&of)
    }

    pub fn is_symmetric(&self) -> bool {
        self.vehicle_profiles
            .iter()
            .all(|profile| profile.travel_costs().is_symmetric())
    }

    pub fn is_homogeneous_fleet(&self) -> bool {
        self.vehicle_profiles.len() == 1
    }

    pub fn has_time_windows(&self) -> bool {
        self.has_time_windows
    }

    pub fn has_capacity(&self) -> bool {
        self.has_capacity
    }

    pub fn has_skills(&self) -> bool {
        !self.skill_registry.is_empty()
    }

    pub fn has_task_dependencies(&self) -> bool {
        self.has_task_dependencies
    }

    pub fn task_dependencies(&self) -> &TaskDependencies {
        &self.task_dependencies
    }

    pub fn average_cost_from_depot(&self, job: &Job) -> f64 {
        match job {
            Job::Shipment(shipment) => {
                let pickup_distance =
                    self.precomputed_average_cost_from_depot[shipment.pickup().location_id().get()];

                let delivery_distance = self.precomputed_average_cost_from_depot
                    [shipment.delivery().location_id().get()];

                let avg_distance = (pickup_distance + delivery_distance) / 2.0;
                -avg_distance
            }
            Job::Service(service) => {
                let distance_from_depot =
                    self.precomputed_average_cost_from_depot[service.location_id().get()];
                -distance_from_depot
            }
        }
    }

    pub fn normalized_demand(&self, index: JobIdx) -> &Capacity {
        &self.precomputed_normalized_demands[index.get()]
    }

    pub fn capacity_dimensions(&self) -> usize {
        self.precomputed_capacity_dimensions
    }

    pub fn set_waiting_duration_weight(&mut self, cost: f64) {
        self.waiting_duration_weight = cost;
    }

    #[instrument(skip_all, level = "debug")]
    fn precompute_neighborhoods(
        locations: &[Location],
        jobs: &[Job],
        vehicle_profiles: &[VehicleProfile],
    ) -> Vec<FxHashSet<ActivityId>> {
        let target_size = 50;
        let num_locations = locations.len();

        // Build location -> activities mapping
        let mut location_activities: Vec<Vec<ActivityId>> = vec![vec![]; num_locations];
        for (job_idx, job) in jobs.iter().enumerate_idx() {
            match job {
                Job::Service(service) => {
                    location_activities[service.location_id().get()]
                        .push(ActivityId::Service(job_idx));
                }
                Job::Shipment(shipment) => {
                    location_activities[shipment.pickup().location_id().get()]
                        .push(ActivityId::ShipmentPickup(job_idx));
                    location_activities[shipment.delivery().location_id().get()]
                        .push(ActivityId::ShipmentDelivery(job_idx));
                }
            }
        }

        // Compute alpha-nearest neighbor locations using 1-tree Held-Karp lower bound
        let profile = &vehicle_profiles[0]; // TODO: handle different profiles
        let alpha_neighbors = alpha_nearest_neighbors(
            num_locations,
            |from, to| {
                let a = LocationIdx::new(from);
                let b = LocationIdx::new(to);
                profile.travel_cost(a, b).min(profile.travel_cost(b, a))
            },
            num_locations - 1, // get all neighbors sorted by alpha
        );

        // For each location, take locations with smallest alpha values until we
        // reach the target neighborhood size in activities
        locations
            .iter()
            .enumerate()
            .map(|(location_id, _)| {
                let mut neighborhood = FxHashSet::default();

                for &neighbor_loc in &alpha_neighbors[location_id] {
                    if neighborhood.len() >= target_size {
                        break;
                    }
                    for &activity in &location_activities[neighbor_loc] {
                        neighborhood.insert(activity);
                    }
                }

                neighborhood
            })
            .collect()
    }

    #[instrument(skip_all, level = "debug")]
    fn precompute_waiting_duration_weight(vehicle_profiles: &[VehicleProfile]) -> f64 {
        let sum = vehicle_profiles
            .iter()
            .map(|profile| {
                let profile_sum = profile
                    .travel_costs()
                    .times()
                    .iter()
                    .zip(profile.travel_costs().costs().iter())
                    .filter_map(|(&time, &cost)| {
                        if time > 0.0 && cost > 0.0 {
                            Some(cost / time)
                        } else {
                            None
                        }
                    })
                    .sum::<f64>();

                profile_sum
                    / (profile.travel_costs().num_locations().pow(2)
                        - profile.travel_costs().num_locations()) as f64
            })
            .sum::<f64>();

        sum / vehicle_profiles.len() as f64
    }

    #[instrument(skip_all, level = "debug")]
    fn precompute_normalized_demands(jobs: &[Job]) -> PrecomputedNormalizedDemands {
        let mut max_capacity: Capacity = Capacity::empty();

        for job in jobs.iter() {
            max_capacity.update_max(job.demand());
        }

        jobs.iter()
            .map(|job| {
                let mut normalized_demand = Capacity::with_dimensions(max_capacity.len());
                zip_longest(job.demand().iter(), max_capacity.iter())
                    .enumerate()
                    .for_each(|(index, (demand, max))| {
                        let normalized = if let (Some(demand), Some(max)) = (demand, max) {
                            if max > 0.0 { demand / max } else { 0.0 }
                        } else {
                            0.0
                        };

                        normalized_demand[index] = normalized;
                    });

                normalized_demand
            })
            .collect()
    }

    #[instrument(skip_all, level = "debug")]
    fn precompute_average_cost_from_depot(
        locations: &[Location],
        vehicles: &[Vehicle],
        profiles: &[VehicleProfile],
    ) -> PrecomputedAverageCostFromDepot {
        let mut precomputed_average_cost_from_depot = Vec::with_capacity(locations.len());

        precomputed_average_cost_from_depot.extend(locations.iter().enumerate_idx().map(
            |(location_id, _location)| {
                vehicles
                    .iter()
                    .filter_map(|vehicle| {
                        vehicle.depot_location_id().map(|depot_location_id| {
                            profiles[vehicle.profile_id()]
                                .travel_cost(depot_location_id, location_id)
                        })
                    })
                    .sum::<f64>()
                    / vehicles.len() as f64
            },
        ));

        precomputed_average_cost_from_depot
    }

    fn collect_skills(vehicles: &[Vehicle], jobs: &[Job]) -> Vec<Skill> {
        let mut skills = FxHashSet::<Skill>::default();

        for vehicle in vehicles {
            skills.extend(vehicle.skills().iter().cloned());
        }

        for job in jobs {
            skills.extend(job.skills().iter().cloned());
        }

        skills.into_iter().collect()
    }
}

#[derive(Default)]
pub struct VehicleRoutingProblemBuilder {
    id: Option<String>,
    services: Option<Vec<Service>>,
    shipments: Option<Vec<Shipment>>,
    locations: Option<Vec<Location>>,
    fleet: Option<Fleet>,
    vehicle_profiles: Option<Vec<VehicleProfile>>,
    distance_method: Option<DistanceMethod>,
    penalize_waiting_duration: Option<bool>,
    relations: Option<Vec<Relation>>,
}

impl VehicleRoutingProblemBuilder {
    pub fn set_distance_method(
        &mut self,
        distance_method: DistanceMethod,
    ) -> &mut VehicleRoutingProblemBuilder {
        self.distance_method = Some(distance_method);
        self
    }

    pub fn set_penalize_waiting_duration(
        &mut self,
        penalize: bool,
    ) -> &mut VehicleRoutingProblemBuilder {
        self.penalize_waiting_duration = Some(penalize);
        self
    }

    pub fn set_services(&mut self, services: Vec<Service>) -> &mut VehicleRoutingProblemBuilder {
        self.services = Some(services);
        self
    }

    pub fn set_shipments(&mut self, shipments: Vec<Shipment>) -> &mut VehicleRoutingProblemBuilder {
        self.shipments = Some(shipments);
        self
    }

    pub fn set_locations(&mut self, locations: Vec<Location>) -> &mut VehicleRoutingProblemBuilder {
        self.locations = Some(locations);
        self
    }

    pub fn add_location(&mut self, location: Location) -> &mut VehicleRoutingProblemBuilder {
        if let Some(locations) = &mut self.locations {
            locations.push(location);
        } else {
            self.locations = Some(vec![location]);
        }

        self
    }

    pub fn add_vehicle_profile(
        &mut self,
        profile: VehicleProfile,
    ) -> &mut VehicleRoutingProblemBuilder {
        if let Some(vehicle_profiles) = &mut self.vehicle_profiles {
            vehicle_profiles.push(profile);
        } else {
            self.vehicle_profiles = Some(vec![profile]);
        }

        self
    }

    pub fn set_vehicle_profiles(
        &mut self,
        profiles: Vec<VehicleProfile>,
    ) -> &mut VehicleRoutingProblemBuilder {
        self.vehicle_profiles = Some(profiles);
        self
    }

    pub fn set_fleet(&mut self, fleet: Fleet) -> &mut VehicleRoutingProblemBuilder {
        self.fleet = Some(fleet);
        self
    }

    pub fn set_id(&mut self, id: String) -> &mut VehicleRoutingProblemBuilder {
        self.id = Some(id);
        self
    }

    pub fn set_relations(&mut self, relations: Vec<Relation>) -> &mut VehicleRoutingProblemBuilder {
        self.relations = Some(relations);
        self
    }

    pub fn build(self) -> Result<VehicleRoutingProblem, VehicleRoutingProblemError> {
        let locations = self
            .locations
            .ok_or(VehicleRoutingProblemError::MissingLocations)?;
        let services = self.services.unwrap_or_default();
        let shipments = self.shipments.unwrap_or_default();

        if services.is_empty() && shipments.is_empty() {
            return Err(VehicleRoutingProblemError::MissingJobs);
        }

        for service in services.iter() {
            if service.location_id().get() >= locations.len() {
                return Err(VehicleRoutingProblemError::LocationIdOutOfBounds(
                    service.location_id().get(),
                ));
            }
        }

        let mut jobs: Vec<Job> = services.into_iter().map(Job::Service).collect();
        jobs.extend(shipments.into_iter().map(Job::Shipment));

        let distance_method = self.distance_method.unwrap_or(DistanceMethod::Haversine);

        let vehicle_profiles = self
            .vehicle_profiles
            .ok_or(VehicleRoutingProblemError::MissingVehicleProfiles)?;

        let fleet = self.fleet.ok_or(VehicleRoutingProblemError::MissingFleet)?;

        VehicleRoutingProblem::try_from_params(VehicleRoutingProblemParams {
            id: self.id.unwrap_or_else(|| Uuid::new_v4().to_string()),
            locations,
            fleet,
            vehicle_profiles,
            jobs,
            distance_method,
            penalize_waiting_duration: self.penalize_waiting_duration.unwrap_or(true),
            relations: self.relations,
        })
    }
}
