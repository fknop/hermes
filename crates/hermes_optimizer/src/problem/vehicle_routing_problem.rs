use jiff::SignedDuration;

use crate::{
    problem::{
        amount::AmountExpression,
        capacity::Capacity,
        fleet::Fleet,
        job::{ActivityId, Job, JobIdx, JobTask},
        service::Service,
        shipment::Shipment,
        vehicle_profile::VehicleProfile,
    },
    solver::constraints::transport_cost_constraint::TRANSPORT_COST_WEIGHT,
    utils::{enumerate_idx::EnumerateIdx, zip_longest::zip_longest},
};

use super::{
    distance_method::DistanceMethod,
    location::{Location, LocationIdx},
    service_location_index::ServiceLocationIndex,
    travel_cost_matrix::{Cost, Distance},
    vehicle::{Vehicle, VehicleIdx},
};

type PrecomputedAverageCostFromDepot = Vec<Cost>;
type PrecomputedNormalizedDemands = Vec<Capacity>;

pub struct VehicleRoutingProblem {
    locations: Vec<Location>,
    fleet: Fleet,
    vehicle_profiles: Vec<VehicleProfile>,
    jobs: Vec<Job>,
    // travel_costs: TravelMatrices,
    service_location_index: ServiceLocationIndex,

    has_time_windows: bool,
    has_capacity: bool,

    precomputed_vehicle_compatibilities: Vec<bool>,
    precomputed_capacity_dimensions: usize,
    precomputed_normalized_demands: PrecomputedNormalizedDemands,
    precomputed_average_cost_from_depot: PrecomputedAverageCostFromDepot,

    /// Normalized weight for converting waiting duration into cost
    waiting_duration_weight: f64,
}

struct VehicleRoutingProblemParams {
    locations: Vec<Location>,
    fleet: Fleet,
    vehicle_profiles: Vec<VehicleProfile>,
    jobs: Vec<Job>,
    // travel_costs: TravelMatrices,
    distance_method: DistanceMethod,
}

impl VehicleRoutingProblem {
    fn new(params: VehicleRoutingProblemParams) -> Self {
        for vehicle in params.fleet.vehicles() {
            if vehicle.profile_id() >= params.vehicle_profiles.len() {
                panic!("Vehicle profile ID out of bounds")
            }
        }

        let service_location_index =
            ServiceLocationIndex::new(&params.locations, &params.jobs, params.distance_method);

        let precomputed_average_cost_from_depot =
            VehicleRoutingProblem::precompute_average_cost_from_depot(
                &params.locations,
                &params.fleet.vehicles(),
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

        let waiting_duration_weight =
            VehicleRoutingProblem::precompute_waiting_duration_weight(&params.vehicle_profiles);

        let precomputed_vehicle_compatibilities =
            VehicleRoutingProblem::precompute_vehicle_compatibilities(
                &params.fleet.vehicles(),
                &params.jobs,
            );

        Self {
            has_time_windows: params.jobs.iter().any(|job| job.has_time_windows()),
            has_capacity: params.jobs.iter().any(|job| !job.demand().is_empty()),
            locations: params.locations,
            fleet: params.fleet,
            vehicle_profiles: params.vehicle_profiles,
            jobs: params.jobs,
            // travel_costs: params.travel_costs,
            service_location_index,
            precomputed_average_cost_from_depot,
            precomputed_normalized_demands,
            precomputed_capacity_dimensions,
            precomputed_vehicle_compatibilities,
            waiting_duration_weight,
        }
    }

    pub fn jobs(&self) -> &[Job] {
        &self.jobs
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

    pub fn locations(&self) -> &[Location] {
        &self.locations
    }

    pub fn location(&self, location_id: LocationIdx) -> &Location {
        &self.locations[location_id]
    }

    pub fn job_task<'a>(&'a self, job_id: ActivityId) -> JobTask<'a> {
        // Can't use match here because if let bindings are experimental
        if let ActivityId::Service(service_id) = job_id
            && let Job::Service(service) = &self.jobs[service_id]
        {
            JobTask::Service(service)
        } else if let ActivityId::ShipmentPickup(shipment_id) = job_id
            && let Job::Shipment(shipment) = &self.jobs[shipment_id]
        {
            JobTask::ShipmentPickup(shipment)
        } else if let ActivityId::ShipmentDelivery(shipment_id) = job_id
            && let Job::Shipment(shipment) = &self.jobs[shipment_id]
        {
            JobTask::ShipmentDelivery(shipment)
        } else {
            panic!("Job {job_id} is not valid");
        }
    }

    #[deprecated(note = "use job location instead")]
    pub fn service_location(&self, service_id: usize) -> &Location {
        if let Job::Service(service) = &self.jobs[service_id] {
            let location_id = service.location_id();
            &self.locations[location_id]
        } else {
            panic!("Job {service_id} is not a service");
        }
    }

    pub fn vehicle_depot_location(&self, vehicle_id: VehicleIdx) -> Option<&Location> {
        let vehicle = &self.fleet.vehicle(vehicle_id);
        vehicle
            .depot_location_id()
            .map(|location_id| &self.locations[location_id])
    }

    pub fn travel_distance(
        &self,
        vehicle: &Vehicle,
        from: LocationIdx,
        to: LocationIdx,
    ) -> Distance {
        let profile_id = vehicle.profile_id();
        self.vehicle_profiles[profile_id].travel_distance(from, to)
    }

    pub fn max_cost(&self) -> Cost {
        self.vehicle_profiles
            .iter()
            .map(|profile| profile.travel_costs().max_cost() * TRANSPORT_COST_WEIGHT)
            .fold(0.0_f64, |a, b| a.max(b))
    }

