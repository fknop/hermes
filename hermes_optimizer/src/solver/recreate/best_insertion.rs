use parking_lot::RwLock;
use rand::{
    rngs::SmallRng,
    seq::{IndexedRandom, SliceRandom},
};

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
}

enum SortMethod {
    Random,
    Demand,
    Far,
    Close,
}

impl SortMethod {
    fn weight(&self) -> usize {
        match self {
            SortMethod::Random => 4,
            SortMethod::Demand => 4,
            SortMethod::Far => 2,
            SortMethod::Close => 1,
        }
    }
}

const METHODS: [SortMethod; 4] = [
    SortMethod::Random,
    SortMethod::Demand,
    SortMethod::Far,
    SortMethod::Close,
];

impl BestInsertion {
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
        let method = METHODS
            .choose_weighted(rng, |method| method.weight())
            .ok()
            .unwrap();

        match method {
            SortMethod::Random => {
                unassigned_services.shuffle(rng);
            }
            SortMethod::Demand => {
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
            SortMethod::Far => {
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
            SortMethod::Close => {
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
        }
    }

    pub fn insert_services(
        unassigned_services: &Vec<ServiceId>,
        solution: &mut WorkingSolution,
        mut context: RecreateContext,
    ) {
        for &service_id in unassigned_services {
            let mut best_insertion: Option<Insertion> = None;
            let mut best_score = Score::MAX;

            let routes = solution.routes();
            for (route_id, route) in routes.iter().enumerate() {
                for position in 0..route.activities().len() {
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
                for vehicle_id in solution.available_vehicles() {
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

        self.sort_unassigned_services(solution.problem(), &mut unassigned_services, context.rng);
        // unassigned_services.shuffle(context.rng);

        BestInsertion::insert_services(&unassigned_services, solution, context);
    }
}
