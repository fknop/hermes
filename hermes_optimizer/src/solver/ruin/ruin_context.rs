use rand::rngs::SmallRng;

pub struct RuinContext<'a> {
    pub rng: &'a mut SmallRng,
    pub num_activities_to_remove: usize,
}
