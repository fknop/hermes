use crate::{
    problem::{
        job::{ActivityId, Job, JobIdx},
        service::Service,
        shipment::Shipment,
        vehicle_routing_problem::VehicleRoutingProblem,
    },
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

impl ServiceInsertion {
    pub fn inserted_activity_ids(&self) -> impl Iterator<Item = ActivityId> + Clone {
        std::iter::once(ActivityId::Service(self.job_index))
    }

    pub fn service<'a>(&self, problem: &'a VehicleRoutingProblem) -> &'a Service {
        match problem.job(self.job_index) {
            Job::Service(service) => service,
            _ => panic!("Job is not a service"),
        }
    }
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

impl ShipmentInsertion {
    pub fn inserted_activity_ids(
        &self,
        route: &WorkingSolutionRoute,
    ) -> impl Iterator<Item = ActivityId> + Clone {
        std::iter::once(ActivityId::ShipmentPickup(self.job_index))
            .chain(route.activity_ids_iter(self.pickup_position, self.delivery_position))
            .chain(std::iter::once(ActivityId::ShipmentDelivery(
                self.job_index,
            )))
    }

    pub fn shipment<'a>(&self, problem: &'a VehicleRoutingProblem) -> &'a Shipment {
        match problem.job(self.job_index) {
            Job::Shipment(shipment) => shipment,
            _ => panic!("Job is not a shipment"),
        }
    }
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
        Job::Shipment(_) => {
            for_each_route_shipment_insertion(solution, route_index, job_index, &mut f)
        }
    }
}

fn route_with_dependencies(
    problem: &VehicleRoutingProblem,
    solution: &WorkingSolution,
    job_id: JobIdx,
) -> Option<RouteIdx> {
    if !problem.task_dependencies().has_in_same_route_dependencies() {
        return None;
    }

    solution
        .routes()
        .iter()
        .enumerate_idx()
        .find_map(|(route_id, route)| {
            if !route.is_empty()
                && problem
                    .task_dependencies()
                    .contains_in_same_route_dependencies_for_insertion(route.jobs_bitset(), job_id)
            {
                Some(route_id)
            } else {
                None
            }
        })
}

#[inline(always)]
fn for_each_service_insertion(
    solution: &WorkingSolution,
    job_index: JobIdx,
    mut f: impl FnMut(Insertion),
) {
    let route_with_deps = route_with_dependencies(solution.problem(), solution, job_index);

    solution
        .routes()
        .iter()
        .enumerate_idx()
        .for_each(|(route_id, route)| {
            // If a route with already assigned dependencies exists and this is not the route, skip
            if let Some(route_with_deps) = route_with_deps
                && route_id != route_with_deps
            {
                return;
            }

            if route.has_maximum_activities(solution.problem()) {
                return;
            }

            if !route.can_deliver_job(solution.problem(), job_index) {
                return;
            }

            let (start, end) = route.insertion_range(ActivityId::Service(job_index));

            (start..=end)
                .filter(|position| {
                    route.in_insertion_neighborhood(
                        solution.problem(),
                        ActivityId::Service(job_index),
                        *position,
                    )
                })
                .for_each(|position| {
                    f(Insertion::Service(ServiceInsertion {
                        route_id,
                        job_index,
                        position,
                    }));
                });
        });
}

fn for_each_route_service_insertion(
    solution: &WorkingSolution,
    route_index: RouteIdx,
    job_index: JobIdx,
    mut f: impl FnMut(Insertion),
) {
    let route = solution.route(route_index);
    // If a route with already assigned dependencies exists and this is not the route, skip
    let route_with_deps = route_with_dependencies(solution.problem(), solution, job_index);

    if let Some(route_with_deps) = route_with_deps
        && route_index != route_with_deps
    {
        return;
    }

    if route.has_maximum_activities(solution.problem()) {
        return;
    }

    if !route.can_deliver_job(solution.problem(), job_index) {
        return;
    }

    let (start, end) = route.insertion_range(ActivityId::Service(job_index));

    for position in start..=end {
        if !route.in_insertion_neighborhood(
            solution.problem(),
            ActivityId::Service(job_index),
            position,
        ) {
            continue;
        }

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
        if route.will_break_maximum_activities(solution.problem(), 2) {
            continue;
        }

        if !route.can_deliver_job(solution.problem(), job_index) {
            continue;
        }

        let (start_pickup, end_pickup) =
            route.insertion_range(ActivityId::ShipmentPickup(job_index));
        let (start_delivery, end_delivery) =
            route.insertion_range(ActivityId::ShipmentDelivery(job_index));

        for pickup_position in start_pickup..=end_pickup {
            if !route.in_insertion_neighborhood(
                solution.problem(),
                ActivityId::ShipmentPickup(job_index),
                pickup_position,
            ) {
                continue;
            }

            for delivery_position in (pickup_position.max(start_delivery))..=end_delivery {
                if !route.in_insertion_neighborhood(
                    solution.problem(),
                    ActivityId::ShipmentDelivery(job_index),
                    delivery_position,
                ) {
                    continue;
                }

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

fn for_each_route_shipment_insertion(
    solution: &WorkingSolution,
    route_index: RouteIdx,
    job_index: JobIdx,
    mut f: impl FnMut(Insertion),
) {
    let route = solution.route(route_index);

    let route_with_deps = route_with_dependencies(solution.problem(), solution, job_index);
    if let Some(route_with_deps) = route_with_deps
        && route_index != route_with_deps
    {
        return;
    }

    if route.will_break_maximum_activities(solution.problem(), 2) {
        return;
    }

    if !route.can_deliver_job(solution.problem(), job_index) {
        return;
    }

    let (start_pickup, end_pickup) = route.insertion_range(ActivityId::ShipmentPickup(job_index));
    let (start_delivery, end_delivery) =
        route.insertion_range(ActivityId::ShipmentDelivery(job_index));

    for pickup_position in start_pickup..=end_pickup {
        if !route.in_insertion_neighborhood(
            solution.problem(),
            ActivityId::ShipmentPickup(job_index),
            pickup_position,
        ) {
            continue;
        }

        for delivery_position in (pickup_position.max(start_delivery))..=end_delivery {
            if !route.in_insertion_neighborhood(
                solution.problem(),
                ActivityId::ShipmentDelivery(job_index),
                delivery_position,
            ) {
                continue;
            }

            f(Insertion::Shipment(ShipmentInsertion {
                route_id: route_index,
                job_index,
                pickup_position,
                delivery_position,
            }));
        }
    }
}
