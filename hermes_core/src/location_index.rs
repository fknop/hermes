use crate::geopoint::GeoPoint;
use crate::graph::Graph;
use rstar::primitives::GeomWithData;
use rstar::{AABB, RTree, RTreeObject};
use std::cell::Cell;

type LocationIndexObject = GeomWithData<GeoPoint, usize>;

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
                        LocationIndexObject::new(coordinates.clone(), edge_id)
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
            .map(|location| location.data)
    }
}
