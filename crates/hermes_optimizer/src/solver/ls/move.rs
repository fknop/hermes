use crate::{
    problem::vehicle_routing_problem::VehicleRoutingProblem,
    solver::{
        ls::{
            cross_exchange::CrossExchangeOperator, inter_mixed_exchange::InterMixedExchange,
            inter_or_opt::InterOrOptOperator, inter_relocate::InterRelocateOperator,
            inter_reverse_two_opt::InterReverseTwoOptOperator, inter_swap::InterSwapOperator,
            inter_two_opt_star::InterTwoOptStarOperator, mixed_exchange::MixedExchangeOperator,
            or_opt::OrOptOperator, relocate::RelocateOperator, swap::SwapOperator,
            swap_star::SwapStar, two_opt::TwoOptOperator,
        },
        solution::{route_id::RouteIdx, working_solution::WorkingSolution},
    },
};

pub trait LocalSearchOperator: Sized {
    fn generate_moves<C>(
        problem: &VehicleRoutingProblem,
        solution: &WorkingSolution,
        pair: (RouteIdx, RouteIdx),
        consumer: C,
    ) where
        C: FnMut(Self);

    fn waiting_cost_delta(&self, solution: &WorkingSolution) -> f64;
    fn transport_cost_delta(&self, solution: &WorkingSolution) -> f64;
    fn fixed_route_cost_delta(&self, _solution: &WorkingSolution) -> f64;
    fn is_valid(&self, solution: &WorkingSolution) -> bool;
    fn apply(&self, problem: &VehicleRoutingProblem, solution: &mut WorkingSolution);
    fn updated_routes(&self) -> Vec<RouteIdx>;

    fn delta(&self, solution: &WorkingSolution) -> f64 {
        self.transport_cost_delta(solution)
            + self.fixed_route_cost_delta(solution)
            + if solution.problem().has_time_windows() {
                self.waiting_cost_delta(solution)
            } else {
                0.0
            }
    }
}

#[derive(Debug)]
pub enum LocalSearchMove {
    /// 2-Opt operator that reverses the segment between two indices start and end in a given route.
    TwoOpt(TwoOptOperator),
    /// Relocate operator that moves an activity from one position to another within the same route.
    Relocate(RelocateOperator),
    /// Swap operator that exchanges two activities at specified positions within the same route.
    Swap(SwapOperator),

    SwapStar(SwapStar),

    /// Swap one with a segment
    MixedExchange(MixedExchangeOperator),

    /// Swap one with a segment in different routes
    InterMixedExchange(InterMixedExchange),

    /// Or-Opt operator that moves a sequence of activities from one position to another within the same route.
    OrOpt(OrOptOperator),

    InterOrOpt(InterOrOptOperator),

    /// Inter-route Relocate operator that moves an activity from one route to another.
    InterRelocate(InterRelocateOperator),

    /// Inter-route Swap operator that exchanges activities between two different routes.
    InterSwap(InterSwapOperator),

    /// Inter-route 2-Opt* operator that exchanges segments between two different routes.
    TwoOptStar(InterTwoOptStarOperator),

    /// Cross-Exchange operator that exchanges segments of activities between two different routes.
    CrossExchange(CrossExchangeOperator),

    InterTwoOptStar(InterTwoOptStarOperator),

    ReverseTwoOpt(InterReverseTwoOptOperator),
}

