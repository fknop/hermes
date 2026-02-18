use crate::problem::{job::ActivityId, vehicle::VehicleIdx};

pub struct InDirectSequenceRelation {
    pub vehicle_id: Option<VehicleIdx>,
    pub activity_ids: Vec<ActivityId>,
}

pub struct InSequenceRelation {
    pub vehicle_id: Option<VehicleIdx>,
    pub activity_ids: Vec<ActivityId>,
}

pub struct InSameRouteRelation {
    pub vehicle_id: Option<VehicleIdx>,
    pub activity_ids: Vec<ActivityId>,
}

pub struct NotInSameRouteRelation {
    pub activity_ids: Vec<ActivityId>,
}

pub enum Relation {
    InSameRoute(InSameRouteRelation),
    NotInSameRoute(NotInSameRouteRelation),
    InSequence(InSequenceRelation),
    InDirectSequence(InDirectSequenceRelation),
}

impl Relation {
    pub fn activity_ids(&self) -> &[ActivityId] {
        match self {
            Relation::InSameRoute(r) => &r.activity_ids,
            Relation::NotInSameRoute(r) => &r.activity_ids,
            Relation::InSequence(r) => &r.activity_ids,
            Relation::InDirectSequence(r) => &r.activity_ids,
        }
    }
}
