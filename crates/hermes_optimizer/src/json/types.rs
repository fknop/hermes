use hermes_matrix_providers::{
    cache::MatricesCache, travel_matrix_client::TravelMatrixClient,
    travel_matrix_provider::TravelMatrixProvider,
};
use jiff::{SignedDuration, Timestamp};
use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use tracing::instrument;

use crate::problem::{
    capacity::Capacity,
    fleet::Fleet,
    job::{ActivityId, JobIdx},
    location::Location,
    relation::{
        InDirectSequenceRelation, InSameRouteRelation, InSequenceRelation, NotInSameRouteRelation,
        Relation,
    },
    service::{Service, ServiceBuilder, ServiceType},
    time_window::TimeWindow,
    travel_cost_matrix::TravelMatrices,
    vehicle::{Vehicle, VehicleBuilder, VehicleIdx, VehicleShift},
    vehicle_profile::VehicleProfile,
    vehicle_routing_problem::{VehicleRoutingProblem, VehicleRoutingProblemBuilder},
};

pub trait FromProblem<T> {
    fn from_problem(value: T, problem: &VehicleRoutingProblem) -> Self;
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename = "VehicleRoutingProblem")]
pub struct JsonVehicleRoutingProblem {
    pub id: Option<String>,
    pub locations: Vec<JsonLocation>,
    pub services: Vec<JsonService>,
    pub vehicle_profiles: Vec<JsonVehicleProfile>,
    pub vehicles: Vec<JsonVehicle>,
    pub relations: Option<Vec<JsonRelation>>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields, rename = "Service")]
pub struct JsonService {
    pub id: String,
    pub location_id: usize,
    pub duration: Option<SignedDuration>,
    pub demand: Option<Vec<f64>>,
    pub skills: Option<Vec<String>>,
    pub time_windows: Option<Vec<TimeWindow>>,

    #[serde(rename = "type")]
    pub service_type: Option<ServiceType>,
}

impl FromProblem<&Service> for JsonService {
    fn from_problem(value: &Service, _problem: &VehicleRoutingProblem) -> Self {
        JsonService {
            id: value.external_id().to_owned(),
            location_id: value.location_id().get(),
            duration: value.duration().into(),
            demand: Some(value.demand().to_vec()),
            skills: Some(
                value
                    .skills()
                    .iter()
                    .map(|skill| skill.to_string())
                    .collect::<Vec<_>>(),
            ),
            time_windows: Some(value.time_windows().to_vec()),
            service_type: value.service_type().into(),
        }
    }
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields, rename = "Location")]
pub struct JsonLocation {
    pub coordinates: [f64; 2],
}

impl FromProblem<&Location> for JsonLocation {
    fn from_problem(value: &Location, _problem: &VehicleRoutingProblem) -> Self {
        JsonLocation {
            coordinates: [value.x(), value.y()],
        }
    }
}

impl From<&JsonLocation> for geo::Point {
    fn from(value: &JsonLocation) -> Self {
        geo::Point::new(value.coordinates[0], value.coordinates[1])
    }
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields, rename = "VehicleProfile")]
pub struct JsonVehicleProfile {
    pub id: String,
    pub cost_provider: TravelMatrixProvider,
}

#[derive(Serialize, Deserialize, JsonSchema)]
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
    pub maximum_activities: Option<usize>,
}

impl FromProblem<&Vehicle> for JsonVehicle {
    fn from_problem(value: &Vehicle, problem: &VehicleRoutingProblem) -> Self {
        JsonVehicle {
            id: value.external_id().to_owned(),
            profile: problem
                .vehicle_profile(value.profile_id())
                .external_id()
                .to_owned(),
            shift: value.shift().map(JsonVehicleShift::from),
            capacity: Some(value.capacity().to_vec()),
            depot_location_id: value.depot_location_id().map(|l| l.get()),
            depot_duration: value.depot_duration().into(),
            should_return_to_depot: value.should_return_to_depot().into(),
            return_depot_duration: value.end_depot_duration().into(),
            skills: Some(
                value
                    .skills()
                    .iter()
                    .map(|skill| skill.to_string())
                    .collect::<Vec<_>>(),
            ),
            maximum_activities: value.maximum_activities(),
        }
    }
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields, rename = "VehicleShift")]
pub struct JsonVehicleShift {
    pub earliest_start: Option<Timestamp>,
    pub latest_start: Option<Timestamp>,
    pub latest_end: Option<Timestamp>,
    pub maximum_transport_duration: Option<SignedDuration>,
    pub maximum_working_duration: Option<SignedDuration>,
}

