use std::f64;

use fxhash::{FxBuildHasher, FxHashMap, FxHashSet};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use tracing::{debug, info, instrument, warn};

use crate::{
    problem::vehicle_routing_problem::VehicleRoutingProblem,
    solver::{
        constraints::constraint::Constraint,
        ls::{
            cross_exchange::CrossExchangeOperator,
            inter_mixed_exchange::InterMixedExchange,
            inter_or_opt::InterOrOptOperator,
            inter_relocate::InterRelocateOperator,
            inter_reverse_two_opt::InterReverseTwoOptOperator,
            inter_swap::InterSwapOperator,
            inter_two_opt_star::InterTwoOptStarOperator,
            mixed_exchange::MixedExchangeOperator,
            r#move::{LocalSearchMove, LocalSearchOperator},
            or_opt::OrOptOperator,
            relocate::RelocateOperator,
            swap::SwapOperator,
            swap_star::find_best_swap_star_move,
            two_opt::TwoOptOperator,
        },
        score::RUN_SCORE_ASSERTIONS,
        solution::{population::Population, route_id::RouteIdx, working_solution::WorkingSolution},
    },
    utils::enumerate_idx::EnumerateIdx,
};

macro_rules! route_idx_index {
    ($t:ty, $output:ty) => {
        // Temporary VehicleId Index
        impl std::ops::Index<RouteIdx> for $t {
            type Output = $output;
            fn index(&self, index: RouteIdx) -> &Self::Output {
                &self[index.get()]
            }
        }

        // Temporary VehicleId IndexMut
        impl std::ops::IndexMut<RouteIdx> for $t {
            fn index_mut(&mut self, index: RouteIdx) -> &mut Self::Output {
                &mut self[index.get()]
            }
        }
    };
}

route_idx_index!(Vec<f64>, f64);
route_idx_index!(Vec<Vec<f64>>, Vec<f64>);
route_idx_index!(Vec<Option<LocalSearchMove>>, Option<LocalSearchMove>);
route_idx_index!(
    Vec<Vec<Option<LocalSearchMove>>>,
    Vec<Option<LocalSearchMove>>
);

type RoutePair = (RouteIdx, RouteIdx);

pub struct LocalSearch {
    constraints: Vec<Constraint>,
    pairs: Vec<RoutePair>,
    state: LocalSearchState,
}

const MAX_DELTA: f64 = 0.0;

impl LocalSearch {
    pub fn new(problem: &VehicleRoutingProblem, constraints: Vec<Constraint>) -> Self {
        let count = problem.vehicles().len();

        let pairs = Vec::with_capacity(count * count);

        LocalSearch {
            constraints,
            pairs,
            state: LocalSearchState::new(),
        }
    }

    #[instrument(skip_all, level = "debug")]
    pub fn intensify(
        &mut self,
        problem: &VehicleRoutingProblem,
        solution: &mut WorkingSolution,
        iterations: usize,
    ) -> usize {
        self.build_pairs(solution);
        for i in 0..iterations {
            if !self.run_iteration(problem, solution, i + 1) {
                return i + 1;
            }
        }

        iterations
    }

    #[instrument(skip_all, level = "debug")]
    pub fn intensify_route(
        &mut self,
        problem: &VehicleRoutingProblem,
        solution: &mut WorkingSolution,
        route: RouteIdx,
    ) {
        self.pairs.clear();
        self.pairs.push((route, route));
        let mut iteration = 0;
        loop {
            iteration += 1;
            if !self.run_iteration(problem, solution, iteration) {
                break;
            }
        }
    }

