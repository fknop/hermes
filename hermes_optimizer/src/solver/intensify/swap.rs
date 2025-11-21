use crate::solver::{
    intensify::intensify_operator::ComputeDelta, solution::working_solution::WorkingSolution,
};

/// **Intra-Route Swap**
///
/// Exchanges the positions of two activities (`first` and `second`) within the same route.
///
/// ```text
/// BEFORE:
///    ... (A) -> [first] -> (B) ... (X) -> [second] -> (Y) ...
///
/// AFTER:
///    ... (A) -> [second] -> (B) ... (X) -> [first] -> (Y) ...
///
/// Note: Handles cases where 'first' and 'second' are adjacent or distant.
/// ```
pub struct SwapOperator {
    route_id: usize,
    first: usize,
    second: usize,
}

impl ComputeDelta for SwapOperator {
    fn compute_delta(&self, solution: &WorkingSolution) -> f64 {
        let problem = solution.problem();
        let route = solution.route(self.route_id);

        let (first, second) = if self.first < self.second {
            (self.first, self.second)
        } else {
            (self.second, self.first)
        };

        let prev_first_loc = route.previous_location_id(problem, first);
        let first_loc = route.location_id(problem, first);
        let next_first_loc = route.next_location_id(problem, first);

        let prev_second_loc = route.previous_location_id(problem, second);
        let second_loc = route.location_id(problem, second);
        let next_second_loc = route.next_location_id(problem, second);

        let current_cost = problem.travel_cost_or_zero(prev_first_loc, first_loc)
            + problem.travel_cost_or_zero(first_loc, next_first_loc)
            + problem.travel_cost_or_zero(prev_second_loc, second_loc)
            + problem.travel_cost_or_zero(second_loc, next_second_loc);

        let new_cost = problem.travel_cost_or_zero(prev_first_loc, second_loc)
            + problem.travel_cost_or_zero(second_loc, next_first_loc)
            + problem.travel_cost_or_zero(prev_second_loc, first_loc)
            + problem.travel_cost_or_zero(first_loc, next_second_loc);

        new_cost - current_cost
    }
}
