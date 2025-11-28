use jiff::{SignedDuration, Timestamp};

use crate::{
    problem::{
        capacity::Capacity,
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
    pub(super) cumulative_load: Capacity,
    pub(super) max_load_until_end: Capacity,
}

impl WorkingSolutionRouteActivity {
    pub fn invalid(job_id: ServiceId) -> Self {
        WorkingSolutionRouteActivity {
            job_id: JobId::Service(job_id),
            arrival_time: jiff::Timestamp::MIN,
            waiting_duration: SignedDuration::ZERO,
            departure_time: jiff::Timestamp::MIN,
            cumulative_load: Capacity::EMPTY,
            max_load_until_end: Capacity::EMPTY,
        }
    }

    pub fn new(
        problem: &VehicleRoutingProblem,
        job_id: ServiceId,
        arrival_time: Timestamp,
        cumulative_load: Capacity,
    ) -> Self {
        let waiting_duration = compute_waiting_duration(problem.service(job_id), arrival_time);
        WorkingSolutionRouteActivity {
            job_id: JobId::Service(job_id),
            arrival_time,
            waiting_duration,
            departure_time: compute_departure_time(problem, arrival_time, waiting_duration, job_id),
            cumulative_load,
            max_load_until_end: Capacity::EMPTY,
        }
    }

    pub fn service<'a>(&self, problem: &'a VehicleRoutingProblem) -> &'a Service {
        problem.service(self.job_id.into())
    }

    pub fn service_id(&self) -> ServiceId {
        self.job_id.into()
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

    pub fn cumulative_load(&self) -> &Capacity {
        &self.cumulative_load
    }

    pub fn max_load_until_end(&self) -> &Capacity {
        &self.max_load_until_end
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
