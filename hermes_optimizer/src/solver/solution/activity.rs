use crate::problem::{
    job::JobId,
    service::{Service, ServiceId},
    vehicle_routing_problem::VehicleRoutingProblem,
};

#[derive(Clone)]
pub struct WorkingSolutionRouteActivity {
    pub(super) job_id: JobId,
}

impl WorkingSolutionRouteActivity {
    pub fn invalid(job_id: JobId) -> Self {
        WorkingSolutionRouteActivity { job_id }
    }

    pub fn new(job_id: ServiceId) -> Self {
        WorkingSolutionRouteActivity {
            job_id: JobId::Service(job_id),
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
}
