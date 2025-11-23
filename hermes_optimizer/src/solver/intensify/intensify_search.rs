use crate::problem::{vehicle::VehicleId, vehicle_routing_problem::VehicleRoutingProblem};

type VehiclePair = (VehicleId, VehicleId);
pub struct IntensifySearch {
    pairs: Vec<VehiclePair>,
    gains: Vec<Vec<f64>>,
}

impl IntensifySearch {
    pub fn new(problem: &VehicleRoutingProblem) -> Self {
        let vehicle_count = problem.vehicles().len();
        let mut gains = Vec::with_capacity(vehicle_count);
        for _ in 0..vehicle_count {
            gains.push(vec![0.0; vehicle_count]);
        }

        let mut pairs = Vec::with_capacity(vehicle_count * vehicle_count);
        for i in 0..vehicle_count {
            for j in 0..vehicle_count {
                pairs.push((i, j))
            }
        }

        IntensifySearch { gains, pairs }
    }

    pub fn run(&mut self) {}
}
