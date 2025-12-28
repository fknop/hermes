use jiff::{SignedDuration, Timestamp};

use crate::{
    problem::{job::ActivityId, vehicle_routing_problem::VehicleRoutingProblem},
    solver::solution::{
        route::WorkingSolutionRoute,
        utils::{
            compute_activity_arrival_time, compute_departure_time,
            compute_first_activity_arrival_time, compute_waiting_duration,
        },
    },
};

#[derive(PartialEq, Eq, Debug)]
pub struct RouteUpdateActivityData {
    pub arrival_time: Timestamp,
    pub departure_time: Timestamp,
    pub waiting_duration: SignedDuration,
    pub job_id: ActivityId,
    pub current_position: Option<usize>,
}

pub struct RouteUpdateIterator<'a, I> {
    problem: &'a VehicleRoutingProblem,
    route: &'a WorkingSolutionRoute,
    jobs_iter: I,

    succeeding_iter: std::slice::Iter<'a, ActivityId>,

    index: usize,
    end: usize,

    previous_job_id: Option<ActivityId>,
    previous_departure_time: Option<Timestamp>,
}

impl<'a, I> RouteUpdateIterator<'a, I>
where
    I: Iterator<Item = ActivityId>,
{
    pub fn new(
        problem: &'a VehicleRoutingProblem,
        route: &'a WorkingSolutionRoute,
        jobs_iter: I,
        start: usize,
        end: usize,
    ) -> Self {
        let succeeding_activities = if end < route.activity_ids.len() {
            &route.activity_ids[end..]
        } else {
            &[]
        };

        let previous_activity = if start > 0 {
            Some(&route.activity(start - 1))
        } else {
            None
        };

        RouteUpdateIterator {
            problem,
            route,
            end,
            index: start,
            jobs_iter,
            succeeding_iter: succeeding_activities.iter(),
            previous_job_id: previous_activity.map(|activity| activity.activity_id),
            previous_departure_time: previous_activity.map(|activity| activity.departure_time),
        }
    }
}

impl<I> Iterator for RouteUpdateIterator<'_, I>
where
    I: Iterator<Item = ActivityId>,
{
    type Item = RouteUpdateActivityData;

    fn next(&mut self) -> Option<Self::Item> {
        let mut job_id = self.jobs_iter.next();

        if job_id.is_none() && self.index >= self.end {
            job_id = self.succeeding_iter.next().copied();
        }

        if let Some(job_id) = job_id {
            let arrival_time = if let Some(previous_job_id) = self.previous_job_id
                && let Some(previous_departure_time) = self.previous_departure_time
            {
                compute_activity_arrival_time(
                    self.problem,
                    self.route.vehicle_id,
                    previous_job_id,
                    previous_departure_time,
                    job_id,
                )
            } else {
                compute_first_activity_arrival_time(self.problem, self.route.vehicle_id, job_id)
            };

            let waiting_duration = compute_waiting_duration(self.problem, job_id, arrival_time);

            let departure_time =
                compute_departure_time(self.problem, arrival_time, waiting_duration, job_id);

            self.previous_job_id = Some(job_id);
            self.previous_departure_time = Some(departure_time);

            let current_position = self.route.jobs.get(&job_id).copied();

            Some(RouteUpdateActivityData {
                arrival_time,
                departure_time,
                waiting_duration,
                job_id,
                current_position,
            })
        } else {
            None
        }
    }
}
