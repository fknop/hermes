use fxhash::FxHashSet;
use rand::seq::IndexedRandom;

use crate::{
    problem::job::{ActivityId, JobIdx},
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

        let mut target_job_id = problem.random_job(rng);
        let mut remaining_to_remove = num_jobs_to_remove;

        while remaining_to_remove > 0 {
            let route_id = solution.route_of_job(target_job_id).unwrap();

            let service_ids = solution
                .route(route_id)
                .activity_ids()
                .iter()
                .map(|job_id| job_id.job_id().get()) // TODO: support shipments
                .collect::<Vec<_>>();

            let mut removed_service_ids: Vec<JobIdx> = vec![];
            if let Some(clusters) = kruskal_cluster(problem, &service_ids)
                && !clusters.is_empty()
            {
                let cluster = clusters.choose(rng).unwrap();
                for &service_id in cluster {
                    let removed = solution.remove_service(service_id.into());
                    if removed {
                        removed_service_ids.push(JobIdx::new(service_id));
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
                if let Some(new_job_id) = solution
                    .problem()
                    .nearest_jobs(ActivityId::Service(
                        removed_service_ids.choose(rng).cloned().unwrap(),
                    ))
                    .find(|&activity_id| {
                        let route_id = solution.route_of_job(activity_id.job_id());
                        if let Some(route_id) = route_id {
                            !ruined_routes.contains(&route_id)
                        } else {
                            false
                        }
                    })
                {
                    target_job_id = new_job_id.job_id();
                } else {
                    // No more services to ruin, exit the loop
                    break;
                }
            }
        }
    }
}
