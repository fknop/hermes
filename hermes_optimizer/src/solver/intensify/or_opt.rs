use crate::solver::{
    intensify::intensify_operator::ComputeDelta, working_solution::WorkingSolution,
};

/// **Intra-Route Or-Opt**
///
/// Moves a consecutive chain of activities of length `count` starting at `from`
/// to a new position `to`.
///
/// ```text
/// BEFORE:
///    ... (A) -> [from -> ... -> end] -> (B) ... (X) -> (Y) ...
///                  ^             ^
///              Start Chain   End Chain
///
/// AFTER:
///    ... (A) -> (B) ... (X) -> [from -> ... -> end] -> (Y) ...
///
/// Effect: Moves a whole cluster of stops to a better location.
/// ```
pub struct OrOptOperator {
    route_id: usize,
    from: usize,
    to: usize,
    count: usize,
}

impl ComputeDelta for OrOptOperator {
    fn compute_delta(&self, solution: &WorkingSolution) -> f64 {
        let problem = solution.problem();
        let route = solution.route(self.route_id);

        let A = route.previous_location_id(problem, self.from);
        let from = route.location_id(problem, self.from);

        let end = route.location_id(problem, self.from + self.count);
        let B = route.next_location_id(problem, self.from + self.count);

        let X = route.location_id(problem, self.to);
        let Y = route.next_location_id(problem, self.to);

        let mut delta = 0.0;

        delta -= problem.travel_cost_or_zero(A, from);
        delta -= problem.travel_cost_or_zero(end, B);
        delta -= problem.travel_cost_or_zero(X, Y);
        delta += problem.travel_cost_or_zero(A, B);
        delta += problem.travel_cost_or_zero(X, from);
        delta += problem.travel_cost_or_zero(end, Y);

        delta
    }
}
