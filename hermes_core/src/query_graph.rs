use fxhash::FxHashMap;

use crate::{
    base_graph::{BaseGraph, BaseGraphEdge},
    edge_direction::EdgeDirection,
    geometry::{
        compute_geometry_distance, create_virtual_geometries,
        create_virtual_geometry_between_points,
    },
    geopoint::GeoPoint,
    graph::{Graph, UndirectedEdgeAccess},
    graph_edge::GraphEdge,
    snap::Snap,
    types::{EdgeId, NodeId},
};

/// A dynamic graph that extends a base graph with virtual nodes and edges
///
/// QueryGraph wraps a base graph and allows for the dynamic addition of virtual nodes
/// and edges during query time. This is particularly useful for routing queries where
/// temporary modifications to the graph are needed without altering the underlying base graph.
///
/// Virtual nodes are typically added when a query point (snap) lies along an existing edge,
/// splitting that edge into two virtual edges connected by the new virtual node.
pub(crate) struct QueryGraph<'a> {
    base_graph: &'a BaseGraph,

    virtual_nodes: usize,
    virtual_edges: Vec<BaseGraphEdge>,
    virtual_edge_geometry: Vec<Vec<GeoPoint>>,

    // New edges for new "virtual" nodes
    virtual_adjacency_list: Vec<Vec<EdgeId>>,

    // New edges for existing nodes in the base graph
    virtual_adjacency_list_existing_nodes: FxHashMap<NodeId, Vec<EdgeId>>,
}

impl<'a> QueryGraph<'a> {
    pub fn from_base_graph(base_graph: &'a BaseGraph, snaps: &mut [Snap]) -> Self {
        let mut query_graph = QueryGraph {
            base_graph,
            virtual_nodes: 0,
            virtual_edge_geometry: Vec::new(),
            virtual_edges: Vec::new(),
            virtual_adjacency_list: Vec::new(),
            virtual_adjacency_list_existing_nodes: FxHashMap::default(),
        };

        for snap in snaps.iter_mut() {
            query_graph.add_edges_from_snap(snap)
        }

        query_graph.add_virtual_edges_between_snaps(snaps);

        query_graph
    }

    /// If an edge A <--> B where the snap is located inside the edge A ---- S ---- B
    /// The snap is not necessarily a node inside the edge but can be arbitrary coordinates between 2 nodes
    ///
    /// For each snap, if the snap is not the start of end of the base edge, we need to create two virtual edges from a new virtual node
    /// If a snap S is located inside an edge A <-----> B, we need to split that edge in two, resulting in two virtual edges A <-> S and S <--> B
    /// The new virtual node V, located at snap S, contains the two new adjacent virtual edges
    /// But we also need to add the virtual edge A <-> S and S <-> B to a new adjacency list for A and B
    fn add_edges_from_snap(&mut self, snap: &mut Snap) {
        let edge_id = snap.edge_id;
        let edge = self.base_graph.edge(edge_id);
        let geometry = self.base_graph.edge_geometry(edge_id);

        let edge_start_node = edge.start_node();
        let edge_end_node = edge.end_node();

        // Point if the first or last node of the edge, no need to create virtual edges
        if geometry[0] == snap.coordinates {
            snap.set_closest_node(edge_start_node);
            return;
        }

        if geometry[geometry.len() - 1] == snap.coordinates {
            snap.set_closest_node(edge_end_node);
            return;
        }

        let (virtual_geometry_1, virtual_geometry_2) =
            create_virtual_geometries(geometry, &snap.coordinates);

        let virtual_node = self.base_graph.node_count() + self.virtual_nodes;

        snap.set_closest_node(virtual_node);

        let virtual_edge_id_1 = self.base_graph.edge_count() + self.virtual_edges.len();
        let virtual_edge_id_2 = virtual_edge_id_1 + 1;

        self.virtual_edges.push(BaseGraphEdge::new(
            virtual_edge_id_1,
            edge_start_node,
            virtual_node,
            compute_geometry_distance(&virtual_geometry_1),
            edge.properties().clone(),
        ));

        self.add_virtual_edge_for_existing_node(virtual_edge_id_1, edge_start_node);
        self.virtual_edge_geometry.push(virtual_geometry_1);

        self.virtual_edges.push(BaseGraphEdge::new(
            virtual_edge_id_2,
            virtual_node,
            edge_end_node,
            compute_geometry_distance(&virtual_geometry_2),
            edge.properties().clone(),
        ));

        self.add_virtual_edge_for_existing_node(virtual_edge_id_2, edge_end_node);
        self.virtual_edge_geometry.push(virtual_geometry_2);

        self.virtual_adjacency_list
            .push(vec![virtual_edge_id_1, virtual_edge_id_2]);

        self.virtual_nodes += 1;
    }

