use std::{f64, sync::Arc};

use geo::ConvexHull;
use jiff::Timestamp;
use rand::rngs::SmallRng;

use crate::{
    problem::{
        amount::AmountExpression,
        capacity::{Capacity, is_capacity_satisfied},
        service::ServiceType,
        vehicle_routing_problem::VehicleRoutingProblem,
    },
    solver::{
        constraints::{
            activity_constraint::ActivityConstraintType, capacity_constraint::CapacityConstraint,
            constraint::Constraint, global_constraint::GlobalConstraintType,
            route_constraint::RouteConstraintType, time_window_constraint::TimeWindowConstraint,
            transport_cost_constraint::TransportCostConstraint,
            vehicle_cost_constraint::VehicleCostConstraint,
            waiting_duration_constraint::WaitingDurationConstraint,
        },
        insertion::{Insertion, NewRouteInsertion},
        recreate::{
            construction_best_insertion::ConstructionBestInsertion,
            recreate_context::RecreateContext,
        },
        score_level::ScoreLevel,
        solution::working_solution::WorkingSolution,
    },
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

        minimum_vehicles = minimum_vehicles.min(minimum_for_vehicle_capacity);
    }

    minimum_vehicles
}

fn compute_convex_hull(problem: &VehicleRoutingProblem) -> (Vec<usize>, Vec<usize>) {
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

    let exterior: Vec<usize> = convex_hull
        .exterior()
        .points()
        .filter_map(|point| {
            problem
                .locations()
                .iter()
                .find(|location| location.x() == point.x() && location.y() == point.y())
                .map(|location| location.id())
        })
        .collect();

    let interior: Vec<usize> = (0..problem.jobs().len())
        .filter(|i| !exterior.contains(&problem.service_location(*i).id()))
        .collect();

    (exterior, interior)
}

