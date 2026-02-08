use tracing::{Level, instrument};

use crate::{
    problem::vehicle_routing_problem::VehicleRoutingProblem,
    solver::{
        ls::r#move::LocalSearchOperator,
        solution::{route_id::RouteIdx, working_solution::WorkingSolution},
    },
};

/// **Intra-Route 2-Opt**
///
/// Reverses the sequence of activities between `from` and `to` (inclusive).
/// This eliminates crossing edges within a single route.
///
/// ```text
/// BEFORE:
///    ... (prev) --x--> [from] -> ... -> [to] --x--> (next) ...
///          ^             ^               ^            ^
///          A             B               C            D
///
/// AFTER (Sequence Reversed):
///    ... (prev) -----> [to] -> ... -> [from] -----> (next) ...
///          ^             ^               ^            ^
///          A             C               B            D
///
/// Edges Removed: (prev->from), (to->next)
/// Edges Added:   (prev->to),   (from->next)
/// ```
#[derive(Debug)]
pub struct TwoOptOperator {
    params: TwoOptParams,
}

#[derive(Debug)]
pub struct TwoOptParams {
    pub route_id: RouteIdx,
    pub from: usize,
    pub to: usize,
}

impl TwoOptOperator {
    pub fn new(params: TwoOptParams) -> Self {
        if params.from >= params.to {
            panic!("TwoOpt: cannot have from >= to")
        }

        TwoOptOperator { params }
    }
}

impl LocalSearchOperator for TwoOptOperator {
    #[instrument(skip_all,level = Level::DEBUG)]
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
            return; // need at least 4 activities to perform 2-opt
        }

        for from in 0..route.activity_ids().len() - 2 {
            for to in (from + 2)..route.activity_ids().len() {
                let op = TwoOptOperator::new(TwoOptParams {
                    route_id: r1,
                    from,
                    to,
                });

                consumer(op)
            }
        }
    }

    fn transport_cost_delta(&self, solution: &WorkingSolution) -> f64 {
        let problem = solution.problem();
        let route = solution.route(self.params.route_id);

        let (_, bwd_delta) = route.transport_cost_delta_update(
            problem,
            self.params.from,
            self.params.to + 1,
            route,
            self.params.from,
            self.params.to + 1,
        );

        bwd_delta
    }

    fn fixed_route_cost_delta(&self, _solution: &WorkingSolution) -> f64 {
        0.0
    }

    fn waiting_cost_delta(&self, solution: &WorkingSolution) -> f64 {
        let route = solution.route(self.params.route_id);

        let delta = route.waiting_duration_change_delta(
            solution.problem(),
            route
                .activity_ids_iter(self.params.from, self.params.to + 1)
                .rev(),
            self.params.from,
            self.params.to + 1,
        );

        solution.problem().waiting_duration_cost(delta)
    }

    fn is_valid(&self, solution: &WorkingSolution) -> bool {
        let route = solution.route(self.params.route_id);

        route.is_valid_change(
            solution.problem(),
            route
                .activity_ids_iter(self.params.from, self.params.to + 1)
                .rev(),
            self.params.from,
            self.params.to + 1,
        )
    }

    fn apply(&self, problem: &VehicleRoutingProblem, solution: &mut WorkingSolution) {
        let route = solution.route_mut(self.params.route_id);
        let job_ids = route
            .activity_ids_iter(self.params.from, self.params.to + 1)
            .rev()
            .collect::<Vec<_>>();
        route.replace_activities(problem, &job_ids, self.params.from, self.params.to + 1);
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
                r#move::LocalSearchOperator,
                two_opt::{TwoOptOperator, TwoOptParams},
            },
            solution::route_id::RouteIdx,
        },
        test_utils::{self, TestRoute},
    };

    #[test]
    fn test_two_opt() {
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

        let operator = TwoOptOperator::new(TwoOptParams {
            route_id: RouteIdx::new(0),
            from: 1,
            to: 4,
        });

        let delta = operator.transport_cost_delta(&solution);

        assert_eq!(delta, 6.0);

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
            vec![0, 4, 3, 2, 1, 5]
        );
    }

    #[test]
    fn test_two_opt_asymmetric() {
        let locations = test_utils::create_location_grid(10, 10);

        let services = test_utils::create_basic_services(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
        let vehicles = test_utils::create_basic_vehicles(vec![0, 0]);
        let problem = Arc::new(test_utils::create_asymmetric_test_problem(
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

        let operator = TwoOptOperator::new(TwoOptParams {
            route_id: RouteIdx::new(0),
            from: 1,
            to: 4,
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
            vec![0, 4, 3, 2, 1, 5]
        );
    }

    #[test]
    fn test_two_opt_end_of_route() {
        let locations = test_utils::create_location_grid(6, 6);

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

        let operator = TwoOptOperator::new(TwoOptParams {
            route_id: RouteIdx::new(0),
            from: 1,
            to: 5,
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
            vec![0, 5, 4, 3, 2, 1]
        );

        let operator = TwoOptOperator::new(TwoOptParams {
            route_id: RouteIdx::new(0),
            from: 1,
            to: 4,
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
            vec![0, 2, 3, 4, 5, 1]
        );
    }

    #[test]
    fn test_two_opt_asymmetric_end_of_route() {
        let locations = test_utils::create_location_grid(10, 10);

        let services = test_utils::create_basic_services(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
        let vehicles = test_utils::create_basic_vehicles(vec![0, 0]);
        let problem = Arc::new(test_utils::create_asymmetric_test_problem(
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

        let operator = TwoOptOperator::new(TwoOptParams {
            route_id: RouteIdx::new(0),
            from: 1,
            to: 5,
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
            vec![0, 5, 4, 3, 2, 1]
        );
    }
}
