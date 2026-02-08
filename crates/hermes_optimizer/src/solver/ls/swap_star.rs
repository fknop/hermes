use std::f64;

use jiff::SignedDuration;

use crate::{
    problem::{
        job::{ActivityId, JobIdx},
        vehicle_routing_problem::VehicleRoutingProblem,
    },
    solver::{
        constraints::{compute_insertion_score::compute_insertion_score, constraint::Constraint},
        insertion::{Insertion, ServiceInsertion, for_each_route_insertion},
        insertion_context::InsertionContext,
        ls::r#move::LocalSearchOperator,
        solution::{
            route::WorkingSolutionRoute, route_id::RouteIdx, working_solution::WorkingSolution,
        },
    },
};

/// The SWAP* operator is based on:
/// Hybrid genetic search for the CVRP: Open-source implementation and SWAP* neighborhood
/// Thibaut Vidal, 2022

#[derive(Default, Clone)]
struct TopThreeInsertions {
    insertions: [Option<(Insertion, f64)>; 3],
}

impl TopThreeInsertions {
    fn is_empty(&self) -> bool {
        self.insertions[0].is_none()
    }

    #[inline(always)]
    fn delta(&self, i: usize) -> f64 {
        self.insertions[i]
            .as_ref()
            .map_or(f64::MAX, |(_, score)| *score)
    }

    fn update(&mut self, insertion: Insertion, delta: f64) {
        if delta < self.delta(0) {
            self.insertions[2] = self.insertions[1].take();
            self.insertions[1] = self.insertions[0].take();
            self.insertions[0] = Some((insertion, delta));
        } else if delta < self.delta(1) {
            self.insertions[2] = self.insertions[1].take();
            self.insertions[1] = Some((insertion, delta));
        } else if delta < self.delta(2) {
            self.insertions[2] = Some((insertion, delta));
        }
    }

    fn iter(&self) -> impl Iterator<Item = &(Insertion, f64)> {
        self.insertions
            .iter()
            .filter_map(|insertion| insertion.as_ref())
    }
}

fn find_top_three_insertions(
    solution: &WorkingSolution,
    constraints: &[Constraint],
    route_id: RouteIdx,
    job_id: JobIdx,
) -> TopThreeInsertions {
    let mut insertions = TopThreeInsertions::default();

    // We do insert on failure here because we want to consider insertions that may become feasible once the other activity is removed from the route.
    let insert_on_failure = true;

    for_each_route_insertion(solution, route_id, job_id, |insertion| {
        let insertion_context =
            InsertionContext::new(solution.problem(), solution, &insertion, insert_on_failure);
        let score = compute_insertion_score(constraints, &insertion_context, None);
        insertions.update(insertion, score.soft_score);
    });

    insertions
}

fn removal_cost_delta(
    problem: &VehicleRoutingProblem,
    route: &WorkingSolutionRoute,
    position: usize,
) -> f64 {
    let vehicle = route.vehicle(problem);
    let previous = route.previous_location_id(problem, position);
    let removed = route.location_id(problem, position);
    let next = route.next_location_id(problem, position);

    let transport_cost_delta = -problem.travel_cost_or_zero(vehicle, previous, removed)
        - problem.travel_cost_or_zero(vehicle, removed, next)
        + problem.travel_cost_or_zero(vehicle, previous, next);

    let waiting_cost_delta = problem.waiting_duration_cost(route.waiting_duration_change_delta(
        problem,
        std::iter::empty(),
        position,
        position,
    ));

    transport_cost_delta + waiting_cost_delta
}

// This function computes the cost delta of moving an activity from source route to target route
fn in_place_delta(
    problem: &VehicleRoutingProblem,
    source: &WorkingSolutionRoute,
    source_position: usize,
    target: &WorkingSolutionRoute,
    target_position: usize,
) -> f64 {
    let (transport_cost_delta, _) = target.transport_cost_delta_update(
        problem,
        target_position,
        target_position + 1,
        source,
        source_position,
        source_position + 1,
    );

    let waiting_cost_delta = problem.waiting_duration_cost(target.waiting_duration_change_delta(
        problem,
        std::iter::once(source.activity_id(source_position)),
        target_position,
        target_position + 1,
    ));

    transport_cost_delta + waiting_cost_delta
}

