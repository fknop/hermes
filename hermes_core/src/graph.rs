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

    pub fn get_from_node(&self) -> usize {
        self.id
    }

    pub fn get_to_node(&self) -> usize {
        self.id
    }
}

pub struct Graph {
    edges: Vec<GraphEdge>,
    geometry: Vec<Vec<LatLng>>,
    adjacency_list: Vec<Vec<usize>>,
}

impl Graph {
    fn new(nodes: usize, edges: usize) -> Graph {
        let adjencency_list = vec![vec![]; nodes];
        Graph {
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
                EdgePropertyMap::new(),
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
        });
        self.geometry.push(geometry);
        self.adjacency_list[from_node].push(edge_id);
        self.adjacency_list[to_node].push(edge_id);
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
        1 // TODO
    }
}
