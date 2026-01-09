use fxhash::{FxBuildHasher, FxHashMap};

use crate::{
    problem::{job::JobIdx, vehicle_routing_problem::VehicleRoutingProblem},
    solver::{
        insertion::Insertion,
        score::Score,
        solution::{route::WorkingSolutionRoute, route_id::RouteIdx},
    },
};

pub struct InsertionCacheEntry {
    pub score: Score,
    pub insertion: Insertion,
}

pub struct InsertionCache {
    cache: FxHashMap<(RouteIdx, usize, JobIdx), InsertionCacheEntry>,
}

impl InsertionCache {
    pub fn new() -> Self {
        Self {
            cache: FxHashMap::default(),
        }
    }

    pub fn get(
        &self,
        route_idx: RouteIdx,
        version: usize,
        job_idx: JobIdx,
    ) -> Option<&InsertionCacheEntry> {
        self.cache.get(&(route_idx, version, job_idx))
    }

    pub fn insert(
        &mut self,
        route_idx: RouteIdx,
        version: usize,
        job_idx: JobIdx,
        score: Score,
        insertion: Insertion,
    ) {
        self.cache.insert(
            (route_idx, version, job_idx),
            InsertionCacheEntry { score, insertion },
        );
    }

    pub fn clear(&mut self, routes: &[WorkingSolutionRoute]) {
        self.cache.retain(|(route_idx, version, _), _| {
            if let Some(route) = routes.get(route_idx.get()) {
                *version == route.version()
            } else {
                false
            }
        });
    }
}
