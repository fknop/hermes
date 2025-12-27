use crate::{
    problem::vehicle_routing_problem::VehicleRoutingProblem,
    solver::{
        intensify::{
            cross_exchange::CrossExchangeOperator, inter_relocate::InterRelocateOperator,
            inter_swap::InterSwapOperator, inter_two_opt_star::InterTwoOptStarOperator,
            or_opt::OrOptOperator, relocate::RelocateOperator, swap::SwapOperator,
            two_opt::TwoOptOperator,
        },
        solution::{route_id::RouteId, working_solution::WorkingSolution},
    },
};

pub trait IntensifyOp {
    fn transport_cost_delta(&self, solution: &WorkingSolution) -> f64;
    fn fixed_route_cost_delta(&self, _solution: &WorkingSolution) -> f64 {
        0.0
    }

    fn is_valid(&self, solution: &WorkingSolution) -> bool;
    fn apply(&self, problem: &VehicleRoutingProblem, solution: &mut WorkingSolution);
    fn updated_routes(&self) -> Vec<RouteId>;

    fn delta(&self, solution: &WorkingSolution) -> f64 {
        self.transport_cost_delta(solution) + self.fixed_route_cost_delta(solution)
    }
}

#[derive(Debug)]
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

    InterTwoOptStar(InterTwoOptStarOperator),
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
            IntensifyOperator::InterTwoOptStar { .. } => "Inter-2-Opt*",
        }
    }

    pub fn delta(&self, solution: &WorkingSolution) -> f64 {
        match self {
            IntensifyOperator::TwoOpt(op) => op.transport_cost_delta(solution),
            IntensifyOperator::Relocate(op) => op.transport_cost_delta(solution),
            IntensifyOperator::Swap(op) => op.transport_cost_delta(solution),
            IntensifyOperator::OrOpt(op) => op.transport_cost_delta(solution),
            IntensifyOperator::InterRelocate(op) => op.transport_cost_delta(solution),
            IntensifyOperator::InterSwap(op) => op.transport_cost_delta(solution),
            IntensifyOperator::TwoOptStar(op) => op.transport_cost_delta(solution),
            IntensifyOperator::CrossExchange(op) => op.transport_cost_delta(solution),
            IntensifyOperator::InterTwoOptStar(op) => op.transport_cost_delta(solution),
        }
    }

    pub fn apply(&self, problem: &VehicleRoutingProblem, solution: &mut WorkingSolution) {
        match self {
            IntensifyOperator::TwoOpt(op) => op.apply(problem, solution),
            IntensifyOperator::Relocate(op) => op.apply(problem, solution),
            IntensifyOperator::Swap(op) => op.apply(problem, solution),
            IntensifyOperator::OrOpt(op) => op.apply(problem, solution),
            IntensifyOperator::InterRelocate(op) => op.apply(problem, solution),
            IntensifyOperator::InterSwap(op) => op.apply(problem, solution),
            IntensifyOperator::TwoOptStar(op) => op.apply(problem, solution),
            IntensifyOperator::CrossExchange(op) => op.apply(problem, solution),
            IntensifyOperator::InterTwoOptStar(op) => op.apply(problem, solution),
        }
    }

    pub fn updated_routes(&self) -> Vec<RouteId> {
        match self {
            IntensifyOperator::TwoOpt(op) => op.updated_routes(),
            IntensifyOperator::Relocate(op) => op.updated_routes(),
            IntensifyOperator::Swap(op) => op.updated_routes(),
            IntensifyOperator::OrOpt(op) => op.updated_routes(),
            IntensifyOperator::InterRelocate(op) => op.updated_routes(),
            IntensifyOperator::InterSwap(op) => op.updated_routes(),
            IntensifyOperator::TwoOptStar(op) => op.updated_routes(),
            IntensifyOperator::CrossExchange(op) => op.updated_routes(),
            IntensifyOperator::InterTwoOptStar(op) => op.updated_routes(),
        }
    }
}
