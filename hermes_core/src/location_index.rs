use crate::base_graph::BaseGraph;
use crate::geopoint::GeoPoint;
use crate::graph::Graph;
use crate::snap::Snap;
use crate::weighting::Weighting;
use geo::HaversineClosestPoint;
use rstar::primitives::GeomWithData;
use rstar::{AABB, PointDistance, RTree, RTreeObject};

struct IndexedLine(geo::LineString);

impl IndexedLine {
    fn new(points: &[GeoPoint]) -> Self {
        IndexedLine(geo::LineString::new(
            points.iter().map(|p| p.into()).collect(),
        ))
    }

    fn line(&self) -> &geo::LineString {
        &self.0
    }
}

impl RTreeObject for IndexedLine {
    type Envelope = AABB<geo::Point>;

    fn envelope(&self) -> Self::Envelope {
        let line = self.line();
        line.envelope()
    }
}

impl PointDistance for IndexedLine {
    fn distance_2(&self, point: &geo::Point) -> f64 {
        self.0.distance_2(point)
    }
}

struct IndexedData {
    edge_id: usize,
}

type LocationIndexObject = GeomWithData<IndexedLine, IndexedData>;

// struct LocationIndexTreeParams;

// impl RTreeParams for LocationIndexTreeParams {
//     type DefaultInsertionStrategy = RStarInsertionStrategy;

//     const MAX_SIZE: usize = 8;
//     const MIN_SIZE: usize = 2;
//     const REINSERTION_COUNT: usize = 5;
// }

pub struct LocationIndex {
    tree: RTree<LocationIndexObject>,
}

impl LocationIndex {
    pub fn build_from_graph(graph: &BaseGraph) -> LocationIndex {
        println!("Building location index");

        let tree: RTree<LocationIndexObject> = RTree::bulk_load(
            (0..graph.edge_count())
                .map(|edge_id| {
                    let geometry = graph.edge_geometry(edge_id);

                    LocationIndexObject::new(IndexedLine::new(geometry), IndexedData { edge_id })
                })
                .collect(),
        );

        println!("Finished building location index");

        LocationIndex { tree }
    }

    pub fn snap(
        &self,
        graph: &BaseGraph,
        weighting: &dyn Weighting,
        coordinates: &GeoPoint,
    ) -> Option<Snap> {
        self.tree
            .nearest_neighbor_iter(&coordinates.into())
            .find(|nearest_neighbor| {
                let edge_id = nearest_neighbor.data.edge_id;
                // We only consider edges that can be accessed by the weighting profile
                weighting.can_access_edge(graph.edge(edge_id))
            })
            .map(|nearest_neighbor| {
                let line = nearest_neighbor.geom().line();

                // Find the closest point on the line so that we can snap to the closest coordinates
                let closest_point: GeoPoint =
                    match line.haversine_closest_point(&coordinates.into()) {
                        geo::Closest::Intersection(point) => point.into(),
                        geo::Closest::SinglePoint(point) => point.into(),
                        geo::Closest::Indeterminate => line.points().next().unwrap().into(),
                    };

                Snap::new(
                    nearest_neighbor.data.edge_id,
                    closest_point,
                    coordinates.haversine_distance(&closest_point),
                )
            })
    }
}
