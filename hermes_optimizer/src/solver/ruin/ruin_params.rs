use super::ruin_strategy::RuinStrategy;

pub struct RuinParams {
    pub ruin_strategies: Vec<(RuinStrategy, u64)>,

    /// Between 0.0 and 1.0, where 1.0 means that the ruin will remove up to 100% of the solution
    pub ruin_maximum_ratio: f64,
}

impl Default for RuinParams {
    fn default() -> Self {
        RuinParams {
            ruin_strategies: vec![(RuinStrategy::Random, 100)],
            ruin_maximum_ratio: 0.7, // Default to removing up to 70% of the solution
        }
    }
}
