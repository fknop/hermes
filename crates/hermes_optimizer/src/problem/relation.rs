use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::problem::{
    external_id::{ExternalActivityId, ExternalJobId},
    job::{ActivityId, Job, JobIdx},
    vehicle::{Vehicle, VehicleIdx},
};

#[derive(Debug)]
pub struct InDirectSequenceRelation {
    pub vehicle_id: Option<VehicleIdx>,
    pub activity_ids: Vec<ActivityId>,
}

#[derive(Debug)]
pub struct InSequenceRelation {
    pub vehicle_id: Option<VehicleIdx>,
    pub activity_ids: Vec<ActivityId>,
}

#[derive(Debug)]
pub struct InSameRouteRelation {
    pub vehicle_id: Option<VehicleIdx>,
    pub job_ids: Vec<JobIdx>,
}

#[derive(Debug)]
pub struct NotInSameRouteRelation {
    pub job_ids: Vec<JobIdx>,
}

#[derive(Debug)]
pub enum Relation {
    InSameRoute(InSameRouteRelation),
    NotInSameRoute(NotInSameRouteRelation),
    InSequence(InSequenceRelation),
    InDirectSequence(InDirectSequenceRelation),
}

#[derive(JsonSchema, Serialize, Deserialize)]
pub struct ExternalInDirectSequenceRelation {
    pub vehicle_id: Option<String>,
    pub ids: Vec<ExternalActivityId>,
}

#[derive(JsonSchema, Serialize, Deserialize)]
pub struct ExternalInSequenceRelation {
    pub vehicle_id: Option<String>,
    pub ids: Vec<ExternalActivityId>,
}

#[derive(JsonSchema, Serialize, Deserialize)]
pub struct ExternalInSameRouteRelation {
    pub vehicle_id: Option<String>,
    pub ids: Vec<ExternalJobId>,
}

#[derive(JsonSchema, Serialize, Deserialize)]
pub struct ExternalNotInSameRouteRelation {
    pub ids: Vec<ExternalJobId>,
}

#[derive(JsonSchema, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ExternalRelation {
    InSameRoute(ExternalInSameRouteRelation),
    NotInSameRoute(ExternalNotInSameRouteRelation),
    InSequence(ExternalInDirectSequenceRelation),
    InDirectSequence(ExternalInDirectSequenceRelation),
}

impl ExternalRelation {
    pub fn try_into_relation(
        self,
        vehicles: &[Vehicle],
        jobs: &[Job],
    ) -> Result<Relation, MalformedRelationError> {
        let relation = match self {
            ExternalRelation::InSameRoute(r) => Relation::InSameRoute(InSameRouteRelation {
                vehicle_id: r
                    .vehicle_id
                    .map(|id| {
                        Self::external_to_internal_vehicle_id(vehicles, &id)
                            .ok_or(MalformedRelationError::UnknownVehicleId(id.to_string()))
                    })
                    .transpose()?,
                job_ids: r
                    .ids
                    .into_iter()
                    .map(|id| {
                        Self::external_to_internal_job_id(jobs, &id)
                            .ok_or(MalformedRelationError::UnknownJobId(id.to_string()))
                    })
                    .collect::<Result<Vec<JobIdx>, _>>()?,
            }),
            ExternalRelation::NotInSameRoute(r) => {
                Relation::NotInSameRoute(NotInSameRouteRelation {
                    job_ids: r
                        .ids
                        .into_iter()
                        .map(|id| {
                            Self::external_to_internal_job_id(jobs, &id)
                                .ok_or(MalformedRelationError::UnknownJobId(id.to_string()))
                        })
                        .collect::<Result<Vec<JobIdx>, _>>()?,
                })
            }
            ExternalRelation::InSequence(r) => Relation::InSequence(InSequenceRelation {
                vehicle_id: r
                    .vehicle_id
                    .map(|id| {
                        Self::external_to_internal_vehicle_id(vehicles, &id)
                            .ok_or(MalformedRelationError::UnknownVehicleId(id.to_string()))
                    })
                    .transpose()?,
                activity_ids: r
                    .ids
                    .into_iter()
                    .map(|id| {
                        id.activity_id(jobs)
                            .ok_or(MalformedRelationError::UnknownActivityId(id.to_string()))
                    })
                    .collect::<Result<Vec<ActivityId>, _>>()?,
            }),
            ExternalRelation::InDirectSequence(r) => {
                Relation::InDirectSequence(InDirectSequenceRelation {
                    vehicle_id: r
                        .vehicle_id
                        .map(|id| {
                            Self::external_to_internal_vehicle_id(vehicles, &id)
                                .ok_or(MalformedRelationError::UnknownVehicleId(id.to_string()))
                        })
                        .transpose()?,
                    activity_ids: r
                        .ids
                        .into_iter()
                        .map(|id| {
                            id.activity_id(jobs)
                                .ok_or(MalformedRelationError::UnknownActivityId(id.to_string()))
                        })
                        .collect::<Result<Vec<ActivityId>, _>>()?,
                })
            }
        };

        Ok(relation)
    }

    fn external_to_internal_job_id(jobs: &[Job], id: &ExternalJobId) -> Option<JobIdx> {
        let position = jobs.iter().position(|job| job.external_id() == id.as_str());
        position.map(JobIdx::new)
    }

    fn external_to_internal_vehicle_id(vehicles: &[Vehicle], id: &str) -> Option<VehicleIdx> {
        let position = vehicles
            .iter()
            .position(|vehicle| vehicle.external_id() == id);

        position.map(VehicleIdx::new)
    }
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum MalformedRelationError {
    #[error("Relations contain cycle")]
    Cycle,

    #[error("Conflicting relations, both in same routes and not in same routes")]
    Conflict,

    #[error("Unknown vehicle ID {0}")]
    UnknownVehicleId(String),

    #[error("Unknown activity ID {0}")]
    UnknownActivityId(String),

    #[error("Unknown job ID {0}")]
    UnknownJobId(String),
}
