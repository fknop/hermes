use super::{
    activity_constraint::ActivityConstraintType,
    global_constraint::{GlobalConstraint, GlobalConstraintType},
    route_constraint::RouteConstraintType,
};

pub enum Constraint {
    Global(GlobalConstraintType),
    Route(RouteConstraintType),
    Activity(ActivityConstraintType),
}

impl Constraint {
    pub fn constraint_name(&self) -> &'static str {
        match self {
            Constraint::Global(c) => todo!(),
            Constraint::Route(c) => todo!(),
            Constraint::Activity(c) => todo!(),
        }
    }
}
