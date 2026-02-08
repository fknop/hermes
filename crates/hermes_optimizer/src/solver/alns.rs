use std::{
    collections::VecDeque,
    sync::{Arc, atomic::AtomicBool},
    thread,
};

use fxhash::FxHashMap;
use jiff::{SignedDuration, Timestamp};
use parking_lot::{MappedRwLockReadGuard, Mutex, RwLock, RwLockReadGuard};
use rand::{Rng, SeedableRng, rngs::SmallRng};
use tracing::{debug, info, warn};

use crate::{
    acceptor::{
        accept_solution::{AcceptSolution, AcceptSolutionContext},
        greedy_solution_acceptor::GreedySolutionAcceptor,
        schrimpf_acceptor::SchrimpfAcceptor,
        simulated_annealing_acceptor::SimulatedAnnealingAcceptor,
        solution_acceptor::SolutionAcceptor,
    },
    problem::vehicle_routing_problem::VehicleRoutingProblem,
    selector::{
        select_best_selector::SelectBestSelector, select_random_selector::SelectRandomSelector,
        select_solution::SelectSolution, select_weighted::SelectWeightedSelector,
        solution_selector::SolutionSelector,
    },
    solver::{
        alns_weights::{AlnsScores, AlnsWeights, UpdateScoreParams},
        constraints::{
            activity_constraint::ActivityConstraintType, capacity_constraint::CapacityConstraint,
            global_constraint::GlobalConstraintType,
            maximum_activities_constraint::MaximumActivitiesConstraint,
            maximum_working_duration_constraint::MaximumWorkingDurationConstraint,
            route_constraint::RouteConstraintType, shift_constraint::ShiftConstraint,
            time_window_constraint::TimeWindowConstraint,
            transport_cost_constraint::TransportCostConstraint,
            vehicle_cost_constraint::VehicleCostConstraint,
            waiting_duration_constraint::WaitingDurationConstraint,
        },
        ls::local_search::LocalSearch,
        noise::NoiseParams,
        score::RUN_SCORE_ASSERTIONS,
        solver_params::SolverParamsDebugOptions,
        statistics::SearchStatisticsIteration,
    },
    timer_debug,
    utils::cancellable_barrier::{CancellableBarrier, WaitResult},
};

use super::{
    accepted_solution::AcceptedSolution,
    constraints::constraint::Constraint,
    construction::construct_solution::construct_solution,
    recreate::{
        recreate_context::RecreateContext, recreate_solution::RecreateSolution,
        recreate_strategy::RecreateStrategy,
    },
    ruin::{ruin_context::RuinContext, ruin_solution::RuinSolution, ruin_strategy::RuinStrategy},
    score::{Score, ScoreAnalysis},
    solution::working_solution::WorkingSolution,
    solver_params::{
        SolverAcceptorStrategy, SolverParams, SolverSelectorStrategy, Termination, Threads,
    },
    statistics::{GlobalStatistics, ScoreEvolutionRow},
};

use super::statistics::{SearchStatistics, ThreadSearchStatistics};

type BestSolutionHandler = Arc<Mutex<dyn FnMut(&AcceptedSolution) + Send + Sync + 'static>>;

pub struct Alns {
    problem: Arc<VehicleRoutingProblem>,
    constraints: Vec<Constraint>,
    params: SolverParams,
    best_solutions: Arc<RwLock<Vec<AcceptedSolution>>>,
    tabu: Arc<RwLock<VecDeque<AcceptedSolution>>>,
    global_alns_ruin_weights: Arc<RwLock<AlnsWeights<RuinStrategy>>>,
    global_alns_recreate_weights: Arc<RwLock<AlnsWeights<RecreateStrategy>>>,
    global_alns_ruin_scores: Arc<RwLock<AlnsScores<RuinStrategy>>>,
    global_alns_recreate_scores: Arc<RwLock<AlnsScores<RecreateStrategy>>>,
    on_best_solution_handler: Option<BestSolutionHandler>,
    is_stopped: Arc<AtomicBool>,
    statistics: Arc<SearchStatistics>,
}

