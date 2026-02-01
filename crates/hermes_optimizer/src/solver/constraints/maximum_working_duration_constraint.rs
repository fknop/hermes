use crate::{
    problem::vehicle_routing_problem::VehicleRoutingProblem,
    solver::{
        constraints::maximum_working_duration_constraint, insertion_context::InsertionContext,
        score::Score, score_level::ScoreLevel, solution::route::WorkingSolutionRoute,
    },
};

use super::route_constraint::RouteConstraint;

pub struct MaximumWorkingDurationConstraint;

const SCORE_LEVEL: ScoreLevel = ScoreLevel::Hard;

impl RouteConstraint for MaximumWorkingDurationConstraint {
    fn score_level(&self) -> ScoreLevel {
        SCORE_LEVEL
    }

    fn compute_score(
        &self,
        problem: &VehicleRoutingProblem,
        route: &WorkingSolutionRoute,
    ) -> Score {
        let vehicle = route.vehicle(problem);
        if let Some(maximum_working_duration) = vehicle.maximum_working_duration() {
            let working_duration = route.end(problem).duration_since(route.start(problem));
            if working_duration > maximum_working_duration {
                return Score::of(
                    self.score_level(),
                    working_duration.as_secs_f64() - maximum_working_duration.as_secs_f64(),
                );
            }
        }

        Score::zero()
    }

