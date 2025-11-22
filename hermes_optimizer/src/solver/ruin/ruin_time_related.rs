use jiff::SignedDuration;

use crate::{
    problem::travel_cost_matrix::Distance, solver::solution::working_solution::WorkingSolution,
};

use super::{ruin_context::RuinContext, ruin_solution::RuinSolution};

pub struct RuinTimeRelated;

impl RuinTimeRelated {
    fn relatedness(
        activity: &RelatednessToTargetActivity,
        max_distance: Distance,
        max_time: SignedDuration,
        distance_influence: f64,
        time_influence: f64,
    ) -> f64 {
        let time_relatedness = if max_time.is_zero() {
            0.0
        } else {
            activity.time.as_secs_f64() / max_time.as_secs_f64()
        };

        let distance_relatedness = if max_distance == 0.0 {
            0.0
        } else {
            activity.distance / max_distance
        };

        time_influence * time_relatedness + distance_influence * distance_relatedness
    }
}

impl RuinSolution for RuinTimeRelated {
    fn ruin_solution<R>(&self, solution: &mut WorkingSolution, context: RuinContext<R>)
    where
        R: rand::Rng,
    {
        let routes = solution.routes();

        let target_route_id = solution.random_non_empty_route(context.rng).unwrap();

        let target_activity_id = context
            .rng
            .random_range(0..routes[target_route_id].activities().len());

        let target_activity = &routes[target_route_id].activities()[target_activity_id];

        let mut max_distance: Distance = 0.0;
        let mut max_time: SignedDuration = SignedDuration::ZERO;

        let mut related_activities: Vec<RelatednessToTargetActivity> = Vec::new();
        for (route_index, route) in routes.iter().enumerate() {
            for (activity_index, activity) in route.activities().iter().enumerate() {
                if target_activity_id == activity_index && target_route_id == route_index {
                    continue; // Skip the target activity itself
                }

                let target_arrival =
                    target_activity.arrival_time() /*+ target_activity.waiting_duration()*/;

                let time_difference = target_arrival
                    .duration_since(
                        activity.arrival_time(), /*+ activity.waiting_duration()*/
                    )
                    .abs();
                let distance = context.problem.travel_distance(
                    target_activity.service(context.problem).location_id(),
                    activity.service(context.problem).location_id(),
                );

                related_activities.push(RelatednessToTargetActivity {
                    service_id: activity.service_id(),
                    time: time_difference,
                    distance,
                });

                max_distance = max_distance.max(distance);
                max_time = max_time.max(time_difference);
            }
        }

        let distance_influence = 1.0;
        let time_influence = 10.0;

        related_activities.sort_by(|a, b| {
            RuinTimeRelated::relatedness(
                a,
                max_distance,
                max_time,
                distance_influence,
                time_influence,
            )
            .total_cmp(&RuinTimeRelated::relatedness(
                b,
                max_distance,
                max_time,
                distance_influence,
                time_influence,
            ))
        });

        let mut remaining_to_remove = context.num_activities_to_remove;

        for related_activity in related_activities {
            if remaining_to_remove == 0 {
                break;
            }

            if solution.remove_service(related_activity.service_id) {
                remaining_to_remove -= 1;
            }
            // solution
            // .remove_service_from_route(related_activity.route_id, related_activity.service_id);
        }
    }
}

struct RelatednessToTargetActivity {
    service_id: usize,
    time: SignedDuration,
    distance: Distance,
}
