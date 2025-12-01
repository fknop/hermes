use std::f64;

use crate::{
    problem::{vehicle::VehicleId, vehicle_routing_problem::VehicleRoutingProblem},
    solver::{
        intensify::{
            cross_exchange::{CrossExchangeOperator, CrossExchangeOperatorParams},
            intensify_operator::{IntensifyOp, IntensifyOperator},
            inter_relocate::{InterRelocateOperator, InterRelocateParams},
            inter_swap::{InterSwapOperator, InterSwapOperatorParams},
            inter_two_opt_star::{InterTwoOptStarOperator, InterTwoOptStarOperatorParams},
            or_opt::{OrOptOperator, OrOptOperatorParams},
            relocate::{RelocateOperator, RelocateOperatorParams},
            swap::{SwapOperator, SwapOperatorParams},
            two_opt::{TwoOptOperator, TwoOptParams},
        },
        solution::working_solution::WorkingSolution,
    },
};

type VehiclePair = (VehicleId, VehicleId);
pub struct IntensifySearch {
    pairs: Vec<VehiclePair>,
    deltas: Vec<Vec<f64>>,
    best_ops: Vec<Vec<Option<IntensifyOperator>>>,
}

const MAX_DELTA: f64 = f64::MAX;

impl IntensifySearch {
    pub fn new(problem: &VehicleRoutingProblem) -> Self {
        let vehicle_count = problem.vehicles().len();
        let mut deltas = Vec::with_capacity(vehicle_count);
        let mut best_ops: Vec<Vec<Option<IntensifyOperator>>> = Vec::with_capacity(vehicle_count);

        for _ in 0..vehicle_count {
            deltas.push(vec![MAX_DELTA; vehicle_count]);

            let mut inner = Vec::with_capacity(vehicle_count);
            inner.resize_with(vehicle_count, || None);
            best_ops.push(inner);
        }

        let mut pairs = Vec::with_capacity(vehicle_count * vehicle_count);
        for i in 0..vehicle_count {
            for j in 0..vehicle_count {
                pairs.push((i, j))
            }
        }

        IntensifySearch {
            deltas,
            pairs,
            best_ops,
        }
    }

    pub fn intensify(
        &mut self,
        problem: &VehicleRoutingProblem,
        solution: &mut WorkingSolution,
        iterations: usize,
    ) {
        for _ in 0..iterations {
            self.run_iteration(problem, solution);
        }
    }

