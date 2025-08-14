use rand::Rng;

use crate::solver::working_solution::WorkingSolution;

use super::{ruin_context::RuinContext, ruin_solution::RuinSolution};

pub struct RuinRadial;

impl RuinSolution for RuinRadial {
    fn ruin_solution(
        &self,
        solution: &mut WorkingSolution,
        RuinContext {
            rng,
            num_activities_to_remove,
            problem,
            ..
        }: RuinContext,
    ) {
        let random_service_id = problem.random_service(rng);

        for service_id in problem
            .nearest_services(random_service_id)
            .take(num_activities_to_remove)
        {
            solution.remove_service(service_id);
        }
    }
}
