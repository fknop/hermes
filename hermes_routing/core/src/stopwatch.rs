use std::{
    fmt::Display,
    time::{Duration, Instant},
};

use tracing::debug;

pub struct Stopwatch {
    start: Instant,
    name: String,
    elapsed_duration: Duration,
}

impl Stopwatch {
    pub fn new(name: String) -> Self {
        Self {
            start: Instant::now(),
            elapsed_duration: Duration::ZERO,
            name,
        }
    }

    pub fn start(&mut self) {
        self.start = Instant::now();
    }

    pub fn stop(&mut self) {
        self.elapsed_duration += self.start.elapsed();
    }

    pub fn total_duration(&self) -> Duration {
        self.elapsed_duration
    }

    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    pub fn report(&self) {
        debug!("{}", self);
    }
}

impl Display for Stopwatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}]: {:?}", self.name, self.elapsed_duration)
    }
}

#[macro_export]
macro_rules! timer {
    ($sw:ident, $block:block) => {
        $sw.start();
        $block;
        $sw.stop();
    };
}
