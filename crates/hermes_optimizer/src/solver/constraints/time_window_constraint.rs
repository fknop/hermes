use jiff::Timestamp;

use crate::{
    problem::{
        job::ActivityId,
        time_window::{TimeWindow, TimeWindows},
        vehicle_routing_problem::VehicleRoutingProblem,
    },
    solver::{
        insertion::Insertion,
        insertion_context::InsertionContext,
        score::Score,
        score_level::ScoreLevel,
        solution::route::{RouteActivityInfo, WorkingSolutionRoute},
    },
};

use super::activity_constraint::ActivityConstraint;

pub struct TimeWindowConstraint {
    score_level: ScoreLevel,
}

impl Default for TimeWindowConstraint {
    fn default() -> Self {
        TimeWindowConstraint {
            score_level: ScoreLevel::Hard,
        }
    }
}

impl TimeWindowConstraint {
    pub fn new(score_level: ScoreLevel) -> Self {
        TimeWindowConstraint { score_level }
    }
}

impl TimeWindowConstraint {
    pub fn compute_time_window_score(
        level: ScoreLevel,
        time_windows: &TimeWindows,
        arrival_time: Timestamp,
    ) -> Score {
        Score::of(level, time_windows.overtime(arrival_time))
    }
}

impl ActivityConstraint for TimeWindowConstraint {
    fn score_level(&self) -> ScoreLevel {
        self.score_level
    }
    fn compute_score(
        &self,
        problem: &VehicleRoutingProblem,
        _route: &WorkingSolutionRoute,
        activity: &RouteActivityInfo,
    ) -> Score {
        TimeWindowConstraint::compute_time_window_score(
            self.score_level(),
            activity.job_task(problem).time_windows(),
            activity.arrival_time(),
        )
    }

    fn compute_insertion_score(&self, context: &InsertionContext) -> Score {
        let problem = context.problem();
        if !problem.has_time_windows() {
            return Score::zero();
        }

        let route = context.insertion.route(context.solution);

        match context.insertion {
            Insertion::Service(insertion) => {
                if route.is_valid_tw_change(
                    problem,
                    std::iter::once(ActivityId::Service(context.insertion.job_idx())),
                    insertion.position,
                    insertion.position,
                ) {
                    return Score::zero();
                } else if !context.insert_on_failure && self.score_level == ScoreLevel::Hard {
                    return Score::hard(1.0);
                }
            }
            Insertion::Shipment(_) => todo!(),
        }

        let mut total_score = Score::zero();

        for data in context.updated_activities_iter() {
            let job_id = data.job_id;
            let service = problem.job_task(job_id);
            total_score += TimeWindowConstraint::compute_time_window_score(
                self.score_level(),
                service.time_windows(),
                data.arrival_time,
            )
        }

        total_score
    }
}
