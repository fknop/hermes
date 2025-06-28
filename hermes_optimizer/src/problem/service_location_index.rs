use geo::HaversineClosestPoint;
use rstar::RTree;
use rstar::primitives::GeomWithData;

use super::location::Location;
use super::service::Service;

pub struct IndexedData {
    service_id: usize,
}

pub type ServiceLocationIndexObject = GeomWithData<geo::Point, IndexedData>;

pub struct ServiceLocationIndex {
    tree: RTree<ServiceLocationIndexObject>,
}

impl ServiceLocationIndex {
    pub fn new(locations: &[Location], services: &[Service]) -> ServiceLocationIndex {
        let tree: RTree<ServiceLocationIndexObject> = RTree::bulk_load(
            services
                .iter()
                .enumerate()
                .map(|(service_id, service)| {
                    let location = &locations[service.location_id()];
                    let point = geo::Point::new(location.x(), location.y());

                    ServiceLocationIndexObject::new(point, IndexedData { service_id })
                })
                .collect(),
        );

        ServiceLocationIndex { tree }
    }

    pub fn nearest_neighbor_iter<'a>(
        &'a self,
        point: geo::Point,
    ) -> impl Iterator<Item = usize> + 'a {
        self.tree
            .nearest_neighbor_iter(&point)
            .map(|geom_with_data| geom_with_data.data.service_id)
    }
}
