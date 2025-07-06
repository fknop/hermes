use super::{ruin_radial::RuinRadial, ruin_strategy::RuinStrategy};

pub struct RuinParams {
    pub ruin_strategies: Vec<(RuinStrategy, u64)>,

    /// Between 0.0 and 1.0, where 1.0 means that the ruin will remove up to 100% of the solution
    pub ruin_minimum_ratio: f64,

    /// Between 0.0 and 1.0, where 1.0 means that the ruin will remove up to 100% of the solution
    pub ruin_maximum_ratio: f64,
}

impl Default for RuinParams {
    fn default() -> Self {
        RuinParams {
            ruin_strategies: vec![
                (RuinStrategy::Random, 50),
                (RuinStrategy::RuinWorst, 50),
                (RuinStrategy::RuinRadial, 200),
            ],
            ruin_minimum_ratio: 0.05,
            ruin_maximum_ratio: 0.3,
        }
    }
}