impl Alns {
    pub fn new(params: SolverParams, problem: Arc<VehicleRoutingProblem>) -> Self {
        if params.terminations.is_empty() {
            panic!(
                "At least one termination condition must be specified in the solver parameters."
            );
        }

        Alns {
            problem: Arc::clone(&problem),
            constraints: Self::create_constraints(),
            best_solutions: Arc::new(RwLock::new(Vec::with_capacity(params.max_solutions))),
            global_alns_ruin_weights: Arc::new(RwLock::new(AlnsWeights::new(
                params.ruin_strategies().clone(),
            ))),
            global_alns_recreate_weights: Arc::new(RwLock::new(AlnsWeights::new(
                params.recreate_strategies().clone(),
            ))),
            global_alns_ruin_scores: Arc::new(RwLock::new(AlnsScores::new(
                params.ruin_strategies().clone(),
            ))),
            global_alns_recreate_scores: Arc::new(RwLock::new(AlnsScores::new(
                params.recreate_strategies().clone(),
            ))),

            tabu: Arc::new(RwLock::new(VecDeque::with_capacity(params.tabu_size))),
            on_best_solution_handler: None,

            is_stopped: Arc::new(AtomicBool::new(false)),
            statistics: Arc::new(SearchStatistics::new(
                params.search_threads.number_of_threads(),
            )),
            params,
        }
    }

    pub fn problem(&self) -> &VehicleRoutingProblem {
        &self.problem
    }

    fn set_initial_solution(&self, solution: WorkingSolution) {
        let (score, score_analysis) = solution.compute_solution_score(&self.constraints);
        self.best_solutions.write().push(AcceptedSolution {
            solution,
            score,
            score_analysis,
        });
    }

    fn create_construction_thread_pool(&self) -> rayon::ThreadPool {
        rayon::ThreadPoolBuilder::new()
            .num_threads(
                // No search threads used in construction so we use those
                self.params.search_threads.number_of_threads()
                    * self.params.insertion_threads.number_of_threads(),
            )
            .build()
            .unwrap()
    }

    fn create_insertion_thread_pool(&self) -> rayon::ThreadPool {
        rayon::ThreadPoolBuilder::new()
            .num_threads(self.params.insertion_threads.number_of_threads())
            .build()
            .unwrap()
    }

    fn create_solution_selector(&self) -> SolutionSelector {
        match self.params.solver_selector {
            SolverSelectorStrategy::SelectBest => SolutionSelector::SelectBest(SelectBestSelector),
            SolverSelectorStrategy::SelectRandom => {
                SolutionSelector::SelectRandom(SelectRandomSelector)
            }
            SolverSelectorStrategy::SelectWeighted => {
                SolutionSelector::SelectWeighted(SelectWeightedSelector)
            }
        }
    }

    fn create_solution_acceptor(&self) -> SolutionAcceptor {
        match self.params.solver_acceptor {
            SolverAcceptorStrategy::Greedy => SolutionAcceptor::Greedy(GreedySolutionAcceptor),
            SolverAcceptorStrategy::Schrimpf => {
                let random_walks = 100;

                // Create a random walk search that accepts any solution.
                // Runs for *random_walk* iterations and compute the standard variation of the scores
                // The initial threshold is set to half of the standard variation
                let shrimpf_initial_threshold_search = Self::new(
                    SolverParams {
                        terminations: vec![Termination::Iterations(random_walks)],
                        max_solutions: random_walks,
                        solver_acceptor: SolverAcceptorStrategy::Any,
                        search_threads: Threads::Single,
                        solver_selector: SolverSelectorStrategy::SelectBest,
                        tabu_enabled: false,
                        run_intensify_search: false,
                        intensify_probability: 0.0,
                        ..self.params.clone()
                    },
                    Arc::clone(&self.problem),
                );

                if let Some(best_solution) = self.best_solution() {
                    shrimpf_initial_threshold_search
                        .set_initial_solution(best_solution.solution.clone());
                }

                shrimpf_initial_threshold_search.run();

                let total_score = shrimpf_initial_threshold_search
                    .best_solutions
                    .read()
                    .iter()
                    .map(|accepted_solution| accepted_solution.score.soft_score)
                    .sum::<f64>();
                let mean = total_score / random_walks as f64;

                let variance = shrimpf_initial_threshold_search
                    .best_solutions
                    .read()
                    .iter()
                    .map(|accepted_solution| (accepted_solution.score.soft_score - mean).powf(2.0))
                    .sum::<f64>()
                    / ((random_walks - 1) as f64);

                let std = variance.sqrt();
                let initial_threshold = std / 2.0;

                debug!(
                    "Schrimpf initial: total_score = {total_score}, mean = {mean}, variance = {variance}, std = {std}, initial_threshold = {initial_threshold}",
                );

                SolutionAcceptor::Schrimpf(SchrimpfAcceptor::new(initial_threshold))
            }
            SolverAcceptorStrategy::SimulatedAnnealing => {
                let initial_temperature_search = Self::new(
                    SolverParams {
                        terminations: vec![Termination::Iterations(1)],
                        max_solutions: 1,
                        solver_acceptor: SolverAcceptorStrategy::Any,
                        search_threads: Threads::Single,
                        solver_selector: SolverSelectorStrategy::SelectBest,
                        tabu_enabled: false,
                        run_intensify_search: false,
                        intensify_probability: 0.0,
                        debug_options: SolverParamsDebugOptions {
                            enable_local_search: false,
                        },
                        ..self.params.clone()
                    },
                    Arc::clone(&self.problem),
                );

                if let Some(best_solution) = self.best_solution() {
                    initial_temperature_search.set_initial_solution(best_solution.solution.clone());
                }

                initial_temperature_search.run();
                let soft_score = initial_temperature_search
                    .best_solution()
                    .unwrap()
                    .score
                    .soft_score;

                let w = 0.3;
                let start_temperature = w * soft_score / (0.5_f64.ln().abs());
                SolutionAcceptor::SimulatedAnnealing(SimulatedAnnealingAcceptor::new(
                    start_temperature,
                    0.99999,
                ))
            }
            SolverAcceptorStrategy::Any => SolutionAcceptor::Any,
        }
    }

