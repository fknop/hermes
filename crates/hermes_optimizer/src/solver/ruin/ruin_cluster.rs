use fxhash::FxHashSet;
use rand::seq::IndexedRandom;

use crate::{
    problem::job::ActivityId,
    solver::solution::{route_id::RouteIdx, working_solution::WorkingSolution},
    utils::kruskal::kruskal_cluster,
};

use super::{ruin_context::RuinContext, ruin_solution::RuinSolution};

// TODO: support shipments
pub struct RuinCluster;

impl RuinSolution for RuinCluster {
    fn ruin_solution<R>(
        &self,
        solution: &mut WorkingSolution,
        RuinContext {
            rng,
            num_jobs_to_remove,
            problem,
            ..
        }: RuinContext<R>,
    ) where
        R: rand::Rng,
    {
        let mut ruined_routes: FxHashSet<RouteIdx> = FxHashSet::default();

        let mut target_service_id = problem.random_job(rng);
        let mut remaining_to_remove = num_jobs_to_remove;

        while remaining_to_remove > 0 {
            let route_id = solution.route_of_service(target_service_id).unwrap();

            let service_ids = solution
                .route(route_id)
                .activity_ids()
                .iter()
                .map(|job_id| job_id.index())
                .collect::<Vec<_>>();

            let mut removed_service_ids = vec![];
            if let Some(clusters) = kruskal_cluster(problem, &service_ids)
                && !clusters.is_empty()
            {
                let cluster = clusters.choose(rng).unwrap();
                for &service_id in cluster {
                    let removed = solution.remove_service(service_id);
                    if removed {
                        removed_service_ids.push(service_id);
                        remaining_to_remove -= 1;
                        if remaining_to_remove == 0 {
                            break;
                        }
                    }
                }

                ruined_routes.insert(route_id);
            } else {
                break;
            }

            if remaining_to_remove > 0 {
                if let Some(new_service_id) = solution
                    .problem()
                    .nearest_jobs(ActivityId::Service(
                        removed_service_ids.choose(rng).cloned().unwrap(),
                    ))
                    .find(|&service_id| {
                        let route_id = solution.route_of_service(service_id.index());
                        if let Some(route_id) = route_id {
                            !ruined_routes.contains(&route_id)
                        } else {
                            false
                        }
                    })
                {
                    target_service_id = new_service_id.index();
                } else {
                    // No more services to ruin, exit the loop
                    break;
                }
            }
        }
    }
}
