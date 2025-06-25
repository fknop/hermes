use crate::solver::insertion_context::InsertionContext;

use super::{
    activity_constraint::{ActivityConstraint, ActivityConstraintType},
    global_constraint::{GlobalConstraint, GlobalConstraintType},
    route_constraint::{RouteConstraint, RouteConstraintType},
};

pub enum Constraint {
    Global(GlobalConstraintType),
    Route(RouteConstraintType),
    Activity(ActivityConstraintType),
}

impl Constraint {
    pub fn compute_insertion_score(
        &self,
        context: &InsertionContext,
    ) -> crate::solver::score::Score {
        match self {
            Constraint::Global(constraint) => constraint.compute_insertion_score(context),
            Constraint::Route(constraint) => constraint.compute_insertion_score(context),
            Constraint::Activity(constraint) => constraint.compute_insertion_score(context),
        }
    }

    pub fn constraint_name(&self) -> &'static str {
        match self {
            Constraint::Global(c) => c.constraint_name(),
            Constraint::Route(c) => c.constraint_name(),
            Constraint::Activity(c) => c.constraint_name(),
        }
    }
}
