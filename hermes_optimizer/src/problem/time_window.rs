use jiff::Timestamp;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct TimeWindow {
    start: Option<Timestamp>,
    end: Option<Timestamp>,
}

impl TimeWindow {
    pub fn new(start: Option<Timestamp>, end: Option<Timestamp>) -> Self {
        TimeWindow { start, end }
    }

    pub fn start(&self) -> Option<Timestamp> {
        self.start
    }

    pub fn end(&self) -> Option<Timestamp> {
        self.end
    }
}

impl TimeWindow {
    pub fn is_satisfied(&self, arrival: Timestamp) -> bool {
        match self.end {
            Some(end) => arrival <= end,
            None => true,
        }
    }

    pub fn overtime(&self, arrival: Timestamp) -> i64 {
        match self.end {
            Some(end) => arrival.as_second() - end.as_second(),
            None => 0,
        }
    }
}

#[derive(Default)]
pub struct TimeWindowBuilder {
    start: Option<Timestamp>,
    end: Option<Timestamp>,
}

impl TimeWindowBuilder {
    pub fn with_start(mut self, start: Timestamp) -> Self {
        self.start = Some(start);
        self
    }

    pub fn with_iso_start(mut self, start: &str) -> Self {
        self.start = Some(start.parse().expect("Error parsing ISO"));
        self
    }

    pub fn with_end(mut self, end: Timestamp) -> Self {
        self.end = Some(end);
        self
    }

    pub fn with_iso_end(mut self, end: &str) -> Self {
        self.end = Some(end.parse().expect("Error parsing ISO"));
        self
    }

    pub fn build(self) -> TimeWindow {
        TimeWindow {
            start: self.start,
            end: self.end,
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_builder() {
        let start: Timestamp = "2025-06-10T08:00:00+02:00".parse().unwrap();
        let end: Timestamp = "2025-06-10T10:00:00+02:00".parse().unwrap();
        let time_window = TimeWindowBuilder::default()
            .with_start(start)
            .with_end(end)
            .build();

        assert_eq!(time_window.start().unwrap(), start);
        assert_eq!(time_window.end().unwrap(), end);
    }

    #[test]
    fn test_iso_builder() {
        let time_window = TimeWindowBuilder::default()
            .with_iso_start("2025-06-10T08:00:00+02:00")
            .with_iso_end("2025-06-10T10:00:00+02:00")
            .build();

        let start = "2025-06-10T08:00:00+02:00".parse().unwrap();
        let end = "2025-06-10T10:00:00+02:00".parse().unwrap();

        assert_eq!(time_window.start().unwrap(), start);
        assert_eq!(time_window.end().unwrap(), end);
    }
}
