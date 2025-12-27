use std::sync::Arc;

use rand::RngCore;

use crate::{
    problem::{
        distance_method::DistanceMethod,
        location::{Location, LocationId},
        service::{Service, ServiceBuilder, ServiceId},
        travel_cost_matrix::TravelMatrices,
        vehicle::{Vehicle, VehicleBuilder},
        vehicle_profile::VehicleProfile,
        vehicle_routing_problem::{VehicleRoutingProblem, VehicleRoutingProblemBuilder},
    },
    solver::{
        insertion::{Insertion, ServiceInsertion},
        solution::{route_id::RouteIdx, working_solution::WorkingSolution},
    },
};

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
            builder.set_profile_id(0);
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
    builder.set_vehicle_profiles(vec![VehicleProfile::new(
        "test_profile".to_owned(),
        TravelMatrices::from_euclidian(&locations),
    )]);
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
        for (index, &service_id) in route.service_ids.iter().enumerate() {
            solution.insert(&Insertion::Service(ServiceInsertion {
                route_id: RouteIdx::new(route_id),
                job_index: service_id,
                position: index,
            }));
        }
    }

    solution
}

pub struct MockRng {
    data: Vec<u64>,
    index: usize,
}

impl MockRng {
    pub fn new(data: Vec<u64>) -> Self {
        MockRng { data, index: 0 }
    }
}

impl RngCore for MockRng {
    fn next_u32(&mut self) -> u32 {
        self.next_u64() as u32
    }

    fn next_u64(&mut self) -> u64 {
        let value = self.data[self.index % self.data.len()];
        self.index = (self.index + 1) % self.data.len();
        value
    }

    fn fill_bytes(&mut self, dst: &mut [u8]) {
        for byte in dst.iter_mut() {
            *byte = 0;
        }
    }
}

#[cfg(test)]
mod tests {
    use rand::Rng;

    use super::*;

    #[test]
    fn test_mock_rng() {
        let data = vec![1, 2, 3, 4];
        let mut rng = MockRng::new(data.clone());

        for &expected in data.iter().cycle().take(8) {
            let value = rng.next_u64();
            assert_eq!(value, expected);
        }
    }

    #[test]
    fn test_random_bool() {
        let data = vec![
            (u64::MAX / 4),
            (u64::MAX / 4),
            (u64::MAX / 4),
            (u64::MAX / 4),
        ];
        let mut rng = MockRng::new(data);

        assert!(!rng.random_bool(0.20));
        assert!(rng.random_bool(0.26));
        assert!(rng.random_bool(0.6));
        assert!(!rng.random_bool(0.10));
    }
}
