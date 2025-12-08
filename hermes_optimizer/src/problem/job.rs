use std::fmt::Display;

use jiff::SignedDuration;

use crate::problem::{
    capacity::Capacity, service::Service, shipment::Shipment, time_window::TimeWindow,
};

#[derive(Hash, Debug, Clone, Copy, Eq, PartialEq)]
pub enum JobId {
    Service(usize),
    ShipmentPickup(usize),
    ShipmentDelivery(usize),
}

impl JobId {
    pub fn is_shipment(&self) -> bool {
        matches!(self, JobId::ShipmentPickup(_) | JobId::ShipmentDelivery(_))
    }

    pub fn index(&self) -> usize {
        match self {
            JobId::Service(id) => *id,
            JobId::ShipmentPickup(id) => *id,
            JobId::ShipmentDelivery(id) => *id,
        }
    }
}

impl Display for JobId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JobId::Service(id) => write!(f, "Service({})", id),
            JobId::ShipmentPickup(id) => write!(f, "ShipmentPickup({})", id),
            JobId::ShipmentDelivery(id) => write!(f, "ShipmentDelivery({})", id),
        }
    }
}

impl From<JobId> for usize {
    fn from(job_id: JobId) -> Self {
        match job_id {
            JobId::Service(id) => id,
            JobId::ShipmentPickup(id) => id,
            JobId::ShipmentDelivery(id) => id,
        }
    }
}

pub enum JobTask<'a> {
    Service(&'a Service),
    ShipmentPickup(&'a Shipment),
    ShipmentDelivery(&'a Shipment),
}

impl JobTask<'_> {
    pub fn time_windows(&self) -> &[TimeWindow] {
        match self {
            JobTask::Service(service) => service.time_windows(),
            JobTask::ShipmentPickup(shipment) => shipment.pickup().time_windows(),
            JobTask::ShipmentDelivery(shipment) => shipment.delivery().time_windows(),
        }
    }

    pub fn location_id(&self) -> usize {
        match self {
            JobTask::Service(service) => service.location_id(),
            JobTask::ShipmentPickup(shipment) => shipment.pickup().location_id(),
            JobTask::ShipmentDelivery(shipment) => shipment.delivery().location_id(),
        }
    }

    pub fn duration(&self) -> SignedDuration {
        match self {
            JobTask::Service(service) => service.duration(),
            JobTask::ShipmentPickup(shipment) => shipment.pickup().duration(),
            JobTask::ShipmentDelivery(shipment) => shipment.delivery().duration(),
        }
    }

    pub fn has_time_windows(&self) -> bool {
        match self {
            JobTask::Service(service) => service.has_time_windows(),
            JobTask::ShipmentPickup(shipment) => shipment.pickup().has_time_windows(),
            JobTask::ShipmentDelivery(shipment) => shipment.delivery().has_time_windows(),
        }
    }

    pub fn time_windows_satisfied(&self, arrival_time: jiff::Timestamp) -> bool {
        self.time_windows()
            .iter()
            .any(|tw| tw.is_satisfied(arrival_time))
    }
}

pub enum Job {
    Service(Service),
    Shipment(Shipment),
}

impl Job {
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
