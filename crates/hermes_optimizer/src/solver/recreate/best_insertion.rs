use std::fmt::Display;

use jiff::Timestamp;
use rand::{Rng, rngs::SmallRng, seq::SliceRandom};
use serde::Serialize;

use crate::{
    problem::{
        amount::AmountExpression,
        job::{Job, JobIdx},
        vehicle_routing_problem::VehicleRoutingProblem,
    },
    solver::{
        insertion::{Insertion, for_each_insertion},
        recreate::recreate_strategy::RecreateStrategy,
        score::{RUN_SCORE_ASSERTIONS, Score},
        solution::working_solution::WorkingSolution,
    },
};

use super::{recreate_context::RecreateContext, recreate_solution::RecreateSolution};

#[derive(Default)]
pub struct BestInsertion {
    sort_method: BestInsertionSortStrategy,
    blink_rate: f64,
}

#[derive(Default, Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize)]
pub enum BestInsertionSortStrategy {
    #[default]
    Random,
    Demand,
    Far,
    Close,
    TimeWindow,
}

impl Display for BestInsertionSortStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Random => write!(f, "Random"),
            Self::Demand => write!(f, "Demand"),
            Self::Far => write!(f, "Far"),
            Self::Close => write!(f, "Close"),
            Self::TimeWindow => write!(f, "TimeWindow"),
        }
    }
}

pub struct BestInsertionParams {
    pub sort_strategy: BestInsertionSortStrategy,
    pub blink_rate: f64,
}

impl BestInsertion {
    pub fn new(
        BestInsertionParams {
            sort_strategy: sort_method,
            blink_rate,
        }: BestInsertionParams,
    ) -> Self {
        BestInsertion {
            sort_method,
            blink_rate,
        }
    }

    pub fn sort_unassigned_jobs(
        &self,
        problem: &VehicleRoutingProblem,
        unassigned_jobs: &mut [JobIdx],
        rng: &mut SmallRng,
    ) {
        match self.sort_method {
            BestInsertionSortStrategy::Random => {
                unassigned_jobs.shuffle(rng);
            }
            BestInsertionSortStrategy::Demand => unassigned_jobs.sort_unstable_by(|a, b| {
                // Not perfect but good enough for sorting purposes.
                let first_demand_a = problem.job(*a).demand().get(0);
                let first_demand_b = problem.job(*b).demand().get(0);

                first_demand_a.total_cmp(&first_demand_b)
            }),
            BestInsertionSortStrategy::Far => {
                unassigned_jobs.sort_unstable_by_key(|&id| match problem.job(id) {
                    Job::Shipment(shipment) => {
                        let pickup_distance =
                            problem.average_cost_from_depot(shipment.pickup().location_id());
                        let delivery_distance =
                            problem.average_cost_from_depot(shipment.delivery().location_id());

                        let avg_distance = (pickup_distance + delivery_distance) / 2.0;
                        -avg_distance.round() as i64
                    }
                    Job::Service(service) => {
                        let distance_from_depot =
                            problem.average_cost_from_depot(service.location_id());
                        -distance_from_depot.round() as i64
                    }
                });
            }
            BestInsertionSortStrategy::Close => {
                unassigned_jobs.sort_unstable_by_key(|&id| match problem.job(id) {
                    Job::Shipment(shipment) => {
                        let pickup_distance =
                            problem.average_cost_from_depot(shipment.pickup().location_id());
                        let delivery_distance =
                            problem.average_cost_from_depot(shipment.delivery().location_id());

                        let avg_distance = (pickup_distance + delivery_distance) / 2.0;
                        avg_distance.round() as i64
                    }
                    Job::Service(service) => {
                        let distance_from_depot =
                            problem.average_cost_from_depot(service.location_id());
                        distance_from_depot.round() as i64
                    }
                })
            }
            BestInsertionSortStrategy::TimeWindow => {
                unassigned_jobs.sort_unstable_by_key(|&job_id| {
                    let time_windows = match problem.job(job_id) {
                        Job::Service(service) => service.time_windows(),
                        Job::Shipment(shipment) => shipment.pickup().time_windows(),
                    };

                    let end = time_windows.end();

                    end.unwrap_or(Timestamp::MAX)
                });
            }
        }
    }

    fn should_blink(&self, rng: &mut SmallRng) -> bool {
        rng.random_bool(self.blink_rate)
    }

    pub fn insert_jobs(
        &self,
        unassigned_jobs: &Vec<JobIdx>,
        solution: &mut WorkingSolution,
        mut context: RecreateContext,
    ) {
        let iteration_seed = context.create_iteration_seed();
        for &job_id in unassigned_jobs {
            let mut best_insertion: Option<Insertion> = None;
            let mut best_score = Score::MAX;
            let noiser_seed = context.create_noiser_seed(iteration_seed, job_id);
            let mut noiser = context.create_noiser(noiser_seed);

            for_each_insertion(solution, job_id, |insertion| {
                if self.should_blink(context.rng) {
                    return;
                }

                let score = noiser.apply_noise(context.compute_insertion_score(
                    solution,
                    &insertion,
                    Some(&best_score),
                ));

                if score < best_score {
                    best_score = score;
                    best_insertion = Some(insertion);
                }
            });

            if context.should_insert(&best_score) {
                if let Some(insertion) = best_insertion {
                    if RUN_SCORE_ASSERTIONS {
                        context.insert_with_score_assertions(
                            solution,
                            insertion.clone(),
                            RecreateStrategy::BestInsertion(self.sort_method),
                        );
                    } else {
                        solution.insert(&insertion);
                    }
                } else {
                    panic!("No insertion possible")
                }
            }
        }
    }
}

impl RecreateSolution for BestInsertion {
    fn recreate_solution(&self, solution: &mut WorkingSolution, context: RecreateContext) {
        let mut unassigned_services: Vec<_> = solution.unassigned_jobs().iter().copied().collect();

        self.sort_unassigned_jobs(context.problem, &mut unassigned_services, context.rng);
        // unassigned_services.shuffle(context.rng);

        self.insert_jobs(&unassigned_services, solution, context);
    }
}
