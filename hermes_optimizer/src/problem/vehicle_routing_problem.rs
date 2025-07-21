use jiff::SignedDuration;

use super::{
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
}

impl VehicleRoutingProblem {
    pub fn services(&self) -> &[Service] {
        &self.services
    }

    pub fn service(&self, index: usize) -> &Service {
        &self.services[index]
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
        self.travel_costs.max_cost()
    }

    pub fn travel_time(&self, from: LocationId, to: LocationId) -> jiff::SignedDuration {
        let travel_time_seconds = self.travel_costs.travel_time(from, to);
        jiff::SignedDuration::from_secs(travel_time_seconds)
    }

    pub fn travel_cost(&self, from: LocationId, to: LocationId) -> Cost {
        self.travel_costs.travel_cost(from, to)
    }

    pub fn acceptable_service_waiting_duration_secs(&self) -> i64 {
        60
    }

    pub fn waiting_cost(&self, waiting_duration: SignedDuration) -> Cost {
        waiting_duration.as_secs_f64() * 5.0
    }

    pub fn route_costs(&self) -> f64 {
        2000.0 // Placeholder for the static cost of a route
    }

    pub fn nearest_services(&self, service_id: ServiceId) -> impl Iterator<Item = ServiceId> {
        let location_id = self.service(service_id).location_id();
        let location = &self.locations[location_id];
        self.service_location_index
            .nearest_neighbor_iter(geo::Point::new(location.x(), location.y()))
    }
}

#[derive(Default)]
pub struct VehicleRoutingProblemBuilder {
    travel_costs: Option<TravelCostMatrix>,
    services: Option<Vec<Service>>,
    locations: Option<Vec<Location>>,
    vehicles: Option<Vec<Vehicle>>,
}

impl VehicleRoutingProblemBuilder {
    pub fn set_travel_costs(
        &mut self,
        travel_costs: TravelCostMatrix,
    ) -> &mut VehicleRoutingProblemBuilder {
        self.travel_costs = Some(travel_costs);
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
        let service_location_index = ServiceLocationIndex::new(&locations, &services);

        VehicleRoutingProblem {
            locations,
            vehicles: self.vehicles.expect("Expected list of vehicles"),
            services,
            travel_costs,
            service_location_index,
        }
    }
}
