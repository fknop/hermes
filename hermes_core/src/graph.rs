use crate::latlng::LatLng;
use crate::osm::osm_reader::OSMData;
use crate::properties::property_map::{
    BACKWARD_EDGE, EdgeDirection, EdgePropertyMap, FORWARD_EDGE,
};
use crate::weighting::Weighting;

pub struct GraphNode {
    id: usize,
}

pub struct GraphEdge {
    id: usize,
    start_node: usize,
    end_node: usize,
    distance: f64,
    pub properties: EdgePropertyMap,
}

impl GraphEdge {
    pub fn distance(&self) -> f64 {
        self.distance
    }

    pub fn start_node(&self) -> usize {
        self.start_node
    }

    pub fn end_node(&self) -> usize {
        self.end_node
    }
}

pub struct Graph {
    nodes: usize,
    edges: Vec<GraphEdge>,
    geometry: Vec<Vec<LatLng>>,
    adjacency_list: Vec<Vec<usize>>,
}

impl Graph {
    fn new(nodes: usize, edges: usize) -> Graph {
        let adjencency_list = vec![vec![]; nodes];
        Graph {
            nodes,
            edges: Vec::with_capacity(edges),
            geometry: Vec::with_capacity(edges),
            adjacency_list: adjencency_list,
        }
    }

    pub fn build_from_osm_data(osm_data: &OSMData) -> Graph {
        let ways = osm_data.ways();
        let nodes = osm_data.nodes();
        let mut graph = Graph::new(nodes.len(), ways.len());

        ways.iter().for_each(|way| {
            graph.add_edge(
                way.start_node(),
                way.end_node(),
                way.properties().clone(),
                osm_data.way_geometry(way.id()),
            )
        });

        graph
    }

    fn add_edge(
        &mut self,
        from_node: usize,
        to_node: usize,
        properties: EdgePropertyMap,
        geometry: Vec<LatLng>,
    ) {
        let edge_id = self.edges.len();
        self.edges.push(GraphEdge {
            id: edge_id,
            start_node: from_node,
            end_node: to_node,
            properties,
            distance: self.compute_distance_for_geometry(&geometry),
        });
        self.geometry.push(geometry);
        self.adjacency_list[from_node].push(edge_id);
        self.adjacency_list[to_node].push(edge_id);
    }

    fn compute_distance_for_geometry(&self, geometry: &Vec<LatLng>) -> f64 {
        let mut distance = 0.0;
        for i in 0..geometry.len() - 1 {
            distance += geometry[i].haversine_distance(&geometry[i + 1])
        }

        distance
    }

    pub fn node_edges(&self, node: usize) -> &[usize] {
        &self.adjacency_list[node]
    }

    pub fn edge(&self, edge: usize) -> &GraphEdge {
        &self.edges[edge]
    }

    pub fn edge_geometry(&self, edge: usize) -> &Vec<LatLng> {
        &self.geometry[edge]
    }
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    pub fn node_count(&self) -> usize {
        self.nodes
    }

    pub fn edge_direction(&self, edge_id: usize, start: usize) -> EdgeDirection {
        let edge = &self.edges[edge_id];

        if edge.start_node == start {
            return FORWARD_EDGE;
        }

        if edge.end_node == start {
            return BACKWARD_EDGE;
        }

        panic!("Tried to get the direction of an unknown edge {}", edge_id)
    }
}
