use crate::{
    problem::job::{Job, JobIdx},
    solver::solution::{
        route::WorkingSolutionRoute, route_id::RouteIdx, working_solution::WorkingSolution,
    },
    utils::enumerate_idx::EnumerateIdx,
};

#[derive(Clone, Debug)]
pub struct ServiceInsertion {
    pub route_id: RouteIdx,
    pub job_index: JobIdx,
    pub position: usize,
}

#[derive(Clone, Debug)]
pub struct ShipmentInsertion {
    pub route_id: RouteIdx,
    pub job_index: JobIdx,

    /// Position of the pickup
    pub pickup_position: usize,

    /// This is the position before the pickup has been inserted
    pub delivery_position: usize,
}

#[derive(Clone, Debug)]
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

    pub fn route_id(&self) -> RouteIdx {
        match self {
            Insertion::Service(ctx) => ctx.route_id,
            Insertion::Shipment(ctx) => ctx.route_id,
        }
    }

    pub fn route<'a>(&self, solution: &'a WorkingSolution) -> &'a WorkingSolutionRoute {
        match self {
            Insertion::Service(ctx) => solution.route(ctx.route_id),
            Insertion::Shipment(ctx) => solution.route(ctx.route_id),
        }
    }
}

#[inline(always)]
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

pub fn for_each_route_insertion(
    solution: &WorkingSolution,
    route_index: RouteIdx,
    job_index: JobIdx,
    mut f: impl FnMut(Insertion),
) {
    let job = solution.problem().job(job_index);

    match job {
        Job::Service(_) => {
            for_each_route_service_insertion(solution, route_index, job_index, &mut f)
        }
        Job::Shipment(_) => todo!(),
    }
}

#[inline(always)]
fn for_each_service_insertion(
    solution: &WorkingSolution,
    job_index: JobIdx,
    mut f: impl FnMut(Insertion),
) {
    solution
        .routes()
        .iter()
        .enumerate_idx()
        .for_each(|(route_id, route)| {
            if !route.has_maximum_activities(solution.problem()) {
                (0..=route.len()).for_each(|position| {
                    f(Insertion::Service(ServiceInsertion {
                        route_id,
                        job_index,
                        position,
                    }));
                });
            }
        });
}

fn for_each_route_service_insertion(
    solution: &WorkingSolution,
    route_index: RouteIdx,
    job_index: JobIdx,
    mut f: impl FnMut(Insertion),
) {
    let route = solution.route(route_index);
    if route.has_maximum_activities(solution.problem()) {
        return;
    }

    for position in 0..=route.len() {
        f(Insertion::Service(ServiceInsertion {
            route_id: route_index,
            job_index,
            position,
        }));
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
