use crate::problem::vehicle_routing_problem::VehicleRoutingProblem;

use super::{best_insertion::BestInsertionSortStrategy, recreate_strategy::RecreateStrategy};

#[derive(Clone, Debug)]
pub struct RecreateParams {
    pub recreate_strategies: Vec<RecreateStrategy>,
    pub insert_on_failure: bool,
}

impl RecreateParams {
    pub fn default_from_problem(problem: &VehicleRoutingProblem) -> Self {
        let mut strategies: Vec<RecreateStrategy> = vec![
            RecreateStrategy::RegretInsertion(2),
            RecreateStrategy::BestInsertion(BestInsertionSortStrategy::Random),
            RecreateStrategy::BestInsertion(BestInsertionSortStrategy::Far),
            RecreateStrategy::BestInsertion(BestInsertionSortStrategy::Close),
        ];

        if problem.has_time_windows() {
            strategies.push(RecreateStrategy::BestInsertion(
                BestInsertionSortStrategy::TimeWindow,
            ));
        }

        if problem.has_capacity() {
            strategies.push(RecreateStrategy::BestInsertion(
                BestInsertionSortStrategy::Demand,
            ));
        }

        Self { ..Self::default() }
    }
}

impl Default for RecreateParams {
    fn default() -> Self {
        RecreateParams {
            insert_on_failure: false,
            recreate_strategies: vec![
                RecreateStrategy::RegretInsertion(2),
                RecreateStrategy::BestInsertion(BestInsertionSortStrategy::Random),
                RecreateStrategy::BestInsertion(BestInsertionSortStrategy::Demand),
                RecreateStrategy::BestInsertion(BestInsertionSortStrategy::Far),
                RecreateStrategy::BestInsertion(BestInsertionSortStrategy::Close),
                RecreateStrategy::BestInsertion(BestInsertionSortStrategy::TimeWindow),
            ],
        }
    }
}
