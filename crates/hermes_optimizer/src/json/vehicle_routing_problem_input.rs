use hermes_matrix_providers::travel_matrix_provider::TravelMatrixProvider;
use jiff::SignedDuration;
use serde::Deserialize;

use crate::problem::{
    capacity::Capacity,
    location::Location,
    service::{ServiceBuilder, ServiceType},
    skill::Skill,
    time_window::TimeWindow,
    vehicle::VehicleBuilder,
    vehicle_routing_problem::{VehicleRoutingProblem, VehicleRoutingProblemBuilder},
};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VehicleRoutingProblemInput {
    pub locations: Vec<LocationInput>,
    pub services: Vec<ServiceInput>,
    pub vehicle_profiles: Vec<VehicleProfileInput>,
    pub vehicles: Vec<VehicleInput>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServiceInput {
    pub id: String,
    pub location_id: usize,
    pub duration: Option<SignedDuration>,
    pub demand: Option<Capacity>,
    pub skills: Option<Vec<String>>,
    pub time_windows: Option<Vec<TimeWindow>>,
    pub service_type: Option<ServiceType>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LocationInput {
    pub coordinates: [f64; 2],
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VehicleProfileInput {
    pub id: String,
    pub cost_provider: TravelMatrixProvider,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VehicleInput {
    pub id: String,
    pub profile: String,
    pub capacity: Option<Capacity>,
    pub depot_location_id: Option<usize>,
    pub depot_duration: Option<SignedDuration>,
    pub should_return_to_depot: Option<bool>,
    pub return_depot_duration: Option<SignedDuration>,
    pub skills: Option<Vec<String>>,
}

impl VehicleRoutingProblemInput {
    pub fn create_problem(self) -> VehicleRoutingProblem {
        let mut builder = VehicleRoutingProblemBuilder::default();

        for location in self.locations.iter() {
            builder.add_location(Location::from_lat_lon(
                location.coordinates[1],
                location.coordinates[0],
            ));
        }

        let services = self
            .services
            .into_iter()
            .map(|service| {
                let mut builder = ServiceBuilder::default();

                builder.set_location_id(service.location_id);
                builder.set_external_id(service.id);

                if let Some(service_type) = service.service_type {
                    builder.set_service_type(service_type);
                }

                if let Some(demand) = service.demand {
                    builder.set_demand(demand);
                }

                if let Some(skills) = service.skills {
                    builder.set_skills(skills);
                }

                if let Some(duration) = service.duration {
                    builder.set_service_duration(duration);
                }

                if let Some(time_windows) = service.time_windows {
                    builder.set_time_windows(time_windows);
                }

                builder.build()
            })
            .collect();

        builder.set_services(services);

        let vehicles = self
            .vehicles
            .into_iter()
            .map(|vehicle| {
                let mut builder = VehicleBuilder::default();

                if let Some(capacity) = vehicle.capacity {
                    builder.set_capacity(capacity);
                }

                if let Some(depot_duration) = vehicle.depot_duration {
                    builder.set_depot_duration(depot_duration);
                }

                if let Some(depot_location_id) = vehicle.depot_location_id {
                    builder.set_depot_location_id(depot_location_id);
                }

                if let Some(should_return) = vehicle.should_return_to_depot {
                    builder.set_return(should_return);
                }

                if let Some(end_duration) = vehicle.return_depot_duration {
                    builder.set_end_depot_duration(end_duration);
                }

                if let Some(skills) = vehicle.skills {
                    builder.set_skills(skills);
                }

                builder.build()
            })
            .collect();

        builder.set_vehicles(vehicles);

        // for profile in &self.vehicle_profiles {
        //     builder.add_vehicle_profile(profile.id.clone(), profile.cost_provider.clone());
        // }
        //
        //
        builder.build()
    }
}
