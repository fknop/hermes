use std::{collections::VecDeque, sync::Arc, thread};

use fxhash::FxHashMap;
use jiff::Timestamp;
use parking_lot::{MappedRwLockReadGuard, RwLock, RwLockReadGuard};
use rand::{Rng, SeedableRng, rngs::SmallRng, seq::IndexedRandom};
use tracing::{debug, info};

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
        select_solution::SelectSolution, select_weighted::SelectWeightedSelector,
        solution_selector::SolutionSelector,
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
    solver_params::{
        SolverAcceptorStrategy, SolverParams, SolverSelectorStrategy, Termination, Threads,
    },
    working_solution::WorkingSolution,
};

pub struct Search {
    problem: Arc<VehicleRoutingProblem>,
    constraints: Vec<Constraint>,
    params: SolverParams,
    best_solutions: Arc<RwLock<Vec<AcceptedSolution>>>,
    tabu: Arc<RwLock<VecDeque<AcceptedSolution>>>,
    solution_selector: SolutionSelector,
    solution_acceptor: SolutionAcceptor,
    on_best_solution_handler: Arc<Option<fn(&AcceptedSolution)>>,
    noise_generator: NoiseGenerator,
    operator_weights: Arc<RwLock<OperatorWeights>>,
    is_stopped: Arc<RwLock<bool>>,
}

impl Search {
    pub fn new(
        params: SolverParams,
        problem: VehicleRoutingProblem,
        constraints: Vec<Constraint>,
    ) -> Self {
        if params.terminations.is_empty() {
            panic!(
                "At least one termination condition must be specified in the solver parameters."
            );
        }

        let solution_selector = match params.solver_selector {
            SolverSelectorStrategy::SelectBest => SolutionSelector::SelectBest(SelectBestSelector),
            SolverSelectorStrategy::SelectRandom => {
                SolutionSelector::SelectRandom(SelectRandomSelector)
            }
            SolverSelectorStrategy::SelectWeighted => {
                SolutionSelector::SelectWeighted(SelectWeightedSelector)
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
            best_solutions: Arc::new(RwLock::new(Vec::with_capacity(params.max_solutions))),
            tabu: Arc::new(RwLock::new(VecDeque::with_capacity(params.tabu_size))),
            solution_selector,
            solution_acceptor,
            on_best_solution_handler: Arc::new(None),
            operator_weights: Arc::new(RwLock::new(OperatorWeights::new(&params))),
            params,
            is_stopped: Arc::new(RwLock::new(false)),
        }
    }

    pub fn on_best_solution(&mut self, callback: fn(&AcceptedSolution)) {
        self.on_best_solution_handler = Arc::new(Some(callback));
    }

    pub fn best_solution(&self) -> Option<MappedRwLockReadGuard<'_, AcceptedSolution>> {
        RwLockReadGuard::try_map(self.best_solutions.read(), |solutions| solutions.first()).ok()
    }

    pub fn stop(&self) {
        *self.is_stopped.write() = true;
    }

    pub fn run(&self) {
        let mut rng = SmallRng::seed_from_u64(2427121);

        let num_threads = self.number_of_threads();

        let initial_solution = construct_solution(
            &self.problem,
            &mut rng,
            &self.constraints,
            &self.noise_generator,
        );

        let (score, score_analysis) = self.compute_solution_score(&initial_solution);

        self.best_solutions.write().push(AcceptedSolution {
            solution: initial_solution,
            score,
            score_analysis,
        });

        debug!("Running search on {} threads", num_threads);
        thread::scope(|s| {
            for thread_index in 0..num_threads {
                let best_solutions = Arc::clone(&self.best_solutions);
                let tabu = Arc::clone(&self.tabu);
                let is_stopped = Arc::clone(&self.is_stopped);

                // let on_best_solution_handler = Arc::clone(&self.on_best_solution_handler);

                let max_iterations = self
                    .params
                    .terminations
                    .iter()
                    .find(|termination| matches!(termination, Termination::Iterations(_)))
                    .map(|termination| {
                        if let Termination::Iterations(max_iterations) = termination {
                            *max_iterations
                        } else {
                            0
                        }
                    });

                let mut thread_rng = SmallRng::from_rng(&mut rng);

                let builder = thread::Builder::new().name(thread_index.to_string());

                let operator_scores = OperatorScores::new(&self.params);
                builder
                    .spawn_scoped(s, move || {
                        let mut state = ThreadedSearchState {
                            start: Timestamp::now(),
                            iterations_without_improvement: 0,
                            operator_scores,
                            best_solutions,
                            tabu,
                            iteration: 0,
                            max_iterations,
                        };

                        loop {
                            state.iteration += 1;

                            if (state.iteration) % 1000 == 0 {
                                info!(
                                    thread = thread::current().name().unwrap_or("main"),
                                    weights = ?self.operator_weights.read(),
                                    "Thread {}: Iteration {}/{}",
                                    thread_index,
                                    state.iteration,
                                    max_iterations.map(|max| max.to_string()).unwrap_or(String::from("N/A"))
                                );
                            }

                            self.perform_iteration(&mut state, &mut thread_rng);

                            let should_terminate = *is_stopped.read() || self.should_terminate(&state);
                            if should_terminate {
                                break;
                            }
                        }
                    })
                    .unwrap();
            }
        });
    }

