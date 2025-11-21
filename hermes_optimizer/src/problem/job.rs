use crate::problem::{service::Service, shipment::Shipment};

pub enum Job {
    Service(Service),
    Shipment(Shipment),
}
