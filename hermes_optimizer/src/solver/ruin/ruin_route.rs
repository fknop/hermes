use std::cmp::Ordering;

use rand::seq::IndexedRandom;

use crate::solver::solution::working_solution::WorkingSolution;

use super::{ruin_context::RuinContext, ruin_solution::RuinSolution};

pub struct RuinRoute;

impl RuinSolution for RuinRoute {
    fn ruin_solution<R>(&self, solution: &mut WorkingSolution, context: RuinContext<R>)
    where
        R: rand::Rng,
    {
        let mut remaining: i64 = context.num_jobs_to_remove as i64;

        let mut routes = solution
            .non_empty_routes_iter()
            .map(|r1| {
                let fit_in_other_route = solution
                    .non_empty_routes_iter()
                    .any(|r2| r1.can_route_capacity_fit_in(context.problem, r2));

                (r1, fit_in_other_route)
            })
            .collect::<Vec<_>>();

        // routes.sort_unstable_by(|r1, r2| {
        //     r1.delivery_load_slack()
        //         .partial_cmp(r2.delivery_load_slack())
        //         .unwrap_or(Ordering::Equal)
        //         .reverse()
        // });

        let no_fit = routes
            .iter()
            .all(|(_, fit_in_other_route)| !fit_in_other_route);

        if let Ok((route, _)) =
            routes.choose_weighted(context.rng, |(route, fit_in_other_route)| {
                if !no_fit && *fit_in_other_route {
                    return 1.0;
                }

                route.duration(context.problem).as_secs_f64() * 0.7
                    + route.total_waiting_duration().as_secs_f64() * 0.3
            })
        {
            solution.remove_route(route.vehicle_id());
        }

        // while remaining > 0 {
        //     // let route_ids = solution
        //     //     .routes()
        //     //     .iter()
        //     //     .enumerate()
        //     //     .filter(|(_, route)| !route.is_empty())
        //     //     .map(|(id, _)| id)
        //     //     .collect::<Vec<usize>>();

        //     // let route_id = route_ids
        //     //     .choose_weighted(context.rng, |&route_id| {
        //     //         let route = solution.route(route_id);
        //     //         route.duration(context.problem).as_secs_f64() * 0.7
        //     //             + route.total_waiting_duration().as_secs_f64() * 0.3
        //     //     })
        //     //     .ok();

        //     if let Some(&route_id) = route_id {
        //         let removed = solution.remove_route(route_id);
        //         remaining -= removed as i64;
        //     } else {
        //         break;
        //     }
        // }
    }
}
