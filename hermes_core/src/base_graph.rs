use std::cmp::max;

use tracing::{debug, info};

use crate::distance::{Distance, Meters};
use crate::edge_direction::EdgeDirection;
use crate::geometry::compute_geometry_distance;
use crate::geopoint::GeoPoint;
use crate::graph::{GeometryAccess, Graph, UndirectedEdgeAccess};
use crate::graph_edge::GraphEdge;
use crate::osm::osm_reader::OsmReader;
use crate::properties::property_map::EdgePropertyMap;
use crate::storage::{read_bytes, write_bytes};
use crate::types::{EdgeId, NodeId};

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Debug)]
pub struct BaseGraphEdge {
    id: EdgeId,
    start_node: NodeId,
    end_node: NodeId,
    distance: Distance<Meters>,
    properties: EdgePropertyMap,
}

impl GraphEdge for BaseGraphEdge {
    fn distance(&self) -> Distance<Meters> {
        self.distance
    }

    fn start_node(&self) -> NodeId {
        self.start_node
    }

    fn end_node(&self) -> NodeId {
        self.end_node
    }

    fn adj_node(&self, node: NodeId) -> NodeId {
        if self.start_node == node {
            self.end_node
        } else {
            self.start_node
        }
    }

    fn properties(&self) -> &EdgePropertyMap {
        &self.properties
    }
}

impl BaseGraphEdge {
    pub fn new(
        id: EdgeId,
        start_node: NodeId,
        end_node: NodeId,
        distance: Distance<Meters>,
        properties: EdgePropertyMap,
    ) -> Self {
        BaseGraphEdge {
            id,
            start_node,
            end_node,
            distance,
            properties,
        }
    }

    pub fn id(&self) -> EdgeId {
        self.id
    }
}

#[derive(Default, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct BaseGraph {
    nodes: usize,
    edges: Vec<BaseGraphEdge>,
    adjacency_list: Vec<Vec<EdgeId>>,
    geometry: Vec<Vec<GeoPoint>>,
}

fn from_bytes(bytes: &[u8]) -> BaseGraph {
    let graph = rkyv::from_bytes::<BaseGraph, rkyv::rancor::Error>(bytes).unwrap();
    info!("Deserialized graph from buffer");
    graph
}

impl BaseGraph {
    pub fn edges(&self) -> &[BaseGraphEdge] {
        &self.edges
    }

    fn add_node(&mut self, node_id: NodeId) {
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

    pub fn save_to_file(&self, path: &str) -> Result<(), std::io::Error> {
        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(self).expect("to_bytes failed");
        write_bytes(&bytes[..], path)
    }

    pub fn from_file(path: &str) -> BaseGraph {
        debug!("Reading from path {}", path);
        let bytes = read_bytes(path);
        debug!("Read from path {}, size {}", path, bytes.len());
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

    pub fn node_edges(&self, node: NodeId) -> &[EdgeId] {
        &self.adjacency_list[node]
    }

    fn add_edge(
        &mut self,
        from_node: NodeId,
        to_node: NodeId,
        properties: EdgePropertyMap,
        geometry: Vec<GeoPoint>,
    ) {
        let edge_id = self.edges.len();
        self.edges.push(BaseGraphEdge {
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
    type Edge = BaseGraphEdge;

    fn is_virtual_node(&self, _: NodeId) -> bool {
        false
    }

    fn edge(&self, edge: EdgeId) -> &BaseGraphEdge {
        &self.edges[edge]
    }

    fn edge_count(&self) -> usize {
        self.edges.len()
    }

    fn node_count(&self) -> usize {
        self.nodes
    }

    fn edge_direction(&self, edge_id: EdgeId, start: NodeId) -> EdgeDirection {
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

impl GeometryAccess for BaseGraph {
    fn edge_geometry(&self, edge: EdgeId) -> &[GeoPoint] {
        &self.geometry[edge][..]
    }

    fn node_geometry(&self, node_id: NodeId) -> &GeoPoint {
        let first_edge_id = self.adjacency_list[node_id][0];
        let edge_geometry = &self.geometry[first_edge_id];
        let edge_direction = self.edge_direction(first_edge_id, node_id);
        match edge_direction {
            EdgeDirection::Forward => &edge_geometry[0],
            EdgeDirection::Backward => &edge_geometry[edge_geometry.len() - 1],
        }
    }
}

impl UndirectedEdgeAccess for BaseGraph {
    type EdgeIterator<'a> = std::iter::Copied<std::slice::Iter<'a, usize>>;

    fn node_edges_iter(&self, node: NodeId) -> Self::EdgeIterator<'_> {
        self.adjacency_list[node].iter().copied()
    }
}
