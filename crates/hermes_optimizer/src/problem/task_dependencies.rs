use fxhash::{FxHashMap, FxHashSet};
use thiserror::Error;

use crate::{
    problem::{
        job::{ActivityId, Job, JobIdx},
        relation::Relation,
        vehicle::VehicleIdx,
    },
    utils::bitset::BitSet,
};

#[derive(Error, Debug, PartialEq, Eq)]
pub enum MalformedRelationError {
    #[error("Relations contain cycle")]
    Cycle,

    #[error("Conflicting relations, both in same routes and not in same routes")]
    Conflict,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TaskDependencyType {
    DirectlyAfter,
    After,
    Before,
    DirectlyBefore,
}

impl TaskDependencyType {
    pub fn rev(&self) -> Self {
        match self {
            TaskDependencyType::DirectlyAfter => TaskDependencyType::DirectlyBefore,
            TaskDependencyType::After => TaskDependencyType::Before,
            TaskDependencyType::Before => TaskDependencyType::After,
            TaskDependencyType::DirectlyBefore => TaskDependencyType::DirectlyAfter,
        }
    }
}

#[derive(Default, Debug)]
struct TaskSequenceList {
    list: FxHashMap<ActivityId, FxHashSet<ActivityId>>,
}

impl TaskSequenceList {
    fn len(&self) -> usize {
        self.list.len()
    }

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn add_edge(&mut self, from: ActivityId, to: ActivityId) {
        self.list.entry(from).or_default().insert(to);
    }

    fn add_edges(&mut self, activity_ids: &[ActivityId]) {
        if activity_ids.len() < 2 {
            return;
        }

        for i in 0..activity_ids.len() {
            for j in i + 1..activity_ids.len() {
                let a = activity_ids[i];
                let b = activity_ids[j];

                self.add_edge(a, b);
            }
        }
    }

    pub fn contains(&self, key: ActivityId, value: ActivityId) -> bool {
        if let Some(neighbors) = self.list.get(&key) {
            neighbors.contains(&value)
        } else {
            false
        }
    }

    pub fn traverse(&self, start: ActivityId) -> impl Iterator<Item = ActivityId> {
        self.list.get(&start).unwrap().iter().copied()
    }

    /// Returns true if the dependency graph contains a cycle. Making the solution impossible.
    fn has_cycle(&self) -> bool {
        let mut visited = FxHashSet::default();
        let mut rec_stack = FxHashSet::default();

        for &node in self.list.keys() {
            if !visited.contains(&node) && self.dfs_cycle_check(node, &mut visited, &mut rec_stack)
            {
                return true;
            }
        }

        false
    }

    /// Recursive DFS to detect back-edges (cycles).
    fn dfs_cycle_check(
        &self,
        node: ActivityId,
        visited: &mut FxHashSet<ActivityId>,
        rec_stack: &mut FxHashSet<ActivityId>,
    ) -> bool {
        // Mark the current node as visited and add it to the recursion stack
        visited.insert(node);
        rec_stack.insert(node);

        if let Some(neighbors) = self.list.get(&node) {
            for &neighbor in neighbors {
                // If not visited, recurse
                if !visited.contains(&neighbor) {
                    if self.dfs_cycle_check(neighbor, visited, rec_stack) {
                        return true;
                    }
                }
                // If the neighbor is already in the current recursion stack, we found a cycle!
                else if rec_stack.contains(&neighbor) {
                    return true;
                }
            }
        }

        // Remove the node from the recursion stack before returning
        rec_stack.remove(&node);
        false
    }
}

#[derive(Default, Debug)]
pub struct TaskDependencies {
    sequence_after: TaskSequenceList,
    sequence_before: TaskSequenceList,
    direct_sequence_after: TaskSequenceList,
    direct_sequence_before: TaskSequenceList,

    fixed_jobs_vehicle: Vec<Option<VehicleIdx>>,

