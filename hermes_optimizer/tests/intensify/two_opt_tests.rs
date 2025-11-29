use std::sync::Arc;

use hermes_optimizer::solver::intensify::{
    intensify_operator::IntensifyOp,
    two_opt::{TwoOptOperator, TwoOptParams},
};

use crate::test_utils::{self, TestRoute};

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

    let two_opt = TwoOptOperator::new(TwoOptParams {
        route_id: 0,
        from: 1,
        to: 4,
    });

    let delta = two_opt.delta(&solution);

    assert_eq!(delta, 6.0);

    two_opt.apply(&problem, &mut solution);

    assert_eq!(
        solution
            .route(0)
            .activities()
            .iter()
            .map(|activity| activity.service_id())
            .collect::<Vec<_>>(),
        vec![0, 4, 3, 2, 1, 5]
    );
}
