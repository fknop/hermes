use std::cmp::max;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};

use crate::distance::{Distance, Meters};
use crate::edge_direction::EdgeDirection;
use crate::geometry::compute_geometry_distance;
use crate::geopoint::GeoPoint;
use crate::graph::Graph;
use crate::osm::osm_reader::OsmReader;
use crate::properties::property_map::EdgePropertyMap;

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct GraphEdge {
    id: usize,
    start_node: usize,
    end_node: usize,
    distance: Distance<Meters>,
    pub properties: EdgePropertyMap,
}

impl GraphEdge {
    pub fn new(
        id: usize,
        start_node: usize,
        end_node: usize,
        distance: Distance<Meters>,
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

    pub fn distance(&self) -> Distance<Meters> {
        self.distance
    }

    pub fn start_node(&self) -> usize {
        self.start_node
    }

    pub fn end_node(&self) -> usize {
        self.end_node
    }

    pub fn adj_node(&self, node: usize) -> usize {
        if self.start_node == node {
            self.end_node
        } else {
            self.start_node
        }
    }
}

#[derive(Default, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
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
    fn add_node(&mut self, node_id: usize) {
        self.nodes = max(self.nodes, node_id + 1);

        if self.nodes > self.adjacency_list.len() {
            // TODO: improve this by setting the capacity in advance
            self.adjacency_list
                .reserve_exact(self.nodes - self.adjacency_list.capacity());

            for _ in 0..(self.nodes - self.adjacency_list.len()) {
                self.adjacency_list.push(vec![]);
            }
        }
    }

    pub fn save_to_file(&self, path: &str) {
        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(self).expect("to_bytes failed");
        let file = File::create(path).expect("failed to create file");
        let mut writer = BufWriter::new(file);

        writer
            .write_all(&bytes[..])
            .expect("failed to write buffer");
        writer.flush().expect("failed to flush buffer");
    }

    pub fn from_file(path: &str) -> BaseGraph {
        println!("Reading from path {}", path);
        let bytes = read_bytes(path);
        println!("Read from path {}, size {}", path, bytes.len());
        from_bytes(&bytes)
    }

    pub fn from_osm_file(path: &str) -> BaseGraph {
        let mut osm_reader = OsmReader::default();

        let mut graph = BaseGraph::default();
        osm_reader.parse_osm_file(path, |edge_segment| {
            graph.add_node(edge_segment.start_node);
            graph.add_node(edge_segment.end_node);
            graph.add_edge(
                edge_segment.start_node,
                edge_segment.end_node,
                edge_segment.properties,
                edge_segment.geometry,
            );
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

    fn node_geometry(&self, node_id: usize) -> &GeoPoint {
        let first_edge_id = self.adjacency_list[node_id][0];
        let edge_geometry = &self.geometry[first_edge_id];
        let edge_direction = self.edge_direction(first_edge_id, node_id);
        match edge_direction {
            EdgeDirection::Forward => &edge_geometry[0],
            EdgeDirection::Backward => &edge_geometry[edge_geometry.len() - 1],
        }
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
            return EdgeDirection::Forward;
        }

        if edge.end_node == start {
            return EdgeDirection::Backward;
        }

        panic!(
            "Node {} is neither the start nor the end of edge {}",
            start, edge_id
        )
    }
}
