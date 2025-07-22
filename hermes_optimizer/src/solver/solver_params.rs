use jiff::SignedDuration;

use super::{recreate::recreate_params::RecreateParams, ruin::ruin_params::RuinParams};

#[derive(Clone, Debug)]
pub struct SolverParams {
    pub termination_maximum_iterations: usize,
    pub termination_maximum_duration: Option<SignedDuration>,
    pub solver_acceptor: SolverAcceptorStrategy,
    pub solver_selector: SolverSelectorStrategy,

    pub max_solutions: usize,

    pub ruin: RuinParams,
    pub recreate: RecreateParams,
    pub threads: Threads,

    pub noise_probability: f64,
    pub noise_level: f64,

    pub alns_segment_iterations: usize,
    pub alns_reaction_factor: f64,
    pub alns_best_factor: f64,
    pub alns_improvement_factor: f64,
    pub alns_accepted_worst_factor: f64,
}

#[derive(Clone, Debug)]
pub enum Threads {
    Single,
    Auto,
    Multi(usize),
}

#[derive(Clone, Debug)]
pub enum SolverAcceptorStrategy {
    Greedy,
    Schrimpf,
}

#[derive(Clone, Debug)]
pub enum SolverSelectorStrategy {
    SelectBest,
    SelectRandom,
}

impl Default for SolverParams {
    fn default() -> Self {
        Self {
            termination_maximum_iterations: 100000,
            termination_maximum_duration: Some(SignedDuration::from_mins(2)),
            max_solutions: 30,
            solver_acceptor: SolverAcceptorStrategy::Schrimpf,
            solver_selector: SolverSelectorStrategy::SelectRandom,
            ruin: RuinParams::default(),
            recreate: RecreateParams::default(),
            threads: Threads::Auto,
            noise_level: 0.1,
            noise_probability: 0.0,

            alns_segment_iterations: 100,
            alns_reaction_factor: 0.7,
            alns_best_factor: 33.0,
            alns_improvement_factor: 9.0,
            alns_accepted_worst_factor: 13.0,
        }
    }
}
