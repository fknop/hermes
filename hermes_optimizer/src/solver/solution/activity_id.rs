use serde::Serialize;

use crate::solver::solution::route::WorkingSolutionRoute;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ActivityId {
    Service(usize),
    // Shipment(usize)
}

impl From<ActivityId> for usize {
    fn from(job_id: ActivityId) -> Self {
        match job_id {
            ActivityId::Service(id) => id,
            // JobId::Shipment(id) => id,
        }
    }
}

pub struct JobIdIterator<'a> {
    route: &'a WorkingSolutionRoute,
    start: usize,
    end: usize,
}

impl<'a> JobIdIterator<'a> {
    pub fn new(route: &'a WorkingSolutionRoute, start: usize, end: usize) -> Self {
        JobIdIterator { route, start, end }
    }
}

impl Iterator for JobIdIterator<'_> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start < self.end {
            let activity = &self.route.activities[self.start];
            self.start += 1;
            Some(activity.job_id.into())
        } else {
            None
        }
    }
}
