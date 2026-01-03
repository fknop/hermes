use parking_lot::Mutex;
use rand::{Rng, SeedableRng, rngs::SmallRng};

use crate::problem::job::JobIdx;

pub struct NoiseGenerator {
    rngs: Vec<Mutex<SmallRng>>,
    pub noise_probability: f64,
    pub noise_level: f64,
    pub max_cost: f64,
}

impl NoiseGenerator {
    pub fn new(
        num_jobs: usize,
        max_cost: f64,
        noise_probability: f64,
        noise_level: f64,
        rng: &mut SmallRng,
    ) -> Self {
        Self {
            rngs: (0..num_jobs)
                .map(|_| Mutex::new(SmallRng::from_rng(rng)))
                .collect(),
            noise_probability,
            noise_level,
            max_cost,
        }
    }

    pub fn create_noise(&self, index: JobIdx) -> f64 {
        let mut rng = self.rngs[index.get()].lock();

        if rng.random_bool(self.noise_probability) {
            self.noise_level * self.max_cost * rng.random_range(0.0..=1.0)
        } else {
            0.0
        }
    }
}
