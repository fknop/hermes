use tracing::{Level, instrument};

use crate::{
    problem::{job::ActivityId, vehicle_routing_problem::VehicleRoutingProblem},
    solver::{
        ls::r#move::LocalSearchOperator,
        solution::{
            route::WorkingSolutionRoute, route_id::RouteIdx, working_solution::WorkingSolution,
        },
    },
};

/// **Intra-Route Mixed Exchange**
///
/// Swaps a single node with a segment within the same route.
///
/// ```text
/// BEFORE:
///    ... (A) -> (B) -> (C) -> [D -> E] -> (F) ...
///               ^              segment
///             node
///
/// AFTER:
///    ... (A) -> [D -> E] -> (C) -> (B) -> (F) ...
///               segment            ^
///                                node
///
/// Effect: Allows asymmetric exchanges between single stops and clusters.
/// ```
#[derive(Debug)]
pub struct MixedExchangeOperator {
    params: MixedExchangeParams,
}

#[derive(Debug)]
pub struct MixedExchangeParams {
    pub route_id: RouteIdx,
    pub position: usize,
    pub segment_start: usize,
    pub segment_length: usize,
}

impl MixedExchangeOperator {
    pub fn new(params: MixedExchangeParams) -> Self {
        if params.segment_length < 2 {
            panic!("MixedExchange: 'segment_length' must be at least 2.");
        }

        if params.segment_start < params.position
            && params.segment_start + params.segment_length > params.position
        {
            panic!("MixedExchange: the segment cannot overlap with the single position")
        }

        MixedExchangeOperator { params }
    }

    /// Returns job IDs [to, ...(from, to), from]
    fn moved_jobs<'a>(
        &'a self,
        route: &'a WorkingSolutionRoute,
    ) -> impl DoubleEndedIterator<Item = ActivityId> + Clone + 'a {
        if self.params.position < self.params.segment_start {
            route
                .activity_ids_iter(
                    self.params.segment_start,
                    self.params.segment_start + self.params.segment_length,
                )
                .chain(route.activity_ids_iter(self.params.position + 1, self.params.segment_start))
                .chain(route.activity_ids_iter(self.params.position, self.params.position + 1))
        } else {
            route
                .activity_ids_iter(self.params.position, self.params.position + 1)
                .chain(route.activity_ids_iter(
                    self.params.segment_start + self.params.segment_length,
                    self.params.position,
                ))
                .chain(route.activity_ids_iter(
                    self.params.segment_start,
                    self.params.segment_start + self.params.segment_length,
                ))
        }
    }
}

/*
 * /// **Intra-Route Mixed Exchange**
///
/// Swaps a single node with a segment within the same route.
///
/// ```text
/// BEFORE:
///    ... (A) -> (B) -> (C) -> [D -> E] -> (F) ...
///               ^              segment
///             node
///
/// AFTER:
///    ... (A) -> [D -> E] -> (C) -> (B) -> (F) ...
///               segment            ^
///                                node
///
/// Effect: Allows asymmetric exchanges between single stops and clusters.
/// ```
 */
impl LocalSearchOperator for MixedExchangeOperator {
    #[instrument(skip_all,level = Level::TRACE)]
    fn generate_moves<C>(
        _problem: &VehicleRoutingProblem,
        solution: &WorkingSolution,
        (r1, r2): (RouteIdx, RouteIdx),
        mut consumer: C,
    ) where
        C: FnMut(Self),
    {
        if r1 != r2 {
            return;
        }

        let route = solution.route(r1);

        if route.len() < 4 {
            return;
        }

        let segment_length = 2;
        for segment_start in 0..route.len() - segment_length + 1 {
            for to in 0..route.len() {
                if to == segment_start {
                    continue;
                }

                if to < segment_start && to + segment_length >= segment_start {
                    continue;
                }

                if to > segment_start && to <= segment_start + segment_length {
                    continue;
                }

                let op = MixedExchangeOperator::new(MixedExchangeParams {
                    route_id: r1,
                    position: to,
                    segment_start,
                    segment_length,
                });

                consumer(op);
            }
        }
    }

