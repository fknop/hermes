use jiff::SignedDuration;
use rand::Rng;

use crate::{problem::travel_cost_matrix::Distance, solver::working_solution::WorkingSolution};

use super::{ruin_context::RuinContext, ruin_solution::RuinSolution};

pub struct RuinTimeRelated;

impl RuinTimeRelated {
    fn relatedness(
        activity: &RelatednessToTargetActivity,
        max_distance: Distance,
        max_time: SignedDuration,
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

        10.0 * time_relatedness + distance_relatedness
    }
}

impl RuinSolution for RuinTimeRelated {
    fn ruin_solution(&self, solution: &mut WorkingSolution, context: RuinContext) {
        let routes = solution.routes();
        let target_route_id = context.rng.random_range(0..routes.len());
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

                let time_difference = target_activity
                    .arrival_time()
                    .duration_since(activity.arrival_time())
                    .abs();
                let distance = context.problem.travel_distance(
                    target_activity.service(context.problem).location_id(),
                    activity.service(context.problem).location_id(),
                );

                related_activities.push(RelatednessToTargetActivity {
                    route_id: route_index,
                    activity_id: activity_index,
                    time: time_difference,
                    distance,
                });

                if distance > max_distance {
                    max_distance = distance;
                }
                if time_difference > max_time {
                    max_time = time_difference;
                }
            }
        }

        related_activities.sort_by(|a, b| {
            RuinTimeRelated::relatedness(a, max_distance, max_time)
                .total_cmp(&RuinTimeRelated::relatedness(b, max_distance, max_time))
        });

        let mut remaining_to_remove = context.num_activities_to_remove;

        for related_activity in related_activities {
            if remaining_to_remove == 0 {
                break;
            }

            solution.remove_activity(related_activity.route_id, related_activity.activity_id);
            remaining_to_remove -= 1;
        }
    }
}

struct RelatednessToTargetActivity {
    route_id: usize,
    activity_id: usize,
    time: SignedDuration,
    distance: Distance,
}
