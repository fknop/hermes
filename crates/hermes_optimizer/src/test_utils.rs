use std::{fs::File, path::PathBuf, str::FromStr, sync::Arc};

use hermes_matrix_providers::{cache::MatricesCache, travel_matrix_client::TravelMatrixClient};
use rand::RngCore;

use crate::{
    json::types::JsonVehicleRoutingProblem,
    problem::{
        distance_method::DistanceMethod,
        fleet::Fleet,
        location::Location,
        service::{Service, ServiceBuilder},
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

struct TestMatricesCache {
    path: PathBuf,
}

impl MatricesCache for TestMatricesCache {
    fn cache<P>(
        &self,
        _provider: &hermes_matrix_providers::travel_matrix_provider::TravelMatrixProvider,
        _points: &[P],
        _matrices: &hermes_matrix_providers::travel_matrices::TravelMatrices,
    ) -> Result<(), anyhow::Error>
    where
        for<'a> &'a P: Into<geo::Point>,
    {
        Ok(())
    }

    fn get_cached<P>(
        &self,
        _provider: &hermes_matrix_providers::travel_matrix_provider::TravelMatrixProvider,
        _points: &[P],
    ) -> Result<Option<hermes_matrix_providers::travel_matrices::TravelMatrices>, anyhow::Error>
    where
        for<'a> &'a P: Into<geo::Point>,
    {
        let file = File::open(&self.path).unwrap();
        let json: hermes_matrix_providers::travel_matrices::TravelMatrices =
            serde_json::from_reader(file).unwrap();

        Ok(Some(json))
    }
}

pub async fn create_test_problem_from_json_file(dir: PathBuf) -> VehicleRoutingProblem {
    let input_path = dir.join("input.json");
    let matrices_path = dir.join("matrices.json");
    let file = File::open(&input_path).unwrap();
    let json: JsonVehicleRoutingProblem = serde_json::from_reader(file).unwrap();
    let cache = TestMatricesCache {
        path: matrices_path,
    };

    let client = TravelMatrixClient::new(cache);

    json.build_problem(&client).await.unwrap()
}

pub fn data_fixture_path(fixture: &str) -> PathBuf {
    let current_working_dir = std::env::current_dir().unwrap();

    current_working_dir
        .join("../../data/optimizer/fixtures/")
        .join(fixture)
        .canonicalize()
        .unwrap()
}

pub fn create_location_grid(rows: usize, cols: usize) -> Vec<Location> {
    let mut locations = Vec::new();

    for y in 0..rows {
        for x in 0..cols {
            let location = Location::from_cartesian(x as f64, y as f64);
            locations.push(location);
        }
    }

    locations
}

pub fn create_locations(locations: Vec<(f64, f64)>) -> Vec<Location> {
    locations
        .iter()
        .map(|&(x, y)| Location::from_cartesian(x, y))
        .collect()
}

pub fn create_basic_services(location_ids: Vec<usize>) -> Vec<Service> {
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

pub fn create_basic_vehicles(location_ids: Vec<usize>) -> Vec<Vehicle> {
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
        TravelMatrices::from_euclidean(&locations, false),
    )]);
    builder.set_services(services);
    builder.set_locations(locations);
    builder.set_fleet(Fleet::Finite(vehicles));

    builder.build()
}

pub struct TestRoute {
    pub vehicle_id: usize,
    pub service_ids: Vec<usize>,
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
                job_index: service_id.into(),
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