    in_same_route_groups: Vec<BitSet>,
    not_in_same_route_groups: Vec<BitSet>,
}

impl TaskDependencies {
    pub fn try_from_jobs_and_relations(
        jobs: &[Job],
        relations: &[Relation],
    ) -> Result<Self, MalformedRelationError> {
        let in_same_route_bitsets: Vec<BitSet> = Vec::new();
        let not_in_same_route_bitsets: Vec<BitSet> = Vec::new();

        let mut task_dependencies = Self {
            in_same_route_groups: in_same_route_bitsets,
            not_in_same_route_groups: not_in_same_route_bitsets,
            fixed_jobs_vehicle: vec![None; jobs.len()],
            ..Self::default()
        };

        #[derive(Debug)]
        struct InSameRouteGroup {
            vehicle_id: Option<VehicleIdx>,
            bitset: BitSet,
        }

        #[derive(Debug)]
        struct NotInSameRouteGroup {
            bitset: BitSet,
        }

        let mut in_same_route_groups: Vec<InSameRouteGroup> = Vec::new();
        let mut not_in_same_route_groups: Vec<NotInSameRouteGroup> = Vec::new();

        for relation in relations {
            match relation {
                Relation::InSequence(r) => {
                    let mut activity_ids = r.activity_ids.to_vec();
                    task_dependencies.sequence_after.add_edges(&activity_ids);
                    activity_ids.reverse();
                    task_dependencies.sequence_before.add_edges(&activity_ids);

                    let mut bitset = BitSet::with_capacity(jobs.len());
                    for activity_id in &r.activity_ids {
                        bitset.insert(activity_id.job_id().get());
                    }

                    in_same_route_groups.push(InSameRouteGroup {
                        vehicle_id: r.vehicle_id,
                        bitset,
                    });
                }
                Relation::InDirectSequence(r) => {
                    let mut activity_ids = r.activity_ids.to_vec();
                    task_dependencies
                        .direct_sequence_after
                        .add_edges(&activity_ids);
                    task_dependencies.sequence_after.add_edges(&activity_ids);

                    activity_ids.reverse();
                    task_dependencies
                        .direct_sequence_before
                        .add_edges(&activity_ids);
                    task_dependencies.sequence_before.add_edges(&activity_ids);

                    let mut bitset = BitSet::with_capacity(jobs.len());
                    for activity_id in &r.activity_ids {
                        bitset.insert(activity_id.job_id().get());
                    }

                    in_same_route_groups.push(InSameRouteGroup {
                        vehicle_id: r.vehicle_id,
                        bitset,
                    });
                }
                Relation::InSameRoute(r) => {
                    let mut bitset = BitSet::with_capacity(jobs.len());
                    for activity_id in &r.activity_ids {
                        bitset.insert(activity_id.job_id().get());
                    }

                    in_same_route_groups.push(InSameRouteGroup {
                        vehicle_id: r.vehicle_id,
                        bitset,
                    });
                }
                Relation::NotInSameRoute(r) => {
                    let mut bitset = BitSet::with_capacity(jobs.len());
                    for activity_id in &r.activity_ids {
                        bitset.insert(activity_id.job_id().get());
                    }

                    not_in_same_route_groups.push(NotInSameRouteGroup { bitset });
                }
            }
        }

        // Merge until stable
        let mut changed = true;
        while changed {
            changed = false;
            for i in 0..in_same_route_groups.len() {
                for j in (i + 1..in_same_route_groups.len()).rev() {
                    if !in_same_route_groups[i]
                        .bitset
                        .is_disjoint(&in_same_route_groups[j].bitset)
                    {
                        let InSameRouteGroup { vehicle_id, bitset } =
                            in_same_route_groups.remove(j);
                        in_same_route_groups[i].bitset.union_with(&bitset);

                        if let Some(group_vehicle_id) = in_same_route_groups[i].vehicle_id {
                            if let Some(vehicle_id) = vehicle_id
                                && group_vehicle_id != vehicle_id
                            {
                                // If jobs in the same group have different vehicle ids, raise a conflict
                                return Err(MalformedRelationError::Conflict);
                            }
                        } else {
                            in_same_route_groups[i].vehicle_id = vehicle_id;
                        }

                        changed = true;
                    }
                }
            }
        }

        for group in in_same_route_groups {
            task_dependencies
                .in_same_route_groups
                .push(group.bitset.clone());

            for job_id in group.bitset.ones() {
                task_dependencies.fixed_jobs_vehicle[job_id] = group.vehicle_id;
            }
        }

        let mut changed = true;
        while changed {
            changed = false;
            for i in 0..not_in_same_route_groups.len() {
                for j in (i + 1..not_in_same_route_groups.len()).rev() {
                    if !not_in_same_route_groups[i]
                        .bitset
                        .is_disjoint(&not_in_same_route_groups[j].bitset)
                    {
                        let NotInSameRouteGroup { bitset } = not_in_same_route_groups.remove(j);
                        not_in_same_route_groups[i].bitset.union_with(&bitset);

                        changed = true;
                    }
                }
            }
        }

        for group in not_in_same_route_groups {
            task_dependencies
                .not_in_same_route_groups
                .push(group.bitset);
        }

        if task_dependencies.sequence_after.has_cycle()
            || task_dependencies.direct_sequence_after.has_cycle()
        {
            return Err(MalformedRelationError::Cycle);
        }

        for same_route_bitset in &task_dependencies.in_same_route_groups {
            for not_same_route_bitset in &task_dependencies.not_in_same_route_groups {
                if same_route_bitset.intersection_count(not_same_route_bitset) > 1 {
                    return Err(MalformedRelationError::Conflict);
                }
            }
        }

        Ok(task_dependencies)
    }

