use jiff::SignedDuration;
use rayon::iter::{IntoParallelRefIterator, ParallelExtend, ParallelIterator};
use uuid::Timestamp;

use crate::{
    problem::{
        amount::AmountExpression,
        capacity::Capacity,
        job::{Job, JobId, JobTask},
        service::{Service, ServiceId},
        shipment::Shipment,
        time_window::TimeWindow,
    },
    solver::constraints::transport_cost_constraint::TRANSPORT_COST_WEIGHT,
    utils::zip_longest::zip_longest,
};

use super::{
    distance_method::DistanceMethod,
    location::{Location, LocationId},
    service_location_index::ServiceLocationIndex,
    travel_cost_matrix::{Cost, Distance, TravelCostMatrix},
    vehicle::{Vehicle, VehicleId},
};

type PrecomputedAverageCostFromDepot = Vec<Cost>;
type PrecomputedNormalizedDemands = Vec<Capacity>;

pub struct VehicleRoutingProblem {
    locations: Vec<Location>,
    vehicles: Vec<Vehicle>,
    jobs: Vec<Job>,
    travel_costs: TravelCostMatrix,
    service_location_index: ServiceLocationIndex,

    has_time_windows: bool,
    has_capacity: bool,

    precomputed_capacity_dimensions: usize,
    precomputed_normalized_demands: PrecomputedNormalizedDemands,
    precomputed_average_cost_from_depot: PrecomputedAverageCostFromDepot,
}

struct VehicleRoutingProblemParams {
    locations: Vec<Location>,
    vehicles: Vec<Vehicle>,
    jobs: Vec<Job>,
    travel_costs: TravelCostMatrix,
    distance_method: DistanceMethod,
}

