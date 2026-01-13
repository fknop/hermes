use std::{f64, sync::Arc};

use geo::ConvexHull;
use jiff::Timestamp;
use rand::rngs::SmallRng;

use crate::{
    problem::{
        amount::AmountExpression,
        capacity::Capacity,
        job::{Job, JobIdx},
        service::ServiceType,
        vehicle_routing_problem::VehicleRoutingProblem,
    },
    solver::{
        constraints::constraint::Constraint,
        insertion::{Insertion, ServiceInsertion},
        noise::NoiseParams,
        recreate::{
            construction_best_insertion::ConstructionBestInsertion,
            recreate_context::RecreateContext,
        },
        solution::{route_id::RouteIdx, working_solution::WorkingSolution},
        solver_params::SolverParams,
    },
    utils::enumerate_idx::EnumerateIdx,
};

/// Kmin = Q / D where Q = total demand and D = vehicle capacity
/// Kmin = max(Q_i / D_i) for each capacity dimension i
fn find_minimum_vehicles(problem: &VehicleRoutingProblem) -> usize {
    let mut minimum_vehicles = problem.vehicles().len();
    let total_demand: Capacity = problem
        .services_iter()
        .filter(|service| service.service_type() == ServiceType::Delivery)
        .fold(Capacity::EMPTY, |total, service| {
            (&total + service.demand()).into()
        });

    for vehicle in problem.vehicles() {
        let mut minimum_for_vehicle_capacity = 0;

        let capacity = vehicle.capacity();

        for (index, capacity_dimension) in capacity.iter().enumerate() {
            let demand = total_demand.get(index);
            if capacity_dimension > 0.0 {
                let required_vehicles = (demand / capacity_dimension).ceil() as usize;

                minimum_for_vehicle_capacity = minimum_for_vehicle_capacity.max(required_vehicles);
            } else if demand > 0.0 {
                // If there's demand but the vehicle has no capacity in this dimension,
                // it can't serve any of that demand.
                minimum_for_vehicle_capacity = problem.vehicles().len();
                break;
            }
        }

        if problem.fleet().is_infinite() {
            minimum_vehicles = minimum_vehicles.max(minimum_for_vehicle_capacity);
        } else {
            minimum_vehicles = minimum_vehicles.min(minimum_for_vehicle_capacity);
        }
    }

    minimum_vehicles
}

fn compute_convex_hull(problem: &VehicleRoutingProblem) -> (Vec<JobIdx>, Vec<JobIdx>) {
    let points = geo::MultiPoint::from(
        problem
            .services_iter()
            .map(|s| {
                let loc = problem.location(s.location_id());
                (loc.x(), loc.y())
            })
            .collect::<Vec<_>>(),
    );
    let convex_hull = points.convex_hull();

    let exterior: Vec<JobIdx> = convex_hull
        .exterior()
        .points()
        .filter_map(|point| {
            problem
                .locations()
                .iter()
                .enumerate_idx()
                .find(|(_idx, location)| location.x() == point.x() && location.y() == point.y())
                .map(|(idx, _)| idx)
        })
        .flat_map(|location_id| {
            problem
                .jobs()
                .iter()
                .enumerate_idx()
                .filter(move |(_, job)| match job {
                    Job::Service(service) => service.location_id() == location_id,
                    Job::Shipment(shipment) => {
                        shipment.pickup().location_id() == location_id
                            || shipment.delivery().location_id() == location_id
                    }
                })
                .map(|(idx, _)| idx)
        })
        .collect();

    let interior: Vec<JobIdx> = (0..problem.jobs().len())
        .map(JobIdx::new)
        .filter(|i| !exterior.contains(i))
        .collect();

    (exterior, interior)
}

