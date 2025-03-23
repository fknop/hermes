use crate::{
    base_graph::{self, BaseGraph, GraphEdge},
    geometry::{compute_geometry_distance, create_virtual_geometries},
    geopoint::GeoPoint,
    graph::Graph,
    properties::property_map::{BACKWARD_EDGE, EdgeDirection, FORWARD_EDGE},
    snap::Snap,
};

pub(crate) struct QueryGraph<'a> {
    base_graph: &'a BaseGraph,

    virtual_nodes: usize,
    virtual_edges: Vec<GraphEdge>,
    virtual_adjacency_list: Vec<Vec<usize>>,
    virtual_edge_geometry: Vec<Vec<GeoPoint>>,
}

impl<'a> QueryGraph<'a> {
    pub fn from_base_graph(base_graph: &'a BaseGraph, snaps: &mut [Snap]) -> Self {
        let mut query_graph = QueryGraph {
            base_graph,
            virtual_nodes: 0,
            virtual_edge_geometry: Vec::new(),
            virtual_edges: Vec::new(),
            virtual_adjacency_list: Vec::new(),
        };

        for snap in snaps.iter_mut() {
            query_graph.add_edges_from_snap(snap)
        }

        query_graph
    }

    /**
     * If an edge A <--> B where the snap is located inside the edge A ---- S ---- B
     * The snap is not necessarily a node inside the edge but can be arbitrary coordinates between 2 nodes
     */
    fn add_edges_from_snap(&mut self, snap: &mut Snap) {
        let edge_id = snap.edge_id;
        let edge = self.base_graph.edge(edge_id);
        let geometry = self.base_graph.edge_geometry(edge_id);

        let edge_start_node = edge.start_node();
        let edge_end_node = edge.end_node();

        // Point if the first or last node of the edge, no need to create virtual edges
        if geometry[0] == snap.coordinates || geometry[geometry.len() - 1] == snap.coordinates {
            return;
        }

        let (virtual_geometry_1, virtual_geometry_2) =
            create_virtual_geometries(geometry, &snap.coordinates);

        let virtual_node = self.base_graph.node_count() + self.virtual_nodes;

        snap.set_closest_node(virtual_node);

        let virtual_edge_id_1 = self.base_graph.edge_count() + self.virtual_edges.len();
        let virtual_edge_id_2 = virtual_edge_id_1 + 1;

        self.virtual_edges.push(GraphEdge::new(
            virtual_edge_id_1,
            virtual_node,
            edge_start_node,
            compute_geometry_distance(&virtual_geometry_1),
            edge.properties.as_reversed(),
        ));

        self.virtual_edge_geometry.push(virtual_geometry_1);

        self.virtual_edges.push(GraphEdge::new(
            virtual_edge_id_2,
            virtual_node,
            edge_end_node,
            compute_geometry_distance(&virtual_geometry_2),
            edge.properties.clone(),
        ));

        self.virtual_edge_geometry.push(virtual_geometry_2);

        println!("Added virtual edge {}", virtual_edge_id_1);
        println!("Added virtual edge {}", virtual_edge_id_2);

        self.virtual_adjacency_list
            .push(vec![virtual_edge_id_1, virtual_edge_id_2]);

        self.virtual_nodes += 1;
    }

    fn is_virtual_edge(&self, edge_id: usize) -> bool {
        edge_id >= self.base_graph.edge_count()
    }

    fn is_virtual_node(&self, node_id: usize) -> bool {
        node_id >= self.base_graph.node_count()
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
    fn virtual_edge(&self, edge_id: usize) -> &GraphEdge {
        &self.virtual_edges[self.virtual_edge_id(edge_id)]
    }
}

impl<'a> Graph for QueryGraph<'a> {
    fn edge_count(&self) -> usize {
        self.base_graph.edge_count() + self.virtual_edges.len()
    }

    fn node_count(&self) -> usize {
        println!(
            "query_graph, node_count {} + {}",
            self.base_graph.node_count(),
            self.virtual_nodes
        );
        self.base_graph.node_count() + self.virtual_nodes
    }

    fn node_edges(&self, node_id: usize) -> &[usize] {
        if self.is_virtual_node(node_id) {
            &self.virtual_adjacency_list[self.virtual_node_id(node_id)]
        } else {
            self.base_graph.node_edges(node_id)
        }
    }

    fn edge(&self, edge_id: usize) -> &GraphEdge {
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

    fn edge_direction(&self, edge_id: usize, start_node_id: usize) -> EdgeDirection {
        if self.is_virtual_edge(edge_id) {
            let edge = self.virtual_edge(edge_id);

            if edge.start_node() == start_node_id {
                return FORWARD_EDGE;
            } else if edge.end_node() == start_node_id {
                return BACKWARD_EDGE;
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
