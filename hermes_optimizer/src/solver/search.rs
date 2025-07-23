use std::{sync::Arc, thread};

use fxhash::FxHashMap;
use jiff::SignedDuration;
use jiff::Timestamp;
use parking_lot::{MappedRwLockReadGuard, RwLock, RwLockReadGuard};
use rand::{Rng, SeedableRng, rngs::SmallRng, seq::IndexedRandom};
use tracing::info;

use crate::{
    acceptor::{
        accept_solution::{AcceptSolution, AcceptSolutionContext},
        greedy_solution_acceptor::GreedySolutionAcceptor,
        schrimpf_acceptor::SchrimpfAcceptor,
        solution_acceptor::SolutionAcceptor,
    },
    problem::vehicle_routing_problem::VehicleRoutingProblem,
    selector::{
        select_best_selector::SelectBestSelector, select_random_selector::SelectRandomSelector,
        select_solution::SelectSolution, solution_selector::SolutionSelector,
    },
};

use super::{
    accepted_solution::AcceptedSolution,
    constraints::constraint::Constraint,
    construction::construct_solution::construct_solution,
    noise::NoiseGenerator,
    recreate::{
        recreate_context::RecreateContext, recreate_solution::RecreateSolution,
        recreate_strategy::RecreateStrategy,
    },
    ruin::{ruin_context::RuinContext, ruin_solution::RuinSolution, ruin_strategy::RuinStrategy},
    score::{Score, ScoreAnalysis},
    solver_params::{SolverAcceptorStrategy, SolverParams, SolverSelectorStrategy, Threads},
    working_solution::WorkingSolution,
};

pub struct Search {
    problem: Arc<VehicleRoutingProblem>,
    constraints: Vec<Constraint>,
    params: SolverParams,
    best_solutions: Arc<RwLock<Vec<AcceptedSolution>>>,
    solution_selector: SolutionSelector,
    solution_acceptor: SolutionAcceptor,
    on_best_solution_handler: Arc<Option<fn(&AcceptedSolution)>>,
    noise_generator: NoiseGenerator,

    ruin_operators: Arc<RwLock<Vec<RuinOperator>>>,
    recreate_operators: Arc<RwLock<Vec<RecreateOperator>>>,
}

impl Search {
    pub fn new(
        params: SolverParams,
        problem: VehicleRoutingProblem,
        constraints: Vec<Constraint>,
    ) -> Self {
        let solution_selector = match params.solver_selector {
            SolverSelectorStrategy::SelectBest => SolutionSelector::SelectBest(SelectBestSelector),
            SolverSelectorStrategy::SelectRandom => {
                SolutionSelector::SelectRandom(SelectRandomSelector)
            }
        };
        let solution_acceptor = match params.solver_acceptor {
            SolverAcceptorStrategy::Greedy => SolutionAcceptor::Greedy(GreedySolutionAcceptor),
            SolverAcceptorStrategy::Schrimpf => SolutionAcceptor::Schrimpf(SchrimpfAcceptor::new()),
        };

        let max_cost = problem.max_cost();

        Search {
            problem: Arc::new(problem),
            noise_generator: NoiseGenerator::new(
                max_cost,
                params.noise_probability,
                params.noise_level,
            ),
            constraints,
            best_solutions: Arc::new(RwLock::new(Vec::new())),
            solution_selector,
            solution_acceptor,
            on_best_solution_handler: Arc::new(None),
            ruin_operators: Arc::new(RwLock::new(
                params
                    .ruin
                    .ruin_strategies
                    .iter()
                    .map(|&strategy| RuinOperator {
                        strategy,
                        weight: 1.0,
                    })
                    .collect(),
            )),
            recreate_operators: Arc::new(RwLock::new(
                params
                    .recreate
                    .recreate_strategies
                    .iter()
                    .map(|&strategy| RecreateOperator {
                        strategy,
                        weight: 1.0,
                    })
                    .collect(),
            )),
            params,
        }
    }

    pub fn on_best_solution(&mut self, callback: fn(&AcceptedSolution)) {
        self.on_best_solution_handler = Arc::new(Some(callback));
    }

