use std::fmt::Display;

use jiff::Timestamp;
use parking_lot::RwLock;
use rand::{
    Rng,
    rngs::SmallRng,
    seq::{IndexedRandom, SliceRandom},
};
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
use serde::Serialize;

use crate::{
    problem::{
        capacity::Capacity, service::ServiceId, travel_cost_matrix::Distance,
        vehicle_routing_problem::VehicleRoutingProblem,
    },
    solver::{
        insertion::{ExistingRouteInsertion, Insertion, NewRouteInsertion},
        score::Score,
        working_solution::WorkingSolution,
    },
};

use super::{recreate_context::RecreateContext, recreate_solution::RecreateSolution};

#[derive(Default)]
pub struct BestInsertion {
    cached_min_max_demand: RwLock<Option<(Capacity, Capacity)>>,
    sort_method: BestInsertionSortMethod,
}

#[derive(Default, Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize)]
pub enum BestInsertionSortMethod {
    #[default]
    Random,
    Demand,
    Far,
    Close,
    TimeWindow,
}

impl Display for BestInsertionSortMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Random => write!(f, "Random"),
            Self::Demand => write!(f, "Demand"),
            Self::Far => write!(f, "Far"),
            Self::Close => write!(f, "Close"),
            Self::TimeWindow => write!(f, "TimeWindow"),
        }
    }
}

impl BestInsertionSortMethod {
    fn weight(&self) -> usize {
        match self {
            BestInsertionSortMethod::Random => 4,
            BestInsertionSortMethod::Demand => 4,
            BestInsertionSortMethod::Far => 2,
            BestInsertionSortMethod::Close => 1,
            BestInsertionSortMethod::TimeWindow => 1,
        }
    }
}

const METHODS: [BestInsertionSortMethod; 4] = [
    BestInsertionSortMethod::Random,
    BestInsertionSortMethod::Demand,
    BestInsertionSortMethod::Far,
    BestInsertionSortMethod::Close,
];

impl BestInsertion {
    pub fn new(sort_method: BestInsertionSortMethod) -> Self {
        BestInsertion {
            sort_method,
            ..Default::default()
        }
    }

    fn min_max_demand(&self, problem: &VehicleRoutingProblem) -> (Capacity, Capacity) {
        let is_none;
        {
            let cached_minimum_demand = self.cached_min_max_demand.read();
            is_none = cached_minimum_demand.is_none();
        }

        if is_none {
            let demands: Vec<&Capacity> = problem
                .services()
                .iter()
                .map(|service| service.demand())
                .collect();

            self.cached_min_max_demand
                .write()
                .replace(Capacity::compute_min_max_capacities(&demands));
        }

        let cached_min_max_demand = self.cached_min_max_demand.read();
        let (min, max) = cached_min_max_demand.as_ref().unwrap();
        (min.clone(), max.clone())
    }

    pub fn sort_unassigned_services(
        &self,
        problem: &VehicleRoutingProblem,
        unassigned_services: &mut [ServiceId],
        rng: &mut SmallRng,
    ) {
        match self.sort_method {
            BestInsertionSortMethod::Random => {
                unassigned_services.shuffle(rng);
            }
            BestInsertionSortMethod::Demand => {
                let (min_demand, max_demand) = self.min_max_demand(problem);
                unassigned_services.sort_by(|a, b| {
                    let demand_a = problem
                        .service(*a)
                        .demand()
                        .normalize(&min_demand, &max_demand);
                    let demand_b = problem
                        .service(*b)
                        .demand()
                        .normalize(&min_demand, &max_demand);

                    demand_b
                        .partial_cmp(&demand_a)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
            }
            BestInsertionSortMethod::Far => {
                unassigned_services.sort_by_key(|&service| {
                    let distance_from_depot = problem
                        .vehicles()
                        .iter()
                        .filter_map(|vehicle| vehicle.depot_location_id())
                        .map(|depot_location_id| {
                            problem.travel_distance(
                                depot_location_id,
                                problem.service_location(service).id(),
                            )
                        })
                        .sum::<Distance>()
                        / problem.vehicles().len() as Distance;

                    -distance_from_depot.round() as i64
                });
            }
            BestInsertionSortMethod::Close => {
                unassigned_services.sort_by_key(|&service| {
                    let distance_from_depot = problem
                        .vehicles()
                        .iter()
                        .filter_map(|vehicle| vehicle.depot_location_id())
                        .map(|depot_location_id| {
                            problem.travel_distance(
                                depot_location_id,
                                problem.service_location(service).id(),
                            )
                        })
                        .sum::<Distance>()
                        / problem.vehicles().len() as Distance;

                    distance_from_depot.round() as i64
                });
            }
            BestInsertionSortMethod::TimeWindow => {
                unassigned_services.sort_by_key(|&service_id| {
                    let service = problem.service(service_id);

                    let end = service
                        .time_windows()
                        .iter()
                        .filter_map(|tw| tw.end())
                        .max();

                    end.unwrap_or(Timestamp::MAX)
                });
            }
        }
    }

    pub fn insert_services(
        unassigned_services: &Vec<ServiceId>,
        solution: &mut WorkingSolution,
        context: RecreateContext,
    ) {
        for &service_id in unassigned_services {
            let mut best_insertion: Option<Insertion> = None;
            let mut best_score = Score::MAX;

            let routes = solution.routes();

            for (route_id, route) in routes.iter().enumerate() {
                for position in 0..=route.activities().len() {
                    let insertion = Insertion::ExistingRoute(ExistingRouteInsertion {
                        route_id,
                        service_id,
                        position,
                    });

                    let score = context.compute_insertion_score(solution, &insertion);

                    if score < best_score {
                        best_score = score;
                        best_insertion = Some(insertion);
                    }
                }
            }

            if solution.has_available_vehicle() {
                for vehicle_id in solution.available_vehicles_iter() {
                    let new_route_insertion = Insertion::NewRoute(NewRouteInsertion {
                        service_id,
                        vehicle_id,
                    });

                    let score = context.compute_insertion_score(solution, &new_route_insertion);

                    if score < best_score {
                        // best_score = score;
                        best_insertion = Some(new_route_insertion);
                    }
                }
            }

            if let Some(insertion) = best_insertion {
                solution.insert_service(&insertion);
            } else {
                panic!("No insertion possible")
            }
        }
    }
}

impl RecreateSolution for BestInsertion {
    fn recreate_solution(&self, solution: &mut WorkingSolution, context: RecreateContext) {
        let mut unassigned_services: Vec<_> =
            solution.unassigned_services().iter().copied().collect();

        self.sort_unassigned_services(context.problem, &mut unassigned_services, context.rng);
        // unassigned_services.shuffle(context.rng);

        BestInsertion::insert_services(&unassigned_services, solution, context);
    }
}
