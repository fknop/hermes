use rand::{RngCore, rngs::SmallRng};

use crate::problem::vehicle_routing_problem::VehicleRoutingProblem;

pub struct RuinContext<'a, R>
where
    R: RngCore,
{
    pub problem: &'a VehicleRoutingProblem,
    pub rng: &'a mut R,
    pub num_activities_to_remove: usize,
}