impl From<&VehicleShift> for JsonVehicleShift {
    fn from(value: &VehicleShift) -> Self {
        JsonVehicleShift {
            earliest_start: value.earliest_start,
            latest_start: value.latest_start,
            latest_end: value.latest_end,
            maximum_transport_duration: value.maximum_transport_duration,
            maximum_working_duration: value.maximum_working_duration,
        }
    }
}

impl From<JsonVehicleShift> for VehicleShift {
    fn from(value: JsonVehicleShift) -> Self {
        VehicleShift {
            earliest_start: value.earliest_start,
            latest_start: value.latest_start,
            latest_end: value.latest_end,
            maximum_transport_duration: value.maximum_transport_duration,
            maximum_working_duration: value.maximum_working_duration,
        }
    }
}

#[derive(JsonSchema)]
pub enum JsonActivityId {
    Pickup(String),
    Delivery(String),
    Service(String),
}

impl JsonActivityId {
    pub fn external_id(&self) -> &str {
        match self {
            JsonActivityId::Pickup(id) => id,
            JsonActivityId::Delivery(id) => id,
            JsonActivityId::Service(id) => id,
        }
    }
}

impl FromProblem<ActivityId> for JsonActivityId {
    fn from_problem(value: ActivityId, problem: &VehicleRoutingProblem) -> Self {
        match value {
            ActivityId::ShipmentPickup(idx) => {
                JsonActivityId::Pickup(problem.job(idx).external_id().to_owned())
            }
            ActivityId::ShipmentDelivery(idx) => {
                JsonActivityId::Delivery(problem.job(idx).external_id().to_owned())
            }
            ActivityId::Service(idx) => {
                JsonActivityId::Service(problem.job(idx).external_id().to_owned())
            }
        }
    }
}

impl std::fmt::Display for JsonActivityId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            JsonActivityId::Pickup(id) => format!("pickup_{id}"),
            JsonActivityId::Delivery(id) => format!("delivery_{id}"),
            JsonActivityId::Service(id) => format!("service_{id}"),
        };

        write!(f, "{}", s)
    }
}

impl Serialize for JsonActivityId {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let s = self.to_string();
        serializer.serialize_str(&s)
    }
}

impl<'de> Deserialize<'de> for JsonActivityId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        if let Some(id) = s.strip_prefix("pickup_") {
            Ok(JsonActivityId::Pickup(id.to_owned()))
        } else if let Some(id) = s.strip_prefix("delivery_") {
            Ok(JsonActivityId::Delivery(id.to_owned()))
        } else if let Some(id) = s.strip_prefix("service_") {
            Ok(JsonActivityId::Service(id.to_owned()))
        } else {
            Ok(JsonActivityId::Service(s.to_owned()))
            // Err(serde::de::Error::custom(format!(
            // "expected prefix pickup_, delivery_, or service_, got: {s}"
            // )))
        }
    }
}

#[derive(JsonSchema, Serialize, Deserialize)]
pub struct JsonInDirectSequenceRelation {
    pub vehicle_id: Option<String>,
    pub ids: Vec<JsonActivityId>,
}

#[derive(JsonSchema, Serialize, Deserialize)]
pub struct JsonInSequenceRelation {
    pub vehicle_id: Option<String>,
    pub ids: Vec<JsonActivityId>,
}

#[derive(JsonSchema, Serialize, Deserialize)]
pub struct JsonInSameRouteRelation {
    pub vehicle_id: Option<String>,
    pub ids: Vec<JsonActivityId>,
}

#[derive(JsonSchema, Serialize, Deserialize)]
pub struct JsonNotInSameRouteRelation {
    pub ids: Vec<JsonActivityId>,
}

#[derive(JsonSchema, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum JsonRelation {
    InSameRoute(JsonInSameRouteRelation),
    NotInSameRoute(JsonNotInSameRouteRelation),
    InSequence(JsonInDirectSequenceRelation),
    InDirectSequence(JsonInDirectSequenceRelation),
}

