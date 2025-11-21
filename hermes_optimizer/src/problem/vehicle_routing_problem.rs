use jiff::SignedDuration;
use rand::{Rng, rngs::SmallRng};

use crate::{
    problem::neighborhood::{BuildNeighborhoodParams, Neighborhoods},
    solver::constraints::transport_cost_constraint::TRANSPORT_COST_WEIGHT,
};

use super::{
    distance_method::DistanceMethod,
    location::{Location, LocationId},
    service::{Service, ServiceId},
    service_location_index::ServiceLocationIndex,
    travel_cost_matrix::{Cost, Distance, TravelCostMatrix},
    vehicle::{Vehicle, VehicleId},
};

pub struct VehicleRoutingProblem {
    locations: Vec<Location>,
    vehicles: Vec<Vehicle>,
    services: Vec<Service>,
    travel_costs: TravelCostMatrix,
    service_location_index: ServiceLocationIndex,
    neighborhoods: Neighborhoods,
}

impl VehicleRoutingProblem {
    pub fn services(&self) -> &[Service] {
        &self.services
    }

    pub fn service(&self, index: usize) -> &Service {
        &self.services[index]
    }

    pub fn random_location<R>(&self, rng: &mut R) -> usize
    where
        R: rand::Rng,
    {
        rng.random_range(0..self.locations.len())
    }

    pub fn random_service<R>(&self, rng: &mut R) -> usize
    where
        R: rand::Rng,
    {
        rng.random_range(0..self.services.len())
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

    pub fn service_location(&self, service_id: ServiceId) -> &Location {
        let service = &self.services[service_id];
        let location_id = service.location_id();
        &self.locations[location_id]
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

    pub fn waiting_cost(&self, waiting_duration: SignedDuration) -> Cost {
        waiting_duration.as_secs_f64() * 10.0
    }

    pub fn fixed_vehicle_costs(&self) -> f64 {
        100000.0 //self.max_cost() // Placeholder for the static cost of a route
    }

    pub fn nearest_services_of_location(
        &self,
        location_id: usize,
    ) -> impl Iterator<Item = ServiceId> {
        let location = &self.locations[location_id];
        self.service_location_index.nearest_neighbor_iter(location)
    }

    pub fn nearest_services(&self, service_id: ServiceId) -> impl Iterator<Item = ServiceId> {
        let location_id = self.service(service_id).location_id();
        self.nearest_services_of_location(location_id)
    }

    pub fn service_neighborhood_iter(
        &self,
        service_id: ServiceId,
    ) -> impl Iterator<Item = ServiceId> {
        self.neighborhoods.neighbors_iter(service_id)
    }

    pub fn is_symmetric(&self) -> bool {
        self.travel_costs.is_symmetric()
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
        let service_location_index = ServiceLocationIndex::new(
            &locations,
            &services,
            // TODO: benchmark which is best ?
            self.distance_method.unwrap_or(DistanceMethod::Haversine),
        );

        // let neighborhoods = Neighborhoods::new(BuildNeighborhoodParams {
        //     services: &services,
        //     location_index: &service_location_index,
        //     locations: &locations,
        // });

        VehicleRoutingProblem {
            locations,
            vehicles: self.vehicles.expect("Expected list of vehicles"),
            services,
            travel_costs,
            service_location_index,
            neighborhoods: Neighborhoods::empty(),
        }
    }
}
