use crate::solver::{
    intensify::intensify_operator::ComputeDelta, working_solution::WorkingSolution,
};

/// **Inter-Route Swap**
///
/// Exchanges an activity `first` in `first_route_id` with `second` in `second_route_id`.
///
/// ```text
/// BEFORE:
///    R1: ... (A) -> [first] -> (B) ...
///    R2: ... (X) -> [second] -> (Y) ...
///
/// AFTER:
///    R1: ... (A) -> [second] -> (B) ...
///    R2: ... (X) -> [first] -> (Y) ...
/// ```
pub struct InterSwapOperator {
    first_route_id: usize,
    second_route_id: usize,
    first: usize,
    second: usize,
}

impl ComputeDelta for InterSwapOperator {
    fn compute_delta(&self, solution: &WorkingSolution) -> f64 {
        let problem = solution.problem();
        let r1 = solution.route(self.first_route_id);
        let r2 = solution.route(self.second_route_id);

        let first = r1.location_id(problem, self.first);
        let second = r2.location_id(problem, self.second);

        let a = r1.previous_location_id(problem, self.first);
        let b = r1.next_location_id(problem, self.first);

        let x = r2.previous_location_id(problem, self.second);
        let y = r2.next_location_id(problem, self.second);

        let mut delta = 0.0;

        // Route 1 cost change
        delta -= problem.travel_cost_or_zero(a, first);
        delta -= problem.travel_cost_or_zero(first, b);
        delta += problem.travel_cost_or_zero(a, second);
        delta += problem.travel_cost_or_zero(second, b);

        // Route 2 cost change
        delta -= problem.travel_cost_or_zero(x, second);
        delta -= problem.travel_cost_or_zero(second, y);
        delta += problem.travel_cost_or_zero(x, first);
        delta += problem.travel_cost_or_zero(first, y);

        delta
    }
}
