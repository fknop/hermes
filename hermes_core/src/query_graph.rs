use crate::{
    base_graph::{BaseGraph, BaseGraphEdge},
    edge_direction::EdgeDirection,
    geometry::{
        compute_geometry_distance, create_virtual_geometries,
        create_virtual_geometry_between_points,
    },
    geopoint::GeoPoint,
    graph::{GeometryAccess, Graph, UndirectedEdgeAccess},
    graph_edge::GraphEdge,
    query_graph_overlay::QueryGraphOverlay,
    snap::Snap,
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
    overlay: QueryGraphOverlay<'a, BaseGraph>,
}

impl<'a> QueryGraph<'a> {
    pub fn from_base_graph(base_graph: &'a BaseGraph, snaps: &mut [Snap]) -> Self {
        let mut query_graph = QueryGraph {
            base_graph,
            overlay: QueryGraphOverlay::from_graph(base_graph),
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

        let virtual_node = self.overlay.add_virtual_node();

        snap.set_closest_node(virtual_node);

        let virtual_edge_id_1 = self.overlay.edge_count();
        let virtual_edge_id_2 = virtual_edge_id_1 + 1;

        self.overlay.add_virtual_edge(
            BaseGraphEdge::new(
                virtual_edge_id_1,
                edge_start_node,
                virtual_node,
                compute_geometry_distance(&virtual_geometry_1),
                edge.properties().clone(),
            ),
            virtual_geometry_1,
        );

        // Connect the start node to the virtual edge
        self.overlay
            .connect_edge(virtual_edge_id_1, edge_start_node);

        self.overlay.add_virtual_edge(
            BaseGraphEdge::new(
                virtual_edge_id_2,
                virtual_node,
                edge_end_node,
                compute_geometry_distance(&virtual_geometry_2),
                edge.properties().clone(),
            ),
            virtual_geometry_2,
        );

        // Connect the end node to the virtual edge
        self.overlay.connect_edge(virtual_edge_id_2, edge_end_node);

        // Connect the virtual node to the virtual edge
        self.overlay.connect_edge(virtual_edge_id_1, virtual_node);
        self.overlay.connect_edge(virtual_edge_id_2, virtual_node);
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

                    let virtual_edge_id = self.overlay.edge_count();

                    // Add the new edge and its geometry
                    self.overlay.add_virtual_edge(
                        BaseGraphEdge::new(
                            virtual_edge_id,
                            start_node,
                            end_node,
                            compute_geometry_distance(&virtual_geometry),
                            edge.properties().clone(),
                        ),
                        virtual_geometry,
                    );

                    // Add the edge to the adjacency list of both virtual nodes
                    let start_virtual_node_id = self.overlay.virtual_node_id(start_node);
                    let end_virtual_node_id = self.overlay.virtual_node_id(end_node);

                    self.overlay
                        .connect_edge(virtual_edge_id, start_virtual_node_id);
                    self.overlay
                        .connect_edge(virtual_edge_id, end_virtual_node_id);
                }
            }
        }
    }
}

impl GeometryAccess for QueryGraph<'_> {
    fn edge_geometry(&self, edge_id: usize) -> &[GeoPoint] {
        if self.overlay.is_virtual_edge(edge_id) {
            self.overlay.virtual_edge_geometry(edge_id)
        } else {
            self.base_graph.edge_geometry(edge_id)
        }
    }

    fn node_geometry(&self, node_id: usize) -> &GeoPoint {
        if self.is_virtual_node(node_id) {
            let first_edge_id = self.overlay.node_virtual_edges(node_id)[0];
            let edge_geometry = self.overlay.virtual_edge_geometry(first_edge_id);
            let edge_direction = self.edge_direction(first_edge_id, node_id);
            match edge_direction {
                EdgeDirection::Forward => &edge_geometry[0],
                EdgeDirection::Backward => &edge_geometry[edge_geometry.len() - 1],
            }
        } else {
            self.base_graph.node_geometry(node_id)
        }
    }
}

impl Graph for QueryGraph<'_> {
    type Edge = BaseGraphEdge;

    fn is_virtual_node(&self, node_id: usize) -> bool {
        node_id >= self.base_graph.node_count()
    }

    fn edge_count(&self) -> usize {
        self.overlay.edge_count()
    }

    fn node_count(&self) -> usize {
        self.overlay.node_count()
    }

    fn edge(&self, edge_id: usize) -> &BaseGraphEdge {
        if self.overlay.is_virtual_edge(edge_id) {
            self.overlay.virtual_edge(edge_id)
        } else {
            self.base_graph.edge(edge_id)
        }
    }

    fn edge_direction(&self, edge_id: usize, start_node_id: usize) -> EdgeDirection {
        if self.overlay.is_virtual_edge(edge_id) {
            let edge = self.overlay.virtual_edge(edge_id);

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
            QueryGraphEdgeIterator::new(&[], self.overlay.node_virtual_edges(node_id))
        } else {
            let virtual_edges = self.overlay.node_virtual_edges(node_id);
            let base_edges = self.base_graph.node_edges(node_id);

            QueryGraphEdgeIterator::new(base_edges, virtual_edges)
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
