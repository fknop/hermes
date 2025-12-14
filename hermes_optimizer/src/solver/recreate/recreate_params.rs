use super::{best_insertion::BestInsertionSortMethod, recreate_strategy::RecreateStrategy};

#[derive(Clone, Debug)]
pub struct RecreateParams {
    pub recreate_strategies: Vec<RecreateStrategy>,
    pub insert_on_failure: bool,
}

impl Default for RecreateParams {
    fn default() -> Self {
        RecreateParams {
            insert_on_failure: true,
            recreate_strategies: vec![
                RecreateStrategy::RegretInsertion,
                RecreateStrategy::BestInsertion(BestInsertionSortMethod::Random),
                RecreateStrategy::BestInsertion(BestInsertionSortMethod::Demand),
                RecreateStrategy::BestInsertion(BestInsertionSortMethod::Far),
                RecreateStrategy::BestInsertion(BestInsertionSortMethod::Close),
                RecreateStrategy::BestInsertion(BestInsertionSortMethod::TimeWindow),
                // RecreateStrategy::CompleteBestInsertion,
            ],
        }
    }
}