impl FromProblem<&Relation> for JsonRelation {
    fn from_problem(relation: &Relation, problem: &VehicleRoutingProblem) -> Self {
        match relation {
            Relation::InSameRoute(rel) => JsonRelation::InSameRoute(JsonInSameRouteRelation {
                vehicle_id: rel
                    .vehicle_id
                    .map(|id| problem.vehicle(id).external_id().to_owned()),
                ids: rel
                    .activity_ids
                    .iter()
                    .map(|&id| JsonActivityId::from_problem(id, problem))
                    .collect(),
            }),
            Relation::NotInSameRoute(rel) => {
                JsonRelation::NotInSameRoute(JsonNotInSameRouteRelation {
                    ids: rel
                        .activity_ids
                        .iter()
                        .map(|&id| JsonActivityId::from_problem(id, problem))
                        .collect(),
                })
            }
            Relation::InSequence(rel) => JsonRelation::InSequence(JsonInDirectSequenceRelation {
                vehicle_id: rel
                    .vehicle_id
                    .map(|id| problem.vehicle(id).external_id().to_owned()),
                ids: rel
                    .activity_ids
                    .iter()
                    .map(|&id| JsonActivityId::from_problem(id, problem))
                    .collect(),
            }),
            Relation::InDirectSequence(rel) => {
                JsonRelation::InDirectSequence(JsonInDirectSequenceRelation {
                    vehicle_id: rel
                        .vehicle_id
                        .map(|id| problem.vehicle(id).external_id().to_owned()),
                    ids: rel
                        .activity_ids
                        .iter()
                        .map(|&id| JsonActivityId::from_problem(id, problem))
                        .collect(),
                })
            }
        }
    }
}

impl JsonVehicleRoutingProblem {
    #[instrument(skip_all, level = "debug")]
    pub async fn build_problem(
        self,
        client: &TravelMatrixClient<impl MatricesCache>,
    ) -> Result<VehicleRoutingProblem, anyhow::Error> {
        let mut builder = VehicleRoutingProblemBuilder::default();

        if let Some(id) = self.id {
            builder.set_id(id);
        }

        let locations = self
            .locations
            .iter()
            .map(|location| {
                Location::from_lat_lon(location.coordinates[1], location.coordinates[0])
            })
            .collect::<Vec<_>>();

        let services: Vec<Service> = self
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

        let vehicles: Vec<Vehicle> = self
            .vehicles
            .into_iter()
            .map(|vehicle| {
                let mut builder = VehicleBuilder::default();

                builder.set_vehicle_id(vehicle.id);

                if let Some(position) = self
                    .vehicle_profiles
                    .iter()
                    .position(|profile| profile.id == vehicle.profile)
                {
                    builder.set_profile_id(position);
                }

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

                if let Some(maximum_activities) = vehicle.maximum_activities {
                    builder.set_maximum_activities(maximum_activities);
                }

                builder.build()
            })
            .collect();

        if let Some(json_relations) = self.relations {
            // TODO: rework this to pass external ID to the problem instance

            let external_to_internal_vehicle_id = |vehicle_id: Option<&str>| {
                if let Some(vehicle_id) = vehicle_id {
                    let position = vehicles
                        .iter()
                        .position(|vehicle| vehicle.external_id() == vehicle_id);

                    position.map(VehicleIdx::new)
                } else {
                    None
                }
            };

            let external_to_internal_service_id = |activity_id: JsonActivityId| -> ActivityId {
                let position = services
                    .iter()
                    .position(|service| service.external_id() == activity_id.external_id())
                    .expect("service not found");

                ActivityId::service(position)
            };

            let relations = json_relations
                .into_iter()
                .map(|relation| match relation {
                    JsonRelation::InSameRoute(rel) => {
                        let vehicle_id = external_to_internal_vehicle_id(rel.vehicle_id.as_deref());

                        Relation::InSameRoute(InSameRouteRelation {
                            vehicle_id,
                            activity_ids: rel
                                .ids
                                .into_iter()
                                .map(external_to_internal_service_id)
                                .collect(),
                        })
                    }
                    JsonRelation::NotInSameRoute(rel) => {
                        Relation::NotInSameRoute(NotInSameRouteRelation {
                            activity_ids: rel
                                .ids
                                .into_iter()
                                .map(external_to_internal_service_id)
                                .collect(),
                        })
                    }
                    JsonRelation::InSequence(rel) => {
                        let vehicle_id = external_to_internal_vehicle_id(rel.vehicle_id.as_deref());
                        Relation::InSequence(InSequenceRelation {
                            vehicle_id,
                            activity_ids: rel
                                .ids
                                .into_iter()
                                .map(external_to_internal_service_id)
                                .collect(),
                        })
                    }
                    JsonRelation::InDirectSequence(rel) => {
                        let vehicle_id = external_to_internal_vehicle_id(rel.vehicle_id.as_deref());
                        Relation::InDirectSequence(InDirectSequenceRelation {
                            vehicle_id,
                            activity_ids: rel
                                .ids
                                .into_iter()
                                .map(external_to_internal_service_id)
                                .collect(),
                        })
                    }
                })
                .collect();

            builder.set_relations(relations);
        }

        builder.set_services(services);
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
        Ok(builder.build()?)
    }
}