    fn run_iteration(
        &mut self,
        problem: &VehicleRoutingProblem,
        solution: &mut WorkingSolution,
        iteration: usize,
    ) -> bool {
        for &(r1, r2) in &self.pairs {
            let v1 = solution.route(r1).version();
            let v2 = solution.route(r2).version();

            assert!(!self.state.contains_key((v1, v2)));
        }

        let results = self
            .pairs
            .par_iter()
            .map(|&(r1, r2)| {
                // Best delta for the pair
                let mut best_delta = self.delta(solution, r1, r2);
                let mut best_move: Option<LocalSearchMove> = None;

                RelocateOperator::generate_moves(problem, solution, (r1, r2), |op| {
                    let delta = op.delta(solution);
                    if delta < best_delta && op.is_valid(solution) {
                        best_delta = delta;
                        best_move = Some(LocalSearchMove::Relocate(op));
                    }
                });

                TwoOptOperator::generate_moves(problem, solution, (r1, r2), |op| {
                    let delta = op.delta(solution);
                    if delta < best_delta && op.is_valid(solution) {
                        best_delta = delta;
                        best_move = Some(LocalSearchMove::TwoOpt(op));
                    }
                });

                OrOptOperator::generate_moves(problem, solution, (r1, r2), |op| {
                    let delta = op.delta(solution);
                    if delta < best_delta && op.is_valid(solution) {
                        best_delta = delta;
                        best_move = Some(LocalSearchMove::OrOpt(op));
                    }
                });

                SwapOperator::generate_moves(problem, solution, (r1, r2), |op| {
                    let delta = op.delta(solution);
                    if delta < best_delta && op.is_valid(solution) {
                        best_delta = delta;
                        best_move = Some(LocalSearchMove::Swap(op));
                    }
                });

                MixedExchangeOperator::generate_moves(problem, solution, (r1, r2), |op| {
                    let delta = op.delta(solution);
                    if delta < best_delta && op.is_valid(solution) {
                        best_delta = delta;
                        best_move = Some(LocalSearchMove::MixedExchange(op));
                    }
                });

                InterSwapOperator::generate_moves(problem, solution, (r1, r2), |op| {
                    let delta = op.delta(solution);
                    if delta < best_delta && op.is_valid(solution) {
                        best_delta = delta;
                        best_move = Some(LocalSearchMove::InterSwap(op));
                    }
                });

                InterMixedExchange::generate_moves(problem, solution, (r1, r2), |op| {
                    let delta = op.delta(solution);
                    if delta < best_delta && op.is_valid(solution) {
                        best_delta = delta;
                        best_move = Some(LocalSearchMove::InterMixedExchange(op));
                    }
                });

                InterRelocateOperator::generate_moves(problem, solution, (r1, r2), |op| {
                    let delta = op.delta(solution);
                    if delta < best_delta && op.is_valid(solution) {
                        best_delta = delta;
                        best_move = Some(LocalSearchMove::InterRelocate(op));
                    }
                });

                InterOrOptOperator::generate_moves(problem, solution, (r1, r2), |op| {
                    let delta = op.delta(solution);
                    if delta < best_delta && op.is_valid(solution) {
                        best_delta = delta;
                        best_move = Some(LocalSearchMove::InterOrOpt(op));
                    }
                });

                CrossExchangeOperator::generate_moves(problem, solution, (r1, r2), |op| {
                    let delta = op.delta(solution);
                    if delta < best_delta && op.is_valid(solution) {
                        best_delta = delta;
                        best_move = Some(LocalSearchMove::CrossExchange(op));
                    }
                });

                InterTwoOptStarOperator::generate_moves(problem, solution, (r1, r2), |op| {
                    let delta = op.delta(solution);
                    if delta < best_delta && op.is_valid(solution) {
                        best_delta = delta;
                        best_move = Some(LocalSearchMove::InterTwoOptStar(op));
                    }
                });

                InterReverseTwoOptOperator::generate_moves(problem, solution, (r1, r2), |op| {
                    let delta = op.delta(solution);
                    if delta < best_delta && op.is_valid(solution) {
                        best_delta = delta;
                        best_move = Some(LocalSearchMove::ReverseTwoOpt(op));
                    }
                });

                if let Some(swap_star) =
                    find_best_swap_star_move(problem, solution, &self.constraints, (r1, r2))
                {
                    let delta = swap_star.delta(solution);
                    if delta < best_delta {
                        best_delta = delta;
                        best_move = Some(LocalSearchMove::SwapStar(swap_star));
                    }
                }

                (r1, r2, best_delta, best_move)
            })
            .collect::<Vec<_>>();

        for (r1, r2, best_delta, best_move) in results {
            if let Some(best_move) = best_move {
                self.state
                    .update_best(solution, r1, r2, best_delta, best_move);
            }
        }

        let mut best_delta = 0.0;
        let mut best_r1 = None;
        let mut best_r2 = None;
        for i in 0..solution.routes().len() {
            for j in 0..solution.routes().len() {
                let delta = self.delta(solution, RouteIdx::new(i), RouteIdx::new(j));
                if delta < best_delta {
                    best_delta = delta;
                    best_r1 = Some(i);
                    best_r2 = Some(j);
                }
            }
        }

        if let (Some(r1), Some(r2)) = (best_r1, best_r2)
            && let Some(op) = self
                .state
                .best_move(solution, RouteIdx::new(r1), RouteIdx::new(r2))
            && best_delta <= -1e-6
        {
            debug!(
                "Apply {} ({}, {}) (d={}) {:?}",
                op.operator_name(),
                r1,
                r2,
                best_delta,
                op
            );

            if RUN_SCORE_ASSERTIONS {
                if !op.is_valid(solution) {
                    tracing::error!(?op, "Operator {} is not valid", op.operator_name());
                    panic!("Stored operator is not valid")
                }

                let d1_before = solution.route(r1.into()).transport_costs(problem);
                let d2_before = solution.route(r2.into()).transport_costs(problem);

                let w1_before = solution.route(r1.into()).total_waiting_duration();
                let w2_before = solution.route(r2.into()).total_waiting_duration();

                let t_delta = op.transport_cost_delta(solution);
                let w_delta = op.waiting_cost_delta(solution);

                // debug!("{:?}", solution.route(r1.into()).activity_ids());
                // debug!("{:?}", solution.route(r2.into()).activity_ids());

                op.apply(problem, solution);

                // debug!("{:?}", solution.route(r1.into()).activity_ids());
                // debug!("{:?}", solution.route(r2.into()).activity_ids());

                let d1_after = solution.route(r1.into()).transport_costs(problem);
                let d2_after = solution.route(r2.into()).transport_costs(problem);

                let w1_after = solution.route(r1.into()).total_waiting_duration();
                let w2_after = solution.route(r2.into()).total_waiting_duration();

                let (d_before, d_after) = if r1 == r2 {
                    (d1_before, d1_after)
                } else {
                    (d1_before + d2_before, d1_after + d2_after)
                };

                let (w_before, w_after) = if r1 == r2 {
                    (
                        problem.waiting_duration_cost(w1_before),
                        problem.waiting_duration_cost(w1_after),
                    )
                } else {
                    (
                        problem.waiting_duration_cost(w1_before + w2_before),
                        problem.waiting_duration_cost(w1_after + w2_after),
                    )
                };

                fn approx_eq(a: f64, b: f64, epsilon: f64) -> bool {
                    (a - b).abs() < epsilon
                }

                assert!(
                    approx_eq(d_before + t_delta, d_after, 1e-9),
                    "Transport cost deviation detected for operator {}, delta does not match the cost after apply. {} {} d={}",
                    op.operator_name(),
                    solution.route(r1.into()).len(),
                    solution.route(r2.into()).len(),
                    t_delta
                );

                assert!(
                    approx_eq(w_before + w_delta, w_after, 1e-9),
                    "Waiting cost deviation detected for operator {}, delta does not match the cost after apply. {} {} d={}",
                    op.operator_name(),
                    solution.route(r1.into()).len(),
                    solution.route(r2.into()).len(),
                    w_delta
                );

                let score = solution.compute_solution_score(&self.constraints);

                if score.0.is_infeasible() {
                    tracing::error!(
                        ?op,
                        "Operator {} broke constraints {:?}",
                        op.operator_name(),
                        score.1
                    );
                    panic!("Score failed after applying operation")
                }
            } else {
                op.apply(problem, solution);
            }

            self.pairs.clear();

            let updated_routes = op.updated_routes();

            for i in 0..solution.routes().len() {
                for &updated_route in &updated_routes {
                    self.pairs.push((RouteIdx::new(i), updated_route));
                    if i != updated_route.get() {
                        self.pairs.push((updated_route, RouteIdx::new(i)));
                    }
                }
            }

            true
        } else {
            false
        }
    }

