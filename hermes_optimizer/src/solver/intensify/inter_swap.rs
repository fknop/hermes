use crate::{
    problem::vehicle_routing_problem::VehicleRoutingProblem,
    solver::{
        intensify::intensify_operator::IntensifyOp, solution::working_solution::WorkingSolution,
    },
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

impl IntensifyOp for InterSwapOperator {
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

    fn is_valid(&self, solution: &WorkingSolution) -> bool {
        todo!()
    }

    fn apply(&self, problem: &VehicleRoutingProblem, solution: &mut WorkingSolution) {
        if let Some(first_service_id) = solution
            .route_mut(self.first_route_id)
            .remove_activity(problem, self.first)
            && let Some(second_service_id) = solution
                .route_mut(self.second_route_id)
                .remove_activity(problem, self.second)
        {
            solution.route_mut(self.first_route_id).insert_service(
                problem,
                self.first,
                second_service_id,
            );
            solution.route_mut(self.second_route_id).insert_service(
                problem,
                self.second,
                first_service_id,
            );
        }
    }
}
