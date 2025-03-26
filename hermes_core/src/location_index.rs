use crate::base_graph::BaseGraph;
use crate::distance::meters;
use crate::geometry::interpolate_geometry;
use crate::geopoint::GeoPoint;
use crate::graph::Graph;
use crate::snap::Snap;
use crate::weighting::Weighting;
use rstar::primitives::GeomWithData;

use rstar::{RStarInsertionStrategy, RTree, RTreeParams};

struct LocationIndexObjectData {
    edge_id: usize,
}

type LocationIndexObject = GeomWithData<GeoPoint, LocationIndexObjectData>;

struct LocationIndexTreeParams;
impl RTreeParams for LocationIndexTreeParams {
    type DefaultInsertionStrategy = RStarInsertionStrategy;

    const MAX_SIZE: usize = 64;
    const MIN_SIZE: usize = 28;
    const REINSERTION_COUNT: usize = 5;
}

pub struct LocationIndex {
    tree: RTree<LocationIndexObject, LocationIndexTreeParams>,
}

impl LocationIndex {
    pub fn build_from_graph(graph: &BaseGraph) -> LocationIndex {
        println!("Building location index");

        let tree: RTree<LocationIndexObject, LocationIndexTreeParams> =
            RTree::bulk_load_with_params(
                (0..graph.edge_count())
                    .flat_map(|edge_id| {
                        let geometry = graph.edge_geometry(edge_id);
                        let interpolated_geometry = interpolate_geometry(geometry, meters!(5));

                        interpolated_geometry.into_iter().map(move |coordinates| {
                            LocationIndexObject::new(
                                coordinates,
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
            .nearest_neighbor(&[coordinates.lon, coordinates.lat])
            .map(|location| location.data.edge_id)
    }

    pub fn snap(
        &self,
        graph: &BaseGraph,
        weighting: &dyn Weighting,
        coordinates: &GeoPoint,
    ) -> Option<Snap> {
        self.tree
            .nearest_neighbor_iter(&[coordinates.lon, coordinates.lat])
            .find(|nearest_neighbor| {
                let edge_id = nearest_neighbor.data.edge_id;
                // We only consider edges that can be accessed by the weighting profile
                weighting.can_access_edge(graph.edge(edge_id))
            })
            .map(|nearest_neighbor| {
                println!("distance {}", coordinates.distance(nearest_neighbor.geom()));
                Snap::new(
                    nearest_neighbor.data.edge_id,
                    *nearest_neighbor.geom(),
                    coordinates.distance(nearest_neighbor.geom()),
                )
            })
    }
}
