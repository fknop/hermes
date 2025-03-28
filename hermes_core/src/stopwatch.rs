use std::{
    fmt::Display,
    time::{Duration, Instant},
};

pub struct Stopwatch<'a> {
    start_time: Instant,
    name: &'a str,
}

impl<'a> Stopwatch<'a> {
    pub fn new(name: &'a str) -> Self {
        Self {
            start_time: Instant::now(),
            name,
        }
    }

    pub fn elapsed(&self) -> Duration {
        Instant::now() - self.start_time
    }

    pub fn report(&self) {
        println!("{}", self);
    }
}

impl Display for Stopwatch<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}]: {:?}", self.name, self.elapsed())
    }
}
