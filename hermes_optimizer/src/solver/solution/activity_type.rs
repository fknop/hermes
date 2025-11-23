use serde::Serialize;

use crate::solver::solution::route::WorkingSolutionRoute;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ActivityType {
    Service(usize),
    ShipmentPickup(usize),
    ShipmentDelivery(usize),
}

impl From<ActivityType> for usize {
    fn from(activity_type: ActivityType) -> Self {
        match activity_type {
            ActivityType::Service(id) => id,
            ActivityType::ShipmentPickup(id) => id,
            ActivityType::ShipmentDelivery(id) => id,
        }
    }
}

pub struct ActivityTypeIterator<'a> {
    route: &'a WorkingSolutionRoute,
    start: usize,
    end: usize,
}

impl<'a> ActivityTypeIterator<'a> {
    pub fn new(route: &'a WorkingSolutionRoute, start: usize, end: usize) -> Self {
        ActivityTypeIterator { route, start, end }
    }
}

impl Iterator for ActivityTypeIterator<'_> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start < self.end {
            let activity = &self.route.activities[self.start];
            self.start += 1;
            Some(activity.activity_type.into())
        } else {
            None
        }
    }
}
