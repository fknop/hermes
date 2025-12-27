use std::collections::BinaryHeap;

use fxhash::FxHashMap;

use crate::problem::{vehicle::VehicleId, vehicle_routing_problem::VehicleRoutingProblem};

struct KruskalEdge {
    from: usize,
    to: usize,
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
    parent: FxHashMap<usize, usize>,
    num_components: usize,
}

impl Dsu {
    fn new(ids: &[usize]) -> Self {
        Dsu {
            parent: ids.iter().fold(FxHashMap::default(), |mut acc, &id| {
                acc.insert(id, id);
                acc
            }),
            num_components: ids.len(),
        }
    }

    fn find(&mut self, i: usize) -> usize {
        let parent = self.parent.get(&i).unwrap_or(&i);
        if *parent == i {
            i
        } else {
            let root = self.find(*parent);
            self.parent.insert(i, root);
            root
        }
    }

    fn union(&mut self, i: usize, j: usize) {
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
    service_ids: &[usize],
) -> Option<Vec<Vec<usize>>> {
    let n = service_ids.len();

    if n <= 2 {
        // If there are 2 or fewer locations, return them as individual clusters
        return Some(service_ids.iter().map(|&id| vec![id]).collect());
    }

    let mut edges = BinaryHeap::new();

    // Create edges between all pairs of locations
    for i in 0..n {
        for j in (i + 1)..n {
            let from = service_ids[i];
            let to = service_ids[j];
            let weight = (problem.travel_cost(
                problem.vehicle(VehicleId::new(0)),
                problem.service_location(from).id(),
                problem.service_location(to).id(),
            ) + problem.travel_cost(
                problem.vehicle(VehicleId::new(0)),
                problem.service_location(to).id(),
                problem.service_location(from).id(),
            )) / 2.0;
            edges.push(KruskalEdge { from, to, weight });
        }
    }

    // Initialize disjoint set union
    let mut dsu = Dsu::new(service_ids);

    let mut clusters: FxHashMap<usize, Vec<usize>> = FxHashMap::default();

    // Process edges in order of weight
    while let Some(edge) = edges.pop() {
        if dsu.num_components <= 2 {
            break;
        }

        dsu.union(edge.from, edge.to);
    }

    for &id in service_ids {
        let root = dsu.find(id);
        clusters.entry(root).or_default().push(id);
    }

    Some(clusters.into_values().collect())
}

#[cfg(test)]
mod tests {
    use std::env;

    use super::*;
    use crate::parsers::{parser::DatasetParser, solomon::SolomonParser};

    #[test]
    fn test_kruskal_cluster() {
        let current_dir = env::current_dir().unwrap();
        let root_directory = current_dir.parent().unwrap();
        let path = root_directory.join("../data/solomon/c1/c101.txt");

        let parser = SolomonParser;
        let problem = parser.parse(path.to_str().unwrap()).unwrap();

        let location_ids = vec![1, 2, 3, 4, 5, 6, 7, 8, 9];

        let clusters = kruskal_cluster(&problem, &location_ids);
        assert!(clusters.is_some());
        let clusters = clusters.unwrap();
        assert!(clusters.len() == 2); // Should create at least two clusters
    }
}
