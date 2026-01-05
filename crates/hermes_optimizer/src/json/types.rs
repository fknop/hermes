use hermes_matrix_providers::{
    cache::MatricesCache, travel_matrix_client::TravelMatrixClient,
    travel_matrix_provider::TravelMatrixProvider,
};
use jiff::{SignedDuration, Timestamp};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::problem::{
    capacity::Capacity,
    fleet::Fleet,
    location::Location,
    service::{ServiceBuilder, ServiceType},
    time_window::TimeWindow,
    travel_cost_matrix::TravelMatrices,
    vehicle::{VehicleBuilder, VehicleShift},
    vehicle_profile::VehicleProfile,
    vehicle_routing_problem::{VehicleRoutingProblem, VehicleRoutingProblemBuilder},
};

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields, rename = "VehicleRoutingProblem")]
pub struct JsonVehicleRoutingProblem {
    pub locations: Vec<JsonLocation>,
    pub services: Vec<JsonService>,
    pub vehicle_profiles: Vec<JsonVehicleProfile>,
    pub vehicles: Vec<JsonVehicle>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields, rename = "Service")]
pub struct JsonService {
    pub id: String,
    pub location_id: usize,
    pub duration: Option<SignedDuration>,
    pub demand: Option<Vec<f64>>,
    pub skills: Option<Vec<String>>,
    pub time_windows: Option<Vec<TimeWindow>>,
    pub service_type: Option<ServiceType>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields, rename = "Location")]
pub struct JsonLocation {
    pub coordinates: [f64; 2],
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields, rename = "VehicleProfile")]
pub struct JsonVehicleProfile {
    pub id: String,
    pub cost_provider: TravelMatrixProvider,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields, rename = "Vehicle")]
pub struct JsonVehicle {
    pub id: String,
    pub profile: String,
    pub shift: Option<JsonVehicleShift>,
    pub capacity: Option<Vec<f64>>,
    pub depot_location_id: Option<usize>,
    pub depot_duration: Option<SignedDuration>,
    pub should_return_to_depot: Option<bool>,
    pub return_depot_duration: Option<SignedDuration>,
    pub skills: Option<Vec<String>>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields, rename = "VehicleShift")]
pub struct JsonVehicleShift {
    pub earliest_start: Option<Timestamp>,
    pub latest_end: Option<Timestamp>,
    pub maximum_transport_duration: Option<SignedDuration>,
    pub maximum_working_duration: Option<SignedDuration>,
}

impl From<JsonVehicleShift> for VehicleShift {
    fn from(value: JsonVehicleShift) -> Self {
        VehicleShift {
            earliest_start: value.earliest_start,
            latest_end: value.latest_end,
            maximum_transport_duration: value.maximum_transport_duration,
            maximum_working_duration: value.maximum_working_duration,
        }
    }
}

impl JsonVehicleRoutingProblem {
    pub async fn build_problem(
        self,
        client: &TravelMatrixClient<impl MatricesCache>,
    ) -> Result<VehicleRoutingProblem, anyhow::Error> {
        let mut builder = VehicleRoutingProblemBuilder::default();

        let locations = self
            .locations
            .iter()
            .map(|location| {
                Location::from_lat_lon(location.coordinates[1], location.coordinates[0])
            })
            .collect::<Vec<_>>();

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
                    builder.set_demand(Capacity::from_vec(demand));
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

                if let Some(shift) = vehicle.shift {
                    builder.set_vehicle_shift(shift.into());
                }

                if let Some(capacity) = vehicle.capacity {
                    builder.set_capacity(Capacity::from_vec(capacity));
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

        builder.set_fleet(Fleet::Finite(vehicles));

        let futures = self
            .vehicle_profiles
            .into_iter()
            .map(|profile| async {
                let travel_matrices = client
                    .fetch_matrix(&locations, profile.cost_provider)
                    .await?;
                Ok::<
                    (
                        String,
                        hermes_matrix_providers::travel_matrices::TravelMatrices,
                    ),
                    anyhow::Error,
                >((profile.id, travel_matrices))
            })
            .collect::<Vec<_>>();

        let results = futures::future::try_join_all(futures).await?;

        builder.set_vehicle_profiles(
            results
                .into_iter()
                .map(|(id, matrices)| {
                    VehicleProfile::new(id, TravelMatrices::from_travel_matrices(matrices))
                })
                .collect(),
        );

        builder.set_locations(locations);
        Ok(builder.build())
    }
}