    pub fn best_solution(&self) -> Option<MappedRwLockReadGuard<'_, AcceptedSolution>> {
        RwLockReadGuard::try_map(self.best_solutions.read(), |solutions| solutions.first()).ok()
    }

    fn create_initial_ruin_scores(&self) -> RuinScores {
        let mut scores = FxHashMap::default();
        for operator in self.ruin_operators.read().iter() {
            scores.insert(
                operator.strategy,
                RuinRecreateScoreEntry {
                    score: 0.0,
                    iterations: 0,
                },
            );
        }
        RuinScores { scores }
    }

    fn create_initial_recreate_scores(&self) -> RecreateScores {
        let mut scores = FxHashMap::default();
        for operator in self.recreate_operators.read().iter() {
            scores.insert(
                operator.strategy,
                RuinRecreateScoreEntry {
                    score: 0.0,
                    iterations: 0,
                },
            );
        }
        RecreateScores { scores }
    }

    pub fn run(&self) {
        let mut rng = SmallRng::seed_from_u64(2427121);

        let num_threads = self.number_of_threads();

        info!("Running search on {} threads", num_threads);
        thread::scope(|s| {
            for thread_index in 0..num_threads {
                let best_solutions = Arc::clone(&self.best_solutions);
                // let on_best_solution_handler = Arc::clone(&self.on_best_solution_handler);

                let mut thread_rng = SmallRng::from_rng(&mut rng);
                let max_iterations = self.params.termination_maximum_iterations;

                let builder = thread::Builder::new().name(thread_index.to_string());

                let mut ruin_scores = self.create_initial_ruin_scores();
                let mut recreate_scores = self.create_initial_recreate_scores();
                builder
                    .spawn_scoped(s, move || {
                        let start = Timestamp::now();
                        for iteration in 0..max_iterations {
                            if (iteration + 1) % 5000 == 0 {
                                info!(
                                    thread = thread::current().name().unwrap_or("main"),
                                    ruin_weights = ?self.ruin_operators.read(),
                                    recreate_weights = ?self.recreate_operators.read(),
                                    "Thread {}: Iteration {}/{}",
                                    thread_index,
                                    iteration + 1,
                                    max_iterations
                                );
                            }

                            self.perform_iteration(
                                &mut thread_rng,
                                &best_solutions,
                                &mut ruin_scores,
                                &mut recreate_scores,
                                iteration,
                            );

                            let search_duration = Timestamp::now().duration_since(start);
                            if let Some(termination_maximum_duration) =
                                self.params.termination_maximum_duration
                                && search_duration > termination_maximum_duration
                            {
                                info!(
                                    thread = thread::current().name().unwrap_or("main"),
                                    "Thread {} stopped after {} iterations because the maximum duration was reached.", thread_index, iteration + 1
                                );
                                break;
                            }
                        }
                    })
                    .unwrap();
            }
        });
    }

    fn perform_iteration(
        &self,
        rng: &mut SmallRng,
        best_solutions: &Arc<RwLock<Vec<AcceptedSolution>>>,
        ruin_scores: &mut RuinScores,
        recreate_scores: &mut RecreateScores,
        iteration: usize,
    ) {
        let (mut working_solution, current_score) = {
            let solutions_guard = best_solutions.read();
            if !solutions_guard.is_empty()
                && let Some(AcceptedSolution {
                    solution, score, ..
                }) = self
                    .solution_selector
                    .select_solution(&solutions_guard, rng)
            {
                (solution.clone(), score.clone())
            } else {
                let solution = construct_solution(
                    &self.problem,
                    rng,
                    &self.constraints,
                    &self.noise_generator,
                );
                let (score, _) = self.compute_solution_score(&solution);
                (solution, score)
            }
        }; // Lock is released here

        let ruin_strategy = self.ruin(&mut working_solution, rng);

        let recreate_strategy = self.recreate(&mut working_solution, rng);

        self.update_solutions(
            working_solution,
            best_solutions,
            ruin_scores,
            recreate_scores,
            IterationInfo {
                iteration,
                ruin_strategy,
                recreate_strategy,
                current_score,
            },
        );
    }

    fn update_solutions(
        &self,
        solution: WorkingSolution,
        best_solutions: &Arc<RwLock<Vec<AcceptedSolution>>>,
        ruin_scores: &mut RuinScores,
        recreate_scores: &mut RecreateScores,
        iteration_info: IterationInfo,
    ) {
        let (score, score_analysis) = self.compute_solution_score(&solution);

        let mut guard = best_solutions.upgradable_read();

        if self.solution_acceptor.accept(
            &guard,
            &solution,
            &score,
            AcceptSolutionContext {
                iteration: iteration_info.iteration,
                max_iterations: self.params.termination_maximum_iterations,
                max_solutions: self.params.max_solutions,
            },
        ) {
            guard.with_upgraded(|guard| {
                let is_best = guard.is_empty() || score < guard[0].score;

                // Evict worst
                if guard.len() + 1 > self.params.max_solutions {
                    guard.pop();
                }

                guard.push(AcceptedSolution {
                    solution,
                    score,
                    score_analysis,
                });
                guard.sort_by(|a, b| a.score.cmp(&b.score));

                if is_best {
                    info!(
                        thread = thread::current().name().unwrap_or("main"),
                        "Score: {:?}", guard[0].score_analysis,
                    );
                    info!("Vehicles {:?}", guard[0].solution.routes().len());

                    if let Some(callback) = self.on_best_solution_handler.as_ref() {
                        callback(&guard[0]);
                    }
                }

                // Update the scores
                if is_best {
                    ruin_scores
                        .update_score(iteration_info.ruin_strategy, self.params.alns_best_factor);
                    recreate_scores.update_score(
                        iteration_info.recreate_strategy,
                        self.params.alns_best_factor,
                    );
                } else if score < iteration_info.current_score {
                    ruin_scores.update_score(
                        iteration_info.ruin_strategy,
                        self.params.alns_improvement_factor,
                    );
                    recreate_scores.update_score(
                        iteration_info.recreate_strategy,
                        self.params.alns_best_factor,
                    );
                } else {
                    ruin_scores.update_score(
                        iteration_info.ruin_strategy,
                        self.params.alns_accepted_worst_factor,
                    );
                    recreate_scores.update_score(
                        iteration_info.recreate_strategy,
                        self.params.alns_accepted_worst_factor,
                    );
                }
            })
        } else {
            ruin_scores.update_score(iteration_info.ruin_strategy, 0.0);
            recreate_scores.update_score(iteration_info.recreate_strategy, 0.0);
        }

        if iteration_info.iteration > 0
            && iteration_info.iteration % self.params.alns_segment_iterations == 0
        {
            for operator in self.ruin_operators.write().iter_mut() {
                if let Some(ruin_score) = ruin_scores.scores.get_mut(&operator.strategy) {
                    operator.update_weight(ruin_score, self.params.alns_reaction_factor);
                    ruin_score.reset();
                }
            }

            for operator in self.recreate_operators.write().iter_mut() {
                if let Some(recreate_score) = recreate_scores.scores.get_mut(&operator.strategy) {
                    operator.update_weight(recreate_score, self.params.alns_reaction_factor);
                    recreate_score.reset();
                }
            }
        }
    }

    fn create_num_activities_to_remove(&self, rng: &mut SmallRng) -> usize {
        let ruin_minimum_ratio = self.params.ruin.ruin_minimum_ratio;
        let ruin_maximum_ratio = self.params.ruin.ruin_maximum_ratio;
        let minimum_ruin_size = ((ruin_minimum_ratio * self.problem.services().len() as f64).ceil()
            as usize)
            .max(self.params.ruin.ruin_minimum_size);

        let maximum_ruin_size =
            ((ruin_maximum_ratio * self.problem.services().len() as f64).floor() as usize)
                .min(self.params.ruin.ruin_maximum_size);

        rng.random_range(minimum_ruin_size..=maximum_ruin_size)
    }

    fn ruin(&self, solution: &mut WorkingSolution, rng: &mut SmallRng) -> RuinStrategy {
        let ruin_strategy = self.select_ruin_strategy(rng);
        ruin_strategy.ruin_solution(
            solution,
            RuinContext {
                problem: &self.problem,
                num_activities_to_remove: self.create_num_activities_to_remove(rng),
                rng,
            },
        );

        ruin_strategy
    }

    fn select_ruin_strategy(&self, rng: &mut SmallRng) -> RuinStrategy {
        self.ruin_operators
            .read()
            .choose_weighted(rng, |operator| operator.weight)
            .map(|operator| operator.strategy)
            .expect("No ruin strategy configured on solver")
    }

    fn recreate(&self, solution: &mut WorkingSolution, rng: &mut SmallRng) -> RecreateStrategy {
        let recreate_strategy = self.select_recreate_strategy(rng);
        recreate_strategy.recreate_solution(
            solution,
            RecreateContext {
                rng,
                constraints: &self.constraints,
                noise_generator: &self.noise_generator,
            },
        );

        recreate_strategy
    }

    fn select_recreate_strategy(&self, rng: &mut SmallRng) -> RecreateStrategy {
        self.recreate_operators
            .read()
            .choose_weighted(rng, |operator| operator.weight)
            .map(|operator| operator.strategy)
            .expect("No recreate strategy configured on solver")
    }

    fn compute_solution_score(&self, solution: &WorkingSolution) -> (Score, ScoreAnalysis) {
        let mut score_analysis = ScoreAnalysis::default();

        for constraint in self.constraints.iter() {
            let score = constraint.compute_score(&self.problem, solution);
            score_analysis
                .scores
                .insert(constraint.constraint_name(), score);
        }

        (score_analysis.total_score(), score_analysis)
    }

    fn number_of_threads(&self) -> usize {
        match self.params.threads {
            Threads::Single => 1,
            Threads::Multi(num) => num,
            Threads::Auto => std::thread::available_parallelism().map_or(1, |n| n.get()),
        }
    }
}

