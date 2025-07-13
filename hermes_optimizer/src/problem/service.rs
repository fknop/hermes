use jiff::SignedDuration;
use serde::Deserialize;

use super::{capacity::Capacity, location::LocationId, time_window::TimeWindow};

pub type ServiceId = usize;

#[derive(Deserialize)]
pub struct Service {
    external_id: String,
    location_id: LocationId,
    time_window: Option<TimeWindow>,
    demand: Capacity,
    service_duration: SignedDuration,
}

impl Service {
    pub fn external_id(&self) -> &str {
        &self.external_id
    }

    pub fn location_id(&self) -> LocationId {
        self.location_id
    }

    pub fn demand(&self) -> &Capacity {
        &self.demand
    }

    pub fn service_duration(&self) -> SignedDuration {
        self.service_duration
    }

    pub fn time_window(&self) -> Option<&TimeWindow> {
        self.time_window.as_ref()
    }
}

#[derive(Default)]
pub struct ServiceBuilder {
    external_id: Option<String>,
    location_id: Option<LocationId>,
    time_window: Option<TimeWindow>,
    demand: Option<Capacity>,
    service_time: Option<SignedDuration>,
}

impl ServiceBuilder {
    pub fn set_external_id(&mut self, external_id: String) -> &mut ServiceBuilder {
        self.external_id = Some(external_id);
        self
    }

    pub fn set_location_id(&mut self, location_id: LocationId) -> &mut ServiceBuilder {
        self.location_id = Some(location_id);
        self
    }

    pub fn set_time_window(&mut self, time_window: TimeWindow) -> &mut ServiceBuilder {
        self.time_window = Some(time_window);
        self
    }

    pub fn set_demand(&mut self, demand: Capacity) -> &mut ServiceBuilder {
        self.demand = Some(demand);
        self
    }

    pub fn set_service_duration(&mut self, service_time: SignedDuration) -> &mut ServiceBuilder {
        self.service_time = Some(service_time);
        self
    }

    pub fn build(self) -> Service {
        Service {
            external_id: self.external_id.expect("Expected service id"),
            location_id: self.location_id.expect("Expected location id"),
            demand: self.demand.unwrap_or_default(),
            service_duration: self.service_time.unwrap_or(SignedDuration::ZERO),
            time_window: self.time_window,
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::problem::time_window::TimeWindowBuilder;

    use super::*;

    #[test]
    fn test_builder() {
        let mut builder = ServiceBuilder::default();
        builder
            .set_external_id(String::from("service_id"))
            .set_location_id(1)
            .set_time_window(
                TimeWindowBuilder::default()
                    .with_iso_start("2025-06-10T08:00:00+02:00")
                    .build(),
            );

        builder.set_demand(Capacity::new(vec![1.0, 2.0, 3.0]));

        let service = builder.build();

        assert_eq!(service.external_id, String::from("service_id"));
        assert_eq!(service.location_id, 1);
        assert_eq!(service.service_duration, SignedDuration::ZERO);
        assert_eq!(service.demand, Capacity::new(vec![1.0, 2.0, 3.0]));
    }
}
