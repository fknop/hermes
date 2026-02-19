use std::collections::BinaryHeap;

use fxhash::FxHashMap;

use crate::problem::{
    job::ActivityId, vehicle::VehicleIdx, vehicle_routing_problem::VehicleRoutingProblem,
};

struct KruskalEdge {
    from: ActivityId,
    to: ActivityId,
    weight: f64,
}

impl PartialEq for KruskalEdge {
    fn eq(&self, other: &Self) -> bool {
        self.weight == other.weight
    }
}

impl Eq for KruskalEdge {}
impl PartialOrd for KruskalEdge {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for KruskalEdge {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other
            .weight
            .partial_cmp(&self.weight)
            .unwrap_or(std::cmp::Ordering::Equal)
    }
}

// Disjoint set union
struct Dsu {
    parent: FxHashMap<ActivityId, ActivityId>,
    num_components: usize,
}

impl Dsu {
    fn new(ids: &[ActivityId]) -> Self {
        Dsu {
            parent: ids.iter().fold(FxHashMap::default(), |mut acc, &id| {
                acc.insert(id, id);
                acc
            }),
            num_components: ids.len(),
        }
    }

    fn find(&mut self, i: ActivityId) -> ActivityId {
        let parent = self.parent.get(&i).unwrap_or(&i);
        if *parent == i {
            i
        } else {
            let root = self.find(*parent);
            self.parent.insert(i, root);
            root
        }
    }

    fn union(&mut self, i: ActivityId, j: ActivityId) {
        let root_i = self.find(i);
        let root_j = self.find(j);
        if root_i != root_j {
            self.parent.insert(root_i, root_j);
            self.num_components -= 1;
        }
    }
}

pub fn kruskal_cluster(
    problem: &VehicleRoutingProblem,
    activity_ids: &[ActivityId],
) -> Option<Vec<Vec<ActivityId>>> {
    let n = activity_ids.len();

    if n <= 2 {
        // If there are 2 or fewer locations, return them as individual clusters
        return Some(activity_ids.iter().map(|&id| vec![id]).collect());
    }

    let mut edges = BinaryHeap::new();

    // Create edges between all pairs of locations
    for i in 0..n {
        for j in (i + 1)..n {
            let from = activity_ids[i];
            let to = activity_ids[j];
            let weight = (problem.travel_cost(
                problem.vehicle(VehicleIdx::new(0)),
                problem.job_activity(from).location_id(),
                problem.job_activity(to).location_id(),
            ) + problem.travel_cost(
                problem.vehicle(VehicleIdx::new(0)),
                problem.job_activity(to).location_id(),
                problem.job_activity(from).location_id(),
            )) / 2.0;
            edges.push(KruskalEdge { from, to, weight });
        }
    }

    // Initialize disjoint set union
    let mut dsu = Dsu::new(activity_ids);

    let mut clusters: FxHashMap<ActivityId, Vec<ActivityId>> = FxHashMap::default();

    // Process edges in order of weight
    while let Some(edge) = edges.pop() {
        if dsu.num_components <= 2 {
            break;
        }

        dsu.union(edge.from, edge.to);
    }

    for &id in activity_ids {
        let root = dsu.find(id);
        clusters.entry(root).or_default().push(id);
    }

    Some(clusters.into_values().collect())
}

#[cfg(test)]
mod tests {
    use std::env;

    use super::*;
    use crate::{parsers::parser::parse_dataset, problem::job::JobIdx};

    #[test]
    fn test_kruskal_cluster() {
        let current_dir = env::current_dir().unwrap();
        let root_directory = current_dir.parent().unwrap();
        let path = root_directory.join("../data/vrptw/solomon/c1/c101.txt");

        let problem = parse_dataset(&path).unwrap();

        let location_ids = [1, 2, 3, 4, 5, 6, 7, 8, 9]
            .into_iter()
            .map(|id| ActivityId::Service(JobIdx::new(id)))
            .collect::<Vec<_>>();

        let clusters = kruskal_cluster(&problem, &location_ids);
        assert!(clusters.is_some());
        let clusters = clusters.unwrap();
        assert!(clusters.len() == 2); // Should create at least two clusters
    }
}
