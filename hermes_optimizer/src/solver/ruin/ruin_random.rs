use crate::solver::working_solution::WorkingSolution;

use super::{ruin_context::RuinContext, ruin_solution::RuinSolution};

pub struct RuinRandom;

impl RuinSolution for RuinRandom {
    fn ruin_solution<R>(
        &self,
        solution: &mut WorkingSolution,
        RuinContext {
            rng,
            num_activities_to_remove,
            ..
        }: RuinContext<R>,
    ) where
        R: rand::Rng,
    {
        for _ in 0..num_activities_to_remove {
            let route_id = solution.random_route(rng);
            let position = solution.route(route_id).random_activity(rng);
            solution.remove_activity(route_id, position);
        }
    }
}
