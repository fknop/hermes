use crate::problem::{
    location::Location,
    service::{Service, ServiceId},
    service_location_index::ServiceLocationIndex,
};

pub struct Neighborhoods {
    neighborhood: Vec<Vec<ServiceId>>,
}

pub struct BuildNeighborhoodParams<'a> {
    pub services: &'a [Service],
    pub locations: &'a [Location],
    pub location_index: &'a ServiceLocationIndex,
}

impl Neighborhoods {
    pub fn empty() -> Self {
        Neighborhoods {
            neighborhood: vec![],
        }
    }

    pub fn new(params: BuildNeighborhoodParams) -> Self {
        Neighborhoods {
            neighborhood: Neighborhoods::build_neighborhood(params),
        }
    }

    pub fn neighbors_iter(&self, service_id: ServiceId) -> impl Iterator<Item = ServiceId> {
        self.neighborhood[service_id].iter().cloned()
    }

    fn build_neighborhood(
        BuildNeighborhoodParams {
            services,
            location_index,
            locations,
        }: BuildNeighborhoodParams,
    ) -> Vec<Vec<ServiceId>> {
        let limit = 40;
        let mut neighborhood: Vec<Vec<ServiceId>> = vec![];

        for (service_id, service) in services.iter().enumerate() {
            let mut neighbors: Vec<ServiceId> = vec![];

            let location_id = service.location_id();
            let location = &locations[location_id];
            neighbors.extend(
                location_index
                    .nearest_neighbor_iter(location)
                    .filter(|&neighbor_service_id| neighbor_service_id != service_id)
                    .take(limit),
            );

            neighborhood.push(neighbors);
        }

        neighborhood
    }
}
