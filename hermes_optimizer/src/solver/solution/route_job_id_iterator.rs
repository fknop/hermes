use serde::Serialize;

use crate::solver::solution::route::WorkingSolutionRoute;

pub struct RouteJobIdIterator<'a> {
    route: &'a WorkingSolutionRoute,
    start: usize,
    end: usize,
}

impl<'a> RouteJobIdIterator<'a> {
    pub fn new(route: &'a WorkingSolutionRoute, start: usize, end: usize) -> Self {
        RouteJobIdIterator { route, start, end }
    }
}

impl Iterator for RouteJobIdIterator<'_> {
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
