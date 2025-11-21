use crate::solver::solution::working_solution::WorkingSolution;

use super::{ruin_context::RuinContext, ruin_solution::RuinSolution};

pub struct RuinRadial;

impl RuinSolution for RuinRadial {
    fn ruin_solution<R>(
        &self,
        solution: &mut WorkingSolution,
        RuinContext {
            rng,
            num_activities_to_remove,
            problem,
            ..
        }: RuinContext<R>,
    ) where
        R: rand::Rng,
    {
        let random_location_id = problem.random_location(rng);

        for service_id in problem
            .nearest_services_of_location(random_location_id)
            .take(num_activities_to_remove)
        {
            solution.remove_service(service_id);
        }

        // let random_service_id = problem.random_service(rng);

        // for service_id in problem
        //     .nearest_services(random_service_id)
        //     .take(num_activities_to_remove)
        // {
        //     solution.remove_service(service_id);
        // }
    }
}
