use geo::{Distance, Haversine};
use rstar::primitives::GeomWithData;
use rstar::{AABB, Envelope, PointDistance, RTree, RTreeObject};

use crate::problem::job::{ActivityId, Job};
use crate::utils::enumerate_idx::EnumerateIdx;

use super::distance_method::DistanceMethod;
use super::location::Location;

pub struct IndexedData {
    job_id: ActivityId,
}

pub enum IndexedPoint {
    Haversine { x: f64, y: f64 },
    Euclidean { x: f64, y: f64 },
}

impl IndexedPoint {
    pub fn x(&self) -> f64 {
        match self {
            IndexedPoint::Haversine { x, .. } | IndexedPoint::Euclidean { x, .. } => *x,
        }
    }

    pub fn y(&self) -> f64 {
        match self {
            IndexedPoint::Haversine { y, .. } | IndexedPoint::Euclidean { y, .. } => *y,
        }
    }
}

impl RTreeObject for IndexedPoint {
    type Envelope = AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        AABB::from_point([self.x(), self.y()])
    }
}

impl PointDistance for IndexedPoint {
    fn distance_2(
        &self,
        point: &<Self::Envelope as Envelope>::Point,
    ) -> <<Self::Envelope as Envelope>::Point as rstar::Point>::Scalar {
        match self {
            IndexedPoint::Haversine { .. } => {
                let distance = Haversine.distance(
                    geo::Point::new(self.x(), self.y()),
                    geo::Point::new(point[0], point[1]),
                );

                distance * distance
            }
            IndexedPoint::Euclidean { .. } => {
                geo::Point::new(self.x(), self.y()).distance_2(&geo::Point::new(point[0], point[1]))
            }
        }
    }
}

pub type ServiceLocationIndexObject = GeomWithData<IndexedPoint, IndexedData>;

pub struct ServiceLocationIndex {
    tree: RTree<ServiceLocationIndexObject>,
}

impl ServiceLocationIndex {
    pub fn new(
        locations: &[Location],
        jobs: &[Job],
        distance_method: DistanceMethod,
    ) -> ServiceLocationIndex {
        let mut location_ids = vec![];

        for (job_id, job) in jobs.iter().enumerate_idx() {
            match job {
                Job::Service(service) => {
                    location_ids.push((ActivityId::Service(job_id), service.location_id()));
                }
                Job::Shipment(shipment) => {
                    location_ids.push((
                        ActivityId::ShipmentPickup(job_id),
                        shipment.pickup().location_id(),
                    ));
                    location_ids.push((
                        ActivityId::ShipmentDelivery(job_id),
                        shipment.delivery().location_id(),
                    ));
                }
            }
        }

        let tree: RTree<ServiceLocationIndexObject> = RTree::bulk_load(
            location_ids
                .iter()
                .map(|&(job_id, location_id)| {
                    let location = &locations[location_id];

                    ServiceLocationIndexObject::new(
                        match distance_method {
                            DistanceMethod::Haversine => IndexedPoint::Haversine {
                                x: location.lon(),
                                y: location.lat(),
                            },
                            DistanceMethod::Euclidean => IndexedPoint::Euclidean {
                                x: location.x(),
                                y: location.y(),
                            },
                        },
                        IndexedData { job_id },
                    )
                })
                .collect(),
        );

        ServiceLocationIndex { tree }
    }

    pub fn nearest_neighbor_iter<'a, P>(&'a self, point: P) -> impl Iterator<Item = ActivityId> + 'a
    where
        P: Into<geo::Point>,
    {
        let point: geo::Point = point.into();
        self.tree
            .nearest_neighbor_iter(&[point.x(), point.y()])
            .map(|geom_with_data| geom_with_data.data.job_id)
    }
}

#[cfg(test)]
mod tests {}
