use crate::solver::{
    insertion::Insertion, insertion_context::InsertionContext, score::Score,
    score_level::ScoreLevel, solution::working_solution::WorkingSolution,
};

use super::global_constraint::GlobalConstraint;

pub struct TransportCostConstraint;

// TODO: update this weight
pub const TRANSPORT_COST_WEIGHT: f64 = 1.0; //70.0;

const SCORE_LEVEL: ScoreLevel = ScoreLevel::Soft;

impl GlobalConstraint for TransportCostConstraint {
    fn score_level(&self) -> ScoreLevel {
        SCORE_LEVEL
    }

    fn compute_score(&self, solution: &WorkingSolution) -> Score {
        let problem = solution.problem();

        let mut cost = 0.0;
        for route in solution.non_empty_routes_iter() {
            let vehicle = route.vehicle(problem);

            if let Some(depot_location_id) = vehicle.depot_location_id() {
                cost += problem.travel_cost(
                    vehicle,
                    depot_location_id,
                    route.first().job_task(problem).location_id(),
                );

                if vehicle.should_return_to_depot() {
                    cost += problem.travel_cost(
                        vehicle,
                        route.last().job_task(problem).location_id(),
                        depot_location_id,
                    )
                }
            }

            for (index, &job_id) in route.activity_ids().iter().enumerate() {
                if index == 0 {
                    // Skip the first activity, as it is already counted with the depot
                    continue;
                }

                let previous_activity_job_id = route.activity_ids()[index - 1];
                cost += problem.travel_cost(
                    vehicle,
                    problem.job_task(previous_activity_job_id).location_id(),
                    problem.job_task(job_id).location_id(),
                )
            }
        }

        Score::of(self.score_level(), cost * TRANSPORT_COST_WEIGHT)
    }

    fn compute_insertion_score(&self, context: &InsertionContext) -> Score {
        let problem = context.problem();

        let route = context.route();

        let position = match context.insertion {
            Insertion::Service(service_insertion) => service_insertion.position,
            Insertion::Shipment(_) => unimplemented!(),
        };

        let previous_location_id = route.previous_location_id(problem, position);
        let next_location_id = route.location_id(problem, position);

        let location_id = match &context.insertion {
            Insertion::Service(service_insertion) => {
                problem.service(service_insertion.job_index).location_id()
            }
            Insertion::Shipment(_) => unimplemented!(),
        };

        let old_cost = problem.travel_cost_or_zero(
            route.vehicle(problem),
            previous_location_id,
            next_location_id,
        );

        let mut new_cost = 0.0;

        new_cost += problem.travel_cost_or_zero(
            route.vehicle(problem),
            previous_location_id,
            Some(location_id),
        );
        new_cost += problem.travel_cost_or_zero(
            route.vehicle(problem),
            Some(location_id),
            next_location_id,
        );

        let travel_cost_delta = new_cost - old_cost;

        Score::of(
            self.score_level(),
            travel_cost_delta * TRANSPORT_COST_WEIGHT,
        )
    }
}