fn create_initial_routes(problem: &VehicleRoutingProblem, solution: &mut WorkingSolution) {
    let k_min = find_minimum_vehicles(problem);

    let (mut exterior, mut interior) = compute_convex_hull(problem);

    let depot_id = problem
        .vehicles()
        .iter()
        .find_map(|v| v.depot_location_id())
        .unwrap();

    // Sort by urgency
    interior.sort_unstable_by(|&a, &b| {
        let service_a = problem.service(a);
        let service_b = problem.service(b);

        let urgency_a = service_a
            .time_windows()
            .iter()
            .filter_map(|time_window| time_window.end())
            .max()
            .map(|end| end - problem.travel_time(depot_id, service_a.location_id()))
            .unwrap_or(Timestamp::MAX); // If no time window end -> no urgency

        let urgency_b = service_b
            .time_windows()
            .iter()
            .filter_map(|time_window| time_window.end())
            .max()
            .map(|end| end - problem.travel_time(depot_id, service_b.location_id()))
            .unwrap_or(Timestamp::MAX); // If no time window end -> no urgency

        urgency_a.cmp(&urgency_b)
    });

    let mut seed_customers: Vec<usize> = Vec::with_capacity(k_min);
    let first_seed = exterior
        .iter()
        .cloned()
        .max_by(|&first, &second| {
            problem
                .travel_cost(depot_id, problem.service_location(first).id())
                .partial_cmp(&problem.travel_cost(depot_id, problem.service_location(second).id()))
                .unwrap()
        })
        .unwrap();
    seed_customers.push(first_seed);
    exterior.retain(|&i| i != first_seed);

    while seed_customers.len() < k_min && (!exterior.is_empty() || !interior.is_empty()) {
        let candidate_j = exterior.iter().cloned().max_by(|&a, &b| {
            let location_id_a = problem.service_location(a).id();
            let location_id_b = problem.service_location(b).id();
            let sum_dist_a = seed_customers
                .iter()
                .map(|&seed| problem.travel_cost(location_id_a, seed))
                .sum::<f64>();
            let sum_dist_b = seed_customers
                .iter()
                .map(|&seed| problem.travel_cost(location_id_b, seed))
                .sum::<f64>();

            sum_dist_a.partial_cmp(&sum_dist_b).unwrap()
        });

        let candidate_i = interior.first().cloned();

        let d_j = candidate_j.map(|j| {
            seed_customers
                .iter()
                .map(|&seed| {
                    problem.travel_cost(
                        problem.service_location(j).id(),
                        problem.service_location(seed).id(),
                    )
                })
                .fold(f64::INFINITY, f64::min)
        });

        let d_i = candidate_i.map(|i| {
            seed_customers
                .iter()
                .map(|&seed| {
                    problem.travel_cost(
                        problem.service_location(i).id(),
                        problem.service_location(seed).id(),
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
        let vehicle_id = solution.available_vehicles_iter().next().unwrap();
        solution.insert_service(&Insertion::NewRoute(NewRouteInsertion {
            service_id: customer,
            vehicle_id,
        }));
    }
}

pub fn construct_solution(
    problem: &Arc<VehicleRoutingProblem>,
    rng: &mut SmallRng,
    constraints: &Vec<Constraint>,
    thread_pool: &rayon::ThreadPool,
) -> WorkingSolution {
    let mut solution = WorkingSolution::new(Arc::clone(problem));

    create_initial_routes(problem, &mut solution);

    let unassigned_services = solution
        .unassigned_services()
        .iter()
        .cloned()
        .collect::<Vec<_>>();

    let depot_id = problem
        .vehicles()
        .iter()
        .find_map(|v| v.depot_location_id())
        .unwrap();

    // unassigned_services.sort_by(|&a, &b| {
    //     let service_a = problem.service(a);
    //     let service_b = problem.service(b);

    //     let urgency_a = service_a
    //         .time_windows()
    //         .iter()
    //         .filter_map(|time_window| time_window.end())
    //         .max()
    //         .map(|end| end - problem.travel_time(depot_id, service_a.location_id()))
    //         .unwrap_or(Timestamp::MAX); // If no time window end -> no urgency

    //     let urgency_b = service_b
    //         .time_windows()
    //         .iter()
    //         .filter_map(|time_window| time_window.end())
    //         .max()
    //         .map(|end| end - problem.travel_time(depot_id, service_b.location_id()))
    //         .unwrap_or(Timestamp::MAX); // If no time window end -> no urgency

    //     urgency_a.cmp(&urgency_b)
    // });

    // let mut services: Vec<_> = (0..problem.services().len()).collect();

    // let vehicles = problem.vehicles();
    // let depot_location = vehicles
    //     .iter()
    //     .filter_map(|vehicle| vehicle.depot_location_id())
    //     .map(|location_id| problem.location(location_id))
    //     .next();

    // let first_service_location = problem.location(problem.service_location(0).id());
    // if let Some(depot_location) = depot_location {
    //     services.sort_by(|&first, &second| {
    //         let first = problem.service_location(first);
    //         let second = problem.service_location(second);

    //         first_service_location
    //             .bearing(first)
    //             .partial_cmp(&first_service_location.bearing(second))
    //             .unwrap()
    //     });
    // }

    // let regret = RegretInsertion::new(3);
    // regret.insert_services(
    //     &mut solution,
    //     RecreateContext {
    //         rng,
    //         constraints: &vec![
    //             Constraint::Global(GlobalConstraintType::TransportCost(TransportCostConstraint)),
    //             Constraint::Route(RouteConstraintType::VehicleCost(VehicleCostConstraint)),
    //         ],
    //         noise_generator: None,
    //         problem,
    //         thread_pool,
    //     },
    // );

    ConstructionBestInsertion::insert_services(
        &mut solution,
        RecreateContext {
            rng,
            // constraints,
            constraints: &vec![
                Constraint::Global(GlobalConstraintType::TransportCost(TransportCostConstraint)),
                Constraint::Route(RouteConstraintType::VehicleCost(VehicleCostConstraint)),
                Constraint::Route(RouteConstraintType::Capacity(CapacityConstraint::new(
                    ScoreLevel::Soft,
                ))),
                Constraint::Activity(ActivityConstraintType::TimeWindow(
                    TimeWindowConstraint::new(ScoreLevel::Soft),
                )),
                Constraint::Route(RouteConstraintType::WaitingDuration(
                    WaitingDurationConstraint,
                )),
            ],
            noise_generator: None,
            problem,
            thread_pool,
        },
    );

    let mut satisfied = false;

    while !satisfied {
        let mut service_to_remove = None;
        for route in solution.routes() {
            for (i, activity) in route.activities().iter().enumerate() {
                let time_window_score = TimeWindowConstraint::compute_time_window_score(
                    ScoreLevel::Hard,
                    activity.service(problem).time_windows(),
                    activity.arrival_time(),
                );

                if time_window_score.hard_score > 0.0 {
                    service_to_remove = Some(activity.service_id());
                    break;
                }

                let vehicle = route.vehicle(problem);

                let load = route.load_at(i);

                if !is_capacity_satisfied(vehicle.capacity(), load) {
                    service_to_remove = Some(activity.service_id());
                    break;
                }
            }

            if service_to_remove.is_some() {
                break;
            }
        }

        if let Some(service_id) = service_to_remove {
            solution.remove_service(service_id);
        } else {
            satisfied = true
        }
    }

    ConstructionBestInsertion::insert_services(
        &mut solution,
        RecreateContext {
            rng,
            constraints,
            noise_generator: None,
            problem,
            thread_pool,
        },
    );

    solution
}
