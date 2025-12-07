use crate::solver::solution::working_solution::WorkingSolution;

use super::{ruin_context::RuinContext, ruin_solution::RuinSolution};

// TODO: support shipments
pub struct RuinRadial;

impl RuinSolution for RuinRadial {
    fn ruin_solution<R>(
        &self,
        solution: &mut WorkingSolution,
        RuinContext {
            rng,
            num_activities_to_remove,
            problem,
            ..
        }: RuinContext<R>,
    ) where
        R: rand::Rng,
    {
        let random_location_id = problem.random_location(rng);

        for job_id in problem
            .nearest_jobs_of_location(random_location_id)
            .take(num_activities_to_remove)
        {
            solution.remove_service(job_id.index());
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::{
        problem::{job::JobId, service::ServiceId},
        solver::ruin::{
            ruin_context::RuinContext, ruin_params::RuinParams, ruin_radial::RuinRadial,
            ruin_solution::RuinSolution,
        },
        test_utils::{self, TestRoute},
    };

    #[test]
    fn test_radial_ruin_basic() {
        let locations = test_utils::create_location_grid(4, 4);

        //
        //  ASCII Schema for coordinates:
        //
        //  Y-axis
        //  ^
        //  |
        //  | (0.0, 3.0) (12) (1.0, 3.0) (13) (2.0, 3.0) (14)  (3.0, 3.0) (15)
        //  |
        //  | (0.0, 2.0) (8) (1.0, 2.0) (9) (2.0, 2.0) (10) (3.0, 2.0) (11)
        //  |
        //  | (0.0, 1.0) (4)  (1.0, 1.0) (5) (2.0, 1.0) (6) (3.0, 1.0) (7)
        //  |
        //  | (0.0, 0.0) (0)  (1.0, 0.0) (1) (2.0, 0.0) (2) (3.0, 0.0) (3)
        //  +------------------------------------------------> X-axis
        let services = test_utils::create_basic_services(vec![1, 6, 8, 10]);
        let vehicles = test_utils::create_basic_vehicles(vec![0]);
        let problem = Arc::new(test_utils::create_test_problem(
            locations, services, vehicles,
        ));

        let mut solution = test_utils::create_test_working_solution(
            Arc::clone(&problem),
            vec![TestRoute {
                vehicle_id: 0,
                service_ids: vec![0, 1, 2, 3],
            }],
        );

        let ruin_radial = RuinRadial;

        let mut rng = test_utils::MockRng::new(vec![0]);

        ruin_radial.ruin_solution(
            &mut solution,
            RuinContext {
                params: &RuinParams::default(),
                problem: &problem,
                rng: &mut rng,
                num_activities_to_remove: 2,
            },
        );

        assert_eq!(
            solution
                .route(0)
                .activities()
                .iter()
                .copied()
                .collect::<Vec<JobId>>(),
            vec![JobId::Service(1), JobId::Service(3)]
        );
    }
}
