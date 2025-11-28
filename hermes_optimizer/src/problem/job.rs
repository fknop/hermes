use crate::problem::{capacity::Capacity, service::Service, shipment::Shipment};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum JobId {
    Service(usize),
    ShipmentPickup(usize),
    ShipmentDelivery(usize),
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