fn create_initial_routes(problem: &VehicleRoutingProblem, solution: &mut WorkingSolution) {
    let k_min = find_minimum_vehicles(problem);

    let (mut exterior, mut interior) = compute_convex_hull(problem);

    let depot_id = problem
        .vehicles()
        .iter()
        // TODO: don't assume there's a depot
        .find_map(|v| v.depot_location_id())
        .unwrap();

    // Sort by urgency
    interior.sort_unstable_by(|&a, &b| {
        let service_a = problem.service(a);
        let service_b = problem.service(b);

        if problem.has_time_windows() {
            let urgency_a = service_a
                .time_windows()
                .iter()
                .filter_map(|time_window| time_window.end())
                .max()
                .map(|end| {
                    end - problem.travel_time(
                        problem.vehicle(0.into()),
                        depot_id,
                        service_a.location_id(),
                    )
                })
                .unwrap_or(Timestamp::MAX); // If no time window end -> no urgency

            let urgency_b = service_b
                .time_windows()
                .iter()
                .filter_map(|time_window| time_window.end())
                .max()
                .map(|end| {
                    end - problem.travel_time(
                        problem.vehicle(0.into()),
                        depot_id,
                        service_b.location_id(),
                    )
                })
                .unwrap_or(Timestamp::MAX); // If no time window end -> no urgency

            urgency_a.cmp(&urgency_b)
        } else if problem.has_capacity() {
            let first_demand_a = service_a.demand().get(0);
            let first_demand_b = service_b.demand().get(0);

            first_demand_a.total_cmp(&first_demand_b).reverse()
        } else {
            let distance_from_depot_to_a = problem.average_cost_from_depot(service_a.location_id());
            let distance_from_depot_to_b = problem.average_cost_from_depot(service_b.location_id());

            distance_from_depot_to_a
                .partial_cmp(&distance_from_depot_to_b)
                .unwrap()
                .reverse()
        }
    });

    let mut seed_customers: Vec<JobIdx> = Vec::with_capacity(k_min);
    let first_seed = exterior
        .iter()
        .cloned()
        .max_by(|&first, &second| {
            problem
                .travel_cost(
                    problem.vehicle(0.into()),
                    depot_id,
                    // TODO: support shipment
                    problem.service(first).location_id(),
                )
                .partial_cmp(&problem.travel_cost(
                    problem.vehicle(0.into()),
                    depot_id,
                    // TODO: support shipment
                    problem.service(second).location_id(),
                ))
                .unwrap()
        })
        .unwrap();
    seed_customers.push(first_seed);
    exterior.retain(|&i| i != first_seed);

    while seed_customers.len() < k_min && (!exterior.is_empty() || !interior.is_empty()) {
        let candidate_j = exterior.iter().cloned().max_by(|&a, &b| {
            let location_id_a = problem.service(a).location_id();
            let location_id_b = problem.service(b).location_id();
            let sum_dist_a = seed_customers
                .iter()
                .map(|&seed| {
                    problem.travel_cost(
                        problem.vehicle(0.into()),
                        location_id_a,
                        problem.service(seed).location_id(),
                    )
                })
                .sum::<f64>();
            let sum_dist_b = seed_customers
                .iter()
                .map(|&seed| {
                    problem.travel_cost(
                        problem.vehicle(0.into()),
                        location_id_b,
                        problem.service(seed).location_id(),
                    )
                })
                .sum::<f64>();

            sum_dist_a.partial_cmp(&sum_dist_b).unwrap()
        });

        let candidate_i = interior.first().cloned();

        let d_j = candidate_j.map(|j| {
            seed_customers
                .iter()
                .map(|&seed| {
                    problem.travel_cost(
                        problem.vehicle(0.into()),
                        problem.service(j).location_id(),
                        problem.service(seed).location_id(),
                    )
                })
                .fold(f64::INFINITY, f64::min)
        });

        let d_i = candidate_i.map(|i| {
            seed_customers
                .iter()
                .map(|&seed| {
                    problem.travel_cost(
                        problem.vehicle(0.into()),
                        problem.service(i).location_id(),
                        problem.service(seed).location_id(),
                    )
                })
                .fold(f64::INFINITY, f64::min)
        });

        let pick_candidate_j = match (d_j, d_i) {
            (Some(d_j), Some(d_i)) => d_j > d_i,
            (Some(_), None) => true,
            (None, Some(_)) => false,
            (None, None) => break,
        };

        if pick_candidate_j {
            let j = candidate_j.unwrap();
            seed_customers.push(j);
            exterior.retain(|&e| e != j);
        } else {
            let i = candidate_i.unwrap();
            seed_customers.push(i);
            interior.remove(0);
        }
    }

    for &customer in &seed_customers {
        if let Some(route_id) = solution
            .routes()
            .iter()
            .enumerate_idx()
            .filter(|(_, route)| route.is_empty())
            .map(|(id, _)| id)
            .next()
        {
            solution.insert(&Insertion::Service(ServiceInsertion {
                route_id: RouteIdx::new(route_id),
                job_index: customer,
                position: 0,
            }));
        }
    }
}

