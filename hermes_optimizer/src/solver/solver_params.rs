use jiff::SignedDuration;

use super::{
    recreate::recreate_params::RecreateParams, ruin::ruin_params::RuinParams, score::Score,
};

#[derive(Clone, Debug)]
pub struct SolverParamsDebugOptions {
    pub enable_local_search: bool,
}

#[derive(Clone, Debug)]
pub struct SolverParams {
    pub terminations: Vec<Termination>,
    pub solver_acceptor: SolverAcceptorStrategy,
    pub solver_selector: SolverSelectorStrategy,

    pub max_solutions: usize,

    pub ruin: RuinParams,
    pub recreate: RecreateParams,

    pub insertion_threads: Threads,
    pub search_threads: Threads,

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

    pub debug_options: SolverParamsDebugOptions,
}

#[derive(Clone, Debug)]
pub enum Termination {
    Duration(SignedDuration),
    Iterations(usize),
    IterationsWithoutImprovement(usize),
    Score(Score),
    VehiclesAndCosts { vehicles: usize, costs: f64 },
}

#[derive(Clone, Debug)]
pub enum Threads {
    Single,
    Auto,
    Multi(usize),
}

impl Threads {
    pub fn number_of_threads(&self) -> usize {
        match self {
            Threads::Single => 1,
            Threads::Multi(num) => *num,
            Threads::Auto => std::thread::available_parallelism().map_or(1, |n| n.get()),
        }
    }
}

#[derive(Clone, Debug)]
pub enum SolverAcceptorStrategy {
    Greedy,
    Schrimpf,
    Any,
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
                Termination::IterationsWithoutImprovement(10000),
                Termination::Iterations(100000),
                Termination::Duration(SignedDuration::from_mins(2)),
            ],
            max_solutions: 10,

            tabu_enabled: false,
            tabu_size: 10,
            tabu_iterations: 1000,
            solver_acceptor: SolverAcceptorStrategy::Schrimpf,
            solver_selector: SolverSelectorStrategy::SelectWeighted,
            ruin: RuinParams::default(),
            recreate: RecreateParams::default(),
            search_threads: Threads::Single,
            insertion_threads: Threads::Multi(4),
            noise_level: 0.15,
            noise_probability: 0.15,

            alns_segment_iterations: 50,
            alns_reaction_factor: 0.8,
            alns_best_factor: 9.0,
            alns_improvement_factor: 6.0,
            alns_accepted_worst_factor: 1.0,

            debug_options: SolverParamsDebugOptions {
                enable_local_search: true,
            },
        }
    }
}
