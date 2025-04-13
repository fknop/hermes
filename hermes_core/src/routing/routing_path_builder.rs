use crate::{
    edge_direction::{self, EdgeDirection},
    geopoint::GeoPoint,
    graph::{GeometryAccess, Graph},
    graph_edge::GraphEdge,
    types::EdgeId,
    weighting::Weighting,
};

use super::routing_path::{RoutingPath, RoutingPathLeg};

pub fn build_routing_path<G>(
    graph: &G,
    weighting: &impl Weighting<G>,
    edges: &[(EdgeId, EdgeDirection)],
) -> RoutingPath
where
    G: Graph + GeometryAccess,
{
    let mut legs: Vec<RoutingPathLeg> = Vec::with_capacity(32);

    for &(edge_id, direction) in edges {
        let edge = graph.edge(edge_id);

        let geometry: Vec<GeoPoint> = if direction == EdgeDirection::Forward {
            graph.edge_geometry(edge_id).to_vec()
        } else {
            graph.edge_geometry(edge_id).iter().rev().cloned().collect()
        };

        let distance = edge.distance();
        let time = weighting.calc_edge_ms(edge, direction);

        legs.push(RoutingPathLeg::new(distance, time, geometry));
    }

    RoutingPath::new(legs)
}
