use super::{best_insertion::BestInsertionSortStrategy, recreate_strategy::RecreateStrategy};

#[derive(Clone, Debug)]
pub struct RecreateParams {
    pub recreate_strategies: Vec<RecreateStrategy>,
    pub insert_on_failure: bool,
}

impl Default for RecreateParams {
    fn default() -> Self {
        RecreateParams {
            insert_on_failure: false,
            recreate_strategies: vec![
                RecreateStrategy::RegretInsertion(2),
                // RecreateStrategy::RegretInsertion(3),
                RecreateStrategy::BestInsertion(BestInsertionSortStrategy::Random),
                RecreateStrategy::BestInsertion(BestInsertionSortStrategy::Demand),
                RecreateStrategy::BestInsertion(BestInsertionSortStrategy::Far),
                RecreateStrategy::BestInsertion(BestInsertionSortStrategy::Close),
                RecreateStrategy::BestInsertion(BestInsertionSortStrategy::TimeWindow),
                // RecreateStrategy::CompleteBestInsertion,
            ],
        }
    }
}
