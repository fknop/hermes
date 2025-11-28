use crate::{
    problem::vehicle_routing_problem::VehicleRoutingProblem,
    solver::{
        intensify::{
            cross_exchange::CrossExchangeOperator, inter_relocate::InterRelocateOperator,
            inter_swap::InterSwapOperator, inter_two_opt_star::InterTwoOptStarOperator,
            or_opt::OrOptOperator, relocate::RelocateOperator, swap::SwapOperator,
            two_opt::TwoOptOperator,
        },
        solution::working_solution::WorkingSolution,
    },
};

pub trait IntensifyOp {
    fn compute_delta(&self, solution: &WorkingSolution) -> f64;
    fn is_valid(&self, solution: &WorkingSolution) -> bool;
    fn apply(&self, problem: &VehicleRoutingProblem, solution: &mut WorkingSolution);
}

pub trait GenerateIntensifyOperators<T = Self> {
    fn generate_operators(&self, solution: &WorkingSolution) -> Vec<T>;
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
            IntensifyOperator::Swap(op) => op.compute_delta(solution),
            IntensifyOperator::OrOpt(op) => op.compute_delta(solution),
            IntensifyOperator::InterRelocate(op) => op.compute_delta(solution),
            IntensifyOperator::InterSwap(op) => op.compute_delta(solution),
            IntensifyOperator::TwoOptStar(op) => op.compute_delta(solution),
            IntensifyOperator::CrossExchange(op) => op.compute_delta(solution),
            _ => unimplemented!(),
        }
    }
}
