use crate::{
    problem::{
        amount::AmountExpression,
        capacity::{is_capacity_satisfied, over_capacity_demand},
        job::Job,
        service::ServiceType,
        vehicle_routing_problem::VehicleRoutingProblem,
    },
    solver::{
        insertion::Insertion, insertion_context::InsertionContext, score::Score,
        score_level::ScoreLevel, solution::route::WorkingSolutionRoute,
    },
};

use super::route_constraint::RouteConstraint;

pub struct CapacityConstraint {
    score_level: ScoreLevel,
}

impl Default for CapacityConstraint {
    fn default() -> Self {
        CapacityConstraint {
            score_level: ScoreLevel::Hard,
        }
    }
}

impl CapacityConstraint {
    pub fn new(score_level: ScoreLevel) -> Self {
        CapacityConstraint { score_level }
    }
}

impl RouteConstraint for CapacityConstraint {
    fn score_level(&self) -> ScoreLevel {
        self.score_level
    }

    fn compute_score(
        &self,
        problem: &VehicleRoutingProblem,
        route: &WorkingSolutionRoute,
    ) -> Score {
        if !problem.has_capacity() {
            return Score::zero();
        }

        let vehicle = route.vehicle(problem);
        let mut score = Score::zero();

        for load in route.current_loads() {
            if !is_capacity_satisfied(vehicle.capacity(), &load) {
                score += Score::of(
                    self.score_level,
                    over_capacity_demand(vehicle.capacity(), &load),
                );
            }
        }

        score
    }

    fn compute_insertion_score(&self, context: &InsertionContext) -> Score {
        let problem = context.problem();
        let job = problem.job(context.insertion.job_index());

        if job.demand().is_empty() {
            return Score::zero();
        }

        let mut score = Score::zero();

        let route = context.insertion.route(context.solution);
        let vehicle = route.vehicle(problem);

        match &context.insertion {
            &Insertion::Service(insertion) => {
                let service = problem.service(insertion.job_index);
                match service.service_type() {
                    ServiceType::Pickup => {
                        if !is_capacity_satisfied(
                            vehicle.capacity(),
                            &(service.demand() + route.bwd_load_peak(insertion.position)),
                        ) {
                            score += Score::of(
                                self.score_level,
                                over_capacity_demand(
                                    vehicle.capacity(),
                                    &(service.demand() + route.bwd_load_peak(insertion.position)),
                                ),
                            )
                        }
                    }
                    ServiceType::Delivery => {
                        if !is_capacity_satisfied(
                            vehicle.capacity(),
                            &(service.demand() + route.fwd_load_peak(insertion.position)),
                        ) {
                            score += Score::of(
                                self.score_level,
                                over_capacity_demand(
                                    vehicle.capacity(),
                                    &(service.demand() + route.fwd_load_peak(insertion.position)),
                                ),
                            );
                        }
                    }
                }
            }
            _ => unimplemented!(),
        }

        score
    }
}
