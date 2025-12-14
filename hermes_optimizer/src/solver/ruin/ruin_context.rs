use rand::RngCore;

use crate::{
    problem::vehicle_routing_problem::VehicleRoutingProblem, solver::ruin::ruin_params::RuinParams,
};

pub struct RuinContext<'a, R>
where
    R: RngCore,
{
    pub params: &'a RuinParams,
    pub problem: &'a VehicleRoutingProblem,
    pub rng: &'a mut R,
    pub num_jobs_to_remove: usize,
}
