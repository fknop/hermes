use crate::{
    problem::vehicle_routing_problem::VehicleRoutingProblem,
    solver::{
        intensify::intensify_operator::IntensifyOp, solution::working_solution::WorkingSolution,
    },
};

/// **Intra-Route Relocate**
///
/// Moves a single activity at `from` to a new position at `to`.
/// The node is inserted *at* index `to` (effectively placing it after the node at `to-1`).
///
/// ```text
/// BEFORE:
///    Route: ... (A) -> [from] -> (C) ... (X) -> (Y) ...
///
/// AFTER:
///    Route: ... (A) -> (C) ... (X) -> [from] -> (Y) ...
///                                      ^
///                               Inserted here
///
/// Edges Modified: (A->from), (from->C), (X->Y)
/// Edges Created:  (A->C),    (X->from), (from->Y)
/// ```
pub struct RelocateOperator {
    route_id: usize,
    from: usize,
    to: usize,
}

impl IntensifyOp for RelocateOperator {
    fn compute_delta(&self, solution: &WorkingSolution) -> f64 {
        let problem = solution.problem();
        let route = solution.route(self.route_id);

        let prev_from = route.previous_location_id(problem, self.from);
        let from = route.location_id(problem, self.from);
        let next_from = route.next_location_id(problem, self.from);

        let prev_to = if self.to < self.from {
            route.location_id(problem, self.to.wrapping_sub(1))
        } else {
            route.location_id(problem, self.to)
        };
        let next_to = route.location_id(problem, self.to);

        let current_cost = problem.travel_cost_or_zero(prev_from, from)
            + problem.travel_cost_or_zero(from, next_from)
            + problem.travel_cost_or_zero(prev_to, next_to);
        let new_cost = problem.travel_cost_or_zero(prev_from, next_from)
            + problem.travel_cost_or_zero(prev_to, from)
            + problem.travel_cost_or_zero(from, next_to);

        new_cost - current_cost
    }

    fn is_valid(&self, solution: &WorkingSolution) -> bool {
        todo!()
    }

    fn apply(&self, problem: &VehicleRoutingProblem, solution: &mut WorkingSolution) {
        let route = solution.route_mut(self.route_id);
        route.move_activity(problem, self.from, self.to);
    }
}
