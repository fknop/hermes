#[cfg(test)]
pub mod test_graph {

    use std::cmp;

    use crate::{
        base_graph::GraphEdge,
        distance::{Distance, Kilometers, Meters},
        edge_direction::EdgeDirection,
        geopoint::GeoPoint,
        graph::Graph,
        kilometers,
        properties::property_map::EdgePropertyMap,
        weighting::{DurationMs, Weight, Weighting},
    };

    pub struct TestGraph {
        nodes: usize,
        edges: Vec<GraphEdge>,
        adjacency_list: Vec<Vec<usize>>,

        mock_geopoint: GeoPoint,
    }

    pub enum RomaniaGraphCity {
        Arad = 1,
        Bucharest = 2,
        Craiova = 3,
        Dobreta = 4,
        Eforie = 5,
        Fagaras = 6,
        Giurgiu = 7,
        Hirsova = 8,
        Iasi = 9,
        Lugoj = 10,
        Mehadia = 11,
        Neamt = 12,
        Oradea = 13,
        Pitesti = 14,
        RimnicuVilcea = 15,
        Sibiu = 16,
        Timisoara = 17,
        Urziceni = 18,
        Vaslui = 19,
        Zerind = 20,
    }

    impl From<RomaniaGraphCity> for usize {
        fn from(value: RomaniaGraphCity) -> Self {
            value as usize
        }
    }

    // https://user-images.githubusercontent.com/43790152/97784960-1a142580-1bc4-11eb-9070-39c03eb16df2.png
    fn get_romania_graph_edges() -> Vec<(RomaniaGraphCity, RomaniaGraphCity, Distance<Kilometers>)>
    {
        vec![
            (
                RomaniaGraphCity::Oradea,
                RomaniaGraphCity::Zerind,
                kilometers!(71),
            ),
            (
                RomaniaGraphCity::Oradea,
                RomaniaGraphCity::Sibiu,
                kilometers!(151),
            ),
            (
                RomaniaGraphCity::Zerind,
                RomaniaGraphCity::Arad,
                kilometers!(75),
            ),
            (
                RomaniaGraphCity::Arad,
                RomaniaGraphCity::Sibiu,
                kilometers!(140),
            ),
            (
                RomaniaGraphCity::Arad,
                RomaniaGraphCity::Timisoara,
                kilometers!(118),
            ),
            (
                RomaniaGraphCity::Timisoara,
                RomaniaGraphCity::Lugoj,
                kilometers!(111),
            ),
            (
                RomaniaGraphCity::Lugoj,
                RomaniaGraphCity::Mehadia,
                kilometers!(70),
            ),
            (
                RomaniaGraphCity::Mehadia,
                RomaniaGraphCity::Dobreta,
                kilometers!(75),
            ),
            (
                RomaniaGraphCity::Dobreta,
                RomaniaGraphCity::Craiova,
                kilometers!(120),
            ),
            (
                RomaniaGraphCity::Craiova,
                RomaniaGraphCity::RimnicuVilcea,
                kilometers!(146),
            ),
            (
                RomaniaGraphCity::Craiova,
                RomaniaGraphCity::Pitesti,
                kilometers!(138),
            ),
            (
                RomaniaGraphCity::RimnicuVilcea,
                RomaniaGraphCity::Pitesti,
                kilometers!(97),
            ),
            (
                RomaniaGraphCity::RimnicuVilcea,
                RomaniaGraphCity::Sibiu,
                kilometers!(80),
            ),
            (
                RomaniaGraphCity::Sibiu,
                RomaniaGraphCity::Fagaras,
                kilometers!(99),
            ),
            (
                RomaniaGraphCity::Fagaras,
                RomaniaGraphCity::Bucharest,
                kilometers!(211),
            ),
            (
                RomaniaGraphCity::Pitesti,
                RomaniaGraphCity::Bucharest,
                kilometers!(101),
            ),
            (
                RomaniaGraphCity::Bucharest,
                RomaniaGraphCity::Giurgiu,
                kilometers!(90),
            ),
            (
                RomaniaGraphCity::Bucharest,
                RomaniaGraphCity::Urziceni,
                kilometers!(85),
            ),
            (
                RomaniaGraphCity::Urziceni,
                RomaniaGraphCity::Hirsova,
                kilometers!(98),
            ),
            (
                RomaniaGraphCity::Hirsova,
                RomaniaGraphCity::Eforie,
                kilometers!(86),
            ),
            (
                RomaniaGraphCity::Urziceni,
                RomaniaGraphCity::Vaslui,
                kilometers!(142),
            ),
            (
                RomaniaGraphCity::Vaslui,
                RomaniaGraphCity::Iasi,
                kilometers!(92),
            ),
            (
                RomaniaGraphCity::Iasi,
                RomaniaGraphCity::Neamt,
                kilometers!(87),
            ),
        ]
    }

