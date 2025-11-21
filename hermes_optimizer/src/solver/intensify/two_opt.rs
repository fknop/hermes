use crate::solver::{
    intensify::intensify_operator::ComputeDelta, working_solution::WorkingSolution,
};

/// **Intra-Route 2-Opt**
///
/// Reverses the sequence of activities between `from` and `to` (inclusive).
/// This eliminates crossing edges within a single route.
///
/// ```text
/// BEFORE:
///    ... (prev) --x--> [from] -> ... -> [to] --x--> (next) ...
///          ^             ^               ^            ^
///          A             B               C            D
///
/// AFTER (Sequence Reversed):
///    ... (prev) -----> [to] -> ... -> [from] -----> (next) ...
///          ^             ^               ^            ^
///          A             C               B            D
///
/// Edges Removed: (prev->from), (to->next)
/// Edges Added:   (prev->to),   (from->next)
/// ```
pub struct TwoOptOperator {
    route_id: usize,
    from: usize,
    to: usize,
}

impl TwoOptOperator {
    fn symmetric_delta(&self, solution: &WorkingSolution) -> f64 {
        let problem = solution.problem();
        let route = solution.route(self.route_id);

        let prev_from = route.previous_location_id(problem, self.from);
        let from = route.location_id(problem, self.from);

        let to = route.location_id(problem, self.to);
        let next_to = route.next_location_id(problem, self.to);

        let current_cost =
            problem.travel_cost_or_zero(prev_from, from) + problem.travel_cost_or_zero(to, next_to);

        let new_cost =
            problem.travel_cost_or_zero(prev_from, to) + problem.travel_cost_or_zero(from, next_to);

        new_cost - current_cost
    }

    fn asymmetric_delta(&self, solution: &WorkingSolution) -> f64 {
        let problem = solution.problem();
        let route = solution.route(self.route_id);

        if self.from >= self.to {
            panic!("TwoOpt: cannot have from >= to")
        }

        let mut delta = self.symmetric_delta(solution);

        // Chain reversal
        for i in self.from..self.to {
            let u = route.location_id(problem, i);
            let v = route.location_id(problem, i + 1);

            // Subtract cost of U -> V
            delta -= problem.travel_cost_or_zero(u, v);
            // Add cost of V -> U
            delta += problem.travel_cost_or_zero(v, u);
        }

        delta
    }
}

impl ComputeDelta for TwoOptOperator {
    fn compute_delta(&self, solution: &WorkingSolution) -> f64 {
        if solution.problem().is_symmetric() {
            self.symmetric_delta(solution)
        } else {
            self.asymmetric_delta(solution)
        }
    }
}