    fn compute_insertion_score(&self, context: &InsertionContext) -> Score {
        let problem = context.problem();
        let route = context.route();
        let vehicle = route.vehicle(problem);

        if vehicle.maximum_working_duration().is_none() {
            return Score::zero();
        }

        let maximum_working_duration = vehicle.maximum_working_duration().unwrap(); // Checked just above
        let new_start = context.compute_vehicle_start();
        let new_end = context.compute_vehicle_end();
        let new_working_duration = new_end.duration_since(new_start);

        if route.is_empty() {
            if new_working_duration > maximum_working_duration {
                return Score::of(
                    self.score_level(),
                    new_working_duration.as_secs_f64() - maximum_working_duration.as_secs_f64(),
                );
            }
        } else {
            // && working_duration > maximum_working_duration
            let current_working_duration = route.end(problem).duration_since(route.start(problem));

            // New violation, old route was not violating the constraint
            if new_working_duration > maximum_working_duration
                && current_working_duration <= maximum_working_duration
            {
                return Score::of(
                    self.score_level(),
                    new_working_duration.as_secs_f64() - maximum_working_duration.as_secs_f64(),
                );

                // Both are violating the constraint, we compute the delta between the two
            } else if current_working_duration > maximum_working_duration
                && new_working_duration > maximum_working_duration
            {
                return Score::of(
                    self.score_level(),
                    (new_working_duration - current_working_duration).as_secs_f64(),
                );
                // Current duration is violating, new one is not
            } else if current_working_duration > maximum_working_duration
                && new_working_duration <= maximum_working_duration
            {
                return Score::of(
                    self.score_level(),
                    (maximum_working_duration - current_working_duration).as_secs_f64(),
                );
            } else {
                return Score::zero();
            }
        }

        Score::zero()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use jiff::SignedDuration;

    use crate::{
        problem::{
            capacity::Capacity,
            fleet::Fleet,
            job::JobIdx,
            service::ServiceBuilder,
            time_window::TimeWindow,
            travel_cost_matrix::TravelMatrices,
            vehicle::{VehicleBuilder, VehicleShift},
            vehicle_profile::VehicleProfile,
            vehicle_routing_problem::{VehicleRoutingProblem, VehicleRoutingProblemBuilder},
        },
        solver::{
            constraints::{
                maximum_working_duration_constraint::MaximumWorkingDurationConstraint,
                route_constraint::RouteConstraint,
            },
            insertion::{Insertion, ServiceInsertion},
            score::Score,
            solution::{route_id::RouteIdx, working_solution::WorkingSolution},
        },
        test_utils,
    };

    fn create_problem() -> VehicleRoutingProblem {
        // 10 locations from (0, 0) to (9, 0)
        let locations = test_utils::create_location_grid(1, 10);

        let mut vehicle_builder = VehicleBuilder::default();
        vehicle_builder.set_depot_location_id(0);
        vehicle_builder.set_capacity(Capacity::from_vec(vec![40.0]));
        vehicle_builder.set_vehicle_id(String::from("vehicle"));
        vehicle_builder.set_profile_id(0);
        vehicle_builder.set_depot_duration(SignedDuration::from_mins(10));
        vehicle_builder.set_vehicle_shift(VehicleShift {
            earliest_start: Some("2025-11-30T08:00:00+02:00".parse().unwrap()),
            latest_start: Some("2025-11-30T08:00:00+02:00".parse().unwrap()),
            latest_end: None,
            maximum_working_duration: Some(SignedDuration::from_hours(1)),
            maximum_transport_duration: None,
        });
        let vehicle = vehicle_builder.build();
        let vehicles = vec![vehicle];

        let mut service_builder = ServiceBuilder::default();
        service_builder.set_external_id(String::from("service_1"));
        service_builder.set_service_duration(SignedDuration::from_mins(10));
        service_builder.set_time_window(TimeWindow::from_iso(
            Some("2025-11-30T08:00:00+02:00"),
            Some("2025-11-30T09:00:00+02:00"),
        ));
        service_builder.set_location_id(1);
        let service_1 = service_builder.build();

        let mut service_builder = ServiceBuilder::default();
        service_builder.set_external_id(String::from("service_2"));
        service_builder.set_service_duration(SignedDuration::from_mins(10));
        service_builder.set_time_window(TimeWindow::from_iso(
            Some("2025-11-30T10:00:00+02:00"),
            Some("2025-11-30T12:00:00+02:00"),
        ));
        service_builder.set_location_id(2);
        let service_2 = service_builder.build();

        let mut service_builder = ServiceBuilder::default();
        service_builder.set_external_id(String::from("service_3"));
        service_builder.set_service_duration(SignedDuration::from_mins(10));
        service_builder.set_time_window(TimeWindow::from_iso(
            Some("2025-11-30T10:00:00+02:00"),
            Some("2025-11-30T12:00:00+02:00"),
        ));
        service_builder.set_location_id(3);
        let service_3 = service_builder.build();

        let services = vec![service_1, service_2, service_3];

        let mut builder = VehicleRoutingProblemBuilder::default();

        builder.set_vehicle_profiles(vec![VehicleProfile::new(
            "test_profile".to_owned(),
            TravelMatrices::from_constant(
                &locations,
                SignedDuration::from_mins(30).as_secs_f64(),
                100.0,
                SignedDuration::from_mins(30).as_secs_f64(),
            ),
        )]);

        builder.set_locations(locations);
        builder.set_fleet(Fleet::Finite(vehicles));
        builder.set_services(services);

        builder.build()
    }

    #[test]
    fn test_maximum_working_duration_constraint() {
        let problem = Arc::new(create_problem());
        let mut solution = WorkingSolution::new(problem.clone());

        solution.insert(&Insertion::Service(ServiceInsertion {
            job_index: JobIdx::new(0),
            position: 0,
            route_id: RouteIdx::new(0),
        }));

        let constraint = MaximumWorkingDurationConstraint;

        {
            let route = solution.route(0.into());
            let score = constraint.compute_score(&problem, route);
            assert_eq!(score, Score::ZERO);

            let duration = route.end(&problem).duration_since(route.start(&problem));
            assert_eq!(duration, SignedDuration::from_mins(50));
        }

        solution.insert(&Insertion::Service(ServiceInsertion {
            job_index: JobIdx::new(1),
            position: 1,
            route_id: RouteIdx::new(0),
        }));

        {
            let route = solution.route(0.into());
            let duration = route.end(&problem).duration_since(route.start(&problem));
            assert_eq!(duration, SignedDuration::from_mins(130));

            let score = constraint.compute_score(&problem, route);
            assert_eq!(
                score,
                Score::hard(SignedDuration::from_mins(130 - 60).as_secs_f64())
            );
        }
    }
}
