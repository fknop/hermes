use std::{f64, sync::Arc};

use geo::ConvexHull;
use jiff::Timestamp;
use rand::rngs::SmallRng;
use tracing::{Level, debug, instrument};

use crate::{
    problem::{
        amount::AmountExpression,
        capacity::Capacity,
        job::{Job, JobIdx},
        location::LocationIdx,
        service::ServiceType,
        time_window::TimeWindows,
        vehicle_routing_problem::VehicleRoutingProblem,
    },
    solver::{
        constraints::constraint::Constraint,
        insertion::{Insertion, ServiceInsertion, ShipmentInsertion},
        ls::local_search::LocalSearch,
        noise::NoiseParams,
        recreate::{
            best_insertion::{BestInsertion, BestInsertionParams, BestInsertionSortStrategy},
            construction_best_insertion::ConstructionBestInsertion,
            recreate_context::RecreateContext,
            recreate_solution::RecreateSolution,
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
        .jobs()
        .iter()
        .filter(|job| match job {
            Job::Shipment(_) => true,
            Job::Service(service) => service.service_type() == ServiceType::Delivery,
        })
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
            .jobs()
            .iter()
            .flat_map(|job| {
                let location_ids = match job {
                    Job::Service(service) => vec![service.location_id()],
                    Job::Shipment(shipment) => {
                        vec![
                            shipment.pickup().location_id(),
                            shipment.delivery().location_id(),
                        ]
                    }
                };

                location_ids
                    .iter()
                    .map(|&location_id| {
                        let loc = problem.location(location_id);
                        (loc.x(), loc.y())
                    })
                    .collect::<Vec<_>>()
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

fn job_time_windows_and_location(job: &Job) -> (&TimeWindows, LocationIdx) {
    match job {
        Job::Service(service) => (service.time_windows(), service.location_id()),
        Job::Shipment(shipment) => {
            let delivery_tw = shipment.delivery().time_windows();

            if delivery_tw.is_empty() {
                (
                    shipment.pickup().time_windows(),
                    shipment.pickup().location_id(),
                )
            } else {
                (delivery_tw, shipment.delivery().location_id())
            }
        }
    }
}

fn between_jobs_travel_cost(problem: &VehicleRoutingProblem, a: &Job, b: &Job) -> f64 {
    match (a, b) {
        (Job::Service(service_a), Job::Service(service_b)) => problem.travel_cost(
            problem.vehicle(0.into()),
            service_a.location_id(),
            service_b.location_id(),
        ),
        (Job::Shipment(shipment_a), Job::Shipment(shipment_b)) => {
            let pickup_a = shipment_a.pickup().location_id();
            let delivery_a = shipment_a.delivery().location_id();
            let pickup_b = shipment_b.pickup().location_id();
            let delivery_b = shipment_b.delivery().location_id();

            problem.travel_cost(problem.vehicle(0.into()), pickup_a, pickup_b)
                + problem.travel_cost(problem.vehicle(0.into()), delivery_a, delivery_b)
        }
        (Job::Service(service_a), Job::Shipment(shipment_b)) => {
            let pickup_b = shipment_b.pickup().location_id();
            let delivery_b = shipment_b.delivery().location_id();

            (problem.travel_cost(problem.vehicle(0.into()), service_a.location_id(), pickup_b)
                + problem.travel_cost(
                    problem.vehicle(0.into()),
                    service_a.location_id(),
                    delivery_b,
                ))
                / 2.0
        }
        (Job::Shipment(shipment_a), Job::Service(service_b)) => {
            let pickup_a = shipment_a.pickup().location_id();
            let delivery_a = shipment_a.delivery().location_id();

            (problem.travel_cost(problem.vehicle(0.into()), pickup_a, service_b.location_id())
                + problem.travel_cost(
                    problem.vehicle(0.into()),
                    delivery_a,
                    service_b.location_id(),
                ))
                / 2.0
        }
    }
}

fn sum_distances(problem: &VehicleRoutingProblem, seed_customers: &[JobIdx], job: &Job) -> f64 {
    seed_customers
        .iter()
        .map(|&seed| between_jobs_travel_cost(problem, problem.job(seed), job))
        .sum::<f64>()
}

#[instrument(skip_all, level = Level::DEBUG)]
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
        let job_a = problem.job(a);
        let job_b = problem.job(b);

        if problem.has_time_windows() {
            let (tw_a, location_a) = job_time_windows_and_location(job_a);
            let (tw_b, location_b) = job_time_windows_and_location(job_b);

            let urgency_a = tw_a
                .iter()
                .filter_map(|time_window| time_window.latest())
                .max()
                .map(|end| {
                    end - problem.travel_time(problem.vehicle(0.into()), depot_id, location_a)
                })
                .unwrap_or(Timestamp::MAX); // If no time window end -> no urgency

            let urgency_b = tw_b
                .iter()
                .filter_map(|time_window| time_window.latest())
                .max()
                .map(|end| {
                    end - problem.travel_time(problem.vehicle(0.into()), depot_id, location_b)
                })
                .unwrap_or(Timestamp::MAX); // If no time window end -> no urgency

            urgency_a.cmp(&urgency_b)
        } else if problem.has_capacity() {
            let first_demand_a = job_a.demand().get(0);
            let first_demand_b = job_b.demand().get(0);

            first_demand_a.total_cmp(&first_demand_b).reverse()
        } else {
            let distance_from_depot_to_a = problem.average_cost_from_depot(job_a);
            let distance_from_depot_to_b = problem.average_cost_from_depot(job_b);

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
                .average_cost_from_depot(problem.job(first))
                .partial_cmp(&problem.average_cost_from_depot(problem.job(second)))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .unwrap();
    seed_customers.push(first_seed);
    exterior.retain(|&i| i != first_seed);

    while seed_customers.len() < k_min && (!exterior.is_empty() || !interior.is_empty()) {
        let candidate_j = exterior.iter().cloned().max_by(|&a, &b| {
            let sum_dist_a = sum_distances(problem, &seed_customers, problem.job(a));
            let sum_dist_b = sum_distances(problem, &seed_customers, problem.job(b));

            sum_dist_a.partial_cmp(&sum_dist_b).unwrap()
        });

        let candidate_i = interior.first().cloned();

        let d_j = candidate_j.map(|j| {
            seed_customers
                .iter()
                .map(|&seed| between_jobs_travel_cost(problem, problem.job(j), problem.job(seed)))
                .fold(f64::INFINITY, f64::min)
        });

        let d_i = candidate_i.map(|i| {
            seed_customers
                .iter()
                .map(|&seed| between_jobs_travel_cost(problem, problem.job(i), problem.job(seed)))
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
            let job = problem.job(customer);

            match job {
                Job::Service(_) => {
                    solution.insert(&Insertion::Service(ServiceInsertion {
                        route_id,
                        job_index: customer,
                        position: 0,
                    }));
                }
                Job::Shipment(_) => {
                    solution.insert(&Insertion::Shipment(ShipmentInsertion {
                        route_id,
                        job_index: customer,
                        pickup_position: 0,
                        delivery_position: 0,
                    }));
                }
            }
        }
    }
}

pub fn construct_solution(
    problem: &Arc<VehicleRoutingProblem>,
    params: &SolverParams,
    rng: &mut SmallRng,
    constraints: &Vec<Constraint>,
) -> WorkingSolution {
    debug!("Start construction heuristic");
    let mut solution = WorkingSolution::new(Arc::clone(problem));
    create_initial_routes(problem, &mut solution);

    let (score, score_analysis) = solution.compute_solution_score(constraints);

    if score.is_infeasible() {
        tracing::error!(
            "create_initial_routes solution rejected due to failure score: {:?}",
            score_analysis,
        );
        panic!("Bug: score should never fail when insert_on_failure is false")
    }

    if problem.jobs().len() > 500 {
        let best_insertion = BestInsertion::new(BestInsertionParams {
            blink_rate: 0.0,
            sort_strategy: BestInsertionSortStrategy::Far,
        });

        best_insertion.recreate_solution(
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
                insert_on_failure: false,
            },
        );
    } else {
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
                insert_on_failure: false,
            },
        );
    }

    let mut local_search = LocalSearch::new(problem, constraints.to_vec());

    let _routes = solution
        .routes()
        .iter()
        .enumerate_idx()
        .filter(|(_, route)| !route.is_empty())
        .map(|(route_id, _)| route_id)
        .collect::<Vec<RouteIdx>>();

    let (score, score_analysis) = solution.compute_solution_score(constraints);

    if score.is_infeasible() {
        tracing::error!(
            "Construction ALNS: solution rejected due to failure score: {:?}",
            score_analysis,
        );
        panic!("Bug: score should never fail when insert_on_failure is false")
    }

    debug!("construct_solution: start local search");

    local_search.intensify(problem, &mut solution, 500);

    let (score, score_analysis) = solution.compute_solution_score(constraints);

    if score.is_infeasible() {
        tracing::error!(
            "Construction LS: solution rejected due to failure score: {:?}",
            score_analysis,
        );
        panic!("Bug: score should never fail when insert_on_failure is false")
    }

    // for &route_id in &routes {
    //     debug!("Intensifying route {}", route_id);
    //     local_search.intensify_route(problem, &mut solution, route_id);
    //     let (score, score_analysis) = solution.compute_solution_score(constraints);

    //     if score.is_infeasible() {
    //         tracing::error!(
    //             "Construction LS: solution rejected due to failure score: {:?}",
    //             score_analysis,
    //         );
    //         panic!("Bug: score should never fail when insert_on_failure is false")
    //     }
    // }

    solution
}
