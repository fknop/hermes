use fxhash::FxHashSet;
use jiff::SignedDuration;

use crate::{
    problem::{
        amount::AmountExpression,
        job::{ActivityId, Job, JobIdx},
        meters::Meters,
    },
    solver::solution::working_solution::WorkingSolution,
};

use super::{ruin_context::RuinContext, ruin_solution::RuinSolution};

pub struct RuinShaw;

const DISTANCE_RELATEDNESS_WEIGHT: f64 = 9.0;
const TIME_RELATEDNESS_WEIGHT: f64 = 3.0;
const DEMAND_RELATEDNESS_WEIGHT: f64 = 2.0;

impl RuinShaw {
    fn relatedness(
        activity: &RelatednessToTargetActivity,
        max_distance: Meters,
        max_time: SignedDuration,
    ) -> f64 {
        let time_relatedness = if max_time.is_zero() {
            0.0
        } else {
            activity.time.as_secs_f64() / max_time.as_secs_f64()
        };

        let distance_relatedness = if max_distance.is_zero() {
            0.0
        } else {
            activity.distance / max_distance
        };

        TIME_RELATEDNESS_WEIGHT * time_relatedness
            + DISTANCE_RELATEDNESS_WEIGHT * distance_relatedness
            + DEMAND_RELATEDNESS_WEIGHT * activity.normalized_demand
    }
}

impl RuinSolution for RuinShaw {
    fn ruin_solution<R>(&self, solution: &mut WorkingSolution, context: RuinContext<R>)
    where
        R: rand::Rng,
    {
        let p = context.params.ruin_shaw_determinism;
        let routes = solution.routes();

        let target_job = solution.random_assigned_job(context.rng).unwrap();

        let mut max_distance: Meters = Meters::ZERO;
        let mut max_time: SignedDuration = SignedDuration::ZERO;

        let mut related_activities: Vec<RelatednessToTargetActivity> = Vec::new();
        let mut processed_jobs = FxHashSet::<JobIdx>::default();

        let target_job_route_id = solution.route_of_job(target_job).unwrap();

        for route in routes.iter() {
            for (pos, activity_id) in route.activity_ids().iter().enumerate() {
                if target_job == activity_id.job_id() {
                    continue; // Skip the target job itself
                }

                if processed_jobs.contains(&activity_id.job_id()) {
                    continue;
                }

                processed_jobs.insert(activity_id.job_id());

                let target_job_route = solution.route(target_job_route_id);

                let time_difference = match (
                    context.problem.job(target_job),
                    context.problem.job(activity_id.job_id()),
                ) {
                    (Job::Service(_), Job::Service(_)) => {
                        let target_job_position = target_job_route
                            .job_position(ActivityId::Service(target_job))
                            .unwrap();

                        let target_activity = target_job_route.activity(target_job_position);
                        let target_arrival = target_activity.arrival_time();

                        let activity = route.activity(pos);

                        target_arrival.duration_since(activity.arrival_time()).abs()
                    }
                    (Job::Shipment(_), Job::Shipment(_)) => {
                        let target_pickup_position = target_job_route
                            .job_position(ActivityId::ShipmentPickup(target_job))
                            .unwrap();

                        let target_delivery_position = target_job_route
                            .job_position(ActivityId::ShipmentPickup(target_job))
                            .unwrap();

                        let target_pickup_activity =
                            target_job_route.activity(target_pickup_position);
                        let target_delivery_activity =
                            target_job_route.activity(target_delivery_position);

                        let pickup_position = route
                            .job_position(ActivityId::ShipmentPickup(activity_id.job_id()))
                            .unwrap();

                        let delivery_position = route
                            .job_position(ActivityId::ShipmentPickup(activity_id.job_id()))
                            .unwrap();

                        let pickup_activity = route.activity(pickup_position);
                        let delivery_activity = route.activity(delivery_position);

                        target_pickup_activity
                            .arrival_time()
                            .duration_since(pickup_activity.arrival_time())
                            .abs()
                            + target_delivery_activity
                                .arrival_time()
                                .duration_since(delivery_activity.arrival_time())
                                .abs()
                    }
                    (Job::Service(_), Job::Shipment(_)) => {
                        let target_job_position = target_job_route
                            .job_position(ActivityId::Service(target_job))
                            .unwrap();

                        let target_activity = target_job_route.activity(target_job_position);

                        let pickup_position = route
                            .job_position(ActivityId::ShipmentPickup(activity_id.job_id()))
                            .unwrap();

                        let delivery_position = route
                            .job_position(ActivityId::ShipmentPickup(activity_id.job_id()))
                            .unwrap();

                        let pickup_activity = route.activity(pickup_position);
                        let delivery_activity = route.activity(delivery_position);

                        (target_activity
                            .arrival_time()
                            .duration_since(pickup_activity.arrival_time())
                            .abs()
                            + target_activity
                                .arrival_time()
                                .duration_since(delivery_activity.arrival_time())
                                .abs())
                            / 2
                    }
                    (Job::Shipment(_), Job::Service(_)) => {
                        let target_pickup_position = target_job_route
                            .job_position(ActivityId::ShipmentPickup(target_job))
                            .unwrap();

                        let target_delivery_position = target_job_route
                            .job_position(ActivityId::ShipmentPickup(target_job))
                            .unwrap();

                        let target_pickup_activity =
                            target_job_route.activity(target_pickup_position);
                        let target_delivery_activity =
                            target_job_route.activity(target_delivery_position);

                        let activity = route.activity(pos);

                        (target_pickup_activity
                            .arrival_time()
                            .duration_since(activity.arrival_time())
                            .abs()
                            + target_delivery_activity
                                .arrival_time()
                                .duration_since(activity.arrival_time())
                                .abs())
                            / 2
                    }
                };

                let distance = context
                    .problem
                    .travel_distance_between_jobs(target_job, activity_id.job_id());

                let demand_difference = (solution.problem().normalized_demand(target_job)
                    - solution.problem().normalized_demand(activity_id.job_id()))
                .iter()
                .map(|value| value.abs())
                .sum::<f64>()
                    / solution.problem().capacity_dimensions() as f64;

                related_activities.push(RelatednessToTargetActivity {
                    job_idx: activity_id.job_id(),
                    time: time_difference,
                    distance,
                    normalized_demand: demand_difference,
                });

                max_distance = max_distance.max(distance);
                max_time = max_time.max(time_difference);
            }
        }

        related_activities.sort_unstable_by(|a, b| {
            RuinShaw::relatedness(a, max_distance, max_time).total_cmp(&RuinShaw::relatedness(
                b,
                max_distance,
                max_time,
            ))
        });

        let mut remaining_to_remove = context.num_jobs_to_remove;

        solution.remove_job(target_job);
        remaining_to_remove -= 1;

        while remaining_to_remove > 0 {
            let y: f64 = context.rng.random_range(0.0..1.0);
            let index = (y.powf(p) * related_activities.len() as f64).floor() as usize;

            if let Some(job_id) = related_activities
                .get(index)
                .map(|candidate| candidate.job_idx)
            {
                solution.remove_job(job_id);
                remaining_to_remove -= 1;
            } else {
                break;
            }
        }
    }
}

struct RelatednessToTargetActivity {
    job_idx: JobIdx,
    time: SignedDuration,
    distance: Meters,
    normalized_demand: f64,
}
