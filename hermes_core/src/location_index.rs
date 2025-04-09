use std::fs::File;
use std::io::{BufReader, BufWriter};

use crate::base_graph::BaseGraph;
use crate::geopoint::GeoPoint;
use crate::graph::Graph;
use crate::snap::Snap;
use crate::stopwatch::{self, Stopwatch};
use crate::storage::write_bytes;
use crate::weighting::Weighting;
use bincode::serde::EncodeError;
use geo::HaversineClosestPoint;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use rstar::primitives::GeomWithData;
use rstar::{AABB, PointDistance, RTree, RTreeObject};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
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

#[derive(Serialize, Deserialize)]
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
        let stopwatch = Stopwatch::new("build_location_index");
        let tree: RTree<LocationIndexObject> = RTree::bulk_load(
            (0..graph.edge_count())
                .map(|edge_id| {
                    let geometry = graph.edge_geometry(edge_id);

                    LocationIndexObject::new(IndexedLine::new(geometry), IndexedData { edge_id })
                })
                .collect(),
        );

        stopwatch.report();

        LocationIndex { tree }
    }

    pub fn save_to_file(&self, path: &str) -> Result<usize, bincode::error::EncodeError> {
        let stopwatch = Stopwatch::new("location_index/save_to_file");
        let mut file = File::create(path).expect("failed to create file");
        let mut writer = BufWriter::new(&mut file);
        let result = bincode::serde::encode_into_std_write(
            &self.tree,
            &mut writer,
            bincode::config::standard(),
        );
        stopwatch.report();
        result
    }

    pub fn load_from_file(path: &str) -> Self {
        let stopwatch = Stopwatch::new("location_index/load_from_file");
        let mut file = File::open(path).expect("failed to open file");
        let mut reader = BufReader::new(&mut file);
        let result: Result<RTree<LocationIndexObject>, bincode::error::DecodeError> =
            bincode::serde::decode_from_std_read(&mut reader, bincode::config::standard());
        let tree = result.unwrap();
        stopwatch.report();
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
