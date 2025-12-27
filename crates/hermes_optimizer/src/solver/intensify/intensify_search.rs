use std::f64;

use tracing::debug;

use crate::{
    problem::{
        job::ActivityId, vehicle::VehicleIdx, vehicle_routing_problem::VehicleRoutingProblem,
    },
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

macro_rules! temporary_vehicle_id_index {
    ($t:ty, $output:ty) => {
        // Temporary VehicleId Index
        impl std::ops::Index<VehicleIdx> for $t {
            type Output = $output;
            fn index(&self, index: VehicleIdx) -> &Self::Output {
                &self[index.get()]
            }
        }

        // Temporary VehicleId IndexMut
        impl std::ops::IndexMut<VehicleIdx> for $t {
            fn index_mut(&mut self, index: VehicleIdx) -> &mut Self::Output {
                &mut self[index.get()]
            }
        }
    };
}

temporary_vehicle_id_index!(Vec<f64>, f64);
temporary_vehicle_id_index!(Vec<Vec<f64>>, Vec<f64>);
temporary_vehicle_id_index!(Vec<Option<IntensifyOperator>>, Option<IntensifyOperator>);
temporary_vehicle_id_index!(
    Vec<Vec<Option<IntensifyOperator>>>,
    Vec<Option<IntensifyOperator>>
);

type VehiclePair = (VehicleIdx, VehicleIdx);

pub struct IntensifySearch {
    pairs: Vec<VehiclePair>,
    deltas: Vec<Vec<f64>>,
    best_ops: Vec<Vec<Option<IntensifyOperator>>>,
}

const MAX_DELTA: f64 = 0.0;

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
                pairs.push((i.into(), j.into()))
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
    ) -> usize {
        for i in 0..iterations {
            if !self.run_iteration(problem, solution) {
                return i + 1;
            }
        }

        iterations
    }

    fn run_iteration(
        &mut self,
        problem: &VehicleRoutingProblem,
        solution: &mut WorkingSolution,
    ) -> bool {
        // TwoOptOperator
        for &(v1, v2) in &self.pairs {
            if v1 != v2 {
                continue;
            }

            let route = solution.route(v1.into());

            if route.len() < 4 {
                continue; // need at least 4 activities to perform 2-opt
            }

            for from in 0..route.activity_ids().len() - 2 {
                for to in (from + 2)..route.activity_ids().len() {
                    let op = TwoOptOperator::new(TwoOptParams {
                        route_id: v1.into(),
                        from,
                        to,
                    });

                    let delta = op.delta(solution);
                    if delta < self.deltas[v1][v2] && op.is_valid(solution) {
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

            let route = solution.route(v1.into());

            for from_pos in 0..route.activity_ids().len() {
                let from_id = route.job_id(from_pos);

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
                        route_id: v1.into(),
                        from: from_pos,
                        to: to_pos,
                    });

                    let delta = op.delta(solution);
                    if delta < self.deltas[v1][v2] && op.is_valid(solution) {
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

            let route = solution.route(v1.into());

            for from_pos in 0..route.activity_ids().len() {
                for to_pos in from_pos + 1..route.activity_ids().len() {
                    let op = SwapOperator::new(SwapOperatorParams {
                        route_id: v1.into(),
                        first: from_pos,
                        second: to_pos,
                    });

                    let delta = op.delta(solution);
                    if delta < self.deltas[v1][v2] && op.is_valid(solution) {
                        self.deltas[v1][v2] = delta;
                        self.best_ops[v1][v2] = Some(IntensifyOperator::Swap(op));
                    }
                }
            }
        }

        // InterSwapOperator
        for &(v1, v2) in &self.pairs {
            if v1 <= v2 {
                continue;
            }

            let from_route = solution.route(v1.into());
            let to_route = solution.route(v2.into());

            for from_pos in 0..from_route.activity_ids().len() {
                for to_pos in 0..to_route.activity_ids().len() {
                    let op = InterSwapOperator::new(InterSwapOperatorParams {
                        first_route_id: v1.into(),
                        second_route_id: v2.into(),
                        first: from_pos,
                        second: to_pos,
                    });

                    let delta = op.delta(solution);
                    if delta < self.deltas[v1][v2] && op.is_valid(solution) {
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

            let from_route = solution.route(v1.into());
            let to_route = solution.route(v2.into());

            for from_pos in 0..from_route.activity_ids().len() {
                let from_job_id = from_route.job_id(from_pos);

                if from_job_id.is_shipment() {
                    continue; // skip shipments for inter-relocate
                }

                for to_pos in 0..=to_route.activity_ids().len() {
                    let op = InterRelocateOperator::new(InterRelocateParams {
                        from_route_id: v1.into(),
                        to_route_id: v2.into(),
                        from: from_pos,
                        to: to_pos,
                    });

                    let delta = op.delta(solution);

                    if delta < self.deltas[v1][v2] && op.is_valid(solution) {
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

            let route = solution.route(v1.into());
            let route_length = route.activity_ids().len();

            for from_pos in 0..route_length {
                for to_pos in from_pos..=route_length {
                    let max_length = to_pos.abs_diff(from_pos).saturating_sub(1);

                    // A chain is at least length 2
                    for chain_length in 2..=max_length {
                        let op = OrOptOperator::new(OrOptOperatorParams {
                            route_id: v1.into(),
                            from: from_pos,
                            to: to_pos,
                            count: chain_length,
                        });

                        let delta = op.delta(solution);
                        if delta < self.deltas[v1][v2] && op.is_valid(solution) {
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

            let from_route = solution.route(v1.into());
            let to_route = solution.route(v2.into());

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
                                first_route_id: v1.into(),
                                second_route_id: v2.into(),

                                first_start: from_pos,
                                second_start: to_pos,
                                first_end: from_pos + from_length - 1,
                                second_end: to_pos + to_length - 1,
                            });
                            let delta = op.delta(solution);
                            if delta < self.deltas[v1][v2] && op.is_valid(solution) {
                                self.deltas[v1][v2] = delta;
                                self.best_ops[v1][v2] = Some(IntensifyOperator::CrossExchange(op));
                            }
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

            let from_route = solution.route(v1.into());
            let to_route = solution.route(v2.into());

            // If the bbox don't intersects, no need to try exchanges
            if !from_route.bbox_intersects(to_route) {
                continue;
            }

            let from_route_length = from_route.activity_ids().len();
            let to_route_length = to_route.activity_ids().len();

            for from_pos in 0..from_route_length - 1 {
                for to_pos in 0..to_route_length - 1 {
                    let op = InterTwoOptStarOperator::new(InterTwoOptStarOperatorParams {
                        first_route_id: v1.into(),
                        second_route_id: v2.into(),

                        first_from: from_pos,
                        second_from: to_pos,
                    });

                    let delta = op.delta(solution);
                    if delta < self.deltas[v1][v2] && op.is_valid(solution) {
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
            debug!(
                "Apply {} - delta = {} (r1 = {}, r2 = {}) (r1.len() = {}, r2.len() = {}), op = {:?}",
                op.operator_name(),
                best_delta,
                v1,
                v2,
                solution.route(v1.into()).activity_ids().len(),
                solution.route(v2.into()).activity_ids().len(),
                op
            );

            op.apply(problem, solution);

            self.pairs.clear();

            let updated_routes = op.updated_routes();
            for &updated_route in &updated_routes {
                self.deltas[updated_route.get()].fill(MAX_DELTA);

                self.best_ops[updated_route.get()].fill_with(|| None);
            }

            for i in 0..solution.routes().len() {
                for &updated_route in &updated_routes {
                    self.deltas[i][updated_route.get()] = MAX_DELTA;
                    self.best_ops[i][updated_route.get()] = None;

                    self.pairs
                        .push((VehicleIdx::new(i), VehicleIdx::new(updated_route.get())));
                    if i != updated_route.get() {
                        self.pairs
                            .push((VehicleIdx::new(updated_route.get()), VehicleIdx::new(i)));
                    }
                }
            }

            true
        } else {
            false
        }
    }
}
