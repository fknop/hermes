use rand::seq::IndexedRandom;

use crate::solver::working_solution::WorkingSolution;

use super::{ruin_context::RuinContext, ruin_solution::RuinSolution};

pub struct RuinRoute;

impl RuinSolution for RuinRoute {
    fn ruin_solution<R>(&self, solution: &mut WorkingSolution, context: RuinContext<R>)
    where
        R: rand::Rng,
    {
        let mut remaining: i64 = context.num_activities_to_remove as i64;

        while remaining > 0 {
            let route_ids: Vec<usize> = (0..solution.routes().len()).collect();

            let route_id = route_ids
                .choose_weighted(context.rng, |&route_id| {
                    let route = solution.route(route_id);
                    route.duration(context.problem).as_secs_f64() * 0.7
                        + route.total_waiting_duration().as_secs_f64() * 0.3
                })
                .ok();

            if let Some(&route_id) = route_id {
                let removed = solution.remove_route(route_id);
                remaining -= removed as i64;
            } else {
                break;
            }
        }
    }
}
