use tracing::{Level, instrument};

use crate::{
    problem::{job::ActivityId, vehicle_routing_problem::VehicleRoutingProblem},
    solver::{
        ls::r#move::LocalSearchOperator,
        solution::{route_id::RouteIdx, working_solution::WorkingSolution},
    },
};

/// **Inter-Route Mixed Exchange**
///
/// Exchanges a single node from one route with a segment from another.
///
/// ```text
/// BEFORE:
///    Route 1: ... (A) -> (B) -> (C) -> (D) ...
///                         ^
///                       node
///    Route 2: ... (X) -> [Y -> Z] -> (W) ...
///                        <─ seg ─>
///
/// AFTER:
///    Route 1: ... (A) -> [Y -> Z] -> (C) -> (D) ...
///                        <─ seg ─>
///    Route 2: ... (X) -> (B) -> (W) ...
///                         ^
///
/// Effect: Asymmetric exchange useful when routes have uneven density.
/// ```
#[derive(Debug)]
pub struct InterMixedExchange {
    params: InterMixedExchangeParams,
}

#[derive(Debug)]
pub struct InterMixedExchangeParams {
    pub from_route_id: RouteIdx,
    pub to_route_id: RouteIdx,

    /// Position is in the 'from' route
    pub position: usize,

    /// Segment start is in the 'to' route
    pub segment_start: usize,
    pub segment_length: usize,
}

impl InterMixedExchange {
    pub fn new(params: InterMixedExchangeParams) -> Self {
        if params.segment_length < 2 {
            panic!("MixedExchange: 'segment_length' must be at least 2.");
        }

        InterMixedExchange { params }
    }

    fn r1_moved_jobs<'a>(
        &'a self,
        solution: &'a WorkingSolution,
    ) -> impl DoubleEndedIterator<Item = ActivityId> + Clone + 'a {
        let route = solution.route(self.params.from_route_id);
        route.activity_ids_iter(self.params.position, self.params.position + 1)
    }

    fn r2_moved_jobs<'a>(
        &'a self,
        solution: &'a WorkingSolution,
    ) -> impl DoubleEndedIterator<Item = ActivityId> + Clone + 'a {
        let route = solution.route(self.params.to_route_id);
        route.activity_ids_iter(
            self.params.segment_start,
            self.params.segment_start + self.params.segment_length,
        )
    }
}

impl LocalSearchOperator for InterMixedExchange {
    #[instrument(skip_all,level = Level::TRACE)]
    fn generate_moves<C>(
        problem: &VehicleRoutingProblem,
        solution: &WorkingSolution,
        (r1, r2): (RouteIdx, RouteIdx),
        mut consumer: C,
    ) where
        C: FnMut(Self),
    {
        if r1 == r2 {
            return;
        }

        let route1 = solution.route(r1);
        let route2 = solution.route(r2);

        if !route1.bbox_intersects(route2) {
            return;
        }

        let segment_length = 2;
        if route1.will_break_maximum_activities(problem, segment_length - 1) {
            return;
        }

        for position in 0..route1.len() {
            for segment_start in 0..route2.len().saturating_sub(segment_length) {
                let params = InterMixedExchangeParams {
                    from_route_id: r1,
                    to_route_id: r2,
                    position,
                    segment_start,
                    segment_length,
                };
                let operator = InterMixedExchange::new(params);
                consumer(operator)
            }
        }
    }

    fn transport_cost_delta(&self, solution: &WorkingSolution) -> f64 {
        let problem = solution.problem();
        let r1 = solution.route(self.params.from_route_id);
        let r2 = solution.route(self.params.to_route_id);
        let v1 = r1.vehicle(problem);
        let v2 = r2.vehicle(problem);

        let from = r1.location_id(problem, self.params.position);
        let from_prev = r1.previous_location_id(problem, self.params.position);
        let from_next = r1.next_location_id(problem, self.params.position);

        let segment_start = r2.location_id(problem, self.params.segment_start);
        let segment_end = r2.location_id(
            problem,
            self.params.segment_start + self.params.segment_length - 1,
        );
        let segment_start_prev = r2.previous_location_id(problem, self.params.segment_start);
        let segment_end_next = r2.next_location_id(
            problem,
            self.params.segment_start + self.params.segment_length - 1,
        );

        let mut delta = 0.0;

        // R1 changes
        delta -= problem.travel_cost_or_zero(v1, from_prev, from);
        delta -= problem.travel_cost_or_zero(v1, from, from_next);
        delta += problem.travel_cost_or_zero(v1, from_prev, segment_start);
        delta += problem.travel_cost_or_zero(v1, segment_end, from_next);

        // R2 changes
        delta -= problem.travel_cost_or_zero(v2, segment_start_prev, segment_start);
        delta -= problem.travel_cost_or_zero(v2, segment_end, segment_end_next);
        delta += problem.travel_cost_or_zero(v2, segment_start_prev, from);
        delta += problem.travel_cost_or_zero(v2, from, segment_end_next);

        delta
    }