    pub fn clear_stale(&mut self, population: &Population) {
        self.state.clear_stale(population);
    }

    fn delta(&self, solution: &WorkingSolution, r1: RouteIdx, r2: RouteIdx) -> f64 {
        self.state.delta(solution, r1, r2)
    }

    fn is_best_delta(
        &self,
        solution: &WorkingSolution,
        operator: &impl LocalSearchOperator,
        best: f64,
    ) -> bool {
        let mut delta = operator.fixed_route_cost_delta(solution);

        if delta < best {
            return true;
        }

        delta += operator.transport_cost_delta(solution);

        if delta < best {
            return true;
        }

        delta += operator.waiting_cost_delta(solution);

        delta < best
    }

    fn build_pairs(&mut self, solution: &WorkingSolution) {
        self.pairs.clear();
        let max = solution.routes().len().pow(2);

        for (i, r1) in solution.routes().iter().enumerate_idx() {
            for (j, r2) in solution.routes().iter().enumerate_idx() {
                let v1 = r1.version();
                let v2 = r2.version();
                if !self.state.contains_key((v1, v2)) {
                    self.pairs.push((i, j))
                }
            }
        }

        debug!(
            "Local Search: Built {} route pairs (max {}). Cache ratio: {}",
            self.pairs.len(),
            max,
            (max - self.pairs.len()) as f64 / max as f64
        );
    }
}

