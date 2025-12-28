use crate::{
    problem::job::{Job, JobIdx},
    solver::solution::{
        route::WorkingSolutionRoute, route_id::RouteIdx, working_solution::WorkingSolution,
    },
    utils::enumerate_idx::EnumerateIdx,
};

#[derive(Clone)]
pub struct ServiceInsertion {
    pub route_id: RouteIdx,
    pub job_index: JobIdx,
    pub position: usize,
}

#[derive(Clone)]
pub struct ShipmentInsertion {
    pub route_id: RouteIdx,
    pub job_index: JobIdx,

    /// Position of the pickup
    pub pickup_position: usize,

    /// This is the position before the pickup has been inserted
    pub delivery_position: usize,
}

#[derive(Clone)]
pub enum Insertion {
    Service(ServiceInsertion),
    Shipment(ShipmentInsertion),
}

impl Insertion {
    pub fn job_idx(&self) -> JobIdx {
        match self {
            Insertion::Service(ctx) => ctx.job_index,
            Insertion::Shipment(ctx) => ctx.job_index,
        }
    }

    pub fn route<'a>(&self, solution: &'a WorkingSolution) -> &'a WorkingSolutionRoute {
        match self {
            Insertion::Service(ctx) => solution.route(ctx.route_id),
            Insertion::Shipment(ctx) => solution.route(ctx.route_id),
        }
    }
}

pub fn for_each_insertion(
    solution: &WorkingSolution,
    job_index: JobIdx,
    mut f: impl FnMut(Insertion),
) {
    let job = solution.problem().job(job_index);

    match job {
        Job::Service(_) => for_each_service_insertion(solution, job_index, &mut f),
        Job::Shipment(_) => for_each_shipment_insertion(solution, job_index, &mut f),
    }
}

fn for_each_service_insertion(
    solution: &WorkingSolution,
    job_index: JobIdx,
    mut f: impl FnMut(Insertion),
) {
    for (route_id, route) in solution.routes().iter().enumerate() {
        for position in 0..=route.len() {
            f(Insertion::Service(ServiceInsertion {
                route_id: RouteIdx::new(route_id),
                job_index,
                position,
            }));
        }
    }
}

fn for_each_shipment_insertion(
    solution: &WorkingSolution,
    job_index: JobIdx,
    mut f: impl FnMut(Insertion),
) {
    for (route_id, route) in solution.routes().iter().enumerate_idx() {
        for pickup_position in 0..=route.len() {
            for delivery_position in pickup_position + 1..=route.len().max(pickup_position + 1) {
                f(Insertion::Shipment(ShipmentInsertion {
                    route_id,
                    job_index,
                    pickup_position,
                    delivery_position,
                }));
            }
        }
    }
}
