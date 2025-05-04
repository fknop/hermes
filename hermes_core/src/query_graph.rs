use crate::{
    base_graph::{BaseGraph, BaseGraphEdge},
    ch::{
        ch_edge::{CHBaseEdge, CHGraphEdge},
        ch_graph::CHGraph,
    },
    constants::{MAX_DURATION, MAX_WEIGHT},
    edge_direction::EdgeDirection,
    geometry::{
        compute_geometry_distance, create_virtual_geometries,
        create_virtual_geometry_between_points,
    },
    geopoint::GeoPoint,
    graph::{DirectedEdgeAccess, GeometryAccess, Graph, UndirectedEdgeAccess, UnfoldEdge},
    graph_edge::GraphEdge,
    query_graph_overlay::QueryGraphOverlay,
    snap::Snap,
    types::{EdgeId, NodeId},
    weighting::{Milliseconds, Weight},
};

/// A dynamic graph that extends a base graph with virtual nodes and edges
///
/// QueryGraph wraps a base graph and allows for the dynamic addition of virtual nodes
/// and edges during query time. This is particularly useful for routing queries where
/// temporary modifications to the graph are needed without altering the underlying base graph.
///
/// Virtual nodes are typically added when a query point (snap) lies along an existing edge,
/// splitting that edge into two virtual edges connected by the new virtual node.
pub(crate) struct QueryGraph<'a, G>
where
    G: Graph + GeometryAccess + BuildVirtualEdge,
{
    graph: &'a G,
    base_graph: &'a BaseGraph,
    overlay: QueryGraphOverlay<'a, G>,
}

