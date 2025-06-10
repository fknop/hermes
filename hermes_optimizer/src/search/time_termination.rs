use std::time::{Duration, Instant};

use super::{search_listener::SearchListener, termination::Termination};

struct TimeTermination {
    start: Instant,
    max_duration: Duration,
}

impl TimeTermination {
    fn new(max_duration: Duration) -> Self {
        Self {
            start: Instant::now(),
            max_duration,
        }
    }
}

impl Termination for TimeTermination {
    fn should_terminate(&self) -> bool {
        self.start.elapsed() >= self.max_duration
    }
}

impl SearchListener for TimeTermination {
    fn search_start(&mut self) {
        self.start = Instant::now()
    }
}