impl LocalSearchMove {
    /// Returns the name of the intensify operator.
    pub fn operator_name(&self) -> &'static str {
        match self {
            LocalSearchMove::TwoOpt { .. } => "Two-Opt",
            LocalSearchMove::Relocate { .. } => "Relocate",
            LocalSearchMove::Swap { .. } => "Swap",
            LocalSearchMove::SwapStar { .. } => "SwapStar",
            LocalSearchMove::OrOpt { .. } => "Or-Opt",
            LocalSearchMove::InterOrOpt { .. } => "Inter Or-Opt",
            LocalSearchMove::InterRelocate { .. } => "Inter-Relocate",
            LocalSearchMove::InterSwap { .. } => "Inter-Swap",
            LocalSearchMove::TwoOptStar { .. } => "Two-Opt*",
            LocalSearchMove::CrossExchange { .. } => "Cross-Exchange",
            LocalSearchMove::InterTwoOptStar { .. } => "Inter-2-Opt*",
            LocalSearchMove::MixedExchange { .. } => "Mixed-Exchange",
            LocalSearchMove::InterMixedExchange { .. } => "Inter-Mixed-Exchange",
            LocalSearchMove::ReverseTwoOpt { .. } => "Inter-Reverse-2-Opt",
        }
    }

    pub fn delta(&self, solution: &WorkingSolution) -> f64 {
        match self {
            LocalSearchMove::TwoOpt(op) => op.delta(solution),
            LocalSearchMove::Relocate(op) => op.delta(solution),
            LocalSearchMove::Swap(op) => op.delta(solution),
            LocalSearchMove::SwapStar(op) => op.delta(solution),
            LocalSearchMove::OrOpt(op) => op.delta(solution),
            LocalSearchMove::InterOrOpt(op) => op.delta(solution),
            LocalSearchMove::InterRelocate(op) => op.delta(solution),
            LocalSearchMove::InterSwap(op) => op.delta(solution),
            LocalSearchMove::TwoOptStar(op) => op.delta(solution),
            LocalSearchMove::CrossExchange(op) => op.delta(solution),
            LocalSearchMove::InterTwoOptStar(op) => op.delta(solution),
            LocalSearchMove::MixedExchange(op) => op.delta(solution),
            LocalSearchMove::InterMixedExchange(op) => op.delta(solution),
            LocalSearchMove::ReverseTwoOpt(op) => op.delta(solution),
        }
    }

    pub fn transport_cost_delta(&self, solution: &WorkingSolution) -> f64 {
        match self {
            LocalSearchMove::TwoOpt(op) => op.transport_cost_delta(solution),
            LocalSearchMove::Relocate(op) => op.transport_cost_delta(solution),
            LocalSearchMove::Swap(op) => op.transport_cost_delta(solution),
            LocalSearchMove::SwapStar(op) => op.transport_cost_delta(solution),
            LocalSearchMove::OrOpt(op) => op.transport_cost_delta(solution),
            LocalSearchMove::InterOrOpt(op) => op.transport_cost_delta(solution),
            LocalSearchMove::InterRelocate(op) => op.transport_cost_delta(solution),
            LocalSearchMove::InterSwap(op) => op.transport_cost_delta(solution),
            LocalSearchMove::TwoOptStar(op) => op.transport_cost_delta(solution),
            LocalSearchMove::CrossExchange(op) => op.transport_cost_delta(solution),
            LocalSearchMove::InterTwoOptStar(op) => op.transport_cost_delta(solution),
            LocalSearchMove::MixedExchange(op) => op.transport_cost_delta(solution),
            LocalSearchMove::InterMixedExchange(op) => op.transport_cost_delta(solution),
            LocalSearchMove::ReverseTwoOpt(op) => op.transport_cost_delta(solution),
        }
    }

    pub fn waiting_cost_delta(&self, solution: &WorkingSolution) -> f64 {
        match self {
            LocalSearchMove::TwoOpt(op) => op.waiting_cost_delta(solution),
            LocalSearchMove::Relocate(op) => op.waiting_cost_delta(solution),
            LocalSearchMove::Swap(op) => op.waiting_cost_delta(solution),
            LocalSearchMove::SwapStar(op) => op.waiting_cost_delta(solution),
            LocalSearchMove::OrOpt(op) => op.waiting_cost_delta(solution),
            LocalSearchMove::InterOrOpt(op) => op.waiting_cost_delta(solution),
            LocalSearchMove::InterRelocate(op) => op.waiting_cost_delta(solution),
            LocalSearchMove::InterSwap(op) => op.waiting_cost_delta(solution),
            LocalSearchMove::TwoOptStar(op) => op.waiting_cost_delta(solution),
            LocalSearchMove::CrossExchange(op) => op.waiting_cost_delta(solution),
            LocalSearchMove::InterTwoOptStar(op) => op.waiting_cost_delta(solution),
            LocalSearchMove::MixedExchange(op) => op.waiting_cost_delta(solution),
            LocalSearchMove::InterMixedExchange(op) => op.waiting_cost_delta(solution),
            LocalSearchMove::ReverseTwoOpt(op) => op.waiting_cost_delta(solution),
        }
    }

    pub fn is_valid(&self, solution: &WorkingSolution) -> bool {
        match self {
            LocalSearchMove::TwoOpt(op) => op.is_valid(solution),
            LocalSearchMove::Relocate(op) => op.is_valid(solution),
            LocalSearchMove::Swap(op) => op.is_valid(solution),
            LocalSearchMove::SwapStar(op) => op.is_valid(solution),
            LocalSearchMove::OrOpt(op) => op.is_valid(solution),
            LocalSearchMove::InterOrOpt(op) => op.is_valid(solution),
            LocalSearchMove::InterRelocate(op) => op.is_valid(solution),
            LocalSearchMove::InterSwap(op) => op.is_valid(solution),
            LocalSearchMove::TwoOptStar(op) => op.is_valid(solution),
            LocalSearchMove::CrossExchange(op) => op.is_valid(solution),
            LocalSearchMove::InterTwoOptStar(op) => op.is_valid(solution),
            LocalSearchMove::MixedExchange(op) => op.is_valid(solution),
            LocalSearchMove::InterMixedExchange(op) => op.is_valid(solution),
            LocalSearchMove::ReverseTwoOpt(op) => op.is_valid(solution),
        }
    }

    pub fn apply(&self, problem: &VehicleRoutingProblem, solution: &mut WorkingSolution) {
        match self {
            LocalSearchMove::TwoOpt(op) => op.apply(problem, solution),
            LocalSearchMove::Relocate(op) => op.apply(problem, solution),
            LocalSearchMove::Swap(op) => op.apply(problem, solution),
            LocalSearchMove::SwapStar(op) => op.apply(problem, solution),
            LocalSearchMove::OrOpt(op) => op.apply(problem, solution),
            LocalSearchMove::InterOrOpt(op) => op.apply(problem, solution),
            LocalSearchMove::InterRelocate(op) => op.apply(problem, solution),
            LocalSearchMove::InterSwap(op) => op.apply(problem, solution),
            LocalSearchMove::TwoOptStar(op) => op.apply(problem, solution),
            LocalSearchMove::CrossExchange(op) => op.apply(problem, solution),
            LocalSearchMove::InterTwoOptStar(op) => op.apply(problem, solution),
            LocalSearchMove::MixedExchange(op) => op.apply(problem, solution),
            LocalSearchMove::InterMixedExchange(op) => op.apply(problem, solution),
            LocalSearchMove::ReverseTwoOpt(op) => op.apply(problem, solution),
        }
    }

    pub fn updated_routes(&self) -> Vec<RouteIdx> {
        match self {
            LocalSearchMove::TwoOpt(op) => op.updated_routes(),
            LocalSearchMove::Relocate(op) => op.updated_routes(),
            LocalSearchMove::Swap(op) => op.updated_routes(),
            LocalSearchMove::SwapStar(op) => op.updated_routes(),
            LocalSearchMove::OrOpt(op) => op.updated_routes(),
            LocalSearchMove::InterOrOpt(op) => op.updated_routes(),
            LocalSearchMove::InterRelocate(op) => op.updated_routes(),
            LocalSearchMove::InterSwap(op) => op.updated_routes(),
            LocalSearchMove::TwoOptStar(op) => op.updated_routes(),
            LocalSearchMove::CrossExchange(op) => op.updated_routes(),
            LocalSearchMove::InterTwoOptStar(op) => op.updated_routes(),
            LocalSearchMove::MixedExchange(op) => op.updated_routes(),
            LocalSearchMove::InterMixedExchange(op) => op.updated_routes(),
            LocalSearchMove::ReverseTwoOpt(op) => op.updated_routes(),
        }
    }
}