impl<'a, G> QueryGraph<'a, G>
where
    G: Graph + GeometryAccess + BuildVirtualEdge,
{
    pub fn from_graph(queried_graph: &'a G, base_graph: &'a BaseGraph, snaps: &mut [Snap]) -> Self {
        let mut query_graph = QueryGraph {
            base_graph,
            graph: queried_graph,
            overlay: QueryGraphOverlay::from_graph(queried_graph),
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
        let edge = self.graph.edge(edge_id);
        let geometry = self.graph.edge_geometry(edge_id);

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
            self.graph.build_virtual_edge(
                virtual_edge_id_1,
                edge_start_node,
                virtual_node,
                &virtual_geometry_1,
                edge,
            ),
            virtual_geometry_1,
        );

        // Connect the start node to the virtual edge
        self.overlay
            .connect_edge(virtual_edge_id_1, edge_start_node);

        self.overlay.add_virtual_edge(
            self.graph.build_virtual_edge(
                virtual_edge_id_2,
                virtual_node,
                edge_end_node,
                &virtual_geometry_2,
                edge,
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

                    let edge = self.graph.edge(snap_i.edge_id);
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
                        self.graph.build_virtual_edge(
                            virtual_edge_id,
                            start_node,
                            end_node,
                            &virtual_geometry,
                            edge,
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

impl<G> GeometryAccess for QueryGraph<'_, G>
where
    G: Graph + GeometryAccess + BuildVirtualEdge,
{
    fn edge_geometry(&self, edge_id: usize) -> &[GeoPoint] {
        if self.overlay.is_virtual_edge(edge_id) {
            self.overlay.virtual_edge_geometry(edge_id)
        } else {
            self.graph.edge_geometry(edge_id)
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
            self.graph.node_geometry(node_id)
        }
    }
}

impl<G> Graph for QueryGraph<'_, G>
where
    G: Graph + GeometryAccess + BuildVirtualEdge,
{
    type Edge = G::Edge;

    fn is_virtual_node(&self, node_id: usize) -> bool {
        node_id >= self.graph.node_count()
    }

    fn edge_count(&self) -> usize {
        self.overlay.edge_count()
    }

    fn node_count(&self) -> usize {
        self.overlay.node_count()
    }

    fn edge(&self, edge_id: usize) -> &G::Edge {
        if self.overlay.is_virtual_edge(edge_id) {
            self.overlay.virtual_edge(edge_id)
        } else {
            self.graph.edge(edge_id)
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
            self.graph.edge_direction(edge_id, start_node_id)
        }
    }
}

impl<G> UndirectedEdgeAccess for QueryGraph<'_, G>
where
    G: Graph + GeometryAccess + BuildVirtualEdge,
{
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

impl<'a> DirectedEdgeAccess for QueryGraph<'a, CHGraph<'a>> {
    type EdgeIterator<'b>
        = QueryGraphEdgeIterator<'b>
    where
        Self: 'b;

    fn node_incoming_edges_iter(&self, node_id: NodeId) -> Self::EdgeIterator<'_> {
        let virtual_edges = self.overlay.node_virtual_edges(node_id);
        if self.is_virtual_node(node_id) {
            QueryGraphEdgeIterator::new(&[], virtual_edges)
        } else {
            let base_edges = self.graph.node_incoming_edges(node_id);

            QueryGraphEdgeIterator::new(base_edges, virtual_edges)
        }
    }

    fn node_outgoing_edges_iter(&self, node_id: NodeId) -> Self::EdgeIterator<'_> {
        let virtual_edges = self.overlay.node_virtual_edges(node_id);
        if self.is_virtual_node(node_id) {
            QueryGraphEdgeIterator::new(&[], virtual_edges)
        } else {
            let base_edges = self.graph.node_outgoing_edges(node_id);

            QueryGraphEdgeIterator::new(base_edges, virtual_edges)
        }
    }
}

impl UnfoldEdge for QueryGraph<'_, CHGraph<'_>> {
    fn unfold_edge(&self, edge_id: EdgeId, edges: &mut Vec<EdgeId>) {
        if self.overlay.is_virtual_edge(edge_id) {
            edges.push(edge_id);
        } else {
            self.graph.unfold_edge(edge_id, edges);
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

pub trait BuildVirtualEdge
where
    Self: Graph,
{
    fn build_virtual_edge(
        &self,
        virtual_edge_id: EdgeId,
        start: NodeId,
        end: NodeId,
        geometry: &[GeoPoint],
        initial_edge: &Self::Edge,
    ) -> Self::Edge;
}

impl BuildVirtualEdge for BaseGraph {
    fn build_virtual_edge(
        &self,
        virtual_edge_id: EdgeId,
        start: NodeId,
        end: NodeId,
        geometry: &[GeoPoint],
        initial_edge: &BaseGraphEdge,
    ) -> BaseGraphEdge {
        BaseGraphEdge::new(
            virtual_edge_id,
            start,
            end,
            compute_geometry_distance(geometry),
            initial_edge.properties().clone(),
        )
    }
}

impl BuildVirtualEdge for CHGraph<'_> {
    fn build_virtual_edge(
        &self,
        virtual_edge_id: EdgeId,
        start: NodeId,
        end: NodeId,
        geometry: &[GeoPoint],
        initial_edge: &CHGraphEdge,
    ) -> CHGraphEdge {
        let distance = compute_geometry_distance(geometry);
        let original_distance = initial_edge.distance();
        let ratio = distance / original_distance;

        match &initial_edge {
            CHGraphEdge::Edge(edge) => {
                let direction =
                    if end == initial_edge.end_node() || start == initial_edge.start_node() {
                        EdgeDirection::Forward
                    } else {
                        EdgeDirection::Backward
                    };

                let (forward_time, backward_time, forward_weight, backward_weight) = match direction
                {
                    EdgeDirection::Forward => (
                        edge.forward_time,
                        edge.backward_time,
                        edge.forward_weight,
                        edge.backward_weight,
                    ),
                    EdgeDirection::Backward => (
                        edge.backward_time,
                        edge.forward_time,
                        edge.backward_weight,
                        edge.forward_weight,
                    ),
                };

                CHGraphEdge::Edge(CHBaseEdge {
                    id: virtual_edge_id,
                    start,
                    end,
                    distance: compute_geometry_distance(geometry),
                    forward_time: if forward_time == MAX_DURATION {
                        MAX_DURATION
                    } else {
                        (forward_time as f64 * ratio).round() as Milliseconds
                    },
                    backward_time: if backward_time == MAX_DURATION {
                        MAX_DURATION
                    } else {
                        (backward_time as f64 * ratio).round() as Milliseconds
                    },
                    forward_weight: if forward_weight == MAX_WEIGHT {
                        MAX_WEIGHT
                    } else {
                        (forward_weight as f64 * ratio).round() as Weight
                    },
                    backward_weight: if backward_weight == MAX_WEIGHT {
                        MAX_WEIGHT
                    } else {
                        (backward_weight as f64 * ratio).round() as Weight
                    },
                })
            }
            _ => panic!("Could not create a virtual edge from a shortcut edge"),
        }
    }
}