    /// If the snaps are on the same edge, we need to create a virtual edge between them
    fn add_virtual_edges_between_snaps(&mut self, snaps: &[Snap]) {
        for i in 0..snaps.len() {
            for j in i + 1..snaps.len() {
                let snap_i = &snaps[i];
                let snap_j = &snaps[j];

                if snap_i.edge_id == snap_j.edge_id {
                    let snap_i_node = snap_i.closest_node();
                    let snap_j_node = snap_j.closest_node();

                    // We only need to create an edge if they are both virtual nodes, otherwise, the edge was alraedy created
                    if !self.is_virtual_node(snap_i_node) || !self.is_virtual_node(snap_j_node) {
                        continue;
                    }

                    let edge = self.base_graph.edge(snap_i.edge_id);
                    let geometry = self.edge_geometry(snap_i.edge_id);
                    let virtual_geometry = create_virtual_geometry_between_points(
                        geometry,
                        (&snap_i.coordinates, &snap_j.coordinates),
                    );

                    let (start_node, end_node) = if virtual_geometry[0] == snap_i.coordinates {
                        (snap_i_node, snap_j_node)
                    } else {
                        (snap_j_node, snap_i_node)
                    };

                    let virtual_edge_id = self.base_graph.edge_count() + self.virtual_edges.len();

                    // Add the new edge
                    self.virtual_edges.push(BaseGraphEdge::new(
                        virtual_edge_id,
                        start_node,
                        end_node,
                        compute_geometry_distance(&virtual_geometry),
                        edge.properties().clone(),
                    ));

                    // Add the geometry for the new edge
                    self.virtual_edge_geometry.push(virtual_geometry);

                    // Add the edge to the adjacency list of both virtual nodes
                    let start_virtual_node_id = self.virtual_node_id(start_node);
                    let end_virtual_node_id = self.virtual_node_id(end_node);

                    self.virtual_adjacency_list[start_virtual_node_id].push(virtual_edge_id);
                    self.virtual_adjacency_list[end_virtual_node_id].push(virtual_edge_id);
                }
            }
        }
    }

    fn add_virtual_edge_for_existing_node(&mut self, edge_id: usize, node_id: usize) {
        match self.virtual_adjacency_list_existing_nodes.get_mut(&node_id) {
            Some(list) => list.push(edge_id),
            None => {
                self.virtual_adjacency_list_existing_nodes
                    .insert(node_id, vec![edge_id]);
            }
        }
    }

    fn is_virtual_edge(&self, edge_id: usize) -> bool {
        edge_id >= self.base_graph.edge_count()
    }

    // Assumes node_id is a virtual node
    fn virtual_node_id(&self, node_id: usize) -> usize {
        node_id - self.base_graph.node_count()
    }

    // Assumes edge_id is a virtual edge
    fn virtual_edge_id(&self, edge_id: usize) -> usize {
        edge_id - self.base_graph.edge_count()
    }

    // Assumes edge_id is a virtual edge
    fn virtual_edge(&self, edge_id: usize) -> &BaseGraphEdge {
        &self.virtual_edges[self.virtual_edge_id(edge_id)]
    }
}