impl VehicleRoutingProblem {
    fn new(params: VehicleRoutingProblemParams) -> Self {
        let service_location_index =
            ServiceLocationIndex::new(&params.locations, &params.jobs, params.distance_method);

        let precomputed_average_cost_from_depot =
            VehicleRoutingProblem::precompute_average_cost_from_depot(
                &params.locations,
                &params.vehicles,
                &params.travel_costs,
            );

        let precomputed_normalized_demands =
            VehicleRoutingProblem::precompute_normalized_demands(&params.jobs);

        let precomputed_capacity_dimensions = params
            .jobs
            .iter()
            .map(|job| job.demand())
            .chain(params.vehicles.iter().map(|vehicle| vehicle.capacity()))
            .map(|capacity| capacity.len())
            .max()
            .unwrap_or(0);

        Self {
            has_time_windows: params.jobs.iter().any(|job| job.has_time_windows()),
            has_capacity: params.jobs.iter().any(|job| !job.demand().is_empty()),
            locations: params.locations,
            vehicles: params.vehicles,
            jobs: params.jobs,
            travel_costs: params.travel_costs,
            service_location_index,
            precomputed_average_cost_from_depot,
            precomputed_normalized_demands,
            precomputed_capacity_dimensions,
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

    pub fn job<Index>(&self, index: Index) -> &Job
    where
        Index: Into<usize>,
    {
        &self.jobs[index.into()]
    }

    pub fn service(&self, index: usize) -> &Service {
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

    pub fn random_location<R>(&self, rng: &mut R) -> usize
    where
        R: rand::Rng,
    {
        rng.random_range(0..self.locations.len())
    }

    pub fn random_job<R>(&self, rng: &mut R) -> usize
    where
        R: rand::Rng,
    {
        rng.random_range(0..self.jobs.len())
    }

    pub fn vehicle(&self, index: usize) -> &Vehicle {
        &self.vehicles[index]
    }

    pub fn vehicles(&self) -> &[Vehicle] {
        &self.vehicles
    }

    pub fn locations(&self) -> &[Location] {
        &self.locations
    }

    pub fn location(&self, location_id: usize) -> &Location {
        &self.locations[location_id]
    }

    pub fn job_task<'a>(&'a self, job_id: JobId) -> JobTask<'a> {
        // Can't use match here because if let bindings are experimental
        if let JobId::Service(service_id) = job_id
            && let Job::Service(service) = &self.jobs[service_id]
        {
            JobTask::Service(service)
        } else if let JobId::ShipmentPickup(shipment_id) = job_id
            && let Job::Shipment(shipment) = &self.jobs[shipment_id]
        {
            JobTask::ShipmentPickup(shipment)
        } else if let JobId::ShipmentDelivery(shipment_id) = job_id
            && let Job::Shipment(shipment) = &self.jobs[shipment_id]
        {
            JobTask::ShipmentDelivery(shipment)
        } else {
            panic!("Job {job_id} is not valid");
        }
    }

    pub fn service_location(&self, service_id: ServiceId) -> &Location {
        if let Job::Service(service) = &self.jobs[service_id] {
            let location_id = service.location_id();
            return &self.locations[location_id];
        } else {
            panic!("Job {service_id} is not a service");
        }
    }

    pub fn vehicle_depot_location(&self, vehicle_id: VehicleId) -> Option<&Location> {
        let vehicle = &self.vehicles[vehicle_id];
        vehicle
            .depot_location_id()
            .map(|location_id| &self.locations[location_id])
    }

    pub fn travel_distance(&self, from: LocationId, to: LocationId) -> Distance {
        self.travel_costs.travel_distance(from, to)
    }

    pub fn max_cost(&self) -> Cost {
        self.travel_costs.max_cost() * TRANSPORT_COST_WEIGHT
    }

    pub fn travel_time(&self, from: LocationId, to: LocationId) -> jiff::SignedDuration {
        let travel_time_seconds = self.travel_costs.travel_time(from, to);
        jiff::SignedDuration::from_secs(travel_time_seconds)
    }

    pub fn travel_cost(&self, from: LocationId, to: LocationId) -> Cost {
        self.travel_costs.travel_cost(from, to)
    }

    pub fn travel_cost_or_zero(&self, from: Option<LocationId>, to: Option<LocationId>) -> Cost {
        if let (Some(from), Some(to)) = (from, to) {
            self.travel_cost(from, to)
        } else {
            0.0
        }
    }

    pub fn acceptable_service_waiting_duration_secs(&self) -> i64 {
        0
    }

    pub fn waiting_duration_weight(&self) -> f64 {
        10.0
    }

    pub fn has_waiting_duration_cost(&self) -> bool {
        self.waiting_duration_weight() > 0.0
    }

    pub fn waiting_duration_cost(&self, waiting_duration: SignedDuration) -> Cost {
        waiting_duration.as_secs_f64() * self.waiting_duration_weight()
    }

    pub fn fixed_vehicle_costs(&self) -> f64 {
        100000.0 //self.max_cost() // Placeholder for the static cost of a route
    }

    pub fn nearest_jobs_of_location(&self, location_id: usize) -> impl Iterator<Item = JobId> {
        let location = &self.locations[location_id];
        self.service_location_index.nearest_neighbor_iter(location)
    }

    pub fn nearest_jobs(&self, job_id: JobId) -> impl Iterator<Item = JobId> {
        let job_location_id = self.job_task(job_id).location_id();
        self.nearest_jobs_of_location(job_location_id)
    }

    pub fn is_symmetric(&self) -> bool {
        self.travel_costs.is_symmetric()
    }

    pub fn has_time_windows(&self) -> bool {
        self.has_time_windows
    }

    pub fn has_capacity(&self) -> bool {
        self.has_capacity
    }

    pub fn average_cost_from_depot(&self, location_id: LocationId) -> Distance {
        self.precomputed_average_cost_from_depot[location_id]
    }

    pub fn normalized_demand(&self, index: usize) -> &Capacity {
        &self.precomputed_normalized_demands[index]
    }

    pub fn capacity_dimensions(&self) -> usize {
        self.precomputed_capacity_dimensions
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
        matrix: &TravelCostMatrix,
    ) -> PrecomputedAverageCostFromDepot {
        let mut precomputed_average_cost_from_depot = Vec::with_capacity(locations.len());

        precomputed_average_cost_from_depot.extend(locations.iter().map(|location_id| {
            vehicles
                .iter()
                .filter_map(|vehicle| vehicle.depot_location_id())
                .map(|depot_location_id| matrix.travel_cost(depot_location_id, location_id.id()))
                .sum::<Distance>()
                / vehicles.len() as Distance
        }));

        precomputed_average_cost_from_depot
    }
}

#[derive(Default)]
pub struct VehicleRoutingProblemBuilder {
    travel_costs: Option<TravelCostMatrix>,
    services: Option<Vec<Service>>,
    locations: Option<Vec<Location>>,
    vehicles: Option<Vec<Vehicle>>,
    distance_method: Option<DistanceMethod>,
}

impl VehicleRoutingProblemBuilder {
    pub fn set_travel_costs(
        &mut self,
        travel_costs: TravelCostMatrix,
    ) -> &mut VehicleRoutingProblemBuilder {
        self.travel_costs = Some(travel_costs);
        self
    }

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

    pub fn set_vehicles(&mut self, vehicles: Vec<Vehicle>) -> &mut VehicleRoutingProblemBuilder {
        self.vehicles = Some(vehicles);
        self
    }

    pub fn build(self) -> VehicleRoutingProblem {
        let locations = self.locations.expect("Expected list of locations");
        let services = self.services.expect("Expected list of services");

        for (index, location) in locations.iter().enumerate() {
            if location.id() != index {
                panic!("Location IDs must be sequential starting from 0");
            }
        }

        for service in services.iter() {
            if service.location_id() >= locations.len() {
                panic!("Service location_id must be within the range of locations");
            }
        }

        let travel_costs = self.travel_costs.expect("Expected travel costs matrix");

        let jobs = services.into_iter().map(Job::Service).collect::<Vec<Job>>();

        let distance_method = self.distance_method.unwrap_or(DistanceMethod::Haversine);

        let vehicles = self.vehicles.expect("Expected list of vehicles");

        VehicleRoutingProblem::new(VehicleRoutingProblemParams {
            locations,
            vehicles,
            jobs,
            travel_costs,
            distance_method,
        })
    }
}
