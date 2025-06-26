use rand::rngs::SmallRng;

pub struct RecreateContext<'a> {
    pub rng: &'a mut SmallRng,
}
