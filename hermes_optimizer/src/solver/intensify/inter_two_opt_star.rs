use crate::{
    problem,
    solver::{
        intensify::intensify_operator::ComputeDelta, solution::working_solution::WorkingSolution,
    },
};

/// **Inter-Route 2-Opt* (Two-Opt Star)**
///
/// Exchanges the **tails** (remaining activities) of two different routes.
///
/// This operator is designed to fix "crossing" routes. If Route 1 and Route 2
/// cross over each other in an 'X' shape, this operator cuts the 'X' at the intersection
/// and reconnects the routes to be parallel, swapping their destinations.
///
/// # Mechanism
/// 1. **Cut R1** after activity `first_from`.
///    - `R1_Head` = Start ... `first_from`
///    - `R1_Tail` = `first_from_next` ... End
/// 2. **Cut R2** after activity `second_from`.
///    - `R2_Head` = Start ... `second_from`
///    - `R2_Tail` = `second_from_next` ... End
/// 3. **Swap:** Connect `R1_Head` -> `R2_Tail` and `R2_Head` -> `R1_Tail`.
///
/// ```text
/// BEFORE (Routes Cross):
///    R1: [Head A] --x--> [Tail A]
///                    \ /
///                     X  <-- Crossing point
///                    / \
///    R2: [Head B] --x--> [Tail B]
///
/// AFTER (Routes Uncrossed):
///    R1: [Head A] -----> [Tail B]  (New Combination)
///
///    R2: [Head B] -----> [Tail A]  (New Combination)
/// ```
///
/// **Note:** Unlike standard 2-Opt, this usually **preserves the direction** of the tails
/// (i.e., it does not reverse the order of activities within the tail).
pub struct InterTwoOptStarOperator {
    first_route_id: usize,
    second_route_id: usize,
    first_from: usize,
    second_from: usize,
}

impl ComputeDelta for InterTwoOptStarOperator {
    fn compute_delta(&self, solution: &WorkingSolution) -> f64 {
        let problem = solution.problem();
        let r1 = solution.route(self.first_route_id);
        let r2 = solution.route(self.second_route_id);
        let first_from = r1.location_id(problem, self.first_from);
        let first_from_next = r1.next_location_id(problem, self.first_from);

        let second_from = r2.location_id(problem, self.second_from);
        let second_from_next = r2.next_location_id(problem, self.second_from);

        let mut delta = 0.0;

        // Remove edges: (first_from -> first_from_next), (second_from -> second_from_next)
        delta -= problem.travel_cost_or_zero(first_from, first_from_next);
        delta -= problem.travel_cost_or_zero(second_from, second_from_next);

        // Add edges: (first_from -> second_from_next), (second_from -> first_from_next)
        delta += problem.travel_cost_or_zero(first_from, second_from_next);
        delta += problem.travel_cost_or_zero(second_from, first_from_next);

        delta
    }
}
