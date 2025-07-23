use super::ruin_strategy::RuinStrategy;

#[derive(Clone, Debug)]
pub struct RuinParams {
    pub ruin_strategies: Vec<RuinStrategy>,

    /// Between 0.0 and 1.0, where 1.0 means that the ruin will remove up to 100% of the solution
    pub ruin_minimum_ratio: f64,

    /// Between 0.0 and 1.0, where 1.0 means that the ruin will remove up to 100% of the solution
    pub ruin_maximum_ratio: f64,

    pub ruin_minimum_size: usize,
    pub ruin_maximum_size: usize,
}

impl Default for RuinParams {
    fn default() -> Self {
        RuinParams {
            ruin_strategies: vec![
                RuinStrategy::RuinString,
                RuinStrategy::RuinTimeRelated,
                RuinStrategy::RuinRadial,
                RuinStrategy::Random,
                RuinStrategy::RuinWorst,
            ],
            ruin_minimum_ratio: 0.05,
            ruin_maximum_ratio: 0.3,
            ruin_minimum_size: 3,
            ruin_maximum_size: 60,
        }
    }
}