    fn transport_cost_delta(&self, solution: &WorkingSolution) -> f64 {
        let problem = solution.problem();
        let route = solution.route(self.params.route_id);
        let vehicle = route.vehicle(problem);

        let segment_start_prev = route.previous_location_id(problem, self.params.segment_start);
        let segment_start = route.location_id(problem, self.params.segment_start);
        let segment_end = route.location_id(
            problem,
            self.params.segment_start + self.params.segment_length - 1,
        );
        let segment_end_next = route.next_location_id(
            problem,
            self.params.segment_start + self.params.segment_length - 1,
        );

        let to = route.location_id(problem, self.params.position);
        let to_prev = route.previous_location_id(problem, self.params.position);
        let to_next = route.next_location_id(problem, self.params.position);

        let mut delta = 0.0;

        delta -= problem.travel_cost_or_zero(vehicle, to_prev, to);
        delta -= problem.travel_cost_or_zero(vehicle, to, to_next);
        delta -= problem.travel_cost_or_zero(vehicle, segment_start_prev, segment_start);
        delta -= problem.travel_cost_or_zero(vehicle, segment_end, segment_end_next);

        delta += problem.travel_cost_or_zero(vehicle, to_prev, segment_start);
        delta += problem.travel_cost_or_zero(vehicle, segment_end, to_next);
        delta += problem.travel_cost_or_zero(vehicle, segment_start_prev, to);
        delta += problem.travel_cost_or_zero(vehicle, to, segment_end_next);

        delta
    }

    fn fixed_route_cost_delta(&self, _solution: &WorkingSolution) -> f64 {
        0.0
    }

    fn waiting_cost_delta(&self, solution: &WorkingSolution) -> f64 {
        let problem = solution.problem();
        let route = solution.route(self.params.route_id);

        let start = self.params.segment_start.min(self.params.position);
        let end =
            (self.params.segment_start + self.params.segment_length - 1).max(self.params.position);

        let delta =
            route.waiting_duration_change_delta(problem, self.moved_jobs(route), start, end + 1);

        problem.waiting_duration_cost(delta)
    }

    fn is_valid(&self, solution: &WorkingSolution) -> bool {
        let problem = solution.problem();
        let route = solution.route(self.params.route_id);

        let start = self.params.segment_start.min(self.params.position);
        let end =
            (self.params.segment_start + self.params.segment_length - 1).max(self.params.position);

        route.is_valid_change(problem, self.moved_jobs(route), start, end + 1)
    }

    fn apply(&self, problem: &VehicleRoutingProblem, solution: &mut WorkingSolution) {
        let start = self.params.segment_start.min(self.params.position);
        let end =
            (self.params.segment_start + self.params.segment_length - 1).max(self.params.position);

        let route = solution.route_mut(self.params.route_id);
        let moved_jobs = self.moved_jobs(route).collect::<Vec<_>>();

        route.replace_activities(problem, &moved_jobs, start, end + 1)
    }

    fn updated_routes(&self) -> Vec<RouteIdx> {
        vec![self.params.route_id]
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::{
        solver::{
            ls::{
                mixed_exchange::{MixedExchangeOperator, MixedExchangeParams},
                r#move::LocalSearchOperator,
            },
            solution::route_id::RouteIdx,
        },
        test_utils::{self, TestRoute},
    };

    #[test]
    fn test_mixed_exchange() {
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
                    service_ids: vec![0, 1, 2, 3, 4, 5, 6, 7],
                },
                TestRoute {
                    vehicle_id: 1,
                    service_ids: vec![8, 9, 10],
                },
            ],
        );

        // Move [1, 2, 3] to position after 4
        let operator = MixedExchangeOperator::new(MixedExchangeParams {
            route_id: RouteIdx::new(0),
            position: 1,
            segment_start: 4,
            segment_length: 3,
        });

        let distance = solution.route(RouteIdx::new(0)).transport_costs(&problem);
        let delta = operator.transport_cost_delta(&solution);
        operator.apply(&problem, &mut solution);
        assert_eq!(
            solution
                .route(RouteIdx::new(0))
                .activity_ids()
                .iter()
                .map(|activity| activity.job_id().get())
                .collect::<Vec<_>>(),
            vec![0, 4, 5, 6, 2, 3, 1, 7],
        );
        assert_eq!(
            solution.route(RouteIdx::new(0)).transport_costs(&problem),
            distance + delta
        );
    }

    #[test]
    fn test_mixed_exchange_segment_before_position() {
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
                    service_ids: vec![0, 1, 2, 3, 4, 5, 6, 7],
                },
                TestRoute {
                    vehicle_id: 1,
                    service_ids: vec![8, 9, 10],
                },
            ],
        );

        // Move [1, 2, 3] to position after 4
        let operator = MixedExchangeOperator::new(MixedExchangeParams {
            route_id: RouteIdx::new(0),
            position: 5,
            segment_start: 1,
            segment_length: 2,
        });

        let distance = solution.route(RouteIdx::new(0)).transport_costs(&problem);
        let delta = operator.transport_cost_delta(&solution);
        operator.apply(&problem, &mut solution);
        assert_eq!(
            solution.route(RouteIdx::new(0)).transport_costs(&problem),
            distance + delta
        );

        assert_eq!(
            solution
                .route(RouteIdx::new(0))
                .activity_ids()
                .iter()
                .map(|activity| activity.job_id().get())
                .collect::<Vec<_>>(),
            vec![0, 5, 3, 4, 1, 2, 6, 7],
        );
    }
}
