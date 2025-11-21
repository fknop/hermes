use crate::solver::{
    intensify::intensify_operator::ComputeDelta, working_solution::WorkingSolution,
};

/// **Inter-Route Relocate**
///
/// Moves an activity `from` in `from_route_id` to position `to` in `to_route_id`.
/// Crucial for load balancing and route elimination.
///
/// ```text
/// BEFORE:
///    R1: ... (A) -> [from] -> (B) ...
///    R2: ... (X) -> (Y) ...
///
/// AFTER:
///    R1: ... (A) -> (B) ...
///    R2: ... (X) -> [from] -> (Y) ...
/// ```
pub struct InterRelocateOperator {
    from_route_id: usize,
    to_route_id: usize,
    from: usize,
    to: usize,
}

impl ComputeDelta for InterRelocateOperator {
    fn compute_delta(&self, solution: &WorkingSolution) -> f64 {
        let problem = solution.problem();
        let r1 = solution.route(self.from_route_id);
        let r2 = solution.route(self.to_route_id);

        let from = r1.location_id(problem, self.from);
        let a = r1.previous_location_id(problem, self.from);
        let b = r1.next_location_id(problem, self.from);

        let x = r2.location_id(problem, self.to);
        let y = r2.location_id(problem, self.to);

        let mut delta = 0.0;

        delta -= problem.travel_cost_or_zero(a, from);
        delta -= problem.travel_cost_or_zero(from, b);
        delta += problem.travel_cost_or_zero(a, b);

        delta -= problem.travel_cost_or_zero(x, y);
        delta += problem.travel_cost_or_zero(x, from);
        delta += problem.travel_cost_or_zero(from, y);

        delta
    }
}
