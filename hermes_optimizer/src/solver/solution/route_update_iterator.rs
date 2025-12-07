use jiff::{SignedDuration, Timestamp};

use crate::{
    problem::{job::JobId, vehicle_routing_problem::VehicleRoutingProblem},
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
    pub job_id: JobId,
}

pub struct RouteUpdateIterator<'a, I> {
    problem: &'a VehicleRoutingProblem,
    route: &'a WorkingSolutionRoute,
    jobs_iter: I,

    succeeding_iter: std::slice::Iter<'a, JobId>,

    index: usize,
    end: usize,

    previous_job_id: Option<JobId>,
    previous_departure_time: Option<Timestamp>,
}

impl<'a, I> RouteUpdateIterator<'a, I>
where
    I: Iterator<Item = JobId>,
{
    pub fn new(
        problem: &'a VehicleRoutingProblem,
        route: &'a WorkingSolutionRoute,
        jobs_iter: I,
        start: usize,
        end: usize,
    ) -> Self {
        let succeeding_activities = if end < route.activities.len() {
            &route.activities[end..]
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
            previous_job_id: previous_activity.map(|activity| activity.job_id),
            previous_departure_time: previous_activity.map(|activity| activity.departure_time),
        }
    }
}

impl<I> Iterator for RouteUpdateIterator<'_, I>
where
    I: Iterator<Item = JobId>,
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
                    previous_job_id.into(),
                    previous_departure_time,
                    job_id.into(),
                )
            } else {
                compute_first_activity_arrival_time(
                    self.problem,
                    self.route.vehicle_id,
                    job_id.into(),
                )
            };

            let waiting_duration =
                compute_waiting_duration(self.problem.service(job_id.into()), arrival_time);

            let departure_time =
                compute_departure_time(self.problem, arrival_time, waiting_duration, job_id.into());

            self.previous_job_id = Some(job_id);
            self.previous_departure_time = Some(departure_time);

            Some(RouteUpdateActivityData {
                arrival_time,
                departure_time,
                waiting_duration,
                job_id,
            })
        } else {
            None
        }
    }
}
