use rand::Rng;

use crate::solver::working_solution::WorkingSolution;

use super::ruin_solution::RuinSolution;

pub struct RuinRandom;

impl RuinSolution for RuinRandom {
    fn ruin_solution(&self, solution: &mut WorkingSolution, num_activities_to_remove: usize) {
        let mut rng = rand::rng();

        for _ in 0..num_activities_to_remove {
            let route_id = rng.random_range(0..solution.routes().len());
            let position = rng.random_range(0..solution.route(route_id).activities().len());
            solution.remove_activity(route_id, position);
        }
    }
}
