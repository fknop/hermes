use super::recreate_strategy::RecreateStrategy;

pub struct RecreateParams {
    pub recreate_strategies: Vec<(RecreateStrategy, u64)>,
}

impl Default for RecreateParams {
    fn default() -> Self {
        RecreateParams {
            recreate_strategies: vec![
                (RecreateStrategy::RegretInsertion, 100),
                (RecreateStrategy::BestInsertion, 100),
            ],
        }
    }
}
