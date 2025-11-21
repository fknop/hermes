use std::sync::Arc;

use hermes_optimizer::{
    problem::{
        distance_method::DistanceMethod,
        location::{Location, LocationId},
        service::{Service, ServiceBuilder, ServiceId},
        travel_cost_matrix::TravelCostMatrix,
        vehicle::{Vehicle, VehicleBuilder},
        vehicle_routing_problem::{VehicleRoutingProblem, VehicleRoutingProblemBuilder},
    },
    solver::{
        insertion::{ExistingRouteInsertion, Insertion, NewRouteInsertion},
        solution::working_solution::WorkingSolution,
    },
};

//
//  ASCII Schema for coordinates:
//
//  Y-axis
//  ^
//  |
//  | (0.0, 3.0)  (1.0, 3.0)  (2.0, 3.0)  (3.0, 3.0)
//  |
//  | (0.0, 2.0)  (1.0, 2.0)  (2.0, 2.0)  (3.0, 2.0)
//  |
//  | (0.0, 1.0)  (1.0, 1.0)  (2.0, 1.0)  (3.0, 1.0)
//  |
//  | (0.0, 0.0)  (1.0, 0.0)  (2.0, 0.0)  (3.0, 0.0)
//  +------------------------------------------------> X-axis
pub fn create_location_grid(rows: usize, cols: usize) -> Vec<Location> {
    let mut locations = Vec::new();

    for y in 0..rows {
        for x in 0..cols {
            let location = Location::from_cartesian(locations.len(), x as f64, y as f64);
            locations.push(location);
        }
    }

    locations
}

pub fn create_locations(locations: Vec<(f64, f64)>) -> Vec<Location> {
    locations
        .iter()
        .enumerate()
        .map(|(index, &(x, y))| Location::from_cartesian(index, x, y))
        .collect()
}

pub fn create_basic_services(location_ids: Vec<LocationId>) -> Vec<Service> {
    location_ids
        .iter()
        .enumerate()
        .map(|(index, &location_id)| {
            let mut builder = ServiceBuilder::default();

            builder.set_location_id(location_id);
            builder.set_external_id(index.to_string());
            builder.build()
        })
        .collect()
}

pub fn create_basic_vehicles(location_ids: Vec<LocationId>) -> Vec<Vehicle> {
    location_ids
        .iter()
        .enumerate()
        .map(|(index, &location_id)| {
            let mut builder = VehicleBuilder::default();
            builder.set_depot_location_id(location_id);
            builder.set_vehicle_id(index.to_string());
            builder.build()
        })
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

pub struct TestRoute {
    pub vehicle_id: usize,
    pub service_ids: Vec<ServiceId>,
}

pub fn create_test_working_solution(
    problem: Arc<VehicleRoutingProblem>,
    routes: Vec<TestRoute>,
) -> WorkingSolution {
    let mut solution = WorkingSolution::new(problem);

    for (route_id, route) in routes.iter().enumerate() {
        for (index, service_id) in route.service_ids.iter().enumerate() {
            if index == 0 {
                solution.insert_service(&Insertion::NewRoute(NewRouteInsertion {
                    vehicle_id: route.vehicle_id,
                    service_id: *service_id,
                }));
            } else {
                solution.insert_service(&Insertion::ExistingRoute(ExistingRouteInsertion {
                    position: index,
                    route_id,
                    service_id: *service_id,
                }));
            }
        }
    }

    solution
}