    fn create_constraints() -> Vec<Constraint> {
        vec![
            // Hard constraints
            Constraint::Route(RouteConstraintType::MaximumJobs(
                MaximumActivitiesConstraint,
            )),
            Constraint::Route(RouteConstraintType::Shift(ShiftConstraint)),
            Constraint::Route(RouteConstraintType::MaximumWorkingDuration(
                MaximumWorkingDurationConstraint,
            )),
            Constraint::Activity(ActivityConstraintType::TimeWindow(
                TimeWindowConstraint::default(),
            )),
            Constraint::Route(RouteConstraintType::Capacity(CapacityConstraint::default())),
            // Soft constraints
            Constraint::Global(GlobalConstraintType::TransportCost(TransportCostConstraint)),
            Constraint::Route(RouteConstraintType::VehicleCost(VehicleCostConstraint)),
            Constraint::Route(RouteConstraintType::WaitingDuration(
                WaitingDurationConstraint,
            )),
        ]
    }

    pub fn on_best_solution<F>(&mut self, callback: F)
    where
        F: FnMut(&AcceptedSolution) + Send + Sync + 'static,
    {
        self.on_best_solution_handler = Some(Arc::new(Mutex::new(callback)));
    }

    pub fn best_solution(&self) -> Option<MappedRwLockReadGuard<'_, AcceptedSolution>> {
        RwLockReadGuard::try_map(self.best_solutions.read(), |solutions| solutions.first()).ok()
    }

    #[cfg(feature = "statistics")]
    pub fn statistics(&self) -> Arc<SearchStatistics> {
        Arc::clone(&self.statistics)
    }

    pub fn weights_cloned(&self) -> (AlnsWeights<RuinStrategy>, AlnsWeights<RecreateStrategy>) {
        (
            self.global_alns_ruin_weights.read().clone(),
            self.global_alns_recreate_weights.read().clone(),
        )
    }

    pub fn stop(&self) {
        self.is_stopped
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }

    fn run_construction(&self, rng: &mut SmallRng) {
        // Solutions already exist, no need to run construction heuristic
        if !self.best_solutions.read().is_empty() {
            return;
        }

        let initial_solution = timer_debug!(
            "Construction",
            construct_solution(
                &self.problem,
                &self.params,
                rng,
                &self.constraints,
                &self.create_construction_thread_pool(),
            )
        );

        let (score, score_analysis) = initial_solution.compute_solution_score(&self.constraints);

        #[cfg(feature = "statistics")]
        {
            self.statistics
                .global_statistics()
                .write()
                .add_best_score(ScoreEvolutionRow {
                    timestamp: Timestamp::now(),
                    score,
                    score_analysis: score_analysis.clone(),
                    thread: 0,
                });
        }

        self.best_solutions.write().push(AcceptedSolution {
            solution: initial_solution,
            score,
            score_analysis,
        });

        if let Some(callback) = &self.on_best_solution_handler {
            callback.lock()(&self.best_solutions.read()[0]);
        }
    }

    pub fn run(&self) {
        self.is_stopped
            .store(false, std::sync::atomic::Ordering::Relaxed);

        let mut rng = SmallRng::seed_from_u64(2427121);
        let start = Timestamp::now();

        self.run_construction(&mut rng);

        if !self.params.debug_options.enable_local_search {
            return;
        }

        let num_threads = self.params.search_threads.number_of_threads();
        // Could just clone this instead of storing in an Arc honestly
        let solution_acceptor = Arc::new(self.create_solution_acceptor());
        let solution_selector = Arc::new(self.create_solution_selector());

        debug!("Running search on {} threads", num_threads);

        let barrier = Arc::new(CancellableBarrier::new(num_threads));

        thread::scope(|s| {
            for thread_index in 0..num_threads {
                let thread_barrier = Arc::clone(&barrier);

                let best_solutions = Arc::clone(&self.best_solutions);
                let tabu = Arc::clone(&self.tabu);

                let global_statistics = Arc::clone(self.statistics.global_statistics());
                let thread_statistics = Arc::clone(self.statistics.thread_statistics(thread_index));

                let solution_acceptor = Arc::clone(&solution_acceptor);
                let solution_selector = Arc::clone(&solution_selector);

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

                builder
                    .spawn_scoped(s, move || {
                        let mut state = ThreadedSearchState {
                            start,
                            thread: thread_index,
                            iteration: 0,
                            iterations_without_improvement: 0,
                            alns_ruin_weights: AlnsWeights::new(
                                self.params.ruin_strategies().clone(),
                            ),
                            alns_recreate_weights: AlnsWeights::new(
                                self.params.recreate_strategies().clone(),
                            ),
                            alns_ruin_scores: AlnsScores::new(
                                self.params.ruin_strategies().clone(),
                            ),
                            alns_recreate_scores: AlnsScores::new(
                                self.params.recreate_strategies().clone(),
                            ),
                            best_solutions,
                            tabu,
                            last_intensify_iteration: None,
                            max_iterations,
                            global_statistics,
                            thread_statistics,
                            insertion_thread_pool: self.create_insertion_thread_pool(),
                            local_search: LocalSearch::new(
                                &self.problem,
                                self.constraints.to_vec(),
                            ),
                            solution_acceptor,
                            solution_selector,
                        };

                        loop {
                            let should_intensify = self.params.run_intensify_search
                                && state.iteration - state.last_intensify_iteration.unwrap_or(0)
                                    > 500;

                            if should_intensify {
                                let best_selector = SelectWeightedSelector;
                                let (
                                    mut working_solution,
                                    current_score,
                                    current_score_analysis,
                                    best_score,
                                    best_unassigned_count,
                                ) = {
                                    let solutions_guard = state.best_solutions.read();
                                    if !solutions_guard.is_empty()
                                        && let Some(AcceptedSolution {
                                            solution,
                                            score,
                                            score_analysis,
                                            ..
                                        }) = best_selector
                                            .select_solution(&solutions_guard, &mut thread_rng)
                                    {
                                        (
                                            solution.clone(),
                                            *score,
                                            score_analysis.clone(),
                                            solutions_guard[0].score,
                                            solutions_guard[0].solution.unassigned_jobs().len(),
                                        )
                                    } else {
                                        panic!("No solutions selected");
                                    }
                                }; // Lock is released here

                                let unassigned_count = working_solution.unassigned_jobs().len();

                                state.insertion_thread_pool.install(|| {
                                    state.local_search.intensify(
                                        &self.problem,
                                        &mut working_solution,
                                        1000,
                                    );
                                });

                                let score =
                                    working_solution.compute_solution_score(&self.constraints);

                                if score.0.is_failure() {
                                    warn!("LocalSearch broke hard constraints");

                                    let analysis = score
                                        .1
                                        .scores
                                        .iter()
                                        .filter(|(_, s)| s.is_failure())
                                        .collect::<FxHashMap<_, _>>();

                                    warn!("{:?}", analysis);
                                }

                                if score.0 > current_score {
                                    warn!(
                                        "Didn't intensify {:?} > {:?} - {} vs {}",
                                        score.0,
                                        current_score,
                                        current_score_analysis
                                            .scores
                                            .get("waiting_duration")
                                            .map(|s| s.soft_score)
                                            .unwrap_or(0.0),
                                        score
                                            .1
                                            .scores
                                            .get("waiting_duration")
                                            .map(|s| s.soft_score)
                                            .unwrap_or(0.0)
                                    );
                                }

                                assert_eq!(
                                    unassigned_count,
                                    working_solution.unassigned_jobs().len()
                                );

                                state.iteration += 1;
                                state.last_intensify_iteration = Some(state.iteration);
                                let iteration_info = IterationInfo::Intensify {
                                    iteration: state.iteration,
                                    current_score,
                                    best_score,
                                    best_unassigned_count,
                                };

                                self.update_solutions(
                                    working_solution,
                                    &mut state,
                                    iteration_info,
                                    &mut thread_rng,
                                );
                            } else {
                                state.iteration += 1;
                                self.run_iteration(&mut state, &mut thread_rng);
                            }

                            if state
                                .iteration
                                .is_multiple_of(self.params.threads_sync_iterations_interval)
                            {
                                // Update accumulated global stats from local stats
                                self.global_alns_ruin_scores
                                    .write()
                                    .accumulate(&mut state.alns_ruin_scores);
                                self.global_alns_recreate_scores
                                    .write()
                                    .accumulate(&mut state.alns_recreate_scores);

                                match thread_barrier.wait() {
                                    WaitResult::Leader => {
                                        debug!("Updating global weights from leader");
                                        // Update global weights
                                        self.global_alns_ruin_weights.write().update_weights(
                                            &mut self.global_alns_ruin_scores.write(),
                                            self.params.alns_reaction_factor,
                                        );

                                        self.global_alns_recreate_weights.write().update_weights(
                                            &mut self.global_alns_recreate_scores.write(),
                                            self.params.alns_reaction_factor,
                                        );
                                    }
                                    WaitResult::Cancelled => {
                                        break;
                                    }
                                    _ => {}
                                }

                                let wait_result = thread_barrier.wait();
                                if wait_result.is_cancelled() {
                                    break;
                                }

                                // Update local weights from global
                                state.alns_ruin_weights =
                                    self.global_alns_ruin_weights.read().clone();

                                state.alns_recreate_weights =
                                    self.global_alns_recreate_weights.read().clone();

                                state.local_search.clear_stale(&self.best_solutions.read());
                            }

                            let is_stopped =
                                self.is_stopped.load(std::sync::atomic::Ordering::Relaxed);
                            let should_terminate = is_stopped || self.should_terminate(&state);
                            if should_terminate {
                                // Make sure other threads stop as well
                                if !is_stopped {
                                    self.stop();
                                }

                                thread_barrier.cancel();
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
                if let Some(best_solution) = state.best_solutions.read().first()
                    && !best_solution.solution.has_unassigned()
                {
                    (best_solution.score * 100.0).round() / 100.0 <= target_score
                } else {
                    false
                }
            }
            Termination::VehiclesAndCosts { vehicles, costs } => {
                if let Some(best_solution) = state.best_solutions.read().first()
                    && !best_solution.solution.has_unassigned()
                {
                    best_solution.solution.total_transport_costs() <= costs
                        && best_solution.solution.non_empty_routes_iter().count() <= vehicles
                } else {
                    false
                }
            }
        }
    }

    fn should_terminate(&self, state: &ThreadedSearchState) -> bool {
        self.params.terminations.iter().any(|termination| {
            if self.check_termination(state, termination) {
                if !matches!(termination, Termination::Iterations(_)) {
                    debug!(
                        thread = thread::current().name().unwrap_or("main"),
                        "Thread {}: Termination condition met: {:?} at iteration {}",
                        thread::current().name().unwrap_or("main"),
                        termination,
                        state.iteration
                    );
                }
                true
            } else {
                false
            }
        })
    }

    fn run_iteration(&self, state: &mut ThreadedSearchState, rng: &mut SmallRng) {
        let (mut working_solution, current_score, best_score, best_unassigned_count) = {
            let solutions_guard = state.best_solutions.read();
            if !solutions_guard.is_empty()
                && let Some(AcceptedSolution {
                    solution, score, ..
                }) = state
                    .solution_selector
                    .select_solution(&solutions_guard, rng)
            {
                (
                    solution.clone(),
                    *score,
                    solutions_guard[0].score,
                    solutions_guard[0].solution.unassigned_jobs().len(),
                )
            } else {
                panic!("No solutions selected");
            }
        }; // Lock is released here

        let (ruin_strategy, recreate_strategy) = self.select_ruin_recreate_strategy(state, rng);

        let now = Timestamp::now();
        self.ruin(&mut working_solution, ruin_strategy, state, rng);

        if !current_score.is_failure() && !self.params.recreate.insert_on_failure {
            let (score, analysis) = working_solution.compute_solution_score(&self.constraints);
            if score.is_failure() {
                tracing::warn!("Ignore iteration due to failing ruin");
                return;
            }

            // tracing::error!(
            //     "Ruin broke hard constraints with strategy {:?} at iteration {}",
            //     ruin_strategy,
            //     state.iteration
            // );
            // tracing::error!("Score: {:?}", analysis);

            // for route in working_solution.non_empty_routes_iter() {
            //     route.dump(&self.problem);
            // }

            // panic!(
            //     "Ruin broke hard constraints with strategy {:?} at iteration {}",
            //     ruin_strategy, state.iteration
            // );
        }

        let ruin_duration = Timestamp::now().duration_since(now);

        let now = Timestamp::now();
        self.recreate(&mut working_solution, recreate_strategy, state, rng);
        let recreate_duration = Timestamp::now().duration_since(now);

        // if rng.random_bool(self.params.intensify_probability) {
        //     state.insertion_thread_pool.install(|| {
        //         state
        //             .local_search
        //             .intensify(self, &self.problem, &mut working_solution, 1000);
        //     });
        // }

        self.update_solutions(
            working_solution,
            state,
            IterationInfo::RuinRecreate {
                iteration: state.iteration,
                ruin_strategy,
                recreate_strategy,
                current_score,
                best_score,
                best_unassigned_count,
                ruin_duration,
                recreate_duration,
            },
            rng,
        );
    }

    fn update_solutions(
        &self,
        solution: WorkingSolution,
        state: &mut ThreadedSearchState,
        iteration_info: IterationInfo,
        rng: &mut SmallRng,
    ) {
        if self.params.tabu_enabled
            && state.iteration > 0
            && state.iteration.is_multiple_of(self.params.tabu_iterations)
        {
            state.tabu.write().clear();
        }

        let (score, score_analysis) = solution.compute_solution_score(&self.constraints);

        if RUN_SCORE_ASSERTIONS && !self.params.recreate.insert_on_failure && score.is_failure() {
            tracing::error!(
                "Solution rejected due to failure score: {:?} {:?}",
                score_analysis,
                iteration_info
            );
            panic!("Bug: score should never fail when insert_on_failure is false")
        }

        let mut guard = state.best_solutions.upgradable_read();

        let is_best = (score < iteration_info.best_score()
            && solution.unassigned_jobs().len() <= iteration_info.best_unassigned_count())
            || solution.unassigned_jobs().len() < iteration_info.best_unassigned_count();

        let improved = score < iteration_info.current_score()
            && solution.unassigned_jobs().len() <= iteration_info.best_unassigned_count();

        if is_best
            || state.solution_acceptor.accept(
                &guard,
                &solution,
                &score,
                AcceptSolutionContext {
                    iteration: state.iteration,
                    max_iterations: state.max_iterations,
                    max_solutions: self.params.max_solutions,
                    rng,
                },
            )
        {
            let is_duplicate = guard.iter().any(|accepted_solution| {
                accepted_solution.score == score
                    && accepted_solution.solution.is_identical(&solution)
            });

            if !is_best {
                state.iterations_without_improvement += 1;
            } else {
                state.iterations_without_improvement = 0;
                #[cfg(feature = "statistics")]
                {
                    state
                        .global_statistics
                        .write()
                        .add_best_score(ScoreEvolutionRow {
                            score,
                            score_analysis: score_analysis.clone(),
                            thread: state.thread,
                            timestamp: Timestamp::now(),
                        });
                }
            }

            #[cfg(feature = "statistics")]
            {
                match iteration_info {
                    IterationInfo::RuinRecreate {
                        current_score,
                        ruin_strategy,
                        recreate_strategy,
                        ruin_duration,
                        recreate_duration,
                        ..
                    } => {
                        state.thread_statistics.write().add_iteration_info(
                            SearchStatisticsIteration::RuinRecreate {
                                timestamp: Timestamp::now(),
                                improved,
                                is_best,
                                recreate_strategy,
                                ruin_strategy,
                                score_before: current_score,
                                score_after: score,
                                ruin_duration,
                                recreate_duration,
                            },
                        );
                    }
                    IterationInfo::Intensify { .. } => {
                        state.thread_statistics.write().add_iteration_info(
                            SearchStatisticsIteration::Intensify {
                                timestamp: Timestamp::now(),
                                improved,
                                is_best,
                            },
                        );
                    }
                }
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
                    if guard.len() + 1 > self.params.max_solutions
                        && let Some(worst_solution) = guard.pop()
                        && self.params.tabu_enabled
                    {
                        let mut guard = state.tabu.write();
                        guard.push_front(worst_solution);
                        if guard.len() > self.params.tabu_size {
                            guard.pop_back();
                        }
                    }

                    if solution.unassigned_jobs().len() < guard[0].solution.unassigned_jobs().len()
                    {
                        guard.retain(|s| {
                            s.solution.unassigned_jobs().len() <= solution.unassigned_jobs().len()
                        });
                    }

                    guard.push(AcceptedSolution {
                        solution,
                        score,
                        score_analysis,
                    });

                    guard.sort_unstable_by(|a, b| {
                        a.solution
                            .unassigned_jobs()
                            .len()
                            .cmp(&b.solution.unassigned_jobs().len())
                            .then(a.score.cmp(&b.score))
                    });

                    if is_best && let Some(callback) = &self.on_best_solution_handler {
                        callback.lock()(&guard[0]);
                    }
                });
            }

            if let Some(strategy) = iteration_info.strategy() {
                state.alns_ruin_scores.update_scores(
                    strategy.0,
                    &self.params,
                    UpdateScoreParams {
                        is_best,
                        improved,
                        accepted: true,
                    },
                );
                state.alns_recreate_scores.update_scores(
                    strategy.1,
                    &self.params,
                    UpdateScoreParams {
                        is_best,
                        improved,
                        accepted: true,
                    },
                );
            }
        } else if let Some(strategy) = iteration_info.strategy() {
            state.alns_ruin_scores.update_scores(
                strategy.0,
                &self.params,
                UpdateScoreParams {
                    is_best: false,
                    improved: false,
                    accepted: false,
                },
            );
            state.alns_recreate_scores.update_scores(
                strategy.1,
                &self.params,
                UpdateScoreParams {
                    is_best: false,
                    improved: false,
                    accepted: false,
                },
            );
        }

        // let segment_size = (self.problem.jobs().len() / 5).clamp(50, 200);

        if state.iteration > 0 {
            if state.iterations_without_improvement > 0
                && state
                    .iterations_without_improvement
                    .is_multiple_of(self.params.alns_iterations_without_improvement_reset)
            {
                state.alns_ruin_weights.reset();
                state.alns_recreate_weights.reset();
                state.alns_ruin_scores.reset();
                state.alns_recreate_scores.reset();
            } else if state
                .iteration
                .is_multiple_of(self.params.alns_segment_iterations)
            {
                state.alns_ruin_weights.update_weights(
                    &mut state.alns_ruin_scores,
                    self.params.alns_reaction_factor,
                );

                state.alns_recreate_weights.update_weights(
                    &mut state.alns_recreate_scores,
                    self.params.alns_reaction_factor,
                );
            }
        }
    }

    fn create_num_jobs_to_remove(&self, _state: &ThreadedSearchState, rng: &mut SmallRng) -> usize {
        // let progress = (state.iteration as f64 / state.max_iterations.unwrap_or(10000) as f64);
        // let stagnation_factor = (state.iterations_without_improvement as f64 / 1000.0).min(1.0);

        // let ruin_minimum_ratio = 0.05 + 0.1 * stagnation_factor;
        // let ruin_maximum_ratio = 0.4 - (0.30 * progress).min(0.30) + 0.2 * stagnation_factor;

        let ruin_minimum_ratio = self.params.ruin.ruin_minimum_ratio;
        let ruin_maximum_ratio = self.params.ruin.ruin_maximum_ratio;

        let minimum_ruin_size =
            (ruin_minimum_ratio * self.problem.jobs().len() as f64).ceil() as usize;
        // .max(self.params.ruin.ruin_minimum_size);

        let maximum_ruin_size =
            (ruin_maximum_ratio * self.problem.jobs().len() as f64).floor() as usize;
        // .min(self.params.ruin.ruin_maximum_size);

        assert!(
            maximum_ruin_size > minimum_ruin_size,
            "{maximum_ruin_size} > {minimum_ruin_size}"
        );
        rng.random_range(minimum_ruin_size..=maximum_ruin_size)
    }

    fn ruin(
        &self,
        solution: &mut WorkingSolution,
        ruin_strategy: RuinStrategy,
        state: &ThreadedSearchState,
        rng: &mut SmallRng,
    ) -> RuinStrategy {
        ruin_strategy.ruin_solution(
            solution,
            RuinContext {
                problem: &self.problem,
                num_jobs_to_remove: self.create_num_jobs_to_remove(state, rng),
                rng,
                params: &self.params.ruin,
            },
        );

        ruin_strategy
    }

    fn recreate(
        &self,
        solution: &mut WorkingSolution,
        recreate_strategy: RecreateStrategy,
        state: &mut ThreadedSearchState,
        rng: &mut SmallRng,
    ) -> RecreateStrategy {
        recreate_strategy.recreate_solution(
            solution,
            RecreateContext {
                rng,
                constraints: &self.constraints,
                noise_params: NoiseParams {
                    max_cost: self.problem.max_cost(),
                    noise_level: self.params.noise_level,
                    noise_probability: self.params.noise_probability,
                },
                problem: &self.problem,
                thread_pool: &state.insertion_thread_pool,
                insert_on_failure: self.params.recreate.insert_on_failure,
            },
        );

        recreate_strategy
    }

    fn select_ruin_recreate_strategy(
        &self,
        state: &ThreadedSearchState,
        rng: &mut SmallRng,
    ) -> (RuinStrategy, RecreateStrategy) {
        let ruin_strategy = state.alns_ruin_weights.select_strategy(rng);
        let recreate_strategy = state.alns_recreate_weights.select_strategy(rng);
        (ruin_strategy, recreate_strategy)
    }
}

#[derive(Debug)]
pub enum IterationInfo {
    RuinRecreate {
        iteration: usize,
        ruin_strategy: RuinStrategy,
        recreate_strategy: RecreateStrategy,
        current_score: Score,
        best_score: Score,
        best_unassigned_count: usize,
        ruin_duration: SignedDuration,
        recreate_duration: SignedDuration,
    },
    Intensify {
        iteration: usize,
        current_score: Score,
        best_score: Score,
        best_unassigned_count: usize,
    },
}

impl IterationInfo {
    fn strategy(&self) -> Option<(RuinStrategy, RecreateStrategy)> {
        match self {
            IterationInfo::RuinRecreate {
                ruin_strategy,
                recreate_strategy,
                ..
            } => Some((*ruin_strategy, *recreate_strategy)),
            _ => None,
        }
    }

    fn best_score(&self) -> Score {
        match self {
            IterationInfo::RuinRecreate { best_score, .. } => *best_score,
            IterationInfo::Intensify { best_score, .. } => *best_score,
        }
    }

    fn best_unassigned_count(&self) -> usize {
        match self {
            IterationInfo::RuinRecreate {
                best_unassigned_count,
                ..
            } => *best_unassigned_count,
            IterationInfo::Intensify {
                best_unassigned_count,
                ..
            } => *best_unassigned_count,
        }
    }

    fn current_score(&self) -> Score {
        match self {
            IterationInfo::RuinRecreate { current_score, .. } => *current_score,
            IterationInfo::Intensify { current_score, .. } => *current_score,
        }
    }
}

struct ThreadedSearchState {
    start: Timestamp,
    thread: usize,
    last_intensify_iteration: Option<usize>,
    iterations_without_improvement: usize,
    alns_ruin_weights: AlnsWeights<RuinStrategy>,
    alns_recreate_weights: AlnsWeights<RecreateStrategy>,
    alns_ruin_scores: AlnsScores<RuinStrategy>,
    alns_recreate_scores: AlnsScores<RecreateStrategy>,
    best_solutions: Arc<RwLock<Vec<AcceptedSolution>>>,
    tabu: Arc<RwLock<VecDeque<AcceptedSolution>>>,
    iteration: usize,
    max_iterations: Option<usize>,
    global_statistics: Arc<RwLock<GlobalStatistics>>,
    thread_statistics: Arc<RwLock<ThreadSearchStatistics>>,
    insertion_thread_pool: rayon::ThreadPool,
    local_search: LocalSearch,
    solution_acceptor: Arc<SolutionAcceptor>,
    solution_selector: Arc<SolutionSelector>,
}
