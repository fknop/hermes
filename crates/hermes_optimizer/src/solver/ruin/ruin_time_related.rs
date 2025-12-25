use jiff::SignedDuration;

use crate::{
    problem::{amount::AmountExpression, travel_cost_matrix::Distance},
    solver::solution::working_solution::WorkingSolution,
};

use super::{ruin_context::RuinContext, ruin_solution::RuinSolution};

// TODO: support shipments
pub struct RuinTimeRelated;

/*
*   fn default() -> Self {
      Self {
          distance_weight: 9.0,
          time_weight: 3.0,
          demand_weight: 2.0,
          same_vehicle_weight: 5.0,
          determinism: 6.0,
      }
  }
*/

const TIME_RELATEDNESS_WEIGHT: f64 = 3.0;
const DISTANCE_RELATEDNESS_WEIGHT: f64 = 9.0;
const DEMAND_RELATEDNESS_WEIGHT: f64 = 2.0;

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

        TIME_RELATEDNESS_WEIGHT * time_relatedness
            + DISTANCE_RELATEDNESS_WEIGHT * distance_relatedness
            + DEMAND_RELATEDNESS_WEIGHT * activity.normalized_demand
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
            .random_range(0..routes[target_route_id].activity_ids().len());

        let target_activity = &routes[target_route_id].activity(target_activity_id);

        let mut max_distance: Distance = 0.0;
        let mut max_time: SignedDuration = SignedDuration::ZERO;

        let mut related_activities: Vec<RelatednessToTargetActivity> = Vec::new();
        for (route_index, route) in routes.iter().enumerate() {
            for (activity_index, _) in route.activity_ids().iter().enumerate() {
                if target_activity_id == activity_index && target_route_id == route_index {
                    continue; // Skip the target activity itself
                }

                let activity = route.activity(activity_index);

                let target_arrival =
                    target_activity.arrival_time() /*+ target_activity.waiting_duration()*/;

                let time_difference = target_arrival
                    .duration_since(
                        activity.arrival_time(), /*+ activity.waiting_duration()*/
                    )
                    .abs();
                let distance = context.problem.travel_distance(
                    target_activity.job_task(context.problem).location_id(),
                    activity.job_task(context.problem).location_id(),
                );

                let demand_difference = (solution.problem().normalized_demand(target_activity_id)
                    - solution
                        .problem()
                        .normalized_demand(activity.job_id().index()))
                .iter()
                .map(|value| value.abs())
                .sum::<f64>()
                    / solution.problem().capacity_dimensions() as f64;

                related_activities.push(RelatednessToTargetActivity {
                    service_id: activity.job_id().index(),
                    time: time_difference,
                    distance,
                    normalized_demand: demand_difference,
                });

                max_distance = max_distance.max(distance);
                max_time = max_time.max(time_difference);
            }
        }

        related_activities.sort_unstable_by(|a, b| {
            RuinTimeRelated::relatedness(a, max_distance, max_time)
                .total_cmp(&RuinTimeRelated::relatedness(b, max_distance, max_time))
        });

        let mut remaining_to_remove = context.num_jobs_to_remove;

        solution.remove_activity(target_route_id, target_activity_id);
        remaining_to_remove -= 1;

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
    normalized_demand: f64,
}
