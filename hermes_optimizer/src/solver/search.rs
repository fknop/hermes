use rand::{Rng, SeedableRng, rngs::SmallRng};

use crate::{
    acceptor::{
        accept_solution::AcceptSolution, greedy_solution_acceptor::GreedySolutionAcceptor,
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
    recreate::{
        recreate_context::RecreateContext, recreate_solution::RecreateSolution,
        recreate_strategy::RecreateStrategy,
    },
    ruin::{ruin_context::RuinContext, ruin_solution::RuinSolution, ruin_strategy::RuinStrategy},
    score::Score,
    solver_params::{SolverAcceptorStrategy, SolverParams, SolverSelectorStrategy},
    working_solution::WorkingSolution,
};

pub struct Search<'a> {
    problem: &'a VehicleRoutingProblem,
    constraints: &'a Vec<Constraint>,
    params: &'a SolverParams,
    best_solutions: Vec<AcceptedSolution<'a>>,
    solution_selector: SolutionSelector,
    solution_acceptor: SolutionAcceptor,
}

impl<'a> Search<'a> {
    pub fn new(
        params: &'a SolverParams,
        problem: &'a VehicleRoutingProblem,
        constraints: &'a Vec<Constraint>,
    ) -> Self {
        let solution_selector = match params.solver_selector {
            SolverSelectorStrategy::SelectBest => SolutionSelector::SelectBest(SelectBestSelector),
            SolverSelectorStrategy::SelectRandom => {
                SolutionSelector::SelectRandom(SelectRandomSelector)
            }
        };
        let solution_acceptor = match params.solver_acceptor {
            SolverAcceptorStrategy::Greedy => SolutionAcceptor::Greedy(GreedySolutionAcceptor),
        };

        Search {
            problem,
            constraints,
            params,
            best_solutions: Vec::new(),
            solution_selector,
            solution_acceptor,
        }
    }

    pub fn best_solutions(&self) -> &[AcceptedSolution] {
        &self.best_solutions
    }

    pub fn run(&mut self) {
        let mut rng = SmallRng::seed_from_u64(123);

        for _ in 0..self.params.max_iterations {
            self.perform_iteration(&mut rng);
        }
    }

    fn perform_iteration(&mut self, rng: &mut SmallRng) {
        let mut working_solution = if !self.best_solutions.is_empty()
            && let Some(AcceptedSolution { solution, .. }) =
                self.solution_selector.select_solution(&self.best_solutions)
        {
            solution.clone()
        } else {
            WorkingSolution::new(self.problem)
        };

        self.ruin(&mut working_solution, rng);
        self.recreate(&mut working_solution, rng);

        self.store_solution(working_solution);
    }

    fn store_solution(&mut self, solution: WorkingSolution<'a>) {
        let score = self.compute_solution_score(&solution);

        if self
            .solution_acceptor
            .accept(&self.best_solutions, &solution, &score)
        {
            self.best_solutions
                .push(AcceptedSolution { solution, score });
            self.best_solutions.sort_by(|a, b| a.score.cmp(&b.score));

            // Evict worst
            if self.best_solutions.len() > self.params.max_solutions {
                self.best_solutions.pop();
            }
        }
    }

    fn ruin(&self, solution: &mut WorkingSolution, rng: &mut SmallRng) {
        let ruin_strategy = self.select_ruin_strategy(rng);
        let ruin_maximum_ratio = self.params.ruin.ruin_maximum_ratio;
        let maximum_ruin_size =
            (ruin_maximum_ratio * self.problem.services().len() as f64).floor() as usize;
        let ruin_size = rng.random_range(0..maximum_ruin_size);

        ruin_strategy.ruin_solution(
            solution,
            RuinContext {
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
        for (strategy, weight) in &self.params.ruin.ruin_strategies {
            if random < *weight {
                return *strategy;
            }
        }

        panic!("No ruin strategy configured on solver");
    }

    fn recreate(&self, solution: &mut WorkingSolution, rng: &mut SmallRng) {
        let recreate_strategy = self.select_recreate_strategy(rng);
        recreate_strategy.recreate_solution(solution, RecreateContext { rng });
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
        for (strategy, weight) in &self.params.recreate.recreate_strategies {
            if random < *weight {
                return *strategy;
            }
        }

        panic!("No ruin strategy configured on solver");
    }

    fn compute_solution_score(&self, solution: &WorkingSolution) -> Score {
        let mut score = Score::zero();

        for constraint in self.constraints.iter() {
            // score += constraint.compute_insertion_score(solution);
        }

        score
    }
}