type VersionPair = (usize, usize);
struct LocalSearchState(FxHashMap<VersionPair, (f64, LocalSearchMove)>);

impl LocalSearchState {
    fn new() -> Self {
        Self(FxHashMap::with_capacity_and_hasher(
            2000,
            FxBuildHasher::default(),
        ))
    }

    fn contains_key(&self, versions: (usize, usize)) -> bool {
        self.0.contains_key(&versions)
    }

    fn delta(&self, solution: &WorkingSolution, r1: RouteIdx, r2: RouteIdx) -> f64 {
        let route_a = solution.route(r1);
        let route_b = solution.route(r2);
        self.0
            .get(&(route_a.version(), route_b.version()))
            .map(|entry| entry.0)
            .unwrap_or(MAX_DELTA)
    }

    fn best_move(
        &self,
        solution: &WorkingSolution,
        r1: RouteIdx,
        r2: RouteIdx,
    ) -> Option<&LocalSearchMove> {
        let route_a = solution.route(r1);
        let route_b = solution.route(r2);
        self.0
            .get(&(route_a.version(), route_b.version()))
            .map(|entry| &entry.1)
    }

    fn update_best(
        &mut self,
        solution: &WorkingSolution,
        r1: RouteIdx,
        r2: RouteIdx,
        delta: f64,
        best_move: LocalSearchMove,
    ) {
        let route_a = solution.route(r1);
        let route_b = solution.route(r2);
        let key = (route_a.version(), route_b.version());
        self.0.insert(key, (delta, best_move));
    }

    fn clear_stale(&mut self, population: &Population) {
        let versions = population
            .solutions()
            .iter()
            .flat_map(|s| s.solution.routes())
            .map(|r| r.version())
            .collect::<FxHashSet<_>>();

        self.0
            .retain(|&k, _| versions.contains(&k.0) && versions.contains(&k.1))
    }
}
