use std::f64;

use fxhash::{FxBuildHasher, FxHashMap, FxHashSet};
use tokio::time::error;
use tracing::{debug, info, warn};

use crate::{
    problem::{job::ActivityId, vehicle_routing_problem::VehicleRoutingProblem},
    solver::{
        accepted_solution::AcceptedSolution,
        alns_search::AlnsSearch,
        ls::{
            cross_exchange::{CrossExchangeOperator, CrossExchangeOperatorParams},
            inter_relocate::{InterRelocateOperator, InterRelocateParams},
            inter_swap::{InterSwapOperator, InterSwapOperatorParams},
            inter_two_opt_star::{InterTwoOptStarOperator, InterTwoOptStarOperatorParams},
            r#move::{LocalSearchMove, LocalSearchOperator},
            or_opt::{OrOptOperator, OrOptOperatorParams},
            relocate::{RelocateOperator, RelocateOperatorParams},
            swap::{SwapOperator, SwapOperatorParams},
            two_opt::{TwoOptOperator, TwoOptParams},
        },
        solution::{
            route::WorkingSolutionRoute, route_id::RouteIdx, working_solution::WorkingSolution,
        },
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
    pairs: Vec<RoutePair>,
    // deltas: Vec<Vec<f64>>,
    // best_ops: Vec<Vec<Option<LocalSearchMove>>>,
    state: LocalSearchState,
}

const MAX_DELTA: f64 = 0.0;

impl LocalSearch {
    pub fn new(problem: &VehicleRoutingProblem) -> Self {
        let count = problem.vehicles().len();
        // let mut deltas = Vec::with_capacity(route_count);
        // let mut best_ops: Vec<Vec<Option<LocalSearchMove>>> = Vec::with_capacity(route_count);

        // for _ in 0..route_count {
        //     deltas.push(vec![MAX_DELTA; route_count]);

        //     let mut inner = Vec::with_capacity(route_count);
        //     inner.resize_with(route_count, || None);
        //     best_ops.push(inner);
        // }

        let pairs = Vec::with_capacity(count * count);

        LocalSearch {
            pairs,
            // deltas,
            // best_ops,
            state: LocalSearchState::new(),
        }
    }

    pub fn intensify(
        &mut self,
        search: &AlnsSearch,
        problem: &VehicleRoutingProblem,
        solution: &mut WorkingSolution,
        iterations: usize,
    ) -> usize {
        self.build_pairs(solution);
        for i in 0..iterations {
            if !self.run_iteration(search, problem, solution, i + 1) {
                return i + 1;
            }
        }

        iterations
    }

    fn run_iteration(
        &mut self,
        search: &AlnsSearch,
        problem: &VehicleRoutingProblem,
        solution: &mut WorkingSolution,
        iteration: usize,
    ) -> bool {
        for &(r1, r2) in &self.pairs {
            let v1 = solution.route(r1).version();
            let v2 = solution.route(r2).version();

            assert!(!self.state.contains_key((v1, v2)));
        }

        // TwoOptOperator
        for &(r1, r2) in &self.pairs {
            if r1 != r2 {
                continue;
            }

            let route = solution.route(r1);

            if route.len() < 4 {
                continue; // need at least 4 activities to perform 2-opt
            }

            for from in 0..route.activity_ids().len() - 2 {
                for to in (from + 2)..route.activity_ids().len() {
                    let op = TwoOptOperator::new(TwoOptParams {
                        route_id: r1,
                        from,
                        to,
                    });

                    let delta = op.delta(solution);
                    if delta < self.delta(solution, r1, r2) && op.is_valid(solution) {
                        self.state.update_best(
                            solution,
                            r1,
                            r2,
                            delta,
                            LocalSearchMove::TwoOpt(op),
                        );
                    }
                }
            }
        }

        // RelocateOperator
        for &(r1, r2) in &self.pairs {
            if r1 != r2 {
                continue;
            }

            let route = solution.route(r1);

            for from_pos in 0..route.activity_ids().len() {
                let from_id = route.activity_id(from_pos);

                let (to_pos_start, to_pos_end) = match from_id {
                    ActivityId::ShipmentPickup(index) => {
                        let delivery_position = route
                            .job_position(ActivityId::ShipmentDelivery(index))
                            .unwrap_or_else(|| {
                                panic!(
                                    "Shipment pickup {from_id} has no delivery in the same route"
                                )
                            });
                        (from_pos + 1, delivery_position)
                    }
                    ActivityId::ShipmentDelivery(index) => {
                        let pickup_position = route
                            .job_position(ActivityId::ShipmentPickup(index))
                            .unwrap_or_else(|| {
                                panic!(
                                    "Shipment delivery {from_id} has no pickup in the same route"
                                )
                            });
                        (pickup_position + 1, route.len())
                    }
                    ActivityId::Service(_) => (0, route.len()),
                };

                for to_pos in to_pos_start..=to_pos_end {
                    if from_pos == to_pos {
                        continue;
                    }

                    if from_pos + 1 == to_pos {
                        continue; // no change in this case
                    }

                    let op = RelocateOperator::new(RelocateOperatorParams {
                        route_id: r1,
                        from: from_pos,
                        to: to_pos,
                    });

                    let delta = op.delta(solution);
                    if delta < self.delta(solution, r1, r2) && op.is_valid(solution) {
                        self.state.update_best(
                            solution,
                            r1,
                            r2,
                            delta,
                            LocalSearchMove::Relocate(op),
                        );
                    }
                }
            }
        }

        // SwapOperator
        for &(r1, r2) in &self.pairs {
            if r1 != r2 {
                continue;
            }

            let route = solution.route(r1);

            for from_pos in 0..route.activity_ids().len() {
                for to_pos in from_pos + 1..route.activity_ids().len() {
                    let op = SwapOperator::new(SwapOperatorParams {
                        route_id: r1,
                        first: from_pos,
                        second: to_pos,
                    });

                    let delta = op.delta(solution);
                    if delta < self.delta(solution, r1, r2) && op.is_valid(solution) {
                        self.state
                            .update_best(solution, r1, r2, delta, LocalSearchMove::Swap(op));
                    }
                }
            }
        }

        // InterSwapOperator
        for &(r1, r2) in &self.pairs {
            if r1 <= r2 {
                continue;
            }

            let from_route = solution.route(r1);
            let to_route = solution.route(r2);

            for from_pos in 0..from_route.activity_ids().len() {
                for to_pos in 0..to_route.activity_ids().len() {
                    let op = InterSwapOperator::new(InterSwapOperatorParams {
                        first_route_id: r1,
                        second_route_id: r2,
                        first: from_pos,
                        second: to_pos,
                    });

                    let delta = op.delta(solution);
                    if delta < self.delta(solution, r1, r2) && op.is_valid(solution) {
                        self.state.update_best(
                            solution,
                            r1,
                            r2,
                            delta,
                            LocalSearchMove::InterSwap(op),
                        );
                    }
                }
            }
        }

        // InterRelocateOperator
        for &(r1, r2) in &self.pairs {
            if r1 == r2 {
                continue;
            }

            let from_route = solution.route(r1);
            let to_route = solution.route(r2);

            for from_pos in 0..from_route.activity_ids().len() {
                let from_activity_id = from_route.activity_id(from_pos);

                if from_activity_id.is_shipment() {
                    continue; // skip shipments for inter-relocate
                }

                for to_pos in 0..=to_route.activity_ids().len() {
                    let op = InterRelocateOperator::new(InterRelocateParams {
                        from_route_id: r1,
                        to_route_id: r2,
                        from: from_pos,
                        to: to_pos,
                    });

                    let delta = op.delta(solution);

                    if delta < self.delta(solution, r1, r2) && op.is_valid(solution) {
                        self.state.update_best(
                            solution,
                            r1,
                            r2,
                            delta,
                            LocalSearchMove::InterRelocate(op),
                        );
                    }
                }
            }
        }

        // OrOptOperator
        for &(r1, r2) in &self.pairs {
            if r1 != r2 {
                continue;
            }

            let route = solution.route(r1);
            let route_length = route.activity_ids().len();

            for from_pos in 0..route_length {
                for to_pos in from_pos..=route_length {
                    let max_length = to_pos.abs_diff(from_pos).saturating_sub(1);

                    // A chain is at least length 2
                    for chain_length in 2..=max_length {
                        let op = OrOptOperator::new(OrOptOperatorParams {
                            route_id: r1,
                            from: from_pos,
                            to: to_pos,
                            count: chain_length,
                        });

                        let delta = op.delta(solution);
                        if delta < self.delta(solution, r1, r2) && op.is_valid(solution) {
                            self.state.update_best(
                                solution,
                                r1,
                                r2,
                                delta,
                                LocalSearchMove::OrOpt(op),
                            );
                        }
                    }
                }
            }
        }

        // CrossExchangeOperator
        for &(r1, r2) in &self.pairs {
            if r1 <= r2 {
                continue;
            }

            let from_route = solution.route(r1);
            let to_route = solution.route(r2);

            // If the bbox don't intersects, no need to try exchanges
            if !from_route.bbox_intersects(to_route) {
                continue;
            }

            let from_route_length = from_route.activity_ids().len();
            let to_route_length = to_route.activity_ids().len();

            for from_pos in 0..from_route_length - 1 {
                for to_pos in 0..to_route_length - 1 {
                    let max_from_chain = from_route_length - from_pos - 1;
                    let max_to_chain = to_route_length - to_pos - 1;

                    // A chain is at least length 2
                    for from_length in 2..=max_from_chain {
                        for to_length in 2..=max_to_chain {
                            let op = CrossExchangeOperator::new(CrossExchangeOperatorParams {
                                first_route_id: r1,
                                second_route_id: r2,

                                first_start: from_pos,
                                second_start: to_pos,
                                first_end: from_pos + from_length - 1,
                                second_end: to_pos + to_length - 1,
                            });
                            let delta = op.delta(solution);
                            if delta < self.delta(solution, r1, r2) && op.is_valid(solution) {
                                self.state.update_best(
                                    solution,
                                    r1,
                                    r2,
                                    delta,
                                    LocalSearchMove::CrossExchange(op),
                                );
                            }
                        }
                    }
                }
            }
        }

        // InterTwoOptStarOperator
        for &(r1, r2) in &self.pairs {
            if r1 <= r2 {
                continue;
            }

            let from_route = solution.route(r1);
            let to_route = solution.route(r2);

            // If the bbox don't intersects, no need to try exchanges
            if !from_route.bbox_intersects(to_route) {
                continue;
            }

            let from_route_length = from_route.activity_ids().len();
            let to_route_length = to_route.activity_ids().len();

            for from_pos in 0..from_route_length - 1 {
                for to_pos in 0..to_route_length - 1 {
                    let op = InterTwoOptStarOperator::new(InterTwoOptStarOperatorParams {
                        first_route_id: r1,
                        second_route_id: r2,

                        first_from: from_pos,
                        second_from: to_pos,
                    });

                    let delta = op.delta(solution);
                    if delta < self.delta(solution, r1, r2) && op.is_valid(solution) {
                        self.state.update_best(
                            solution,
                            r1,
                            r2,
                            delta,
                            LocalSearchMove::InterTwoOptStar(op),
                        );
                    }
                }
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
        {
            let score_before = search.compute_solution_score(solution);

            if score_before.0.is_failure() {
                panic!("Score before is failure, don't apply");
            }

            if !op.is_valid(solution) {
                tracing::error!(?op, "Operator {} is not valid", op.operator_name());
                panic!("Stored operator is not valid")
            }

            info!("Apply {} ({}, {})", op.operator_name(), r1, r2);
            op.apply(problem, solution);

            let score = search.compute_solution_score(solution);
            if !score_before.0.is_failure() && score.0.is_failure() {
                tracing::error!(
                    "Iteration {}: Operator {} ({}, {}) broke hard constraint {:?}",
                    iteration,
                    op.operator_name(),
                    r1,
                    r2,
                    score.1
                );

                tracing::error!("{:?}", op);

                for (index, route) in solution.routes().iter().enumerate() {
                    println!("Route {}: {:?}", index, route.activity_ids());
                }

                panic!("BUG!")
            }

            self.pairs.clear();

            let updated_routes = op.updated_routes();
            // for &updated_route in &updated_routes {
            //     self.deltas[updated_route.get()].fill(MAX_DELTA);
            //     self.best_ops[updated_route.get()].fill_with(|| None);
            // }

            for i in 0..solution.routes().len() {
                for &updated_route in &updated_routes {
                    // self.deltas[i][updated_route.get()] = MAX_DELTA;
                    // self.best_ops[i][updated_route.get()] = None;

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

    pub fn clear_stale(&mut self, accepted_solutions: &[AcceptedSolution]) {
        self.state.clear_stale(accepted_solutions);
    }

    fn delta(&self, solution: &WorkingSolution, r1: RouteIdx, r2: RouteIdx) -> f64 {
        self.state.delta(solution, r1, r2)
    }

    fn build_pairs(&mut self, solution: &WorkingSolution) {
        self.pairs.clear();

        for (i, r1) in solution.routes().iter().enumerate_idx() {
            for (j, r2) in solution.routes().iter().enumerate_idx() {
                let v1 = r1.version();
                let v2 = r2.version();
                if !self.state.contains_key((v1, v2)) {
                    self.pairs.push((i, j))
                }
            }
        }
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

    fn clear_stale(&mut self, solutions: &[AcceptedSolution]) {
        let versions = solutions
            .iter()
            .flat_map(|s| s.solution.routes())
            .map(|r| r.version())
            .collect::<FxHashSet<_>>();

        self.0
            .retain(|&k, _| versions.contains(&k.0) && versions.contains(&k.1))
    }
}
