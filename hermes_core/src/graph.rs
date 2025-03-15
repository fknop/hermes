use crate::latlng::LatLng;
use crate::osm::osm_reader::OSMData;
use crate::properties::property_map::{
    BACKWARD_EDGE, EdgeDirection, EdgePropertyMap, FORWARD_EDGE,
};

pub struct GraphNode {
    id: usize,
}

pub struct GraphEdge {
    id: usize,
    from_node: usize,
    to_node: usize,
    distance: f64,
    pub properties: EdgePropertyMap,
}

impl GraphEdge {
    pub fn get_distance(&self) -> f64 {
        self.distance
    }

    pub fn get_from_node(&self) -> usize {
        self.from_node
    }

    pub fn get_to_node(&self) -> usize {
        self.to_node
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
        let ways = osm_data.get_ways();
        let nodes = osm_data.get_nodes();
        let mut graph = Graph::new(nodes.len(), ways.len());

        ways.iter().for_each(|way| {
            graph.add_edge(
                way.get_from_node(),
                way.get_to_node(),
                way.get_properties().clone(),
                osm_data.get_way_geometry(way.get_id()),
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
            from_node,
            to_node,
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

    pub fn get_node_edges(&self, node: usize) -> &[usize] {
        &self.adjacency_list[node]
    }

    pub fn get_edge(&self, edge: usize) -> &GraphEdge {
        &self.edges[edge]
    }

    pub fn get_edge_geometry(&self, edge: usize) -> &Vec<LatLng> {
        &self.geometry[edge]
    }
    pub fn get_edge_count(&self) -> usize {
        self.edges.len()
    }

    pub fn get_node_count(&self) -> usize {
        self.nodes
    }

    pub fn get_edge_direction(&self, edge_id: usize, start: usize) -> EdgeDirection {
        let edge = &self.edges[edge_id];

        if edge.from_node == start {
            return FORWARD_EDGE;
        }

        if edge.to_node == start {
            return BACKWARD_EDGE;
        }

        panic!("Tried to get the direction of an unknown edge {}", edge_id)
    }
}
