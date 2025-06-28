use rand::Rng;

use crate::{problem::service::ServiceId, solver::working_solution::WorkingSolution};

use super::{ruin_context::RuinContext, ruin_solution::RuinSolution};

pub struct RuinRadial;

impl RuinSolution for RuinRadial {
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

        let problem = solution.problem();
        let random_service_id = rng.random_range(0..problem.services().len());

        let service_ids: Vec<ServiceId> = problem
            .nearest_services(random_service_id)
            .take(num_activities_to_remove)
            .collect();

        for service_id in service_ids {
            solution.remove_service(service_id);
        }
    }
}