impl Graph for QueryGraph<'_> {
    type Edge = BaseGraphEdge;

    fn is_virtual_node(&self, node_id: usize) -> bool {
        node_id >= self.base_graph.node_count()
    }

    fn edge_count(&self) -> usize {
        self.base_graph.edge_count() + self.virtual_edges.len()
    }

    fn node_count(&self) -> usize {
        self.base_graph.node_count() + self.virtual_nodes
    }

    fn edge(&self, edge_id: usize) -> &BaseGraphEdge {
        if self.is_virtual_edge(edge_id) {
            self.virtual_edge(edge_id)
        } else {
            self.base_graph.edge(edge_id)
        }
    }

    fn edge_geometry(&self, edge_id: usize) -> &[GeoPoint] {
        if self.is_virtual_edge(edge_id) {
            let virtual_edge_id = self.virtual_edge_id(edge_id);
            &self.virtual_edge_geometry[virtual_edge_id]
        } else {
            self.base_graph.edge_geometry(edge_id)
        }
    }

    fn node_geometry(&self, node_id: usize) -> &GeoPoint {
        if self.is_virtual_node(node_id) {
            let first_edge_id = self.virtual_adjacency_list[self.virtual_node_id(node_id)][0];
            let edge_geometry = &self.virtual_edge_geometry[self.virtual_edge_id(first_edge_id)];
            let edge_direction = self.edge_direction(first_edge_id, node_id);
            match edge_direction {
                EdgeDirection::Forward => &edge_geometry[0],
                EdgeDirection::Backward => &edge_geometry[edge_geometry.len() - 1],
            }
        } else {
            self.base_graph.node_geometry(node_id)
        }
    }

    fn edge_direction(&self, edge_id: usize, start_node_id: usize) -> EdgeDirection {
        if self.is_virtual_edge(edge_id) {
            let edge = self.virtual_edge(edge_id);

            if edge.start_node() == start_node_id {
                return EdgeDirection::Forward;
            } else if edge.end_node() == start_node_id {
                return EdgeDirection::Backward;
            }

            panic!(
                "Node {} is neither the start nor the end of edge {}",
                start_node_id, edge_id
            )
        } else {
            self.base_graph.edge_direction(edge_id, start_node_id)
        }
    }
}

impl UndirectedEdgeAccess for QueryGraph<'_> {
    type EdgeIterator<'b>
        = QueryGraphEdgeIterator<'b>
    where
        Self: 'b;
    fn node_edges_iter(&self, node_id: usize) -> Self::EdgeIterator<'_> {
        if self.is_virtual_node(node_id) {
            QueryGraphEdgeIterator::new(
                &[],
                &self.virtual_adjacency_list[self.virtual_node_id(node_id)],
            )
        } else {
            let virtual_edges = self.virtual_adjacency_list_existing_nodes.get(&node_id);
            let base_edges = self.base_graph.node_edges(node_id);

            match virtual_edges {
                Some(virtual_edges) => QueryGraphEdgeIterator::new(base_edges, virtual_edges),
                None => QueryGraphEdgeIterator::new(base_edges, &[]),
            }
        }
    }
}

/// An iterator that combines base edges and virtual edges from a QueryGraph
///
/// This iterator will first yield all base edges, followed by all virtual edges.
/// It is used internally by the QueryGraph to provide a unified view of both
/// the original graph edges and dynamically added virtual edges.
pub struct QueryGraphEdgeIterator<'a> {
    base_edges: &'a [usize],
    virtual_edges: &'a [usize],
    index: usize,
}

impl<'a> QueryGraphEdgeIterator<'a> {
    fn new(base_edges: &'a [usize], virtual_edges: &'a [usize]) -> Self {
        QueryGraphEdgeIterator {
            base_edges,
            virtual_edges,
            index: 0,
        }
    }
}

impl Iterator for QueryGraphEdgeIterator<'_> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.base_edges.len() {
            let edge = self.base_edges[self.index];
            self.index += 1;
            return Some(edge);
        }

        let virtual_index = self.index - self.base_edges.len();

        if virtual_index < self.virtual_edges.len() {
            let edge = self.virtual_edges[virtual_index];
            self.index += 1;
            return Some(edge);
        }

        None
    }
}
