use fxhash::FxHashSet;
use jiff::{SignedDuration, Timestamp};
use serde::{Deserialize, Serialize};

use crate::{
    define_index_newtype,
    problem::{job::Job, skill::Skill, vehicle_profile::VehicleProfileIdx},
};

use super::{capacity::Capacity, location::LocationIdx};

define_index_newtype!(VehicleIdx, Vehicle);

#[derive(Serialize, Debug, Clone)]
pub struct Vehicle {
    external_id: String,
    vehicle_profile_id: VehicleProfileIdx,
    shift: Option<VehicleShift>,
    capacity: Capacity,
    depot_location_id: Option<LocationIdx>,
    depot_duration: Option<SignedDuration>,
    end_depot_duration: Option<SignedDuration>,
    should_return_to_depot: bool,
    maximum_activities: Option<usize>,
    skills: FxHashSet<Skill>,
}

impl Vehicle {
    pub fn external_id(&self) -> &str {
        &self.external_id
    }

    pub fn profile_id(&self) -> VehicleProfileIdx {
        self.vehicle_profile_id
    }

    pub fn shift(&self) -> Option<&VehicleShift> {
        self.shift.as_ref()
    }

    pub fn capacity(&self) -> &Capacity {
        &self.capacity
    }

    pub fn skills(&self) -> &FxHashSet<Skill> {
        &self.skills
    }

    pub fn depot_location_id(&self) -> Option<LocationIdx> {
        self.depot_location_id
    }

    pub fn earliest_start_time(&self) -> Option<Timestamp> {
        self.shift.as_ref().and_then(|shift| shift.earliest_start)
    }

    pub fn latest_start_time(&self) -> Option<Timestamp> {
        self.shift.as_ref().and_then(|shift| shift.latest_start)
    }

    pub fn maximum_transport_duration(&self) -> Option<SignedDuration> {
        self.shift
            .as_ref()
            .and_then(|shift| shift.maximum_transport_duration)
    }

    pub fn maximum_working_duration(&self) -> Option<SignedDuration> {
        self.shift
            .as_ref()
            .and_then(|shift| shift.maximum_working_duration)
    }

    pub fn latest_end_time(&self) -> Option<Timestamp> {
        self.shift.as_ref().and_then(|shift| shift.latest_end)
    }

    pub fn should_return_to_depot(&self) -> bool {
        self.should_return_to_depot
    }

    pub fn set_should_return_to_depot(&mut self, should_return_to_depot: bool) {
        self.should_return_to_depot = should_return_to_depot;
    }

    pub fn maximum_activities(&self) -> Option<usize> {
        self.maximum_activities
    }

    pub fn depot_duration(&self) -> SignedDuration {
        self.depot_duration.unwrap_or(SignedDuration::ZERO)
    }

    pub fn set_shift(&mut self, shift: VehicleShift) {
        self.shift = Some(shift);
    }

    pub fn set_depot_location(&mut self, depot_location_id: LocationIdx) {
        self.depot_location_id = Some(depot_location_id);
    }

    pub fn end_depot_duration(&self) -> SignedDuration {
        self.end_depot_duration.unwrap_or(SignedDuration::ZERO)
    }

    pub fn is_compatible_with(&self, job: &Job) -> bool {
        self.skills.is_subset(job.skills())
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Default)]
pub struct VehicleShift {
    pub(crate) earliest_start: Option<Timestamp>,
    pub(crate) latest_start: Option<Timestamp>,
    pub(crate) latest_end: Option<Timestamp>,
    pub(crate) maximum_transport_duration: Option<SignedDuration>,
    pub(crate) maximum_working_duration: Option<SignedDuration>,
}

impl VehicleShift {
    pub fn maximum_transport_duration(&self) -> Option<SignedDuration> {
        self.maximum_transport_duration
    }

    pub fn maximum_working_duration(&self) -> Option<SignedDuration> {
        self.maximum_working_duration
    }

    pub fn earliest_start(&self) -> Option<Timestamp> {
        self.earliest_start
    }

    pub fn latest_start(&self) -> Option<Timestamp> {
        self.latest_start
    }

    pub fn latest_end(&self) -> Option<Timestamp> {
        self.latest_end
    }
}

#[derive(Default)]
pub struct VehicleShiftBuilder {
    earliest_start: Option<Timestamp>,
    latest_start: Option<Timestamp>,
    latest_end: Option<Timestamp>,
    maximum_transport_duration: Option<SignedDuration>,
    maximum_working_duration: Option<SignedDuration>,
}