    pub fn has_in_same_route_dependencies(&self) -> bool {
        !self.in_same_route_groups.is_empty()
    }

    pub fn has_not_in_same_route_dependencies(&self) -> bool {
        !self.not_in_same_route_groups.is_empty()
    }

    pub fn fixed_vehicle_for_job(&self, job_id: JobIdx) -> Option<VehicleIdx> {
        self.fixed_jobs_vehicle[job_id.get()]
    }

    pub fn traverse(
        &self,
        activity_id: ActivityId,
        dependency_type: TaskDependencyType,
    ) -> impl Iterator<Item = ActivityId> {
        match dependency_type {
            TaskDependencyType::After => self.sequence_after.traverse(activity_id),
            TaskDependencyType::DirectlyAfter => self.direct_sequence_after.traverse(activity_id),
            TaskDependencyType::Before => self.sequence_before.traverse(activity_id),
            TaskDependencyType::DirectlyBefore => self.direct_sequence_before.traverse(activity_id),
        }
    }

    pub fn is(
        &self,
        first: ActivityId,
        dependency_type: TaskDependencyType,
        second: ActivityId,
    ) -> bool {
        match dependency_type {
            TaskDependencyType::After => self.sequence_after.contains(second, first),
            TaskDependencyType::DirectlyAfter => self.direct_sequence_after.contains(second, first),
            TaskDependencyType::Before => self.sequence_before.contains(second, first),
            TaskDependencyType::DirectlyBefore => {
                self.direct_sequence_before.contains(second, first)
            }
        }
    }

    pub fn contains_not_in_same_route_dependencies(
        &self,
        route_bitset: &BitSet,
        segment: &BitSet,
    ) -> bool {
        for not_in_same_route_bitset in &self.not_in_same_route_groups {
            if !not_in_same_route_bitset.intersects(segment) {
                continue;
            }

            if route_bitset.intersects(not_in_same_route_bitset) {
                return true;
            }
        }

        false
    }

    pub fn contains_in_same_route_dependencies_for_unassigned_job(
        &self,
        job_id: JobIdx,
        route_bitset: &BitSet,
    ) -> bool {
        self.in_same_route_groups
            .iter()
            .any(|bs| bs.contains(job_id.get()) && route_bitset.intersects(bs))
    }

