use std::fmt::Display;

use jiff::Timestamp;
use rand::{Rng, rngs::SmallRng, seq::SliceRandom};
use serde::Serialize;

use crate::{
    problem::{
        amount::AmountExpression, job::JobId, service::ServiceId, travel_cost_matrix::Distance,
        vehicle_routing_problem::VehicleRoutingProblem,
    },
    solver::{
        insertion::{ExistingRouteInsertion, Insertion, NewRouteInsertion},
        score::Score,
        solution::working_solution::WorkingSolution,
    },
};

use super::{recreate_context::RecreateContext, recreate_solution::RecreateSolution};

#[derive(Default)]
pub struct BestInsertion {
    sort_method: BestInsertionSortMethod,
    blink_rate: f64,
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

pub struct BestInsertionParams {
    pub sort_method: BestInsertionSortMethod,
    pub blink_rate: f64,
}

impl BestInsertion {
    pub fn new(
        BestInsertionParams {
            sort_method,
            blink_rate,
        }: BestInsertionParams,
    ) -> Self {
        BestInsertion {
            sort_method,
            blink_rate,
        }
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
            BestInsertionSortMethod::Demand => unassigned_services.sort_by(|a, b| {
                // Not perfect but good enough for sorting purposes.
                let first_demand_a = problem.job(*a).demand().get(0);
                let first_demand_b = problem.job(*b).demand().get(0);

                first_demand_a.total_cmp(&first_demand_b)
            }),
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

    fn should_blink(&self, rng: &mut SmallRng) -> bool {
        rng.random_bool(self.blink_rate)
    }

    pub fn insert_services(
        &self,
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
                    if self.should_blink(context.rng) {
                        continue;
                    }

                    let insertion = if route.is_empty() {
                        Insertion::NewRoute(NewRouteInsertion {
                            service_id,
                            vehicle_id: route.vehicle_id(),
                        })
                    } else {
                        Insertion::ExistingRoute(ExistingRouteInsertion {
                            route_id,
                            service_id,
                            position,
                        })
                    };

                    let score = context.compute_insertion_score(solution, &insertion);

                    if score < best_score {
                        best_score = score;
                        best_insertion = Some(insertion);
                    }
                }
            }

            // if solution.has_available_vehicle() {
            //     for vehicle_id in solution.available_vehicles_iter() {
            //         let new_route_insertion = Insertion::NewRoute(NewRouteInsertion {
            //             service_id,
            //             vehicle_id,
            //         });

            //         let score = context.compute_insertion_score(solution, &new_route_insertion);

            //         if score < best_score {
            //             best_score = score;
            //             best_insertion = Some(new_route_insertion);
            //         }
            //     }
            // }

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

        self.insert_services(&unassigned_services, solution, context);
    }
}