    fn check_termination(&self, state: &ThreadedSearchState, termination: &Termination) -> bool {
        match *termination {
            Termination::Iterations(max_iterations) => state.iteration >= max_iterations,
            Termination::Duration(max_duration) => {
                Timestamp::now().duration_since(state.start) > max_duration
            }
            Termination::IterationsWithoutImprovement(max_iterations_without_improvement) => {
                state.iterations_without_improvement >= max_iterations_without_improvement
            }
            Termination::Score(target_score) => {
                if let Some(best_solution) = state.best_solutions.read().first() {
                    (best_solution.score * 100.0).round() / 100.0 <= target_score
                } else {
                    false
                }
            }
        }
    }

    fn should_terminate(&self, state: &ThreadedSearchState) -> bool {
        self.params.terminations.iter().any(|termination| {
            if self.check_termination(state, termination) {
                debug!(
                    thread = thread::current().name().unwrap_or("main"),
                    "Thread {}: Termination condition met: {:?}",
                    thread::current().name().unwrap_or("main"),
                    termination
                );
                true
            } else {
                false
            }
        })
    }

    fn perform_iteration(&self, state: &mut ThreadedSearchState, rng: &mut SmallRng) {
        let (mut working_solution, current_score) = {
            let solutions_guard = state.best_solutions.read();
            if !solutions_guard.is_empty()
                && let Some(AcceptedSolution {
                    solution, score, ..
                }) = self
                    .solution_selector
                    .select_solution(&solutions_guard, rng)
            {
                (solution.clone(), *score)
            } else {
                panic!("No solutions selected");
            }
        }; // Lock is released here

        let ruin_strategy = self.ruin(&mut working_solution, state, rng);

        let recreate_strategy = self.recreate(&mut working_solution, rng);

        self.update_solutions(
            working_solution,
            state,
            IterationInfo {
                iteration: state.iteration,
                ruin_strategy,
                recreate_strategy,
                current_score,
            },
        );
    }

