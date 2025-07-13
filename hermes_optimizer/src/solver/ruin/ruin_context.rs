use rand::rngs::SmallRng;

use crate::problem::vehicle_routing_problem::VehicleRoutingProblem;

pub struct RuinContext<'a> {
    pub problem: &'a VehicleRoutingProblem,
    pub rng: &'a mut SmallRng,
    pub num_activities_to_remove: usize,
}