    pub fn travel_time(
        &self,
        vehicle: &Vehicle,
        from: LocationIdx,
        to: LocationIdx,
    ) -> jiff::SignedDuration {
        let profile_id = vehicle.profile_id();
        self.vehicle_profiles[profile_id].travel_time(from, to)
    }

    pub fn travel_cost(&self, vehicle: &Vehicle, from: LocationIdx, to: LocationIdx) -> Cost {
        let profile_id = vehicle.profile_id();
        self.vehicle_profiles[profile_id].travel_cost(from, to)
    }

    pub fn travel_cost_or_zero(
        &self,
        vehicle: &Vehicle,
        from: Option<LocationIdx>,
        to: Option<LocationIdx>,
    ) -> Cost {
        if let (Some(from), Some(to)) = (from, to) {
            self.travel_cost(vehicle, from, to)
        } else {
            0.0
        }
    }

    pub fn acceptable_service_waiting_duration_secs(&self) -> i64 {
        0
    }

    pub fn waiting_duration_weight(&self) -> f64 {
        self.waiting_duration_weight
    }

    pub fn has_waiting_duration_cost(&self) -> bool {
        self.waiting_duration_weight() > 0.0
    }

    pub fn waiting_duration_cost(&self, waiting_duration: SignedDuration) -> Cost {
        waiting_duration.as_secs_f64() * self.waiting_duration_weight()
    }

    pub fn unassigned_job_cost(&self) -> Cost {
        // Should always be more worth to assign a job than leave it unassigned
        self.fixed_vehicle_costs() + 1.0
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
        let job_location_id = self.job_task(job_id).location_id();
        self.nearest_jobs_of_location(job_location_id)
    }

    pub fn is_symmetric(&self) -> bool {
        self.vehicle_profiles
            .iter()
            .all(|profile| profile.travel_costs().is_symmetric())
    }

    pub fn has_time_windows(&self) -> bool {
        self.has_time_windows
    }

    pub fn has_capacity(&self) -> bool {
        self.has_capacity
    }

    pub fn average_cost_from_depot(&self, location_id: LocationIdx) -> Distance {
        self.precomputed_average_cost_from_depot[location_id.get()]
    }

    pub fn normalized_demand(&self, index: JobIdx) -> &Capacity {
        &self.precomputed_normalized_demands[index.get()]
    }

    pub fn capacity_dimensions(&self) -> usize {
        self.precomputed_capacity_dimensions
    }

    pub fn is_service_compatible_with_vehicle(
        &self,
        vehicle_index: usize,
        job_index: usize,
    ) -> bool {
        let index = (vehicle_index * self.fleet.vehicles().len()) + job_index;
        self.precomputed_vehicle_compatibilities[index]
    }

    fn precompute_vehicle_compatibilities(vehicles: &[Vehicle], jobs: &[Job]) -> Vec<bool> {
        let mut compatibilities = vec![true; vehicles.len() * jobs.len()];

        for (vehicle_index, vehicle) in vehicles.iter().enumerate() {
            for (job_index, job) in jobs.iter().enumerate() {
                //  from * self.num_locations + to
                let index = (vehicle_index * vehicles.len()) + job_index;
                if !vehicle.is_compatible_with(job) {
                    compatibilities[index] = false;
                }
            }
        }

        compatibilities
    }

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

    fn precompute_average_cost_from_depot(
        locations: &[Location],
        vehicles: &[Vehicle],
        profiles: &[VehicleProfile],
    ) -> PrecomputedAverageCostFromDepot {
        let mut precomputed_average_cost_from_depot = Vec::with_capacity(locations.len());

        precomputed_average_cost_from_depot.extend(locations.iter().enumerate_idx().map(
            |(location_id, location)| {
                vehicles
                    .iter()
                    .filter_map(|vehicle| {
                        if let Some(depot_location_id) = vehicle.depot_location_id() {
                            Some(
                                profiles[vehicle.profile_id()]
                                    .travel_cost(depot_location_id, location_id),
                            )
                        } else {
                            None
                        }
                    })
                    .sum::<Distance>()
                    / vehicles.len() as Distance
            },
        ));

        precomputed_average_cost_from_depot
    }
}

#[derive(Default)]
pub struct VehicleRoutingProblemBuilder {
    services: Option<Vec<Service>>,
    locations: Option<Vec<Location>>,
    fleet: Option<Fleet>,
    vehicle_profiles: Option<Vec<VehicleProfile>>,
    distance_method: Option<DistanceMethod>,
}

impl VehicleRoutingProblemBuilder {
    pub fn set_distance_method(
        &mut self,
        distance_method: DistanceMethod,
    ) -> &mut VehicleRoutingProblemBuilder {
        self.distance_method = Some(distance_method);
        self
    }

    pub fn set_services(&mut self, services: Vec<Service>) -> &mut VehicleRoutingProblemBuilder {
        self.services = Some(services);
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

    pub fn build(self) -> VehicleRoutingProblem {
        let locations = self.locations.expect("Expected list of locations");
        let services = self.services.expect("Expected list of services");

        // for (index, location) in locations.iter().enumerate_idx() {
        //     if location.id() != index {
        //         panic!("Location IDs must be sequential starting from 0");
        //     }
        // }

        for service in services.iter() {
            if service.location_id().get() >= locations.len() {
                panic!("Service location_id must be within the range of locations");
            }
        }

        let jobs = services.into_iter().map(Job::Service).collect::<Vec<Job>>();

        let distance_method = self.distance_method.unwrap_or(DistanceMethod::Haversine);

        let vehicle_profiles = self
            .vehicle_profiles
            .expect("Expected list of vehicle profiles");

        VehicleRoutingProblem::new(VehicleRoutingProblemParams {
            locations,
            fleet: self.fleet.expect("Expected fleet"),
            vehicle_profiles,
            jobs,
            distance_method,
        })
    }
}
