use crate::base_graph::BaseGraph;
use crate::geopoint::GeoPoint;
use crate::graph::Graph;
use crate::snap::Snap;
use crate::weighting::Weighting;
use geo::HaversineClosestPoint;
use rstar::primitives::GeomWithData;
use rstar::{AABB, PointDistance, RStarInsertionStrategy, RTree, RTreeObject, RTreeParams};

struct IndexedLine(geo::Line);

impl IndexedLine {
    fn new(start: &GeoPoint, end: &GeoPoint) -> Self {
        IndexedLine(geo::Line::new(start, end))
    }

    fn line(&self) -> &geo::Line {
        &self.0
    }
}

impl RTreeObject for IndexedLine {
    type Envelope = AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        let line = self.line();
        AABB::from_corners([line.start.x, line.start.y], [line.end.x, line.end.y])
    }
}

impl PointDistance for IndexedLine {
    fn distance_2(&self, point: &[f64; 2]) -> f64 {
        let point = geo::Point::new(point[0], point[1]);
        self.0.distance_2(&point)
    }
}

struct IndexedData {
    edge_id: usize,
}

type LocationIndexObject = GeomWithData<IndexedLine, IndexedData>;

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

                        geometry.windows(2).map(move |c| {
                            LocationIndexObject::new(
                                IndexedLine::new(&c[0], &c[1]),
                                IndexedData { edge_id },
                            )
                        })
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
            .nearest_neighbor_iter(&[coordinates.lon(), coordinates.lat()])
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
                        geo::Closest::Indeterminate => line.start.into(),
                    };

                Snap::new(
                    nearest_neighbor.data.edge_id,
                    closest_point,
                    coordinates.haversine_distance(&closest_point),
                )
            })
    }
}