    fn update_solutions(
        &self,
        solution: WorkingSolution,
        state: &mut ThreadedSearchState,
        iteration_info: IterationInfo,
    ) {
        if self.params.tabu_enabled
            && iteration_info.iteration > 0
            && iteration_info.iteration % self.params.tabu_iterations == 0
        {
            state.tabu.write().clear();
        }

        let (score, score_analysis) = self.compute_solution_score(&solution);

        let mut guard = state.best_solutions.upgradable_read();

        if self.solution_acceptor.accept(
            &guard,
            &solution,
            &score,
            AcceptSolutionContext {
                iteration: iteration_info.iteration,
                max_iterations: state.max_iterations,
                max_solutions: self.params.max_solutions,
            },
        ) {
            let is_duplicate = guard.iter().any(|accepted_solution| {
                accepted_solution.score == score
                    && accepted_solution.solution.is_identical(&solution)
            });

            let is_best = guard.is_empty() || score < guard[0].score;
            if !is_best {
                state.iterations_without_improvement += 1;
            } else {
                state.iterations_without_improvement = 0;
            }

            let is_tabu = self.params.tabu_enabled
                && state.tabu.read().iter().any(|accepted_solution| {
                    accepted_solution.score == score
                        && accepted_solution.solution.is_identical(&solution)
                });

            // Don't store it if it's a duplicate
            if !is_duplicate && !is_tabu {
                guard.with_upgraded(|guard| {
                    // Evict worst
                    if guard.len() + 1 > self.params.max_solutions {
                        if let Some(worst_solution) = guard.pop() {
                            if self.params.tabu_enabled {
                                let mut guard = state.tabu.write();
                                guard.push_front(worst_solution);
                                if guard.len() > self.params.tabu_size {
                                    guard.pop_back();
                                }
                            }
                        }
                    }

                    guard.push(AcceptedSolution {
                        solution,
                        score,
                        score_analysis,
                    });
                    guard.sort_by(|a, b| a.score.cmp(&b.score));

                    if is_best {
                        // info!(
                        //     thread = thread::current().name().unwrap_or("main"),
                        //     "Score: {:?}", guard[0].score_analysis,
                        // );
                        // info!("Vehicles {:?}", guard[0].solution.routes().len());

                        if let Some(callback) = self.on_best_solution_handler.as_ref() {
                            callback(&guard[0]);
                        }
                    }
                });
            }

            // Update the scores
            if is_best {
                state
                    .operator_scores
                    .update_ruin_score(iteration_info.ruin_strategy, self.params.alns_best_factor);
                state.operator_scores.update_recreate_score(
                    iteration_info.recreate_strategy,
                    self.params.alns_best_factor,
                );
            } else if score < iteration_info.current_score {
                state.operator_scores.update_ruin_score(
                    iteration_info.ruin_strategy,
                    self.params.alns_improvement_factor,
                );
                state.operator_scores.update_recreate_score(
                    iteration_info.recreate_strategy,
                    self.params.alns_improvement_factor,
                );
            } else {
                state.operator_scores.update_ruin_score(
                    iteration_info.ruin_strategy,
                    self.params.alns_accepted_worst_factor,
                );
                state.operator_scores.update_recreate_score(
                    iteration_info.recreate_strategy,
                    self.params.alns_accepted_worst_factor,
                );
            }
        } else {
            state.iterations_without_improvement += 1;
            state
                .operator_scores
                .update_ruin_score(iteration_info.ruin_strategy, 0.0);
            state
                .operator_scores
                .update_recreate_score(iteration_info.recreate_strategy, 0.0);
        }

        if iteration_info.iteration > 0
            && iteration_info.iteration % self.params.alns_segment_iterations == 0
        {
            for operator in self.operator_weights.write().ruin.iter_mut() {
                if let Some(ruin_score) = state
                    .operator_scores
                    .ruin_scores
                    .get_mut(&operator.strategy)
                {
                    operator.update_weight(ruin_score, self.params.alns_reaction_factor);
                    ruin_score.reset();
                }
            }

            for operator in self.operator_weights.write().recreate.iter_mut() {
                if let Some(recreate_score) = state
                    .operator_scores
                    .recreate_scores
                    .get_mut(&operator.strategy)
                {
                    operator.update_weight(recreate_score, self.params.alns_reaction_factor);
                    recreate_score.reset();
                }
            }
        }
    }