struct RuinRecreateScoreEntry {
    pub score: f64,
    pub iterations: usize,
}

impl RuinRecreateScoreEntry {
    pub fn reset(&mut self) {
        self.score = 0.0;
        self.iterations = 0;
    }
}

#[derive(Debug)]
pub struct RuinOperator {
    pub strategy: RuinStrategy,
    pub weight: f64,
}

impl RuinOperator {
    fn update_weight(&mut self, entry: &RuinRecreateScoreEntry, reaction_factor: f64) {
        let new_weight = if entry.iterations == 0 {
            (1.0 - reaction_factor) * self.weight
        } else {
            (1.0 - reaction_factor) * self.weight
                + reaction_factor * (entry.score / entry.iterations as f64)
        };

        self.weight = new_weight.max(0.05);
    }
}

struct RuinScores {
    pub scores: FxHashMap<RuinStrategy, RuinRecreateScoreEntry>,
}

impl RuinScores {
    pub fn update_score(&mut self, strategy: RuinStrategy, score: f64) {
        if let Some(entry) = self.scores.get_mut(&strategy) {
            entry.score += score;
            entry.iterations += 1;
        } else {
            self.scores.insert(
                strategy,
                RuinRecreateScoreEntry {
                    score,
                    iterations: 1,
                },
            );
        }
    }
}

#[derive(Debug)]
pub struct RecreateOperator {
    pub strategy: RecreateStrategy,
    pub weight: f64,
}

impl RecreateOperator {
    fn update_weight(&mut self, entry: &RuinRecreateScoreEntry, reaction_factor: f64) {
        let new_weight = if entry.iterations == 0 {
            (1.0 - reaction_factor) * self.weight
        } else {
            (1.0 - reaction_factor) * self.weight
                + reaction_factor * (entry.score / entry.iterations as f64)
        };

        self.weight = new_weight.max(0.05);
    }
}

struct RecreateScores {
    pub scores: FxHashMap<RecreateStrategy, RuinRecreateScoreEntry>,
}

impl RecreateScores {
    pub fn update_score(&mut self, strategy: RecreateStrategy, score: f64) {
        if let Some(entry) = self.scores.get_mut(&strategy) {
            entry.score += score;
            entry.iterations += 1;
        } else {
            self.scores.insert(
                strategy,
                RuinRecreateScoreEntry {
                    score,
                    iterations: 1,
                },
            );
        }
    }
}

struct IterationInfo {
    pub iteration: usize,
    pub ruin_strategy: RuinStrategy,
    pub recreate_strategy: RecreateStrategy,
    pub current_score: Score,
}
