use crate::graph::Graph;
use crate::latlng::LatLng;
use rstar::primitives::GeomWithData;
use rstar::{AABB, RTree, RTreeObject};

type LocationIndexObject = GeomWithData<[f64; 2], usize>;

pub struct LocationIndex {
    tree: RTree<LocationIndexObject>,
}

// impl RTreeObject for LatLng {
//     type Envelope = AABB<[f64; 2]>;
//     fn envelope(&self) -> Self::Envelope {
//         AABB::from_point([self.lng, self.lat])
//     }
// }

impl LocationIndex {
    pub fn build_from_graph(graph: &Graph) -> LocationIndex {
        let tree: RTree<LocationIndexObject> = RTree::bulk_load(
            (0..graph.edge_count())
                .flat_map(|edge_id| {
                    let geometry = graph.edge_geometry(edge_id);
                    geometry.iter().map(move |coordinates| {
                        LocationIndexObject::new(coordinates.into(), edge_id)
                    })
                })
                .collect(),
        );

        LocationIndex { tree }
    }

    pub fn closest(&self, coordinates: &LatLng) -> Option<usize> {
        self.tree
            .nearest_neighbor(&coordinates.into())
            .map(|location| location.data)
    }
}