pub fn construct_solution(
    problem: &Arc<VehicleRoutingProblem>,
    params: &SolverParams,
    rng: &mut SmallRng,
    constraints: &Vec<Constraint>,
    thread_pool: &rayon::ThreadPool,
) -> WorkingSolution {
    let mut solution = WorkingSolution::new(Arc::clone(problem));
    create_initial_routes(problem, &mut solution);

    // ConstructionBestInsertion::insert_services(
    //     &mut solution,
    //     RecreateContext {
    //         rng,
    //         // constraints,
    //         constraints: &vec![
    //             Constraint::Global(GlobalConstraintType::TransportCost(TransportCostConstraint)),
    //             Constraint::Route(RouteConstraintType::VehicleCost(VehicleCostConstraint)),
    //             Constraint::Route(RouteConstraintType::Capacity(CapacityConstraint::new(
    //                 ScoreLevel::Soft,
    //             ))),
    //             Constraint::Activity(ActivityConstraintType::TimeWindow(
    //                 TimeWindowConstraint::new(ScoreLevel::Soft),
    //             )),
    //             Constraint::Route(RouteConstraintType::WaitingDuration(
    //                 WaitingDurationConstraint,
    //             )),
    //         ],
    //         noise_generator: None,
    //         problem,
    //         thread_pool,
    //         insert_on_failure: true,
    //     },
    // );

    // let mut satisfied = false;

    // while !satisfied {
    //     let mut job_to_remove = None;
    //     for route in solution.routes() {
    //         for (i, _) in route.activity_ids().iter().enumerate() {
    //             let activity = route.activity(i);
    //             let time_window_score = TimeWindowConstraint::compute_time_window_score(
    //                 ScoreLevel::Hard,
    //                 activity.job_task(problem).time_windows(),
    //                 activity.arrival_time(),
    //             );

    //             if time_window_score.hard_score > 0.0 {
    //                 job_to_remove = Some(activity.job_id());
    //                 break;
    //             }

    //             let vehicle = route.vehicle(problem);

    //             let load = route.load_at(i);

    //             if !is_capacity_satisfied(vehicle.capacity(), load) {
    //                 job_to_remove = Some(activity.job_id());
    //                 break;
    //             }
    //         }

    //         if job_to_remove.is_some() {
    //             break;
    //         }
    //     }

    //     if let Some(service_id) = job_to_remove {
    //         solution.remove_job(service_id);
    //         solution.resync();
    //     } else {
    //         satisfied = true
    //     }
    // }

    // solution.resync();

    ConstructionBestInsertion::insert_services(
        &mut solution,
        RecreateContext {
            rng,
            constraints,
            noise_params: NoiseParams {
                max_cost: problem.max_cost(),
                noise_level: params.noise_level,
                noise_probability: params.noise_probability,
            },
            problem,
            thread_pool,
            insert_on_failure: false,
        },
    );

    solution
}
