use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::slice::Iter;

use crate::geometry::compute_geometry_distance;
use crate::geopoint::GeoPoint;
use crate::graph::Graph;
use crate::osm::osm_reader::OSMData;
use crate::properties::property_map::{
    BACKWARD_EDGE, EdgeDirection, EdgePropertyMap, FORWARD_EDGE,
};

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct GraphEdge {
    id: usize,
    start_node: usize,
    end_node: usize,
    distance: f64,
    pub properties: EdgePropertyMap,
}

impl GraphEdge {
    pub fn new(
        id: usize,
        start_node: usize,
        end_node: usize,
        distance: f64,
        properties: EdgePropertyMap,
    ) -> Self {
        GraphEdge {
            id,
            start_node,
            end_node,
            distance,
            properties,
        }
    }

    pub fn id(&self) -> usize {
        self.id
    }

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

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct BaseGraph {
    nodes: usize,
    edges: Vec<GraphEdge>,
    geometry: Vec<Vec<GeoPoint>>,
    adjacency_list: Vec<Vec<usize>>,
}

fn read_bytes(path: &str) -> Vec<u8> {
    let file = File::open(path).expect("Cannot open file");
    let mut reader = BufReader::new(file);
    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer).unwrap();
    buffer
}

fn from_bytes(bytes: &[u8]) -> BaseGraph {
    let graph = rkyv::from_bytes::<BaseGraph, rkyv::rancor::Error>(bytes).unwrap();
    println!("Deserialized graph from buffer");
    graph
}

impl BaseGraph {
    pub fn save_to_file(&self, path: &str) {
        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(self).expect("to_bytes failed");
        let file = File::create(path).expect("failed to create file");
        let mut writer = BufWriter::new(file);

        writer.write(&bytes[..]).expect("failed to write buffer");
        writer.flush().expect("failed to flush buffer");
    }

    pub fn from_file(path: &str) -> BaseGraph {
        println!("Reading from path {}", path);
        let bytes = read_bytes(path);
        println!("Read from path {}, size {}", path, bytes.len());
        from_bytes(&bytes)
    }

    fn new(nodes: usize, edges: usize) -> BaseGraph {
        let adjencency_list = vec![vec![]; nodes];
        BaseGraph {
            nodes,
            edges: Vec::with_capacity(edges),
            geometry: Vec::with_capacity(edges),
            adjacency_list: adjencency_list,
        }
    }

    pub fn from_osm_data(osm_data: &OSMData) -> BaseGraph {
        let ways = osm_data.ways();
        let nodes = osm_data.nodes();
        let mut graph = BaseGraph::new(nodes.len(), ways.len());

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

    pub fn node_edges(&self, node: usize) -> &[usize] {
        &self.adjacency_list[node]
    }

    fn add_edge(
        &mut self,
        from_node: usize,
        to_node: usize,
        properties: EdgePropertyMap,
        geometry: Vec<GeoPoint>,
    ) {
        let edge_id = self.edges.len();
        self.edges.push(GraphEdge {
            id: edge_id,
            start_node: from_node,
            end_node: to_node,
            properties,
            distance: compute_geometry_distance(&geometry),
        });
        self.geometry.push(geometry);
        self.adjacency_list[from_node].push(edge_id);
        self.adjacency_list[to_node].push(edge_id);
    }
}

impl Graph for BaseGraph {
    type EdgeIterator<'a> = std::iter::Copied<std::slice::Iter<'a, usize>>;

    fn node_edges_iter(&self, node: usize) -> Self::EdgeIterator<'_> {
        self.adjacency_list[node].iter().copied()
    }

    fn edge(&self, edge: usize) -> &GraphEdge {
        &self.edges[edge]
    }

    fn edge_geometry(&self, edge: usize) -> &[GeoPoint] {
        &self.geometry[edge][..]
    }

    fn edge_count(&self) -> usize {
        self.edges.len()
    }

    fn node_count(&self) -> usize {
        self.nodes
    }

    fn edge_direction(&self, edge_id: usize, start: usize) -> EdgeDirection {
        let edge = &self.edges[edge_id];

        if edge.start_node == start {
            return FORWARD_EDGE;
        }

        if edge.end_node == start {
            return BACKWARD_EDGE;
        }

        panic!(
            "Node {} is neither the start nor the end of edge {}",
            start, edge_id
        )
    }
}
