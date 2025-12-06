use jiff::SignedDuration;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

use super::{capacity::Capacity, location::LocationId, time_window::TimeWindow};

pub type ServiceId = usize;

#[derive(Deserialize, Serialize, Debug, Copy, Clone, Default, PartialEq, Eq)]
pub enum ServiceType {
    Pickup,
    #[default]
    Delivery,
}

type TimeWindows = SmallVec<[TimeWindow; 1]>;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Service {
    external_id: String,
    location_id: LocationId,
    time_windows: TimeWindows,
    demand: Capacity,
    service_duration: SignedDuration,

    #[serde(default = "ServiceType::default")]
    service_type: ServiceType,
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

    pub fn duration(&self) -> SignedDuration {
        self.service_duration
    }

    pub fn service_type(&self) -> ServiceType {
        self.service_type
    }

    pub fn time_windows(&self) -> &[TimeWindow] {
        &self.time_windows
    }

    pub fn has_time_windows(&self) -> bool {
        self.time_windows.iter().any(|tw| !tw.is_empty())
    }

    pub fn time_windows_satisfied(&self, arrival_time: jiff::Timestamp) -> bool {
        self.time_windows
            .iter()
            .any(|tw| tw.is_satisfied(arrival_time))
    }
}

#[derive(Default)]
pub struct ServiceBuilder {
    external_id: Option<String>,
    location_id: Option<LocationId>,
    time_windows: Option<Vec<TimeWindow>>,
    demand: Option<Capacity>,
    service_duration: Option<SignedDuration>,
    service_type: Option<ServiceType>,
}

impl ServiceBuilder {
    pub fn set_external_id(&mut self, external_id: String) -> &mut ServiceBuilder {
        self.external_id = Some(external_id);
        self
    }

    pub fn set_service_type(&mut self, service_type: ServiceType) -> &mut ServiceBuilder {
        self.service_type = Some(service_type);
        self
    }

    pub fn set_location_id(&mut self, location_id: LocationId) -> &mut ServiceBuilder {
        self.location_id = Some(location_id);
        self
    }

    pub fn set_time_window(&mut self, time_window: TimeWindow) -> &mut ServiceBuilder {
        if let Some(time_windows) = &mut self.time_windows {
            time_windows.push(time_window)
        } else {
            self.time_windows = Some(vec![time_window]);
        }

        self
    }

    pub fn set_time_windows(&mut self, time_window: Vec<TimeWindow>) -> &mut ServiceBuilder {
        self.time_windows = Some(time_window);
        self
    }

    pub fn set_demand(&mut self, demand: Capacity) -> &mut ServiceBuilder {
        self.demand = Some(demand);
        self
    }

    pub fn set_service_duration(&mut self, service_time: SignedDuration) -> &mut ServiceBuilder {
        self.service_duration = Some(service_time);
        self
    }

    pub fn build(self) -> Service {
        Service {
            external_id: self.external_id.expect("Expected service id"),
            location_id: self.location_id.expect("Expected location id"),
            demand: self.demand.unwrap_or_default(),
            service_duration: self.service_duration.unwrap_or(SignedDuration::ZERO),
            time_windows: SmallVec::from_vec(self.time_windows.unwrap_or_default()),
            service_type: self.service_type.unwrap_or(ServiceType::Delivery),
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

        builder.set_demand(Capacity::from_vec(vec![1.0, 2.0, 3.0]));

        let service = builder.build();

        assert_eq!(service.external_id, String::from("service_id"));
        assert_eq!(service.location_id, 1);
        assert_eq!(service.service_duration, SignedDuration::ZERO);
        assert_eq!(service.demand, Capacity::from_vec(vec![1.0, 2.0, 3.0]));
    }
}
