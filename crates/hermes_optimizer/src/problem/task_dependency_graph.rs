use fxhash::{FxHashMap, FxHashSet};

use crate::problem::{job::ActivityId, relation::Relation};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TaskDependencyType {
    DirectlyAfter,
    After,
}

#[derive(Debug, Eq, PartialEq)]
pub struct TaskDependencyEdge {
    activity_id: ActivityId,
    edge_type: TaskDependencyType,
}

#[derive(Default, Debug)]
pub struct TaskDependencyGraph {
    adjacency_list: FxHashMap<ActivityId, Vec<TaskDependencyEdge>>,
}

impl TaskDependencyGraph {
    pub fn from_relations(relations: &[Relation]) -> Self {
        let mut graph = Self::default();

        for relation in relations {
            match relation {
                Relation::InSequence(r) => {
                    graph.add_edges(&r.activity_ids, TaskDependencyType::After)
                }
                Relation::InDirectSequence(r) => {
                    graph.add_edges(&r.activity_ids, TaskDependencyType::DirectlyAfter)
                }
                Relation::InSameRoute(_) | Relation::NotInSameRoute(_) => {}
            }
        }
        graph
    }

    /// Returns true if the dependency graph contains a cycle. Making the solution impossible.
    pub fn has_cycle(&self) -> bool {
        let mut visited = FxHashSet::default();
        let mut rec_stack = FxHashSet::default();

        for &node in self.adjacency_list.keys() {
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

        if let Some(neighbors) = self.adjacency_list.get(&node) {
            for neighbor in neighbors {
                // If not visited, recurse
                if !visited.contains(&neighbor.activity_id) {
                    if self.dfs_cycle_check(neighbor.activity_id, visited, rec_stack) {
                        return true;
                    }
                }
                // If the neighbor is already in the current recursion stack, we found a cycle!
                else if rec_stack.contains(&neighbor.activity_id) {
                    return true;
                }
            }
        }

        // Remove the node from the recursion stack before returning
        rec_stack.remove(&node);
        false
    }

    fn add_edges(&mut self, activity_ids: &[ActivityId], edge_type: TaskDependencyType) {
        if activity_ids.len() < 2 {
            return;
        }

        for window in activity_ids.windows(2) {
            let from = window[0];
            let to = window[1];

            self.add_edge(from, to, edge_type);
        }
    }

    fn add_edge(&mut self, from: ActivityId, to: ActivityId, edge_type: TaskDependencyType) {
        self.adjacency_list
            .entry(from)
            .or_default()
            .push(TaskDependencyEdge {
                activity_id: to,
                edge_type,
            });
    }
}

#[cfg(test)]
mod tests {
    use crate::problem::{job::ActivityId, relation::*};

    use super::*;

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

        let graph = TaskDependencyGraph::from_relations(&relations);

        assert_eq!(graph.adjacency_list.len(), 3);
        assert_eq!(
            graph.adjacency_list[&ActivityId::service(0)],
            vec![
                TaskDependencyEdge {
                    activity_id: ActivityId::service(1),
                    edge_type: TaskDependencyType::After,
                },
                TaskDependencyEdge {
                    activity_id: ActivityId::service(1),
                    edge_type: TaskDependencyType::DirectlyAfter,
                }
            ]
        );
        assert_eq!(
            graph.adjacency_list[&ActivityId::service(1)],
            vec![TaskDependencyEdge {
                activity_id: ActivityId::service(2),
                edge_type: TaskDependencyType::After,
            }]
        );
        assert_eq!(
            graph.adjacency_list[&ActivityId::service(2)],
            vec![TaskDependencyEdge {
                activity_id: ActivityId::service(3),
                edge_type: TaskDependencyType::After,
            }]
        );

        assert!(!graph.has_cycle())
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

        let graph = TaskDependencyGraph::from_relations(&relations);

        assert!(graph.has_cycle());

        let relations = vec![Relation::InSequence(InSequenceRelation {
            vehicle_id: None,
            activity_ids: vec![
                ActivityId::service(0),
                ActivityId::service(1),
                ActivityId::service(0),
                ActivityId::service(3),
            ],
        })];

        let graph = TaskDependencyGraph::from_relations(&relations);

        assert!(graph.has_cycle())
    }
}
