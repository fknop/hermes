use crate::{
    base_graph::BaseGraph,
    constants::{INVALID_EDGE, INVALID_NODE, MAX_DURATION, MAX_WEIGHT},
    graph::Graph,
    meters,
    storage::{read_bytes, write_bytes},
    types::{EdgeId, NodeId},
};

use super::{
    ch_edge::{CHBaseEdge, CHGraphEdge},
    shortcut::Shortcut,
};

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct CHStorage {
    nodes: usize,
    edges: Vec<CHGraphEdge>,

    /// For each node, a list the incoming edges into this node
    incoming_edges: Vec<Vec<EdgeId>>,

    /// For each node, a list the outgoing edges from this node
    outgoing_edges: Vec<Vec<EdgeId>>,
}

impl CHStorage {
    pub fn save_to_file(&self, path: &str) -> Result<(), std::io::Error> {
        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(self).expect("to_bytes failed");
        write_bytes(&bytes[..], path)
    }

    pub fn from_file(path: &str) -> Self {
        println!("Reading from path {}", path);
        let bytes = read_bytes(path);
        println!("Read from path {}, size {}", path, bytes.len());
        let data = rkyv::from_bytes::<Self, rkyv::rancor::Error>(&bytes[..]).unwrap();
        println!("Deserialized ch storage from buffer");
        data
    }

    pub fn new(base_graph: &BaseGraph) -> Self {
        let edges = vec![
            CHGraphEdge::Edge(CHBaseEdge {
                edge_id: INVALID_EDGE,
                start: INVALID_NODE,
                end: INVALID_NODE,
                forward_weight: MAX_WEIGHT,
                backward_weight: MAX_WEIGHT,
                backward_time: MAX_DURATION,
                forward_time: MAX_DURATION,
                distance: meters!(0)
            });
            base_graph.edge_count()
        ];
        let ranks = vec![0; base_graph.node_count()];
        let incoming_edges = vec![Vec::new(); base_graph.node_count()];
        let outgoing_edges = vec![Vec::new(); base_graph.node_count()];

        Self {
            nodes: base_graph.node_count(),
            edges,
            incoming_edges,
            outgoing_edges,
        }
    }

    pub fn add_edge(&mut self, edge: CHBaseEdge) {
        if edge.forward_weight != MAX_WEIGHT {
            self.outgoing_edges[edge.start].push(edge.edge_id);
            self.incoming_edges[edge.end].push(edge.edge_id);
        }

        if edge.backward_weight != MAX_WEIGHT {
            self.incoming_edges[edge.start].push(edge.edge_id);
            self.outgoing_edges[edge.end].push(edge.edge_id);
        }

        let edge_id = edge.edge_id;
        self.edges[edge_id] = CHGraphEdge::Edge(edge);
    }

    pub fn add_shortcut(&mut self, shortcut: Shortcut) {
        let edge_id = self.edges.len();
        self.outgoing_edges[shortcut.start].push(edge_id);
        self.incoming_edges[shortcut.end].push(edge_id);

        self.edges.push(CHGraphEdge::Shortcut(shortcut));
    }

    pub fn nodes_count(&self) -> usize {
        self.nodes
    }

    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    pub fn edge(&self, edge_id: EdgeId) -> &CHGraphEdge {
        &self.edges[edge_id]
    }

    pub fn incoming_edges(&self, node_id: NodeId) -> &[EdgeId] {
        &self.incoming_edges[node_id]
    }

    pub fn outgoing_edges(&self, node_id: NodeId) -> &[EdgeId] {
        &self.outgoing_edges[node_id]
    }
}