    fn fixed_route_cost_delta(&self, _solution: &WorkingSolution) -> f64 {
        0.0
    }

    fn waiting_cost_delta(&self, solution: &WorkingSolution) -> f64 {
        let problem = solution.problem();
        let r1 = solution.route(self.params.from_route_id);
        let r2 = solution.route(self.params.to_route_id);

        let r1_activity_ids = self.r1_moved_jobs(solution);
        let r2_activity_ids = self.r2_moved_jobs(solution);

        let delta = r1.waiting_duration_change_delta(
            problem,
            r2_activity_ids,
            self.params.position,
            self.params.position + 1,
        ) + r2.waiting_duration_change_delta(
            problem,
            r1_activity_ids,
            self.params.segment_start,
            self.params.segment_start + self.params.segment_length,
        );

        problem.waiting_duration_cost(delta)
    }

    fn is_valid(&self, solution: &WorkingSolution) -> bool {
        let problem = solution.problem();
        let r1 = solution.route(self.params.from_route_id);
        let r2 = solution.route(self.params.to_route_id);

        let r1_activity_ids = self.r1_moved_jobs(solution);
        let r2_activity_ids = self.r2_moved_jobs(solution);

        r1.is_valid_change(
            problem,
            r2_activity_ids,
            self.params.position,
            self.params.position + 1,
        ) && r2.is_valid_change(
            problem,
            r1_activity_ids,
            self.params.segment_start,
            self.params.segment_start + self.params.segment_length,
        )
    }

    fn apply(&self, problem: &VehicleRoutingProblem, solution: &mut WorkingSolution) {
        let r1_activity_ids = self.r1_moved_jobs(solution).collect::<Vec<_>>();
        let r2_activity_ids = self.r2_moved_jobs(solution).collect::<Vec<_>>();

        let r1 = solution.route_mut(self.params.from_route_id);

        r1.replace_activities(
            problem,
            &r2_activity_ids,
            self.params.position,
            self.params.position + 1,
        );

        let r2 = solution.route_mut(self.params.to_route_id);

        r2.replace_activities(
            problem,
            &r1_activity_ids,
            self.params.segment_start,
            self.params.segment_start + self.params.segment_length,
        );
    }

    fn updated_routes(&self) -> Vec<RouteIdx> {
        vec![self.params.from_route_id, self.params.to_route_id]
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::{
        solver::{
            ls::{
                inter_mixed_exchange::{InterMixedExchange, InterMixedExchangeParams},
                r#move::LocalSearchOperator,
            },
            solution::route_id::RouteIdx,
        },
        test_utils::{self, TestRoute},
    };

    #[test]
    fn test_inter_mixed_exchange() {
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
        let operator = InterMixedExchange::new(InterMixedExchangeParams {
            from_route_id: RouteIdx::new(1),
            to_route_id: RouteIdx::new(0),
            position: 1, // 9
            segment_start: 2,
            segment_length: 3, // 2, 3, 4
        });

        let distance0 = solution.route(RouteIdx::new(0)).transport_costs(&problem);
        let distance1 = solution.route(RouteIdx::new(1)).transport_costs(&problem);
        let delta = operator.transport_cost_delta(&solution);
        operator.apply(&problem, &mut solution);
        assert_eq!(
            solution.route(RouteIdx::new(0)).transport_costs(&problem)
                + solution.route(RouteIdx::new(1)).transport_costs(&problem),
            distance0 + distance1 + delta
        );

        assert_eq!(
            solution
                .route(RouteIdx::new(0))
                .activity_ids()
                .iter()
                .map(|activity| activity.job_id().get())
                .collect::<Vec<_>>(),
            vec![0, 1, 9, 5, 6, 7],
        );

        assert_eq!(
            solution
                .route(RouteIdx::new(1))
                .activity_ids()
                .iter()
                .map(|activity| activity.job_id().get())
                .collect::<Vec<_>>(),
            vec![8, 2, 3, 4, 10],
        );
    }
}