    pub fn contains_in_same_route_dependencies(
        &self,
        route_bitset: &BitSet,
        segment: &BitSet,
    ) -> bool {
        for in_same_route_bitset in &self.in_same_route_groups {
            if !in_same_route_bitset.intersects(segment) {
                continue;
            }

            if route_bitset.intersects(in_same_route_bitset) {
                return true;
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::problem::{
        job::ActivityId,
        relation::*,
        service::{Service, ServiceBuilder},
        task_dependencies,
    };

    use super::*;

    fn create_dummy_job() -> Job {
        let mut service_builder = ServiceBuilder::default();
        service_builder.set_external_id("dummy".to_owned());
        service_builder.set_location_id(0);
        let service = service_builder.build();
        Job::Service(service)
    }

    #[test]
    fn test_from_relations() {
        let dummy_jobs = (0..10).map(|_| create_dummy_job()).collect::<Vec<_>>();

        let relations = vec![
            Relation::InSequence(InSequenceRelation {
                vehicle_id: None,
                activity_ids: vec![
                    ActivityId::service(0),
                    ActivityId::service(1),
                    ActivityId::service(2),
                    ActivityId::service(3),
                ],
            }),
            Relation::InDirectSequence(InDirectSequenceRelation {
                vehicle_id: None,
                activity_ids: vec![ActivityId::service(0), ActivityId::service(1)],
            }),
        ];

        let dependencies =
            TaskDependencies::try_from_jobs_and_relations(&dummy_jobs, &relations).unwrap();

        assert_eq!(dependencies.sequence_after.len(), 3);
        assert_eq!(
            dependencies.sequence_after.list[&ActivityId::service(0)],
            [
                ActivityId::service(1),
                ActivityId::service(2),
                ActivityId::service(3)
            ]
            .into_iter()
            .collect()
        );
        assert_eq!(
            dependencies.sequence_after.list[&ActivityId::service(1)]
                .iter()
                .copied()
                .collect::<Vec<_>>(),
            vec![ActivityId::service(2), ActivityId::service(3)]
        );
        assert_eq!(
            dependencies.sequence_after.list[&ActivityId::service(2)]
                .iter()
                .copied()
                .collect::<Vec<_>>(),
            vec![ActivityId::service(3)]
        );

        assert!(!dependencies.sequence_after.has_cycle())
    }

    #[test]
    fn test_has_cycle() {
        let dummy_jobs = (0..10).map(|_| create_dummy_job()).collect::<Vec<_>>();

        let relations = vec![
            Relation::InSequence(InSequenceRelation {
                vehicle_id: None,
                activity_ids: vec![
                    ActivityId::service(0),
                    ActivityId::service(1),
                    ActivityId::service(2),
                    ActivityId::service(3),
                ],
            }),
            Relation::InDirectSequence(InDirectSequenceRelation {
                vehicle_id: None,
                activity_ids: vec![ActivityId::service(0), ActivityId::service(1)],
            }),
            Relation::InDirectSequence(InDirectSequenceRelation {
                vehicle_id: None,
                activity_ids: vec![ActivityId::service(3), ActivityId::service(0)],
            }),
        ];

        let error =
            TaskDependencies::try_from_jobs_and_relations(&dummy_jobs, &relations).unwrap_err();

        assert_eq!(error, MalformedRelationError::Cycle);

        let relations = vec![Relation::InSequence(InSequenceRelation {
            vehicle_id: None,
            activity_ids: vec![
                ActivityId::service(0),
                ActivityId::service(1),
                ActivityId::service(0),
                ActivityId::service(3),
            ],
        })];

        let error =
            TaskDependencies::try_from_jobs_and_relations(&dummy_jobs, &relations).unwrap_err();

        assert_eq!(error, MalformedRelationError::Cycle)
    }

    #[test]
    fn test_from_relations_in_same_route() {
        let dummy_jobs = (0..10).map(|_| create_dummy_job()).collect::<Vec<_>>();
        let relations = vec![
            Relation::InSameRoute(InSameRouteRelation {
                vehicle_id: Some(VehicleIdx::new(0)),
                activity_ids: vec![ActivityId::service(0), ActivityId::service(1)],
            }),
            Relation::InSameRoute(InSameRouteRelation {
                vehicle_id: None,
                activity_ids: vec![ActivityId::service(2), ActivityId::service(3)],
            }),
            Relation::InSameRoute(InSameRouteRelation {
                vehicle_id: None,
                activity_ids: vec![ActivityId::service(3), ActivityId::service(4)],
            }),
            Relation::InSameRoute(InSameRouteRelation {
                vehicle_id: None,
                activity_ids: vec![ActivityId::service(4), ActivityId::service(5)],
            }),
        ];

        let dependencies =
            TaskDependencies::try_from_jobs_and_relations(&dummy_jobs, &relations).unwrap();

        assert_eq!(dependencies.in_same_route_groups.len(), 2);

        assert_eq!(
            dependencies.in_same_route_groups[0]
                .ones()
                .collect::<Vec<_>>(),
            vec![0, 1]
        );
        for i in 0..=1 {
            assert_eq!(dependencies.fixed_jobs_vehicle[i], Some(VehicleIdx::new(0)));
        }

        assert_eq!(
            dependencies.in_same_route_groups[1]
                .ones()
                .collect::<Vec<_>>(),
            vec![2, 3, 4, 5]
        );
        for i in 2..=5 {
            assert_eq!(dependencies.fixed_jobs_vehicle[i], None);
        }
    }

    #[test]
    fn test_from_relations_not_in_same_route() {
        let dummy_jobs = (0..10).map(|_| create_dummy_job()).collect::<Vec<_>>();
        let relations = vec![
            Relation::NotInSameRoute(NotInSameRouteRelation {
                activity_ids: vec![ActivityId::service(0), ActivityId::service(1)],
            }),
            Relation::NotInSameRoute(NotInSameRouteRelation {
                activity_ids: vec![ActivityId::service(2), ActivityId::service(3)],
            }),
            Relation::NotInSameRoute(NotInSameRouteRelation {
                activity_ids: vec![ActivityId::service(3), ActivityId::service(4)],
            }),
            Relation::NotInSameRoute(NotInSameRouteRelation {
                activity_ids: vec![ActivityId::service(4), ActivityId::service(5)],
            }),
        ];

        let dependencies =
            TaskDependencies::try_from_jobs_and_relations(&dummy_jobs, &relations).unwrap();

        assert_eq!(dependencies.not_in_same_route_groups.len(), 2);
        assert_eq!(
            dependencies.not_in_same_route_groups[0]
                .ones()
                .collect::<Vec<_>>(),
            vec![0, 1]
        );

        assert_eq!(
            dependencies.not_in_same_route_groups[1]
                .ones()
                .collect::<Vec<_>>(),
            vec![2, 3, 4, 5]
        );
    }

    #[test]
    fn test_conflict() {
        let dummy_jobs = (0..10).map(|_| create_dummy_job()).collect::<Vec<_>>();
        let relations = vec![
            Relation::InSameRoute(InSameRouteRelation {
                vehicle_id: None,
                activity_ids: vec![ActivityId::service(0), ActivityId::service(1)],
            }),
            Relation::InSameRoute(InSameRouteRelation {
                vehicle_id: None,
                activity_ids: vec![ActivityId::service(1), ActivityId::service(2)],
            }),
            Relation::NotInSameRoute(NotInSameRouteRelation {
                activity_ids: vec![ActivityId::service(0), ActivityId::service(2)],
            }),
        ];

        let error =
            TaskDependencies::try_from_jobs_and_relations(&dummy_jobs, &relations).unwrap_err();

        assert_eq!(error, MalformedRelationError::Conflict);
    }

    #[test]
    fn test_vehicle_conflict() {
        let dummy_jobs = (0..10).map(|_| create_dummy_job()).collect::<Vec<_>>();
        let relations = vec![
            Relation::InSameRoute(InSameRouteRelation {
                vehicle_id: Some(VehicleIdx::new(0)),
                activity_ids: vec![ActivityId::service(0), ActivityId::service(1)],
            }),
            Relation::InSameRoute(InSameRouteRelation {
                vehicle_id: Some(VehicleIdx::new(1)),
                activity_ids: vec![ActivityId::service(1), ActivityId::service(2)],
            }),
        ];

        let error =
            TaskDependencies::try_from_jobs_and_relations(&dummy_jobs, &relations).unwrap_err();

        assert_eq!(error, MalformedRelationError::Conflict);
    }
}
