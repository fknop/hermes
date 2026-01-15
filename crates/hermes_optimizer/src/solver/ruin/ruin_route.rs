use rand::seq::IndexedRandom;
use tracing::warn;

use crate::solver::solution::{route_id::RouteIdx, working_solution::WorkingSolution};

use super::{ruin_context::RuinContext, ruin_solution::RuinSolution};

pub struct RuinRoute;

impl RuinSolution for RuinRoute {
    fn ruin_solution<R>(&self, solution: &mut WorkingSolution, context: RuinContext<R>)
    where
        R: rand::Rng,
    {
        let mut remaining: i64 = context.num_jobs_to_remove as i64;

        while remaining > 0 && !solution.is_empty() {
            let routes = solution
                .routes()
                .iter()
                .enumerate()
                .filter(|(_, route)| !route.is_empty())
                .map(|(index, r1)| {
                    let fit_in_other_route = solution
                        .non_empty_routes_iter()
                        .any(|r2| r1.can_route_capacity_fit_in(context.problem, r2));

                    (RouteIdx::new(index), r1, fit_in_other_route)
                })
                .collect::<Vec<_>>();

            if routes.is_empty() {
                break;
            }

            let no_fit = routes
                .iter()
                .all(|(_, _, fit_in_other_route)| !fit_in_other_route);

            let all_fit = routes
                .iter()
                .all(|(_, _, fit_in_other_route)| *fit_in_other_route);

            if let Ok((route_id, _, _)) =
                routes.choose_weighted(context.rng, |(_, route, fit_in_other_route)| {
                    let is_full = route.has_maximum_activities(solution.problem());
                    let full_weight = if is_full { 2.0 } else { 1.0 };
                    if no_fit || all_fit {
                        return full_weight
                            * (route.duration(context.problem).as_secs_f64() * 0.7
                                + route.total_waiting_duration().as_secs_f64() * 0.3);
                    }

                    full_weight * if *fit_in_other_route { 10.0 } else { 1.0 }
                })
            {
                let removed = solution.remove_route(*route_id);
                remaining -= removed as i64;
            } else {
                warn!("RuinRoute: could not select a route to remove");
                break;
            }
        }
    }
}
