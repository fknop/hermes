use fxhash::{FxHashMap, FxHashSet};

use crate::{
    problem::{
        job::{ActivityId, Job, JobIdx},
        relation::Relation,
        vehicle::VehicleIdx,
    },
    utils::bitset::BitSet,
};

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
pub struct TaskDependencyGraph {
    edges: FxHashMap<ActivityId, FxHashSet<ActivityId>>,
}

impl TaskDependencyGraph {
    pub fn len(&self) -> usize {
        self.edges.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn add_edge(&mut self, from: ActivityId, to: ActivityId) {
        self.edges.entry(from).or_default().insert(to);
    }

    fn add_edges(&mut self, activity_ids: &[ActivityId]) {
        if activity_ids.len() < 2 {
            return;
        }

        for window in activity_ids.windows(2) {
            let from = window[0];
            let to = window[1];

            self.add_edge(from, to);
        }
    }

    pub fn traverse(&self, start: ActivityId) -> TaskDependencyGraphIterator<'_> {
        TaskDependencyGraphIterator {
            graph: self,
            stack: vec![start],
        }
    }

    /// Returns true if the dependency graph contains a cycle. Making the solution impossible.
    pub fn has_cycle(&self) -> bool {
        let mut visited = FxHashSet::default();
        let mut rec_stack = FxHashSet::default();

        for &node in self.edges.keys() {
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

        if let Some(neighbors) = self.edges.get(&node) {
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

pub struct TaskDependencyGraphIterator<'a> {
    graph: &'a TaskDependencyGraph,
    stack: Vec<ActivityId>,
}

impl<'a> Iterator for TaskDependencyGraphIterator<'a> {
    type Item = ActivityId;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.stack.pop()?;
        if let Some(neighbors) = self.graph.edges.get(&node) {
            self.stack.extend(neighbors);
        }
        Some(node)
    }
}

#[derive(Default, Debug)]
pub struct TaskDependencies {
    directly_after_graph: TaskDependencyGraph,
    after_graph: TaskDependencyGraph,
    before_graph: TaskDependencyGraph,
    directly_before_graph: TaskDependencyGraph,

    fixed_jobs_vehicle: Vec<Option<VehicleIdx>>,

    in_same_route_bitsets: Vec<BitSet>,
    not_in_same_route_bitsets: Vec<BitSet>,
}

impl TaskDependencies {
    pub fn from_jobs_and_relations(jobs: &[Job], relations: &[Relation]) -> Self {
        let mut in_same_route_bitsets: Vec<BitSet> = Vec::with_capacity(jobs.len());
        let mut not_in_same_route_bitsets: Vec<BitSet> = Vec::with_capacity(jobs.len());

        in_same_route_bitsets.resize_with(jobs.len(), || {
            let mut bitset = BitSet::with_capacity(jobs.len());
            bitset.fill_ones();
            bitset
        });
        not_in_same_route_bitsets.resize_with(jobs.len(), || BitSet::with_capacity(jobs.len()));

        let mut task_dependencies = Self {
            in_same_route_bitsets,
            not_in_same_route_bitsets,
            fixed_jobs_vehicle: vec![None; jobs.len()],
            ..Self::default()
        };

        struct InSameRouteGroup {
            vehicle_id: Option<VehicleIdx>,
            bitset: BitSet,
        }

        struct NotInSameRouteGroup {
            bitset: BitSet,
        }

        let mut in_same_route_groups: Vec<InSameRouteGroup> = Vec::new();
        let mut not_in_same_route_groups: Vec<NotInSameRouteGroup> = Vec::new();

        for relation in relations {
            match relation {
                Relation::InSequence(r) => {
                    let mut activity_ids = r.activity_ids.to_vec();
                    task_dependencies.after_graph.add_edges(&activity_ids);
                    activity_ids.reverse();
                    task_dependencies.before_graph.add_edges(&activity_ids);

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
                        .directly_after_graph
                        .add_edges(&activity_ids);
                    task_dependencies.after_graph.add_edges(&activity_ids);

                    activity_ids.reverse();
                    task_dependencies
                        .directly_before_graph
                        .add_edges(&activity_ids);
                    task_dependencies.before_graph.add_edges(&activity_ids);

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
                        if in_same_route_groups[i].vehicle_id.is_none() {
                            in_same_route_groups[i].vehicle_id = vehicle_id;
                        }
                        changed = true;
                    }
                }
            }
        }

        for group in in_same_route_groups {
            for job_id in group.bitset.ones() {
                task_dependencies.in_same_route_bitsets[job_id] = group.bitset.clone();
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
            for job_id in group.bitset.ones() {
                task_dependencies.not_in_same_route_bitsets[job_id] = group.bitset.clone();
            }
        }

        task_dependencies
    }

    pub fn has_in_same_route_dependencies(&self) -> bool {
        !self.in_same_route_bitsets.is_empty()
    }

    pub fn has_not_in_same_route_dependencies(&self) -> bool {
        !self.not_in_same_route_bitsets.is_empty()
    }

    pub fn fixed_vehicle_for_job(&self, job_id: JobIdx) -> Option<VehicleIdx> {
        self.fixed_jobs_vehicle[job_id.get()]
    }

    pub fn traverse(
        &self,
        activity_id: ActivityId,
        dependency_type: TaskDependencyType,
    ) -> TaskDependencyGraphIterator<'_> {
        match dependency_type {
            TaskDependencyType::After => self.after_graph.traverse(activity_id),
            TaskDependencyType::DirectlyAfter => self.directly_after_graph.traverse(activity_id),
            TaskDependencyType::Before => self.before_graph.traverse(activity_id),
            TaskDependencyType::DirectlyBefore => self.directly_before_graph.traverse(activity_id),
        }
    }

    pub fn contains_not_in_same_route_dependencies(
        &self,
        route_bitset: &BitSet,
        segment: &BitSet,
    ) -> bool {
        for not_in_same_route_bitset in &self.not_in_same_route_bitsets {
            if !not_in_same_route_bitset.intersects(segment) {
                continue;
            }

            if route_bitset.intersects(not_in_same_route_bitset) {
                return true;
            }
        }

        false
    }

    pub fn contains_in_same_route_dependencies(
        &self,
        route_bitset: &BitSet,
        segment: &BitSet,
    ) -> bool {
        for in_same_route_bitset in &self.in_same_route_bitsets {
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
    use crate::problem::{
        job::ActivityId,
        relation::*,
        service::{Service, ServiceBuilder},
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

        let dependencies = TaskDependencies::from_jobs_and_relations(&[], &relations);

        assert_eq!(dependencies.after_graph.len(), 3);
        assert_eq!(
            dependencies.after_graph.edges[&ActivityId::service(0)]
                .iter()
                .copied()
                .collect::<Vec<_>>(),
            vec![ActivityId::service(1)]
        );
        assert_eq!(
            dependencies.after_graph.edges[&ActivityId::service(1)]
                .iter()
                .copied()
                .collect::<Vec<_>>(),
            vec![ActivityId::service(2)]
        );
        assert_eq!(
            dependencies.after_graph.edges[&ActivityId::service(2)]
                .iter()
                .copied()
                .collect::<Vec<_>>(),
            vec![ActivityId::service(3)]
        );

        assert!(!dependencies.after_graph.has_cycle())
    }

    #[test]
    fn test_has_cycle() {
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

        let dependencies = TaskDependencies::from_jobs_and_relations(&[], &relations);

        assert!(dependencies.after_graph.has_cycle());

        let relations = vec![Relation::InSequence(InSequenceRelation {
            vehicle_id: None,
            activity_ids: vec![
                ActivityId::service(0),
                ActivityId::service(1),
                ActivityId::service(0),
                ActivityId::service(3),
            ],
        })];

        let graph = TaskDependencies::from_jobs_and_relations(&[], &relations);

        assert!(graph.after_graph.has_cycle())
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

        let dependencies = TaskDependencies::from_jobs_and_relations(&dummy_jobs, &relations);

        for i in 0..=1 {
            assert_eq!(
                dependencies.in_same_route_bitsets[i]
                    .ones()
                    .collect::<Vec<_>>(),
                vec![0, 1]
            );
            assert_eq!(dependencies.fixed_jobs_vehicle[i], Some(VehicleIdx::new(0)));
        }

        for i in 2..=5 {
            assert_eq!(
                dependencies.in_same_route_bitsets[i]
                    .ones()
                    .collect::<Vec<_>>(),
                vec![2, 3, 4, 5]
            );
            assert_eq!(dependencies.fixed_jobs_vehicle[i], None);
        }

        for i in 6..10 {
            assert_eq!(
                dependencies.in_same_route_bitsets[i]
                    .ones()
                    .collect::<Vec<_>>(),
                vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]
            );
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

        let dependencies = TaskDependencies::from_jobs_and_relations(&dummy_jobs, &relations);

        for i in 0..=1 {
            assert_eq!(
                dependencies.not_in_same_route_bitsets[i]
                    .ones()
                    .collect::<Vec<_>>(),
                vec![0, 1]
            );
        }

        for i in 2..=5 {
            assert_eq!(
                dependencies.not_in_same_route_bitsets[i]
                    .ones()
                    .collect::<Vec<_>>(),
                vec![2, 3, 4, 5]
            );
        }

        for i in 6..10 {
            assert_eq!(
                dependencies.not_in_same_route_bitsets[i]
                    .ones()
                    .collect::<Vec<_>>(),
                vec![] as Vec<usize>
            );
        }
    }
}
