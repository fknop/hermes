use crate::{
    problem::{location::LocationId, vehicle_routing_problem::VehicleRoutingProblem},
    solver::working_solution::WorkingSolution,
};

trait ComputeDelta {
    fn compute_delta(&self, solution: &WorkingSolution) -> f64;
}

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

impl ComputeDelta for TwoOptOperator {
    fn compute_delta(&self, solution: &WorkingSolution) -> f64 {
        let problem = solution.problem();
        let route = solution.route(self.route_id);

        let prev_from = route.previous_location_id(problem, self.from);
        let from = route.location_id(problem, self.from);

        let to = route.location_id(problem, self.to);
        let next_to = route.next_location_id(problem, self.to);

        let current_cost =
            travel_cost(problem, prev_from, from) + travel_cost(problem, to, next_to);
        let new_cost = travel_cost(problem, prev_from, to) + travel_cost(problem, from, next_to);

        new_cost - current_cost
    }
}

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

impl ComputeDelta for RelocateOperator {
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

        let current_cost = travel_cost(problem, prev_from, from)
            + travel_cost(problem, from, next_from)
            + travel_cost(problem, prev_to, next_to);
        let new_cost = travel_cost(problem, prev_from, next_from)
            + travel_cost(problem, prev_to, from)
            + travel_cost(problem, from, next_to);

        new_cost - current_cost
    }
}

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

/// **Inter-Route 2-Opt* (Two-Opt Star)**
///
/// Exchanges the "tails" of two routes.
/// Breaks the edge after `first_from` in R1 and `second_from` in R2, swapping the remainders.
/// Excellent for resolving crossing routes in Euclidean space.
///
/// ```text
/// BEFORE:
///    R1: Start ... (A) --x--> (B) ... End
///                   ^ first_from
///    R2: Start ... (X) --x--> (Y) ... End
///                   ^ second_from
///
/// AFTER:
///    R1: Start ... (A) -----> (Y) ... End (Old R2 Tail)
///    R2: Start ... (X) -----> (B) ... End (Old R1 Tail)
/// ```
pub struct InterTwoOptStarOperator {
    first_route_id: usize,
    second_route_id: usize,
    first_from: usize,
    first_to: usize,
    second_from: usize,
    second_to: usize,
}

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
    first_from: usize,
    first_to: usize,
    second_from: usize,
    second_to: usize,
}

pub enum IntensifyOperator {
    /// 2-Opt operator that reverses the segment between two indices start and end in a given route.
    TwoOpt(TwoOptOperator),
    /// Relocate operator that moves an activity from one position to another within the same route.
    Relocate(RelocateOperator),
    /// Swap operator that exchanges two activities at specified positions within the same route.
    Swap(SwapOperator),
    /// Or-Opt operator that moves a sequence of activities from one position to another within the same route.
    OrOpt(OrOptOperator),

    /// Inter-route Relocate operator that moves an activity from one route to another.
    InterRelocate(InterRelocateOperator),

    /// Inter-route Swap operator that exchanges activities between two different routes.
    InterSwap(InterSwapOperator),

    /// Inter-route 2-Opt* operator that exchanges segments between two different routes.
    TwoOptStar(InterTwoOptStarOperator),

    /// Cross-Exchange operator that exchanges segments of activities between two different routes.
    CrossExchange(CrossExchangeOperator),
}

impl IntensifyOperator {
    /// Returns the name of the intensify operator.
    pub fn operator_name(&self) -> &'static str {
        match self {
            IntensifyOperator::TwoOpt { .. } => "Two-Opt",
            IntensifyOperator::Relocate { .. } => "Relocate",
            IntensifyOperator::Swap { .. } => "Swap",
            IntensifyOperator::OrOpt { .. } => "Or-Opt",
            IntensifyOperator::InterRelocate { .. } => "Inter-Relocate",
            IntensifyOperator::InterSwap { .. } => "Inter-Swap",
            IntensifyOperator::TwoOptStar { .. } => "Two-Opt*",
            IntensifyOperator::CrossExchange { .. } => "Cross-Exchange",
        }
    }

    pub fn compute_delta(&self, solution: &WorkingSolution) -> f64 {
        match self {
            IntensifyOperator::TwoOpt(op) => op.compute_delta(solution),
            IntensifyOperator::Relocate(op) => op.compute_delta(solution),
            _ => unimplemented!(),
        }
    }
}

fn travel_cost(
    problem: &VehicleRoutingProblem,
    from: Option<LocationId>,
    to: Option<LocationId>,
) -> f64 {
    if let (Some(from), Some(to)) = (from, to) {
        problem.travel_cost(from, to)
    } else {
        0.0
    }
}
