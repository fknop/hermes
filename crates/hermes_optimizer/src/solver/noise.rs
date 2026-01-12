use rand::{Rng, SeedableRng, rngs::SmallRng};

use crate::solver::score::Score;

#[derive(Clone)]
pub struct NoiseParams {
    pub max_cost: f64,
    pub noise_probability: f64,
    pub noise_level: f64,
}

pub struct JobNoiser {
    params: NoiseParams,
    rng: SmallRng,
}

impl JobNoiser {
    pub fn new(seed: u64, params: NoiseParams) -> Self {
        Self {
            params,
            rng: SmallRng::seed_from_u64(seed),
        }
    }

    pub fn create_noise(&mut self) -> f64 {
        if self.rng.random_bool(self.params.noise_probability) {
            self.params.noise_level * self.params.max_cost * self.rng.random_range(0.0..=1.0)
        } else {
            0.0
        }
    }

    pub fn apply_noise(&mut self, score: Score) -> Score {
        score + Score::soft(self.create_noise())
    }
}
