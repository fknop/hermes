use jiff::{SignedDuration, Timestamp};

use crate::{
    problem::{
        job::JobId,
        service::{Service, ServiceId},
        vehicle_routing_problem::VehicleRoutingProblem,
    },
    solver::solution::utils::{compute_departure_time, compute_waiting_duration},
};

#[derive(Clone)]
pub struct WorkingSolutionRouteActivity {
    pub(super) job_id: JobId,
    pub(super) arrival_time: Timestamp,
    pub(super) departure_time: Timestamp,
    pub(super) waiting_duration: SignedDuration,
}

impl WorkingSolutionRouteActivity {
    pub fn invalid(job_id: JobId) -> Self {
        WorkingSolutionRouteActivity {
            job_id,
            arrival_time: jiff::Timestamp::MIN,
            waiting_duration: SignedDuration::ZERO,
            departure_time: jiff::Timestamp::MIN,
        }
    }

    pub fn new(
        problem: &VehicleRoutingProblem,
        job_id: ServiceId,
        arrival_time: Timestamp,
    ) -> Self {
        let waiting_duration = compute_waiting_duration(problem.service(job_id), arrival_time);
        WorkingSolutionRouteActivity {
            job_id: JobId::Service(job_id),
            arrival_time,
            waiting_duration,
            departure_time: compute_departure_time(problem, arrival_time, waiting_duration, job_id),
        }
    }

    pub fn service<'a>(&self, problem: &'a VehicleRoutingProblem) -> &'a Service {
        problem.service(self.job_id.into())
    }

    pub fn service_id(&self) -> ServiceId {
        self.job_id.into()
    }

    pub fn job_id(&self) -> JobId {
        self.job_id
    }

    pub fn arrival_time(&self) -> Timestamp {
        self.arrival_time
    }

    pub fn departure_time(&self) -> Timestamp {
        self.departure_time
    }

    pub fn waiting_duration(&self) -> SignedDuration {
        self.waiting_duration
    }

    pub(super) fn update_arrival_time(
        &mut self,
        problem: &VehicleRoutingProblem,
        arrival_time: Timestamp,
    ) {
        self.arrival_time = arrival_time;
        self.waiting_duration =
            compute_waiting_duration(problem.service(self.job_id.into()), arrival_time);
        self.departure_time = compute_departure_time(
            problem,
            self.arrival_time,
            self.waiting_duration,
            self.job_id.into(),
        );
    }
}
