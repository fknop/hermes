use std::fmt::Display;

use serde::Serialize;

use crate::solver::working_solution::WorkingSolution;

use super::{
    ruin_cluster::RuinCluster, ruin_context::RuinContext, ruin_radial::RuinRadial,
    ruin_random::RuinRandom, ruin_route::RuinRoute, ruin_solution::RuinSolution,
    ruin_string::RuinString, ruin_time_related::RuinTimeRelated, ruin_worst::RuinWorst,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
pub enum RuinStrategy {
    Random,
    RuinRadial,
    RuinWorst,
    RuinString,
    RuinTimeRelated,
    RuinCluster,
    RuinRoute,
}

impl Display for RuinStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Random => write!(f, "Random"),
            Self::RuinRadial => write!(f, "RuinRadial"),
            Self::RuinWorst => write!(f, "RuinWorst"),
            Self::RuinString => write!(f, "RuinString"),
            Self::RuinTimeRelated => write!(f, "RuinTimeRelated"),
            Self::RuinCluster => write!(f, "RuinCluster"),
            Self::RuinRoute => write!(f, "RuinRoute"),
        }
    }
}

impl RuinSolution for RuinStrategy {
    fn ruin_solution(&self, solution: &mut WorkingSolution, context: RuinContext) {
        match self {
            RuinStrategy::Random => {
                let strategy = RuinRandom;
                strategy.ruin_solution(solution, context);
            }
            RuinStrategy::RuinRadial => {
                let strategy = RuinRadial;
                strategy.ruin_solution(solution, context);
            }
            RuinStrategy::RuinWorst => {
                let strategy = RuinWorst;
                strategy.ruin_solution(solution, context);
            }
            RuinStrategy::RuinString => {
                let strategy = RuinString::default();
                strategy.ruin_solution(solution, context);
            }
            RuinStrategy::RuinTimeRelated => {
                let strategy = RuinTimeRelated;
                strategy.ruin_solution(solution, context);
            }
            RuinStrategy::RuinCluster => {
                let strategy = RuinCluster;
                strategy.ruin_solution(solution, context);
            }
            RuinStrategy::RuinRoute => {
                let strategy = RuinRoute;
                strategy.ruin_solution(solution, context);
            }
        }
    }
}