pub fn find_best_swap_star_move(
    problem: &VehicleRoutingProblem,
    solution: &WorkingSolution,
    constraints: &[Constraint],
    (r1, r2): (RouteIdx, RouteIdx),
) -> Option<SwapStar> {
    if r1 <= r2 {
        // Operator is symmetric
        return None;
    }

    let route1 = solution.route(r1);
    let route2 = solution.route(r2);

    if !route1.bbox_intersects(route2) {
        return None;
    }

    let mut best_delta = 0.0;
    let mut best_move: Option<SwapStar> = None;
    let mut top_insertions_r1 = vec![TopThreeInsertions::default(); route2.len()];
    let mut top_insertions_r2 = vec![TopThreeInsertions::default(); route1.len()];

    for (position, activity_id) in route1.activity_ids().iter().enumerate() {
        let job_id = activity_id.job_id();

        let job = problem.job(job_id);

        // Don't support shipment for Swap*
        if job.is_shipment() {
            continue;
        }

        top_insertions_r2[position] = find_top_three_insertions(solution, constraints, r2, job_id);
    }

    for (position, activity_id) in route2.activity_ids().iter().enumerate() {
        let job_id = activity_id.job_id();

        let job = problem.job(job_id);

        // Don't support shipment for Swap*
        if job.is_shipment() {
            continue;
        }

        top_insertions_r1[position] = find_top_three_insertions(solution, constraints, r1, job_id);
    }

    for (p1, _) in route1.activity_ids().iter().enumerate() {
        let r2_insertions = &top_insertions_r2[p1];

        // No valid insertions for p1 in r2, skip
        if r2_insertions.is_empty() {
            continue;
        }

        let r1_removal_cost = removal_cost_delta(problem, route1, p1);

        for (p2, _) in route2.activity_ids().iter().enumerate() {
            let r1_insertions = &top_insertions_r1[p2];

            // No valid insertions of p2 in r1, skip
            if r1_insertions.is_empty() {
                continue;
            }

            let mut best_moves: Vec<SwapStar> = Vec::with_capacity(16);

            let r2_removal_cost = removal_cost_delta(problem, route2, p2);

            let r1_in_place_delta = in_place_delta(problem, route2, p2, route1, p1);
            let r2_in_place_delta = in_place_delta(problem, route1, p1, route2, p2);

            let delta = r1_in_place_delta + r2_in_place_delta;

            // Option 1: Both in-place swaps
            if delta < best_delta {
                best_moves.push(SwapStar::new(SwapStarParams {
                    first_route: r1,
                    second_route: r2,
                    first_position: p1,
                    second_position: p2,
                    first_insertion: p1,
                    second_insertion: p2,
                    delta,
                }));
            }

            for insertion2 in r2_insertions.iter() {
                match insertion2.0 {
                    Insertion::Service(ServiceInsertion { position, .. }) => {
                        if position == p2 || position == p2 + 1 {
                            continue;
                        }

                        let r2_delta = r2_removal_cost + insertion2.1;
                        let delta = r1_in_place_delta + r2_delta;
                        // Option 2: p1 in place, p2 inserted
                        if delta < best_delta {
                            best_moves.push(SwapStar::new(SwapStarParams {
                                first_route: r1,
                                second_route: r2,
                                first_position: p1,
                                second_position: p2,
                                first_insertion: p1,
                                second_insertion: position,
                                delta,
                            }));
                        }
                    }
                    Insertion::Shipment(_) => continue,
                }
            }

            for insertion1 in r1_insertions.iter() {
                match &insertion1.0 {
                    Insertion::Service(i1) => {
                        if i1.position == p1 || i1.position == p1 + 1 {
                            continue;
                        }

                        let r1_delta = r1_removal_cost + insertion1.1;

                        let delta = r1_delta + r2_in_place_delta;
                        // Option 3: p2 in-place, p1 inserted
                        if delta < best_delta {
                            best_moves.push(SwapStar::new(SwapStarParams {
                                first_route: r1,
                                second_route: r2,
                                first_position: p1,
                                second_position: p2,
                                first_insertion: i1.position,
                                second_insertion: p2,
                                delta,
                            }));
                        }

                        for insertion2 in r2_insertions.iter() {
                            match &insertion2.0 {
                                Insertion::Service(i2) => {
                                    if i2.position == p2 || i2.position == p2 + 1 {
                                        continue;
                                    }

                                    let r2_delta = r2_removal_cost + insertion2.1;

                                    // Option 4: Both inserted
                                    let delta = r1_delta + r2_delta;
                                    if delta < best_delta {
                                        best_moves.push(SwapStar::new(SwapStarParams {
                                            first_route: r1,
                                            second_route: r2,
                                            first_position: p1,
                                            second_position: p2,
                                            first_insertion: i1.position,
                                            second_insertion: i2.position,
                                            delta,
                                        }));
                                    }
                                }
                                Insertion::Shipment(_) => continue,
                            }
                        }
                    }
                    Insertion::Shipment(_) => continue,
                }
            }

            best_moves.sort_by(|s1, s2| s1.params.delta.total_cmp(&s2.params.delta));

            for swap_move in best_moves {
                if swap_move.is_valid(solution) {
                    best_delta = swap_move.delta(solution);
                    best_move = Some(swap_move);
                    break;
                }
            }
        }
    }

    best_move
}