    fn run_iteration(&mut self, problem: &VehicleRoutingProblem, solution: &mut WorkingSolution) {
        // TwoOptOperator
        for &(v1, v2) in &self.pairs {
            if v1 != v2 {
                continue;
            }

            let route = solution.route(v1);
            for from in 0..route.activities().len() {
                for to in (from + 2)..route.activities().len() {
                    let op = TwoOptOperator::new(TwoOptParams {
                        route_id: v1,
                        from,
                        to,
                    });

                    let delta = op.delta(solution);
                    if delta <= self.deltas[v1][v2] && op.is_valid(solution) {
                        self.deltas[v1][v2] = delta;
                        self.best_ops[v1][v2] = Some(IntensifyOperator::TwoOpt(op));
                    }
                }
            }
        }

        // RelocateOperator
        for &(v1, v2) in &self.pairs {
            if v1 != v2 {
                continue;
            }

            let route = solution.route(v1);

            for from_pos in 0..route.activities().len() {
                for to_pos in 0..=route.activities().len() {
                    if from_pos == to_pos {
                        continue;
                    }

                    if from_pos + 1 == to_pos {
                        continue; // no change in this case
                    }

                    let op = RelocateOperator::new(RelocateOperatorParams {
                        route_id: v1,
                        from: from_pos,
                        to: to_pos,
                    });

                    let delta = op.delta(solution);
                    if delta <= self.deltas[v1][v2] && op.is_valid(solution) {
                        self.deltas[v1][v2] = delta;
                        self.best_ops[v1][v2] = Some(IntensifyOperator::Relocate(op));
                    }
                }
            }
        }

        // SwapOperator
        for &(v1, v2) in &self.pairs {
            if v1 != v2 {
                continue;
            }

            let route = solution.route(v1);

            for from_pos in 0..route.activities().len() {
                for to_pos in from_pos + 1..route.activities().len() {
                    let op = SwapOperator::new(SwapOperatorParams {
                        route_id: v1,
                        first: from_pos,
                        second: to_pos,
                    });

                    let delta = op.delta(solution);
                    if delta <= self.deltas[v1][v2] && op.is_valid(solution) {
                        self.deltas[v1][v2] = delta;
                        self.best_ops[v1][v2] = Some(IntensifyOperator::Swap(op));
                    }
                }
            }
        }

        // InterSwapOperator
        for &(v1, v2) in &self.pairs {
            if v1 == v2 {
                continue;
            }

            let from_route = solution.route(v1);
            let to_route = solution.route(v2);

            for from_pos in 0..from_route.activities().len() {
                for to_pos in 0..to_route.activities().len() {
                    let op = InterSwapOperator::new(InterSwapOperatorParams {
                        first_route_id: v1,
                        second_route_id: v2,
                        first: from_pos,
                        second: to_pos,
                    });

                    let delta = op.delta(solution);
                    if delta <= self.deltas[v1][v2] && op.is_valid(solution) {
                        self.deltas[v1][v2] = delta;
                        self.best_ops[v1][v2] = Some(IntensifyOperator::InterSwap(op));
                    }
                }
            }
        }

        // InterRelocateOperator
        for &(v1, v2) in &self.pairs {
            if v1 == v2 {
                continue;
            }

            let from_route = solution.route(v1);
            let to_route = solution.route(v2);

            for from_pos in 0..from_route.activities().len() {
                let from_job_id = from_route.job_id_at(from_pos);

                if from_job_id.is_shipment() {
                    continue; // skip shipments for inter-relocate
                }

                for to_pos in 0..=to_route.activities().len() {
                    let op = InterRelocateOperator::new(InterRelocateParams {
                        from_route_id: v1,
                        to_route_id: v2,
                        from: from_pos,
                        to: to_pos,
                    });

                    let delta = op.delta(solution);
                    if delta <= self.deltas[v1][v2] && op.is_valid(solution) {
                        self.deltas[v1][v2] = delta;
                        self.best_ops[v1][v2] = Some(IntensifyOperator::InterRelocate(op));
                    }
                }
            }
        }

        // OrOptOperator
        for &(v1, v2) in &self.pairs {
            if v1 != v2 {
                continue;
            }

            let route = solution.route(v1);
            let route_length = route.activities().len();

            for from_pos in 0..route_length {
                for to_pos in 0..=route_length {
                    let max_length = to_pos.abs_diff(from_pos);

                    // A chain is at least length 2
                    for chain_length in 2..=max_length {
                        let op = OrOptOperator::new(OrOptOperatorParams {
                            route_id: v1,
                            from: from_pos,
                            to: to_pos,
                            count: chain_length,
                        });

                        let delta = op.delta(solution);
                        if delta <= self.deltas[v1][v2] && op.is_valid(solution) {
                            self.deltas[v1][v2] = delta;
                            self.best_ops[v1][v2] = Some(IntensifyOperator::OrOpt(op));
                        }
                    }
                }
            }
        }

        // CrossExchangeOperator
        for &(v1, v2) in &self.pairs {
            if v1 <= v2 {
                continue;
            }

            let from_route = solution.route(v1);
            let to_route = solution.route(v2);

            // If the bbox don't intersects, no need to try exchanges
            if !from_route.bbox_intersects(to_route) {
                continue;
            }

            let from_route_length = from_route.activities().len();
            let to_route_length = to_route.activities().len();

            for from_pos in 0..from_route_length - 1 {
                for to_pos in 0..to_route_length - 1 {
                    let max_from_chain = from_route_length - from_pos;
                    let max_to_chain = to_route_length - to_pos;
                    let max_chain_length = max_from_chain.min(max_to_chain);

                    // A chain is at least length 2
                    for chain_length in 2..=max_chain_length {
                        let op = CrossExchangeOperator::new(CrossExchangeOperatorParams {
                            first_route_id: v1,
                            second_route_id: v2,

                            first_start: from_pos,
                            second_start: to_pos,
                            first_end: from_pos + chain_length,
                            second_end: to_pos + chain_length,
                        });

                        let delta = op.delta(solution);
                        if delta <= self.deltas[v1][v2] && op.is_valid(solution) {
                            self.deltas[v1][v2] = delta;
                            self.best_ops[v1][v2] = Some(IntensifyOperator::CrossExchange(op));
                        }
                    }
                }
            }
        }

        // InterTwoOptStarOperator
        for &(v1, v2) in &self.pairs {
            if v1 <= v2 {
                continue;
            }

            let from_route = solution.route(v1);
            let to_route = solution.route(v2);

            // If the bbox don't intersects, no need to try exchanges
            if !from_route.bbox_intersects(to_route) {
                continue;
            }

            let from_route_length = from_route.activities().len();
            let to_route_length = to_route.activities().len();

            for from_pos in 0..from_route_length {
                for to_pos in 0..to_route_length {
                    let op = InterTwoOptStarOperator::new(InterTwoOptStarOperatorParams {
                        first_route_id: v1,
                        second_route_id: v2,

                        first_from: from_pos,
                        second_from: to_pos,
                    });

                    let delta = op.delta(solution);
                    if delta <= self.deltas[v1][v2] && op.is_valid(solution) {
                        self.deltas[v1][v2] = delta;
                        self.best_ops[v1][v2] = Some(IntensifyOperator::InterTwoOptStar(op));
                    }
                }
            }
        }

        let mut best_delta = 0.0;
        let mut best_v1 = None;
        let mut best_v2 = None;
        for i in 0..solution.routes().len() {
            for j in 0..solution.routes().len() {
                if self.deltas[i][j] < best_delta {
                    best_delta = self.deltas[i][j];
                    best_v1 = Some(i);
                    best_v2 = Some(j);
                }
            }
        }

        if let (Some(v1), Some(v2)) = (best_v1, best_v2)
            && let Some(op) = &self.best_ops[v1][v2]
        {
            op.apply(problem, solution);

            self.pairs.clear();

            let updated_routes = op.updated_routes();
            for &updated_route in &updated_routes {
                self.deltas[updated_route].fill(MAX_DELTA);
                self.best_ops[updated_route].fill_with(|| None);
            }

            for i in 0..solution.routes().len() {
                for &updated_route in &updated_routes {
                    self.deltas[i][updated_route] = MAX_DELTA;
                    self.best_ops[i][updated_route] = None;

                    self.pairs.push((i, updated_route));
                    if i != updated_route {
                        self.pairs.push((updated_route, i));
                    }
                }
            }
        }
    }
}
