use crate::solver::solution::working_solution::WorkingSolution;

use super::{ruin_context::RuinContext, ruin_solution::RuinSolution};

pub struct RuinRandom;

impl RuinSolution for RuinRandom {
    fn ruin_solution<R>(
        &self,
        solution: &mut WorkingSolution,
        RuinContext {
            rng,
            num_jobs_to_remove,
            ..
        }: RuinContext<R>,
    ) where
        R: rand::Rng,
    {
        for _ in 0..num_jobs_to_remove {
            if let Some(route_id) = solution.random_non_empty_route(rng) {
                let route = solution.route(route_id);
                let position = route.random_activity(rng);
                solution.remove_job(route.job_id(position));
            } else {
                break;
            }
        }
    }
}
