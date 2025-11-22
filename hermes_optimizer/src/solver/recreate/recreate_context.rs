use rand::rngs::SmallRng;

use crate::{
    problem::vehicle_routing_problem::VehicleRoutingProblem,
    solver::{
        constraints::{compute_insertion_score::compute_insertion_score, constraint::Constraint},
        insertion::{ExistingRouteInsertion, Insertion, NewRouteInsertion},
        insertion_context::compute_insertion_context,
        noise::NoiseGenerator,
        score::Score,
        solution::working_solution::WorkingSolution,
    },
};

pub struct RecreateContext<'a> {
    pub rng: &'a mut SmallRng,
    pub constraints: &'a Vec<Constraint>,
    pub problem: &'a VehicleRoutingProblem,
    pub noise_generator: Option<&'a NoiseGenerator>,
    pub thread_pool: &'a rayon::ThreadPool,
}

impl<'a> RecreateContext<'a> {
    pub fn compute_insertion_score(
        &self,
        solution: &WorkingSolution,
        insertion: &Insertion,
    ) -> Score {
        // Temporary check until enum is reworked
        match insertion {
            Insertion::NewRoute(NewRouteInsertion { vehicle_id, .. }) => {
                if !solution.route(*vehicle_id).is_empty() {
                    panic!("NewRouteInsertion should only be used on empty routes");
                }
            }
            Insertion::ExistingRoute(ExistingRouteInsertion { route_id, .. }) => {
                if solution.route(*route_id).is_empty() {
                    panic!("ExistingRouteInsertion shouldn't be used on empty routes");
                }
            }
        };

        let context = compute_insertion_context(self.problem, solution, insertion);
        compute_insertion_score(self.constraints, &context)
            + self.noise_generator.map_or(Score::ZERO, |noise_generator| {
                Score::soft(noise_generator.create_noise(context.insertion.service_id()))
            })
    }
}
