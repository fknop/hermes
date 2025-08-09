use jiff::SignedDuration;

use super::{
    recreate::recreate_params::RecreateParams, ruin::ruin_params::RuinParams, score::Score,
};

#[derive(Clone, Debug)]
pub struct SolverParams {
    pub terminations: Vec<Termination>,
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
    pub tabu_enabled: bool,
    pub tabu_size: usize,
    pub tabu_iterations: usize,
}

#[derive(Clone, Debug)]
pub enum Termination {
    Duration(SignedDuration),
    Iterations(usize),
    IterationsWithoutImprovement(usize),
    Score(Score),
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
    SelectWeighted,
}

impl Default for SolverParams {
    fn default() -> Self {
        Self {
            terminations: vec![
                Termination::IterationsWithoutImprovement(5000),
                Termination::Iterations(100000),
                Termination::Duration(SignedDuration::from_mins(2)),
            ],
            max_solutions: 30,

            tabu_enabled: false,
            tabu_size: 10,
            tabu_iterations: 1000,
            solver_acceptor: SolverAcceptorStrategy::Schrimpf,
            solver_selector: SolverSelectorStrategy::SelectWeighted,
            ruin: RuinParams::default(),
            recreate: RecreateParams::default(),
            threads: Threads::Multi(8),
            noise_level: 0.15,
            noise_probability: 0.2,

            alns_segment_iterations: 100,
            alns_reaction_factor: 0.8,
            alns_best_factor: 33.0,
            alns_improvement_factor: 20.0,
            alns_accepted_worst_factor: 13.0,
        }
    }
}
