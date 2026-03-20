use schemars::JsonSchema;

use crate::problem::job::{ActivityId, Job};

#[derive(JsonSchema)]
pub enum ExternalActivityId {
    ShipmentPickup(String),
    ShipmentDelivery(String),
    Service(String),
}

#[derive(JsonSchema, serde::Serialize, serde::Deserialize)]
pub struct ExternalJobId(pub String);

impl ExternalJobId {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn to_string(&self) -> String {
        self.0.to_owned()
    }
}

impl ExternalActivityId {
    pub fn external_id(&self) -> &str {
        match self {
            ExternalActivityId::ShipmentPickup(id) => id,
            ExternalActivityId::ShipmentDelivery(id) => id,
            ExternalActivityId::Service(id) => id,
        }
    }

    pub fn activity_id(&self, jobs: &[Job]) -> Option<ActivityId> {
        jobs.iter()
            .position(|job| match job {
                Job::Service(service) => {
                    service.external_id() == self.external_id()
                        && matches!(self, ExternalActivityId::Service(_))
                }
                Job::Shipment(shipment) => {
                    shipment.external_id() == self.external_id()
                        && matches!(
                            self,
                            ExternalActivityId::ShipmentPickup(_)
                                | ExternalActivityId::ShipmentDelivery(_)
                        )
                }
            })
            .map(|idx| match self {
                ExternalActivityId::ShipmentPickup(_) => ActivityId::shipment_pickup(idx),
                ExternalActivityId::ShipmentDelivery(_) => ActivityId::shipment_delivery(idx),
                ExternalActivityId::Service(_) => ActivityId::service(idx),
            })
    }
}

impl std::fmt::Display for ExternalActivityId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExternalActivityId::ShipmentPickup(id) => write!(f, "shipment_{}", id),
            ExternalActivityId::ShipmentDelivery(id) => write!(f, "shipment_delivery_{}", id),
            ExternalActivityId::Service(id) => write!(f, "service_{}", id),
        }
    }
}

impl serde::Serialize for ExternalActivityId {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> serde::Deserialize<'de> for ExternalActivityId {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        if let Some(id) = s.strip_prefix("pickup_") {
            Ok(ExternalActivityId::ShipmentPickup(id.to_owned()))
        } else if let Some(id) = s.strip_prefix("delivery_") {
            Ok(ExternalActivityId::ShipmentDelivery(id.to_owned()))
        } else if let Some(id) = s.strip_prefix("service_") {
            Ok(ExternalActivityId::Service(id.to_owned()))
        } else {
            Ok(ExternalActivityId::Service(s))
        }
    }
}