impl VehicleShiftBuilder {
    pub fn set_earliest_start(&mut self, earliest_start: Timestamp) -> &mut VehicleShiftBuilder {
        self.earliest_start = Some(earliest_start);
        self
    }

    pub fn set_latest_start(&mut self, latest_start: Timestamp) -> &mut VehicleShiftBuilder {
        self.latest_start = Some(latest_start);
        self
    }

    pub fn set_latest_end(&mut self, latest_end: Timestamp) -> &mut VehicleShiftBuilder {
        self.latest_end = Some(latest_end);
        self
    }

    pub fn set_maximum_transport_duration(
        &mut self,
        maximum_transport_duration: SignedDuration,
    ) -> &mut VehicleShiftBuilder {
        self.maximum_transport_duration = Some(maximum_transport_duration);
        self
    }

    pub fn set_maximum_working_duration(
        &mut self,
        maximum_working_duration: SignedDuration,
    ) -> &mut VehicleShiftBuilder {
        self.maximum_working_duration = Some(maximum_working_duration);
        self
    }

    pub fn build(self) -> VehicleShift {
        VehicleShift {
            earliest_start: self.earliest_start,
            latest_start: self.latest_start,
            latest_end: self.latest_end,
            maximum_transport_duration: self.maximum_transport_duration,
            maximum_working_duration: self.maximum_working_duration,
        }
    }
}

#[derive(Default)]
pub struct VehicleBuilder {
    external_id: Option<String>,
    vehicle_profile_id: Option<usize>,
    shift: Option<VehicleShift>,
    capacity: Option<Capacity>,
    depot_location_id: Option<usize>,
    should_return_to_depot: Option<bool>,
    depot_duration: Option<SignedDuration>,
    end_depot_duration: Option<SignedDuration>,
    skills: Option<Vec<Skill>>,
    maximum_activities: Option<usize>,
}

impl VehicleBuilder {
    pub fn set_profile_id(&mut self, vehicle_profile_id: usize) -> &mut VehicleBuilder {
        self.vehicle_profile_id = Some(vehicle_profile_id);
        self
    }

    pub fn set_vehicle_id(&mut self, external_id: String) -> &mut VehicleBuilder {
        self.external_id = Some(external_id);
        self
    }

    pub fn set_maximum_activities(&mut self, maximum_activities: usize) -> &mut VehicleBuilder {
        self.maximum_activities = Some(maximum_activities);
        self
    }

    pub fn set_vehicle_shift(&mut self, shift: VehicleShift) -> &mut VehicleBuilder {
        self.shift = Some(shift);
        self
    }

    pub fn set_capacity(&mut self, capacity: Capacity) -> &mut VehicleBuilder {
        self.capacity = Some(capacity);
        self
    }

    pub fn set_depot_location_id(&mut self, depot_location_id: usize) -> &mut VehicleBuilder {
        self.depot_location_id = Some(depot_location_id);
        self
    }

    pub fn set_return(&mut self, should_return_to_depot: bool) -> &mut VehicleBuilder {
        // This method is not used in the current implementation but can be added for future use
        self.should_return_to_depot = Some(should_return_to_depot);
        self
    }

    pub fn set_depot_duration(&mut self, duration: SignedDuration) -> &mut VehicleBuilder {
        self.depot_duration = Some(duration);
        self
    }

    pub fn set_end_depot_duration(&mut self, duration: SignedDuration) -> &mut VehicleBuilder {
        self.end_depot_duration = Some(duration);
        self
    }

    pub fn set_skills(&mut self, skills: Vec<String>) -> &mut VehicleBuilder {
        self.skills = Some(skills.into_iter().map(Skill::new).collect());
        self
    }

    pub fn build(self) -> Vehicle {
        Vehicle {
            external_id: self.external_id.expect("External ID is required"),
            vehicle_profile_id: self
                .vehicle_profile_id
                .expect("Vehicle profile ID is required")
                .into(),
            shift: self.shift,
            capacity: self.capacity.unwrap_or(Capacity::EMPTY),
            depot_location_id: self.depot_location_id.map(|id| id.into()),
            should_return_to_depot: self.should_return_to_depot.unwrap_or(false),
            depot_duration: self.depot_duration,
            end_depot_duration: self.end_depot_duration,
            skills: FxHashSet::from_iter(self.skills.unwrap_or_default()),
            maximum_activities: self.maximum_activities,
        }
    }
}