#[derive(Debug)]
pub struct SwapStarParams {
    pub first_route: RouteIdx,
    pub second_route: RouteIdx,
    pub first_position: usize,
    pub second_position: usize,
    pub first_insertion: usize,
    pub second_insertion: usize,
    pub delta: f64,
}

#[derive(Debug)]
pub struct SwapStar {
    params: SwapStarParams,
}

impl SwapStar {
    pub fn new(params: SwapStarParams) -> Self {
        if params.first_route == params.second_route {
            panic!("SwapStar: first_route cannot be equal to second_route");
        }

        if params.first_insertion == params.first_position + 1 {
            panic!(
                "SwapStar: first_insertion cannot be equal to first_position + 1 as it is equal to being first_position when first_position is removed"
            )
        }

        if params.second_insertion == params.second_position + 1 {
            panic!(
                "SwapStar: second_insertion cannot be equal to second_position + 1 as it is equal to being second_position when second_position is removed"
            )
        }

        SwapStar { params }
    }
}

impl LocalSearchOperator for SwapStar {
    fn generate_moves<C>(
        _problem: &VehicleRoutingProblem,
        _solution: &WorkingSolution,
        _pair: (RouteIdx, RouteIdx),
        _consumer: C,
    ) where
        C: FnMut(Self),
    {
    }

    fn waiting_cost_delta(&self, solution: &WorkingSolution) -> f64 {
        let mut waiting_duration_delta = SignedDuration::ZERO;
        let problem = solution.problem();
        let route1 = solution.route(self.params.first_route);
        let route2 = solution.route(self.params.second_route);

        let r1_activity_id = route1.activity_id(self.params.first_position);
        let r2_activity_id = route2.activity_id(self.params.second_position);

        if self.params.first_insertion > self.params.first_position {
            waiting_duration_delta += route1.waiting_duration_change_delta(
                problem,
                route1
                    .activity_ids_iter(self.params.first_position + 1, self.params.first_insertion)
                    .chain(std::iter::once(r2_activity_id)),
                self.params.first_position,
                self.params.first_insertion,
            );
        } else {
            waiting_duration_delta += route1.waiting_duration_change_delta(
                problem,
                std::iter::once(r2_activity_id).chain(
                    route1
                        .activity_ids_iter(self.params.first_insertion, self.params.first_position),
                ),
                self.params.first_insertion,
                self.params.first_position + 1,
            );
        };

        if self.params.second_insertion > self.params.second_position {
            waiting_duration_delta += route2.waiting_duration_change_delta(
                problem,
                route2
                    .activity_ids_iter(
                        self.params.second_position + 1,
                        self.params.second_insertion,
                    )
                    .chain(std::iter::once(r1_activity_id)),
                self.params.second_position,
                self.params.second_insertion,
            );
        } else {
            waiting_duration_delta +=
                route2.waiting_duration_change_delta(
                    problem,
                    std::iter::once(r1_activity_id).chain(route2.activity_ids_iter(
                        self.params.second_insertion,
                        self.params.second_position,
                    )),
                    self.params.second_insertion,
                    self.params.second_position + 1,
                );
        };

        problem.waiting_duration_cost(waiting_duration_delta)
    }

