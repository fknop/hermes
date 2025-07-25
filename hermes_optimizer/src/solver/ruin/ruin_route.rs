use rand::Rng;

use crate::solver::working_solution::WorkingSolution;

use super::{ruin_context::RuinContext, ruin_solution::RuinSolution};

pub struct RuinRoute;

impl RuinSolution for RuinRoute {
    fn ruin_solution(&self, solution: &mut WorkingSolution, RuinContext { rng, .. }: RuinContext) {
        if solution.routes().is_empty() {
            return;
        }

        let route_id = rng.random_range(0..solution.routes().len());
        solution.remove_service(route_id);
    }
}
