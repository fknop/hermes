use std::{sync::Arc, thread};

use parking_lot::{
    MappedMutexGuard, MappedRwLockReadGuard, Mutex, MutexGuard, RwLock, RwLockReadGuard,
};
use rand::{Rng, SeedableRng, rngs::SmallRng};
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

        Search {
            problem: Arc::new(problem),
            constraints,
            params,
            best_solutions: Arc::new(RwLock::new(Vec::new())),
            solution_selector,
            solution_acceptor,
            on_best_solution_handler: Arc::new(None),
        }
    }

    pub fn on_best_solution(&mut self, callback: fn(&AcceptedSolution)) {
        self.on_best_solution_handler = Arc::new(Some(callback));
    }

    pub fn best_solution(&self) -> Option<MappedRwLockReadGuard<'_, AcceptedSolution>> {
        RwLockReadGuard::try_map(self.best_solutions.read(), |solutions| solutions.first()).ok()
    }

    pub fn run(&self) {
        let mut rng = SmallRng::seed_from_u64(2427121);

        let num_threads = self.number_of_threads();

        info!("Running search on {} threads", num_threads);
        thread::scope(|s| {
            for thread_index in 0..num_threads {
                let best_solutions = Arc::clone(&self.best_solutions);
                let on_best_solution_handler = Arc::clone(&self.on_best_solution_handler);

                let mut thread_rng = SmallRng::from_rng(&mut rng);
                let max_iterations = self.params.max_iterations;

                let builder = thread::Builder::new().name(thread_index.to_string());

                builder
                    .spawn_scoped(s, move || {
                        for iteration in 0..max_iterations {
                            if (iteration + 1) % 5000 == 0 {
                                info!(
                                    thread = thread::current().name().unwrap_or("main"),
                                    "Thread {}: Iteration {}/{}",
                                    thread_index,
                                    iteration + 1,
                                    max_iterations
                                );
                            }

                            self.perform_iteration(&mut thread_rng, &best_solutions, iteration);
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
        iteration: usize,
    ) {
        let mut working_solution = {
            let solutions_guard = best_solutions.read();
            if !solutions_guard.is_empty()
                && let Some(AcceptedSolution { solution, .. }) = self
                    .solution_selector
                    .select_solution(&solutions_guard, rng)
            {
                solution.clone()
            } else {
                construct_solution(&self.problem, rng, &self.constraints)
            }
        }; // Lock is released here

        self.ruin(&mut working_solution, rng);

        self.recreate(&mut working_solution, rng);

        self.store_solution(working_solution, best_solutions, iteration);
    }

    fn store_solution(
        &self,
        solution: WorkingSolution,
        best_solutions: &Arc<RwLock<Vec<AcceptedSolution>>>,
        iteration: usize,
    ) {
        let (score, score_analysis) = self.compute_solution_score(&solution);

        let mut guard = best_solutions.upgradable_read();

        if self.solution_acceptor.accept(
            &guard,
            &solution,
            &score,
            AcceptSolutionContext {
                iteration,
                max_iterations: self.params.max_iterations,
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

                for i in 0..guard.len() - 1 {
                    assert!(guard[i].score <= guard[i + 1].score);
                }

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
            })
        }
    }

    fn ruin(&self, solution: &mut WorkingSolution, rng: &mut SmallRng) {
        let ruin_strategy = self.select_ruin_strategy(rng);
        let ruin_minimum_ratio = self.params.ruin.ruin_minimum_ratio;
        let ruin_maximum_ratio = self.params.ruin.ruin_maximum_ratio;

        let minimum_ruin_size =
            (ruin_minimum_ratio * self.problem.services().len() as f64).ceil() as usize;

        let maximum_ruin_size =
            (ruin_maximum_ratio * self.problem.services().len() as f64).floor() as usize;

        let ruin_size = rng.random_range(minimum_ruin_size..maximum_ruin_size);

        ruin_strategy.ruin_solution(
            solution,
            RuinContext {
                problem: &self.problem,
                rng,
                num_activities_to_remove: ruin_size,
            },
        );
    }

    fn select_ruin_strategy(&self, rng: &mut SmallRng) -> RuinStrategy {
        let total_weight: u64 = self
            .params
            .ruin
            .ruin_strategies
            .iter()
            .map(|strategy| strategy.1)
            .sum();

        let random = rng.random_range(0..total_weight);

        let mut cumulative_weight = 0;
        for (strategy, weight) in &self.params.ruin.ruin_strategies {
            cumulative_weight += weight;

            if random < cumulative_weight {
                return *strategy;
            }
        }

        panic!("No ruin strategy configured on solver");
    }

    fn recreate(&self, solution: &mut WorkingSolution, rng: &mut SmallRng) {
        let recreate_strategy = self.select_recreate_strategy(rng);
        recreate_strategy.recreate_solution(
            solution,
            RecreateContext {
                rng,
                constraints: &self.constraints,
            },
        );
    }

    fn select_recreate_strategy(&self, rng: &mut SmallRng) -> RecreateStrategy {
        let total_weight: u64 = self
            .params
            .recreate
            .recreate_strategies
            .iter()
            .map(|strategy| strategy.1)
            .sum();

        let random = rng.random_range(0..total_weight);
        let mut cumulative_weight = 0;

        for (strategy, weight) in &self.params.recreate.recreate_strategies {
            cumulative_weight += weight;

            if random < cumulative_weight {
                return *strategy;
            }
        }

        panic!("No ruin strategy configured on solver");
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