    /// Only called for score assertions
    fn transport_cost_delta(&self, solution: &WorkingSolution) -> f64 {
        let mut delta = 0.0;
        let problem = solution.problem();
        let route1 = solution.route(self.params.first_route);
        let route2 = solution.route(self.params.second_route);

        let r1_location_id = route1.location_id(problem, self.params.first_position);
        let r2_location_id = route2.location_id(problem, self.params.second_position);

        // Removal cost of route 1
        delta -= problem.travel_cost_or_zero(
            route1.vehicle(problem),
            route1.previous_location_id(problem, self.params.first_position),
            route1.location_id(problem, self.params.first_position),
        );

        delta -= problem.travel_cost_or_zero(
            route1.vehicle(problem),
            route1.location_id(problem, self.params.first_position),
            route1.next_location_id(problem, self.params.first_position),
        );

        // Addition cost of route 1
        if self.params.first_insertion == self.params.first_position {
            delta += problem.travel_cost_or_zero(
                route1.vehicle(problem),
                route1.previous_location_id(problem, self.params.first_position),
                r2_location_id,
            );
            delta += problem.travel_cost_or_zero(
                route1.vehicle(problem),
                r2_location_id,
                route1.next_location_id(problem, self.params.first_position),
            )
        } else {
            delta += problem.travel_cost_or_zero(
                route1.vehicle(problem),
                route1.previous_location_id(problem, self.params.first_position),
                route1.next_location_id(problem, self.params.first_position),
            );

            delta += problem.travel_cost_or_zero(
                route1.vehicle(problem),
                route1.previous_location_id(problem, self.params.first_insertion),
                r2_location_id,
            );

            delta += problem.travel_cost_or_zero(
                route1.vehicle(problem),
                r2_location_id,
                route1
                    .location_id(problem, self.params.first_insertion)
                    .or_else(|| route1.end_location(problem)),
            );

            delta -= problem.travel_cost_or_zero(
                route1.vehicle(problem),
                route1.previous_location_id(problem, self.params.first_insertion),
                route1
                    .location_id(problem, self.params.first_insertion)
                    .or_else(|| route1.end_location(problem)),
            )
        }

        // Removal cost of route 2
        delta -= problem.travel_cost_or_zero(
            route2.vehicle(problem),
            route2.previous_location_id(problem, self.params.second_position),
            route2.location_id(problem, self.params.second_position),
        );

        delta -= problem.travel_cost_or_zero(
            route2.vehicle(problem),
            route2.location_id(problem, self.params.second_position),
            route2.next_location_id(problem, self.params.second_position),
        );

        // Addition cost of route 2
        if self.params.second_insertion == self.params.second_position {
            delta += problem.travel_cost_or_zero(
                route2.vehicle(problem),
                route2.previous_location_id(problem, self.params.second_position),
                r1_location_id,
            );
            delta += problem.travel_cost_or_zero(
                route2.vehicle(problem),
                r1_location_id,
                route2.next_location_id(problem, self.params.second_position),
            )
        } else {
            delta += problem.travel_cost_or_zero(
                route2.vehicle(problem),
                route2.previous_location_id(problem, self.params.second_position),
                route2.next_location_id(problem, self.params.second_position),
            );

            delta += problem.travel_cost_or_zero(
                route2.vehicle(problem),
                route2.previous_location_id(problem, self.params.second_insertion),
                r1_location_id,
            );

            delta += problem.travel_cost_or_zero(
                route2.vehicle(problem),
                r1_location_id,
                route2
                    .location_id(problem, self.params.second_insertion)
                    .or_else(|| route2.end_location(problem)),
            );

            delta -= problem.travel_cost_or_zero(
                route2.vehicle(problem),
                route2.previous_location_id(problem, self.params.second_insertion),
                route2
                    .location_id(problem, self.params.second_insertion)
                    .or_else(|| route2.end_location(problem)),
            )
        }

        delta
    }

    fn fixed_route_cost_delta(&self, _solution: &WorkingSolution) -> f64 {
        0.0
    }

    // fn delta(&self, _solution: &WorkingSolution) -> f64 {
    //     self.params.delta
    // }

