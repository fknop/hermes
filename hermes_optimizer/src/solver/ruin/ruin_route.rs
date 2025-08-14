use std::f64;

use rand::seq::{IndexedRandom, IteratorRandom};

use crate::solver::working_solution::WorkingSolution;

use super::{ruin_context::RuinContext, ruin_solution::RuinSolution};

pub struct RuinRoute;

impl RuinSolution for RuinRoute {
    fn ruin_solution(&self, solution: &mut WorkingSolution, context: RuinContext) {
        // let mut route_to_ruin = None;
        // let mut max = f64::MIN;

        // for (route_id, route) in solution.routes().iter().enumerate() {
        //     let cost = route.duration(context.problem).as_secs_f64() * 0.7
        //         + route.total_waiting_duration().as_secs_f64() * 0.3;

        //     if cost > max {
        //         max = cost;
        //         route_to_ruin = Some(route_id);
        //     }
        // }

        if let Some(route_id) = solution
            .routes()
            .iter()
            .enumerate()
            .map(|(route_id, _)| route_id)
            .choose(context.rng)
        {
            solution.remove_route(route_id);
        }

        // if let Some(route_to_ruin) = route_to_ruin {
        //     solution.remove_route(route_to_ruin);
        // }
    }
}
