use std::fmt::Display;

use fxhash::FxHashSet;
use jiff::SignedDuration;

use crate::{
    define_index_newtype,
    problem::{
        capacity::Capacity, location::LocationIdx, service::Service, shipment::Shipment,
        skill::Skill, time_window::TimeWindows,
    },
};

define_index_newtype!(JobIdx, Job);

#[derive(Hash, Debug, Clone, Copy, Eq, PartialEq)]
pub enum ActivityId {
    Service(JobIdx),
    ShipmentPickup(JobIdx),
    ShipmentDelivery(JobIdx),
}

impl ActivityId {
    pub fn service(idx: impl Into<JobIdx>) -> Self {
        ActivityId::Service(idx.into())
    }

    pub fn shipment_pickup(idx: impl Into<JobIdx>) -> Self {
        ActivityId::ShipmentPickup(idx.into())
    }

    pub fn shipment_delivery(idx: impl Into<JobIdx>) -> Self {
        ActivityId::ShipmentDelivery(idx.into())
    }

    pub fn is_shipment(&self) -> bool {
        matches!(
            self,
            ActivityId::ShipmentPickup(_) | ActivityId::ShipmentDelivery(_)
        )
    }

    pub fn job_id(&self) -> JobIdx {
        match self {
            ActivityId::Service(id) => *id,
            ActivityId::ShipmentPickup(id) => *id,
            ActivityId::ShipmentDelivery(id) => *id,
        }
    }
}

impl Display for ActivityId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ActivityId::Service(id) => write!(f, "Service({})", id),
            ActivityId::ShipmentPickup(id) => write!(f, "ShipmentPickup({})", id),
            ActivityId::ShipmentDelivery(id) => write!(f, "ShipmentDelivery({})", id),
        }
    }
}

impl From<ActivityId> for JobIdx {
    fn from(activity_id: ActivityId) -> Self {
        activity_id.job_id()
    }
}

pub enum JobActivity<'a> {
    Service(&'a Service),
    ShipmentPickup(&'a Shipment),
    ShipmentDelivery(&'a Shipment),
}

impl JobActivity<'_> {
    pub fn time_windows(&self) -> &TimeWindows {
        match self {
            JobActivity::Service(service) => service.time_windows(),
            JobActivity::ShipmentPickup(shipment) => shipment.pickup().time_windows(),
            JobActivity::ShipmentDelivery(shipment) => shipment.delivery().time_windows(),
        }
    }

    pub fn location_id(&self) -> LocationIdx {
        match self {
            JobActivity::Service(service) => service.location_id(),
            JobActivity::ShipmentPickup(shipment) => shipment.pickup().location_id(),
            JobActivity::ShipmentDelivery(shipment) => shipment.delivery().location_id(),
        }
    }

    pub fn duration(&self) -> SignedDuration {
        match self {
            JobActivity::Service(service) => service.duration(),
            JobActivity::ShipmentPickup(shipment) => shipment.pickup().duration(),
            JobActivity::ShipmentDelivery(shipment) => shipment.delivery().duration(),
        }
    }

    pub fn has_time_windows(&self) -> bool {
        match self {
            JobActivity::Service(service) => service.has_time_windows(),
            JobActivity::ShipmentPickup(shipment) => shipment.pickup().has_time_windows(),
            JobActivity::ShipmentDelivery(shipment) => shipment.delivery().has_time_windows(),
        }
    }
}

#[derive(Debug)]
pub enum Job {
    Service(Service),
    Shipment(Shipment),
}

impl Job {
    pub fn skills(&self) -> &FxHashSet<Skill> {
        match self {
            Job::Service(service) => service.skills(),
            Job::Shipment(shipment) => shipment.skills(),
        }
    }

    pub fn external_id(&self) -> &str {
        match self {
            Job::Service(service) => service.external_id(),
            Job::Shipment(shipment) => shipment.external_id(),
        }
    }

    pub fn demand(&self) -> &Capacity {
        match self {
            Job::Service(service) => service.demand(),
            Job::Shipment(shipment) => shipment.demand(),
        }
    }

    pub fn has_time_windows(&self) -> bool {
        match self {
            Job::Service(service) => service.has_time_windows(),
            Job::Shipment(shipment) => shipment.has_time_windows(),
        }
    }
}
