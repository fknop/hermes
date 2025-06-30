use crate::solver::{
    insertion::Insertion, insertion_context::InsertionContext, score::Score,
    working_solution::WorkingSolutionRoute,
};

use super::route_constraint::RouteConstraint;

pub struct VehicleCostConstraint;

const VEHICLE_COST: i64 = 2000;

impl RouteConstraint for VehicleCostConstraint {
    fn compute_score(&self, _: &WorkingSolutionRoute) -> Score {
        Score::soft(VEHICLE_COST)
    }

    fn compute_insertion_score(&self, context: &InsertionContext) -> Score {
        match context.insertion {
            Insertion::ExistingRoute(_) => Score::soft(0),
            Insertion::NewRoute(_) => Score::soft(VEHICLE_COST),
        }
    }
}
