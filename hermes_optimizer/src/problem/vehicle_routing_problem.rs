use super::{
    location::Location,
    service::{Service, ServiceId},
    travel_cost_matrix::{Cost, Distance, Time, TravelCostMatrix},
    vehicle::{Vehicle, VehicleId},
};

pub struct VehicleRoutingProblem {
    locations: Vec<Location>,
    vehicles: Vec<Vehicle>,
    services: Vec<Service>,
    travel_costs: TravelCostMatrix,
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

    pub fn travel_distance(&self, from: &Location, to: &Location) -> Distance {
        self.travel_costs.travel_distance(from.id(), to.id())
    }

    pub fn travel_time(&self, from: &Location, to: &Location) -> jiff::SignedDuration {
        let travel_time_seconds = self.travel_costs.travel_time(from.id(), to.id());
        jiff::SignedDuration::from_secs(travel_time_seconds)
    }

    pub fn travel_cost(&self, from: &Location, to: &Location) -> Cost {
        self.travel_costs.travel_cost(from.id(), to.id())
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
    pub fn with_travel_costs(mut self, travel_costs: TravelCostMatrix) -> Self {
        self.travel_costs = Some(travel_costs);
        self
    }

    pub fn with_services(mut self, services: Vec<Service>) -> Self {
        self.services = Some(services);
        self
    }

    pub fn with_locations(mut self, locations: Vec<Location>) -> Self {
        self.locations = Some(locations);
        self
    }

    pub fn with_vehicles(mut self, vehicles: Vec<Vehicle>) -> Self {
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

        VehicleRoutingProblem {
            locations,
            vehicles: self.vehicles.expect("Expected list of vehicles"),
            services,
            travel_costs: self.travel_costs.expect("Missing travel_costs"),
        }
    }
}