    fn create_num_activities_to_remove(
        &self,
        state: &ThreadedSearchState,
        rng: &mut SmallRng,
    ) -> usize {
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

    fn ruin(
        &self,
        solution: &mut WorkingSolution,
        state: &ThreadedSearchState,
        rng: &mut SmallRng,
    ) -> RuinStrategy {
        let ruin_strategy = self.select_ruin_strategy(rng);
        ruin_strategy.ruin_solution(
            solution,
            RuinContext {
                problem: &self.problem,
                num_activities_to_remove: self.create_num_activities_to_remove(state, rng),
                rng,
            },
        );

        ruin_strategy
    }

    fn select_ruin_strategy(&self, rng: &mut SmallRng) -> RuinStrategy {
        self.operator_weights.read().select_ruin_strategy(rng)
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
        self.operator_weights.read().select_recreate_strategy(rng)
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

#[derive(Debug)]
struct OperatorWeights {
    ruin: Vec<Operator<RuinStrategy>>,
    recreate: Vec<Operator<RecreateStrategy>>,
}

impl OperatorWeights {
    fn new(params: &SolverParams) -> Self {
        let ruin = params
            .ruin
            .ruin_strategies
            .iter()
            .map(|&strategy| Operator {
                strategy,
                weight: 1.0,
            })
            .collect();

        let recreate = params
            .recreate
            .recreate_strategies
            .iter()
            .map(|&strategy| Operator {
                strategy,
                weight: 1.0,
            })
            .collect();

        OperatorWeights { ruin, recreate }
    }

    fn select_ruin_strategy(&self, rng: &mut SmallRng) -> RuinStrategy {
        self.ruin
            .choose_weighted(rng, |operator| operator.weight)
            .map(|operator| operator.strategy)
            .expect("No ruin strategy configured on solver")
    }

    fn select_recreate_strategy(&self, rng: &mut SmallRng) -> RecreateStrategy {
        self.recreate
            .choose_weighted(rng, |operator| operator.weight)
            .map(|operator| operator.strategy)
            .expect("No recreate strategy configured on solver")
    }

    fn reset(&mut self) {
        for operator in self.ruin.iter_mut() {
            operator.reset();
        }
        for operator in self.recreate.iter_mut() {
            operator.reset();
        }
    }
}

struct ScoreEntry {
    pub score: f64,
    pub iterations: usize,
}

impl ScoreEntry {
    pub fn reset(&mut self) {
        self.score = 0.0;
        self.iterations = 0;
    }
}

#[derive(Debug)]
pub struct Operator<T> {
    pub strategy: T,
    pub weight: f64,
}

impl<T> Operator<T> {
    fn update_weight(&mut self, entry: &ScoreEntry, reaction_factor: f64) {
        let new_weight = if entry.iterations == 0 {
            (1.0 - reaction_factor) * self.weight
        } else {
            (1.0 - reaction_factor) * self.weight
                + reaction_factor * (entry.score / entry.iterations as f64)
        };

        self.weight = new_weight.max(0.1);
    }

    fn reset(&mut self) {
        self.weight = 1.0;
    }
}

struct OperatorScores {
    ruin_scores: FxHashMap<RuinStrategy, ScoreEntry>,
    recreate_scores: FxHashMap<RecreateStrategy, ScoreEntry>,
}

impl OperatorScores {
    pub fn new(params: &SolverParams) -> Self {
        let ruin_scores = params
            .ruin
            .ruin_strategies
            .iter()
            .map(|&strategy| {
                (
                    strategy,
                    ScoreEntry {
                        score: 0.0,
                        iterations: 0,
                    },
                )
            })
            .collect();

        let recreate_scores = params
            .recreate
            .recreate_strategies
            .iter()
            .map(|&strategy| {
                (
                    strategy,
                    ScoreEntry {
                        score: 0.0,
                        iterations: 0,
                    },
                )
            })
            .collect();

        OperatorScores {
            ruin_scores,
            recreate_scores,
        }
    }

    pub fn update_ruin_score(&mut self, strategy: RuinStrategy, score: f64) {
        if let Some(entry) = self.ruin_scores.get_mut(&strategy) {
            entry.score += score;
            entry.iterations += 1;
        } else {
            self.ruin_scores.insert(
                strategy,
                ScoreEntry {
                    score,
                    iterations: 1,
                },
            );
        }
    }

    pub fn update_recreate_score(&mut self, strategy: RecreateStrategy, score: f64) {
        if let Some(entry) = self.recreate_scores.get_mut(&strategy) {
            entry.score += score;
            entry.iterations += 1;
        } else {
            self.recreate_scores.insert(
                strategy,
                ScoreEntry {
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

struct ThreadedSearchState {
    start: Timestamp,
    iterations_without_improvement: usize,
    operator_scores: OperatorScores,
    best_solutions: Arc<RwLock<Vec<AcceptedSolution>>>,
    tabu: Arc<RwLock<VecDeque<AcceptedSolution>>>,
    iteration: usize,
    max_iterations: Option<usize>,
}
