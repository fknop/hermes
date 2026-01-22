use jiff::SignedDuration;

use crate::solver::{
    recreate::recreate_strategy::RecreateStrategy, ruin::ruin_strategy::RuinStrategy,
};

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

    pub threads_sync_iterations_interval: usize,

    pub noise_probability: f64,
    pub noise_level: f64,

    pub alns_iterations_without_improvement_reset: usize,
    pub alns_segment_iterations: usize,
    pub alns_reaction_factor: f64,
    pub alns_best_factor: f64,
    pub alns_improvement_factor: f64,
    pub alns_accepted_worst_factor: f64,
    pub tabu_enabled: bool,
    pub tabu_size: usize,
    pub tabu_iterations: usize,

    pub intensify_probability: f64,
    pub run_intensify_search: bool,
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
    SimulatedAnnealing,
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

            tabu_enabled: true,
            tabu_size: 5,
            tabu_iterations: 500,
            solver_acceptor: SolverAcceptorStrategy::Schrimpf,
            solver_selector: SolverSelectorStrategy::SelectWeighted,
            ruin: RuinParams::default(),
            recreate: RecreateParams::default(),
            search_threads: Threads::Multi(1),
            insertion_threads: Threads::Multi(4),
            noise_level: 0.025,
            noise_probability: 0.15,

            alns_iterations_without_improvement_reset: 4000,
            alns_segment_iterations: 50,
            threads_sync_iterations_interval: 250,
            alns_reaction_factor: 0.3,
            alns_best_factor: 33.0,
            alns_improvement_factor: 9.0,
            alns_accepted_worst_factor: 3.0,

            run_intensify_search: true,

            intensify_probability: 1.0,

            debug_options: SolverParamsDebugOptions {
                enable_local_search: true,
            },
        }
    }
}

impl SolverParams {
    pub fn ruin_strategies(&self) -> &Vec<RuinStrategy> {
        &self.ruin.ruin_strategies
    }

    pub fn recreate_strategies(&self) -> &Vec<RecreateStrategy> {
        &self.recreate.recreate_strategies
    }
}
