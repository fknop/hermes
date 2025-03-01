use crate::latlng::LatLng;
use crate::osm::osm_reader::OSMData;
use crate::properties::property_map::EdgePropertyMap;

pub struct GraphNode {
    id: usize,
}

pub struct GraphEdge {
    id: usize,
    from_node: usize,
    to_node: usize,
    pub properties: EdgePropertyMap,
}

impl GraphEdge {
    pub fn get_distance(&self) -> f64 {
        return 100.0; // TODO
    }
}

pub struct Graph {
    nodes: Vec<GraphNode>,
    edges: Vec<GraphEdge>,
    geometry: Vec<Vec<LatLng>>,

    first_node_edges: Vec<usize>,
}

impl Graph {
    fn new() -> Graph {
        Graph {
            nodes: Vec::new(),
            edges: Vec::new(),
            geometry: Vec::new(),
            first_node_edges: Vec::new(),
        }
    }

    fn build_graph(&mut self, osm_data: &OSMData) {
        for way in osm_data.get_ways() {}
    }

    pub fn get_node_edges(&self, node: usize) -> &[GraphEdge] {
        let first_edge = self.first_node_edges[node];
        let last_edge = self.first_node_edges[node + 1];
        &self.edges[first_edge..last_edge]
    }

    pub fn get_edge(&self, edge: usize) -> &GraphEdge {
        &self.edges[edge]
    }
}
