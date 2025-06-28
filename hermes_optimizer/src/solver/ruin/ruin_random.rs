use rand::Rng;

use crate::solver::working_solution::WorkingSolution;

use super::{ruin_context::RuinContext, ruin_solution::RuinSolution};

pub struct RuinRandom;

impl RuinSolution for RuinRandom {
    fn ruin_solution(
        &self,
        solution: &mut WorkingSolution,
        RuinContext {
            rng,
            num_activities_to_remove,
            ..
        }: RuinContext,
    ) {
        if solution.routes().is_empty() {
            return;
        }

        for _ in 0..num_activities_to_remove {
            let route_id = rng.random_range(0..solution.routes().len());
            let position = rng.random_range(0..solution.route(route_id).activities().len());
            solution.remove_activity(route_id, position);
        }
    }
}
