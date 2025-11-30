use crate::{
    problem::{
        amount::AmountExpression,
        capacity::{Capacity, is_capacity_satisfied, over_capacity_demand},
        service::ServiceType,
        vehicle_routing_problem::VehicleRoutingProblem,
    },
    solver::{
        insertion_context::InsertionContext,
        score::Score,
        score_level::ScoreLevel,
        solution::route::WorkingSolutionRoute,
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

impl CapacityConstraint {
    fn compute_capacity_score(
        &self,
        vehicle_capacity: &Capacity,
        initial_load: &Capacity,
        cumulative_load: &Capacity,
    ) -> Score {
        let load = initial_load + cumulative_load;
        if is_capacity_satisfied(vehicle_capacity, &load) {
            Score::zero()
        } else {
            Score::of(
                self.score_level,
                over_capacity_demand(vehicle_capacity, &load),
            )
        }
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
        let service = problem.service(context.insertion.service_id());

        if service.demand().is_empty() {
            return Score::zero();
        }

        let mut score = Score::zero();

        let route = context.insertion.route(context.solution);
        let vehicle = route.vehicle(problem);

        match service.service_type() {
            ServiceType::Pickup => {
                if !is_capacity_satisfied(
                    vehicle.capacity(),
                    &(service.demand() + route.bwd_load_peak(context.insertion.position())),
                ) {
                    score += Score::of(
                        self.score_level,
                        over_capacity_demand(
                            vehicle.capacity(),
                            &(service.demand() + route.bwd_load_peak(context.insertion.position())),
                        ),
                    );
                }
            }
            ServiceType::Delivery => {
                if !is_capacity_satisfied(
                    vehicle.capacity(),
                    &(service.demand() + route.fwd_load_peak(context.insertion.position())),
                ) {
                    score += Score::of(
                        self.score_level,
                        over_capacity_demand(
                            vehicle.capacity(),
                            &(service.demand() + route.fwd_load_peak(context.insertion.position())),
                        ),
                    );
                }
            }
        }

        score
    }
}
