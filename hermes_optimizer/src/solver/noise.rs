use rand::{Rng, rngs::SmallRng};

pub struct NoiseGenerator {
    noise_probability: f64,
    noise_level: f64,
    max_cost: f64,
}

impl NoiseGenerator {
    pub fn new(max_cost: f64, noise_probability: f64, noise_level: f64) -> Self {
        NoiseGenerator {
            noise_probability,
            noise_level,
            max_cost,
        }
    }

    pub fn create_noise(&self, rng: &mut SmallRng) -> f64 {
        if rng.random_bool(self.noise_probability) {
            self.noise_level * self.max_cost * rng.random_range(0.0..=1.0)
        } else {
            0.0
        }
    }
}
