use crate::{
    problem::vehicle_routing_problem::VehicleRoutingProblem,
    solver::{
        constraints::activity_constraint::ActivityConstraint,
        insertion_context::InsertionContext,
        score::Score,
        score_level::ScoreLevel,
        solution::route::{RouteActivityInfo, WorkingSolutionRoute},
    },
};

const SCORE_LEVEL: ScoreLevel = ScoreLevel::Hard;

#[derive(Clone)]
pub struct SkillConstraint;

impl ActivityConstraint for SkillConstraint {
    fn score_level(&self) -> ScoreLevel {
        SCORE_LEVEL
    }

    fn compute_score(
        &self,
        problem: &VehicleRoutingProblem,
        route: &WorkingSolutionRoute,
        activity: &RouteActivityInfo,
    ) -> Score {
        let vehicle = route.vehicle(problem);
        let job = activity.job(problem);

        if job.skills_satisfied_by_vehicle(vehicle) {
            Score::zero()
        } else {
            panic!("bug: should not be possible to break skill constraint");
        }
    }

    fn compute_insertion_score(&self, context: &InsertionContext) -> Score {
        let vehicle = context.route().vehicle(context.problem());
        let job = context.problem.job(context.insertion.job_idx());

        if job.skills_satisfied_by_vehicle(vehicle) {
            Score::zero()
        } else {
            panic!("bug: should not be possible to break skill constraint");
        }
    }
}