    impl TestGraph {
        pub fn new() -> Self {
            Self {
                nodes: 0,
                edges: Vec::new(),
                adjacency_list: Vec::new(),
                mock_geopoint: GeoPoint::new(0.0, 0.0),
            }
        }

        pub fn create_romania_graph() -> Self {
            let mut graph = TestGraph::new();

            let dataset = get_romania_graph_edges();

            for (start, end, distance) in dataset {
                graph.add_edge(start as usize, end as usize, distance);
            }

            graph
        }

        fn add_node(&mut self, node_id: usize) {
            self.nodes = cmp::max(self.nodes, node_id + 1);

            if self.nodes > self.adjacency_list.len() {
                // TODO: improve this by setting the capacity in advance
                self.adjacency_list
                    .reserve_exact(self.nodes - self.adjacency_list.capacity());

                for _ in 0..(self.nodes - self.adjacency_list.len()) {
                    self.adjacency_list.push(vec![]);
                }
            }
        }

        fn add_edge<D: Into<Distance<Meters>>>(
            &mut self,
            start_node: usize,
            end_node: usize,
            distance: D,
            // geometry: Vec<GeoPoint>,
        ) {
            self.add_node(start_node);
            self.add_node(end_node);
            let edge_id = self.edges.len();
            self.edges.push(GraphEdge::new(
                edge_id,
                start_node,
                end_node,
                distance.into(),
                EdgePropertyMap::new(),
            ));

            // self.geometry.push(geometry);
            self.adjacency_list[start_node].push(edge_id);
            self.adjacency_list[end_node].push(edge_id);
        }
    }

    impl Graph for TestGraph {
        type EdgeIterator<'a> = std::iter::Copied<std::slice::Iter<'a, usize>>;

        fn edge_count(&self) -> usize {
            self.edges.len()
        }

        fn node_count(&self) -> usize {
            self.nodes
        }

        fn is_virtual_node(&self, _: usize) -> bool {
            false
        }

        fn node_edges_iter(&self, node: usize) -> Self::EdgeIterator<'_> {
            self.adjacency_list[node].iter().copied()
        }

        fn edge(&self, edge: usize) -> &GraphEdge {
            &self.edges[edge]
        }

        fn edge_geometry(&self, _: usize) -> &[GeoPoint] {
            &[]
        }

        fn node_geometry(&self, _node_id: usize) -> &GeoPoint {
            &self.mock_geopoint
        }

        fn edge_direction(&self, edge_id: usize, start: usize) -> EdgeDirection {
            let edge = &self.edges[edge_id];

            if edge.start_node() == start {
                return EdgeDirection::Forward;
            }

            if edge.end_node() == start {
                return EdgeDirection::Backward;
            }

            panic!(
                "Node {} is neither the start nor the end of edge {}",
                start, edge_id
            )
        }
    }

    pub struct TestWeighting;

    impl Weighting for TestWeighting {
        fn calc_edge_weight(&self, edge: &GraphEdge, _: EdgeDirection) -> Weight {
            edge.distance().value() as Weight
        }

        fn calc_edge_ms(&self, edge: &GraphEdge, _: EdgeDirection) -> DurationMs {
            let speed_kmh = 120.0;
            let speed_ms = speed_kmh / 3.6;

            let distance = edge.distance().value();

            (distance / speed_ms).round() as DurationMs
        }
    }
}
