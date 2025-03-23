use crate::geopoint::GeoPoint;
use crate::graph::Graph;
use crate::properties::property_map::FORWARD_EDGE;
use crate::snap::Snap;
use crate::weighting::Weighting;
use rstar::primitives::GeomWithData;

use rstar::{AABB, RTree, RTreeObject};
use std::cell::Cell;
use std::cmp::min;

struct LocationIndexObjectData {
    edge_id: usize,
}

type LocationIndexObject = GeomWithData<GeoPoint, LocationIndexObjectData>;

pub struct LocationIndex {
    tree: RTree<LocationIndexObject>,
}

impl LocationIndex {
    pub fn build_from_graph(graph: &Graph) -> LocationIndex {
        println!("Building location index");

        let tree: RTree<LocationIndexObject> = RTree::bulk_load(
            (0..graph.edge_count())
                .flat_map(|edge_id| {
                    let geometry = graph.edge_geometry(edge_id);
                    geometry.iter().map(move |coordinates| {
                        LocationIndexObject::new(
                            coordinates.clone(),
                            LocationIndexObjectData { edge_id },
                        )
                    })
                })
                .collect(),
        );

        println!("Finished building location index");

        LocationIndex { tree }
    }

    pub fn closest(&self, coordinates: &GeoPoint) -> Option<usize> {
        self.tree
            .nearest_neighbor(&[coordinates.lng, coordinates.lat])
            .map(|location| location.data.edge_id)
    }

    pub fn snap(
        &self,
        graph: &Graph,
        weighting: &dyn Weighting,
        coordinates: &GeoPoint,
    ) -> Option<Snap> {
        self.tree
            .nearest_neighbor_iter_with_distance_2(&[coordinates.lng, coordinates.lat])
            .filter(|(nearest_neighbor, _)| {
                let edge_id = nearest_neighbor.data.edge_id;
                println!("edge_id: {}", edge_id);

                // We only consider edges that can be accessed by the weighting profile
                weighting.can_access_edge(graph.edge(edge_id))
            })
            .next()
            .map(|(nearest_neighbor, distance)| Snap {
                edge_id: nearest_neighbor.data.edge_id,
                distance,
            })
    }
}
