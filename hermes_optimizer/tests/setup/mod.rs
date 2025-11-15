use hermes_optimizer::problem::{
    distance_method::DistanceMethod,
    location::Location,
    service::Service,
    travel_cost_matrix::TravelCostMatrix,
    vehicle::Vehicle,
    vehicle_routing_problem::{VehicleRoutingProblem, VehicleRoutingProblemBuilder},
};

pub fn create_locations(locations: &[(f64, f64)]) -> Vec<Location> {
    locations
        .iter()
        .enumerate()
        .map(|(index, &(x, y))| Location::from_cartesian(index, x, y))
        .collect()
}

pub fn create_test_problem(
    locations: Vec<Location>,
    services: Vec<Service>,
    vehicles: Vec<Vehicle>,
) -> VehicleRoutingProblem {
    let mut builder = VehicleRoutingProblemBuilder::default();

    builder.set_distance_method(DistanceMethod::Euclidean);
    builder.set_travel_costs(TravelCostMatrix::from_haversine(&locations));
    builder.set_services(services);
    builder.set_locations(locations);
    builder.set_vehicles(vehicles);

    builder.build()
}