    fn is_valid(&self, solution: &WorkingSolution) -> bool {
        let problem = solution.problem();
        let route1 = solution.route(self.params.first_route);
        let route2 = solution.route(self.params.second_route);

        let r1_activity_id = route1.activity_id(self.params.first_position);
        let r2_activity_id = route2.activity_id(self.params.second_position);

        let mut is_valid = true;

        if self.params.first_insertion > self.params.first_position {
            is_valid = is_valid
                && route1.is_valid_change(
                    problem,
                    route1
                        .activity_ids_iter(
                            self.params.first_position + 1,
                            self.params.first_insertion,
                        )
                        .chain(std::iter::once(r2_activity_id)),
                    self.params.first_position,
                    self.params.first_insertion,
                );
        } else {
            is_valid = is_valid
                && route1.is_valid_change(
                    problem,
                    std::iter::once(r2_activity_id).chain(route1.activity_ids_iter(
                        self.params.first_insertion,
                        self.params.first_position,
                    )),
                    self.params.first_insertion,
                    self.params.first_position + 1,
                );
        };

        if self.params.second_insertion > self.params.second_position {
            is_valid = is_valid
                && route2.is_valid_change(
                    problem,
                    route2
                        .activity_ids_iter(
                            self.params.second_position + 1,
                            self.params.second_insertion,
                        )
                        .chain(std::iter::once(r1_activity_id)),
                    self.params.second_position,
                    self.params.second_insertion,
                );
        } else {
            is_valid = is_valid
                && route2.is_valid_change(
                    problem,
                    std::iter::once(r1_activity_id).chain(route2.activity_ids_iter(
                        self.params.second_insertion,
                        self.params.second_position,
                    )),
                    self.params.second_insertion,
                    self.params.second_position + 1,
                );
        };

        is_valid
    }

    fn apply(&self, problem: &VehicleRoutingProblem, solution: &mut WorkingSolution) {
        let r1_activity_id = solution
            .route(self.params.first_route)
            .activity_id(self.params.first_position);
        let r2_activity_id = solution
            .route(self.params.second_route)
            .activity_id(self.params.second_position);

        let first_route_activities: Vec<ActivityId> = if self.params.first_insertion
            > self.params.first_position
        {
            solution
                .route(self.params.first_route)
                .activity_ids_iter(self.params.first_position + 1, self.params.first_insertion)
                .chain(std::iter::once(r2_activity_id))
                .collect()
        } else {
            std::iter::once(r2_activity_id)
                .chain(
                    solution
                        .route(self.params.first_route)
                        .activity_ids_iter(self.params.first_insertion, self.params.first_position),
                )
                .collect()
        };

        let second_route_activities: Vec<ActivityId> =
            if self.params.second_insertion > self.params.second_position {
                solution
                    .route(self.params.second_route)
                    .activity_ids_iter(
                        self.params.second_position + 1,
                        self.params.second_insertion,
                    )
                    .chain(std::iter::once(r1_activity_id))
                    .collect()
            } else {
                std::iter::once(r1_activity_id)
                    .chain(solution.route(self.params.second_route).activity_ids_iter(
                        self.params.second_insertion,
                        self.params.second_position,
                    ))
                    .collect()
            };

        let route1 = solution.route_mut(self.params.first_route);
        if self.params.first_insertion > self.params.first_position {
            route1.replace_activities(
                problem,
                &first_route_activities,
                self.params.first_position,
                self.params.first_insertion,
            );
        } else {
            route1.replace_activities(
                problem,
                &first_route_activities,
                self.params.first_insertion,
                self.params.first_position + 1,
            );
        };

        let route2 = solution.route_mut(self.params.second_route);
        if self.params.second_insertion > self.params.second_position {
            route2.replace_activities(
                problem,
                &second_route_activities,
                self.params.second_position,
                self.params.second_insertion,
            );
        } else {
            route2.replace_activities(
                problem,
                &second_route_activities,
                self.params.second_insertion,
                self.params.second_position + 1,
            );
        };
    }

