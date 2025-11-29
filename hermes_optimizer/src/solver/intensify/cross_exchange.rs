use crate::{
    problem::vehicle_routing_problem::VehicleRoutingProblem,
    solver::{
        intensify::intensify_operator::IntensifyOp, solution::working_solution::WorkingSolution,
    },
};

/// **Cross-Exchange**
///
/// Swaps a sub-sequence of activities from Route 1 with a sub-sequence from Route 2.
///
/// ```text
/// BEFORE:
///    R1: ... (A) -> [f_start ... f_end] -> (B) ...
///    R2: ... (X) -> [s_start ... s_end] -> (Y) ...
///
/// AFTER:
///    R1: ... (A) -> [s_start ... s_end] -> (B) ...
///    R2: ... (X) -> [f_start ... f_end] -> (Y) ...
/// ```
pub struct CrossExchangeOperator {
    first_route_id: usize,
    second_route_id: usize,
    first_start: usize,
    first_end: usize,
    second_start: usize,
    second_end: usize,
}

impl IntensifyOp for CrossExchangeOperator {
    fn delta(&self, solution: &WorkingSolution) -> f64 {
        let problem = solution.problem();

        let r1 = solution.route(self.first_route_id);
        let r2 = solution.route(self.second_route_id);

        let previous_first_start = r1.previous_location_id(problem, self.first_start);
        let first_start = r1.location_id(problem, self.first_start);
        let first_end = r1.location_id(problem, self.first_end);
        let next_first_end = r1.next_location_id(problem, self.first_end);

        let previous_second_start = r2.previous_location_id(problem, self.second_start);
        let second_start = r2.location_id(problem, self.second_start);
        let second_end = r2.location_id(problem, self.second_end);
        let next_second_end = r2.next_location_id(problem, self.second_end);

        let mut delta = 0.0;

        // Route 1 cost change
        delta -= problem.travel_cost_or_zero(previous_first_start, first_start);
        delta -= problem.travel_cost_or_zero(first_end, next_first_end);
        delta += problem.travel_cost_or_zero(previous_first_start, second_start);
        delta += problem.travel_cost_or_zero(second_end, next_first_end);

        // Route 2 cost change
        delta -= problem.travel_cost_or_zero(previous_second_start, second_start);
        delta -= problem.travel_cost_or_zero(second_end, next_second_end);
        delta += problem.travel_cost_or_zero(previous_second_start, first_start);
        delta += problem.travel_cost_or_zero(first_end, next_second_end);

        delta
    }

    fn is_valid(&self, solution: &WorkingSolution) -> bool {
        todo!()
    }

    fn apply(&self, problem: &VehicleRoutingProblem, solution: &mut WorkingSolution) {
        todo!()
    }

    fn updated_routes(&self) -> Vec<usize> {
        vec![self.first_route_id, self.second_route_id]
    }
}
