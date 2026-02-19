use fxhash::FxHashSet;
use rand::seq::IndexedRandom;

use crate::{
    problem::job::{ActivityId, JobIdx},
    solver::solution::{route_id::RouteIdx, working_solution::WorkingSolution},
    utils::kruskal::kruskal_cluster,
};

use super::{ruin_context::RuinContext, ruin_solution::RuinSolution};

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

        let mut target_job_id = solution.random_assigned_job(rng);

        let mut remaining_to_remove = num_jobs_to_remove;

        while remaining_to_remove > 0
            && let Some(target) = target_job_id
        {
            let route_id = solution.route_of_job(target).unwrap();

            let mut removed_activity_ids: Vec<ActivityId> = vec![];
            if let Some(clusters) =
                kruskal_cluster(problem, solution.route(route_id).activity_ids())
                && !clusters.is_empty()
            {
                let cluster = clusters.choose(rng).unwrap();
                for &activity_id in cluster {
                    let removed = solution.remove_activity(activity_id);
                    if removed {
                        removed_activity_ids.push(activity_id);
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
                target_job_id = solution
                    .problem()
                    .nearest_jobs(removed_activity_ids.choose(rng).copied().unwrap())
                    .find(|&activity_id| {
                        let route_id = solution.route_of_activity(activity_id);
                        if let Some(route_id) = route_id {
                            !ruined_routes.contains(&route_id)
                        } else {
                            false
                        }
                    })
                    .map(|activity_id| activity_id.job_id());
            }
        }
    }
}