    fn updated_routes(&self) -> Vec<RouteIdx> {
        vec![self.params.first_route, self.params.second_route]
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::{
        problem::job::JobIdx,
        solver::{
            insertion::{Insertion, ServiceInsertion},
            ls::{
                r#move::LocalSearchOperator,
                swap_star::{SwapStar, SwapStarParams, TopThreeInsertions},
            },
            solution::route_id::RouteIdx,
        },
        test_utils::{self, TestRoute},
    };

    #[test]
    fn top_3_insertions_test() {
        let mut top3 = TopThreeInsertions::default();

        // The actual values don't matter here
        let test_insertion = Insertion::Service(ServiceInsertion {
            job_index: JobIdx::new(0),
            position: 0,
            route_id: RouteIdx::new(0),
        });

        top3.update(test_insertion.clone(), 10.0);
        assert_eq!(top3.delta(0), 10.0);
        assert_eq!(top3.delta(1), f64::MAX);
        assert_eq!(top3.delta(2), f64::MAX);

        top3.update(test_insertion.clone(), 5.0);
        assert_eq!(top3.delta(0), 5.0);
        assert_eq!(top3.delta(1), 10.0);
        assert_eq!(top3.delta(2), f64::MAX);

        top3.update(test_insertion.clone(), 7.0);
        assert_eq!(top3.delta(0), 5.0);
        assert_eq!(top3.delta(1), 7.0);
        assert_eq!(top3.delta(2), 10.0);

        top3.update(test_insertion.clone(), 6.0);
        assert_eq!(top3.delta(0), 5.0);
        assert_eq!(top3.delta(1), 6.0);
        assert_eq!(top3.delta(2), 7.0);

        top3.update(test_insertion.clone(), 11.0);
        assert_eq!(top3.delta(0), 5.0);
        assert_eq!(top3.delta(1), 6.0);
        assert_eq!(top3.delta(2), 7.0);
    }

    #[test]
    fn test_swap_star_both_in_place_apply() {
        let locations = test_utils::create_location_grid(10, 10);

        let services = test_utils::create_basic_services(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
        let vehicles = test_utils::create_basic_vehicles(vec![0, 0]);
        let problem = Arc::new(test_utils::create_test_problem(
            locations, services, vehicles,
        ));

        let mut solution = test_utils::create_test_working_solution(
            Arc::clone(&problem),
            vec![
                TestRoute {
                    vehicle_id: 0,
                    service_ids: vec![0, 1, 2, 3, 4, 5],
                },
                TestRoute {
                    vehicle_id: 1,
                    service_ids: vec![6, 7, 8, 9, 10],
                },
            ],
        );

        let operator = SwapStar::new(SwapStarParams {
            first_route: RouteIdx::new(0),
            second_route: RouteIdx::new(1),
            first_position: 2,  // the 2
            second_position: 3, // The 9
            first_insertion: 2,
            second_insertion: 3,
            delta: 0.0, // We don't care about the actual delta for this test
        });

        let distances = solution.route(0.into()).transport_costs(&problem)
            + solution.route(1.into()).transport_costs(&problem);
        let delta = operator.transport_cost_delta(&solution);
        operator.apply(&problem, &mut solution);
        assert_eq!(
            solution.route(0.into()).transport_costs(&problem)
                + solution.route(1.into()).transport_costs(&problem),
            distances + delta,
        );

        assert_eq!(
            solution
                .route(0.into())
                .activity_ids()
                .iter()
                .map(|activity| activity.job_id().get())
                .collect::<Vec<_>>(),
            vec![0, 1, 9, 3, 4, 5],
        );

        assert_eq!(
            solution
                .route(1.into())
                .activity_ids()
                .iter()
                .map(|activity| activity.job_id().get())
                .collect::<Vec<_>>(),
            vec![6, 7, 8, 2, 10],
        );
    }

    #[test]
    fn test_swap_star_r1_in_place_apply() {
        let locations = test_utils::create_location_grid(10, 10);

        let services = test_utils::create_basic_services(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
        let vehicles = test_utils::create_basic_vehicles(vec![0, 0]);
        let problem = Arc::new(test_utils::create_test_problem(
            locations, services, vehicles,
        ));

        let mut solution = test_utils::create_test_working_solution(
            Arc::clone(&problem),
            vec![
                TestRoute {
                    vehicle_id: 0,
                    service_ids: vec![0, 1, 2, 3, 4, 5],
                },
                TestRoute {
                    vehicle_id: 1,
                    service_ids: vec![6, 7, 8, 9, 10],
                },
            ],
        );

        let operator = SwapStar::new(SwapStarParams {
            first_route: RouteIdx::new(0),
            second_route: RouteIdx::new(1),
            first_position: 2,  // the 2
            second_position: 3, // The 9
            first_insertion: 2,
            second_insertion: 1,
            delta: 0.0, // We don't care about the actual delta for this test
        });

        let distances = solution.route(0.into()).transport_costs(&problem)
            + solution.route(1.into()).transport_costs(&problem);
        let delta = operator.transport_cost_delta(&solution);
        operator.apply(&problem, &mut solution);
        assert_eq!(
            solution.route(0.into()).transport_costs(&problem)
                + solution.route(1.into()).transport_costs(&problem),
            distances + delta,
        );

        assert_eq!(
            solution
                .route(0.into())
                .activity_ids()
                .iter()
                .map(|activity| activity.job_id().get())
                .collect::<Vec<_>>(),
            vec![0, 1, 9, 3, 4, 5],
        );

        assert_eq!(
            solution
                .route(1.into())
                .activity_ids()
                .iter()
                .map(|activity| activity.job_id().get())
                .collect::<Vec<_>>(),
            vec![6, 2, 7, 8, 10],
        );
    }

    #[test]
    fn test_swap_star_r2_in_place_apply() {
        let locations = test_utils::create_location_grid(10, 10);

        let services = test_utils::create_basic_services(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
        let vehicles = test_utils::create_basic_vehicles(vec![0, 0]);
        let problem = Arc::new(test_utils::create_test_problem(
            locations, services, vehicles,
        ));

        let mut solution = test_utils::create_test_working_solution(
            Arc::clone(&problem),
            vec![
                TestRoute {
                    vehicle_id: 0,
                    service_ids: vec![0, 1, 2, 3, 4, 5],
                },
                TestRoute {
                    vehicle_id: 1,
                    service_ids: vec![6, 7, 8, 9, 10],
                },
            ],
        );

        let operator = SwapStar::new(SwapStarParams {
            first_route: RouteIdx::new(0),
            second_route: RouteIdx::new(1),
            first_position: 2,  // the 2
            second_position: 3, // The 9
            first_insertion: 1,
            second_insertion: 3,
            delta: 0.0, // We don't care about the actual delta for this test
        });

        let distances = solution.route(0.into()).transport_costs(&problem)
            + solution.route(1.into()).transport_costs(&problem);
        let delta = operator.transport_cost_delta(&solution);
        operator.apply(&problem, &mut solution);
        assert_eq!(
            solution.route(0.into()).transport_costs(&problem)
                + solution.route(1.into()).transport_costs(&problem),
            distances + delta,
        );

        assert_eq!(
            solution
                .route(0.into())
                .activity_ids()
                .iter()
                .map(|activity| activity.job_id().get())
                .collect::<Vec<_>>(),
            vec![0, 9, 1, 3, 4, 5],
        );

        assert_eq!(
            solution
                .route(1.into())
                .activity_ids()
                .iter()
                .map(|activity| activity.job_id().get())
                .collect::<Vec<_>>(),
            vec![6, 7, 8, 2, 10],
        );
    }

    #[test]
    fn test_swap_star_none_in_place_apply() {
        let locations = test_utils::create_location_grid(10, 10);

        let services = test_utils::create_basic_services(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
        let vehicles = test_utils::create_basic_vehicles(vec![0, 0]);
        let problem = Arc::new(test_utils::create_test_problem(
            locations, services, vehicles,
        ));

        let mut solution = test_utils::create_test_working_solution(
            Arc::clone(&problem),
            vec![
                TestRoute {
                    vehicle_id: 0,
                    service_ids: vec![0, 1, 2, 3, 4, 5],
                },
                TestRoute {
                    vehicle_id: 1,
                    service_ids: vec![6, 7, 8, 9, 10],
                },
            ],
        );

        let operator = SwapStar::new(SwapStarParams {
            first_route: RouteIdx::new(0),
            second_route: RouteIdx::new(1),
            first_position: 2,  // the 2
            second_position: 3, // The 9
            first_insertion: 4,
            second_insertion: 1,
            delta: 0.0, // We don't care about the actual delta for this test
        });

        let distances = solution.route(0.into()).transport_costs(&problem)
            + solution.route(1.into()).transport_costs(&problem);
        let delta = operator.transport_cost_delta(&solution);
        operator.apply(&problem, &mut solution);
        assert_eq!(
            solution.route(0.into()).transport_costs(&problem)
                + solution.route(1.into()).transport_costs(&problem),
            distances + delta,
        );

        assert_eq!(
            solution
                .route(0.into())
                .activity_ids()
                .iter()
                .map(|activity| activity.job_id().get())
                .collect::<Vec<_>>(),
            vec![0, 1, 3, 9, 4, 5],
        );

        assert_eq!(
            solution
                .route(1.into())
                .activity_ids()
                .iter()
                .map(|activity| activity.job_id().get())
                .collect::<Vec<_>>(),
            vec![6, 2, 7, 8, 10],
        );
    }

    #[test]
    fn test_swap_star_end_of_route() {
        let locations = test_utils::create_location_grid(10, 10);

        let services = test_utils::create_basic_services(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
        let vehicles = test_utils::create_basic_vehicles(vec![0, 0]);
        let problem = Arc::new(test_utils::create_test_problem(
            locations, services, vehicles,
        ));

        let mut solution = test_utils::create_test_working_solution(
            Arc::clone(&problem),
            vec![
                TestRoute {
                    vehicle_id: 0,
                    service_ids: vec![0, 1, 2, 3, 4, 5],
                },
                TestRoute {
                    vehicle_id: 1,
                    service_ids: vec![6, 7, 8, 9, 10],
                },
            ],
        );

        let operator = SwapStar::new(SwapStarParams {
            first_route: RouteIdx::new(0),
            second_route: RouteIdx::new(1),
            first_position: 2,  // the 2
            second_position: 3, // The 9
            first_insertion: 6,
            second_insertion: 5,
            delta: 0.0, // We don't care about the actual delta for this test
        });

        let distances = solution.route(0.into()).transport_costs(&problem)
            + solution.route(1.into()).transport_costs(&problem);
        let delta = operator.transport_cost_delta(&solution);
        operator.apply(&problem, &mut solution);
        assert_eq!(
            solution.route(0.into()).transport_costs(&problem)
                + solution.route(1.into()).transport_costs(&problem),
            distances + delta,
        );

        assert_eq!(
            solution
                .route(0.into())
                .activity_ids()
                .iter()
                .map(|activity| activity.job_id().get())
                .collect::<Vec<_>>(),
            vec![0, 1, 3, 4, 5, 9],
        );

        assert_eq!(
            solution
                .route(1.into())
                .activity_ids()
                .iter()
                .map(|activity| activity.job_id().get())
                .collect::<Vec<_>>(),
            vec![6, 7, 8, 10, 2],
        );
    }

    #[test]
    fn test_swap_star_end_of_route_with_return() {
        let locations = test_utils::create_location_grid(10, 10);

        let services = test_utils::create_basic_services(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
        let mut vehicles = test_utils::create_basic_vehicles(vec![0, 0]);
        vehicles[0].set_should_return_to_depot(true);
        vehicles[1].set_should_return_to_depot(true);
        let problem = Arc::new(test_utils::create_test_problem(
            locations, services, vehicles,
        ));

        let mut solution = test_utils::create_test_working_solution(
            Arc::clone(&problem),
            vec![
                TestRoute {
                    vehicle_id: 0,
                    service_ids: vec![0, 1, 2, 3, 4, 5],
                },
                TestRoute {
                    vehicle_id: 1,
                    service_ids: vec![6, 7, 8, 9, 10],
                },
            ],
        );

        let operator = SwapStar::new(SwapStarParams {
            first_route: RouteIdx::new(0),
            second_route: RouteIdx::new(1),
            first_position: 2,  // the 2
            second_position: 3, // The 9
            first_insertion: 6,
            second_insertion: 5,
            delta: 0.0, // We don't care about the actual delta for this test
        });

        let distances = solution.route(0.into()).transport_costs(&problem)
            + solution.route(1.into()).transport_costs(&problem);
        let delta = operator.transport_cost_delta(&solution);
        operator.apply(&problem, &mut solution);
        assert_eq!(
            solution.route(0.into()).transport_costs(&problem)
                + solution.route(1.into()).transport_costs(&problem),
            distances + delta,
        );

        assert_eq!(
            solution
                .route(0.into())
                .activity_ids()
                .iter()
                .map(|activity| activity.job_id().get())
                .collect::<Vec<_>>(),
            vec![0, 1, 3, 4, 5, 9],
        );

        assert_eq!(
            solution
                .route(1.into())
                .activity_ids()
                .iter()
                .map(|activity| activity.job_id().get())
                .collect::<Vec<_>>(),
            vec![6, 7, 8, 10, 2],
        );
    }
}
