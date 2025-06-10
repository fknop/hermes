use std::time::Instant;

pub struct TimeWindow {
    start: Option<i64>,
    end: Option<i64>,
}

impl TimeWindow {
    pub fn start(&self) -> Option<i64> {
        self.start
    }

    pub fn end(&self) -> Option<i64> {
        self.end
    }
}

#[derive(Default)]
pub struct TimeWindowBuilder {
    start: Option<i64>,
    end: Option<i64>,
}

impl TimeWindowBuilder {
    pub fn with_start(mut self, start: i64) -> Self {
        self.start = Some(start);
        self
    }

    pub fn with_iso_start(mut self, start: &str) -> Self {
        self.start = Some(
            chrono::DateTime::parse_from_rfc3339(start)
                .unwrap()
                .timestamp(),
        );
        self
    }

    pub fn with_end(mut self, end: i64) -> Self {
        self.end = Some(end);
        self
    }

    pub fn with_iso_end(mut self, end: &str) -> Self {
        self.end = Some(
            chrono::DateTime::parse_from_rfc3339(end)
                .unwrap()
                .timestamp(),
        );
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
        let start = chrono::DateTime::parse_from_rfc3339("2025-06-10T08:00:00+02:00").unwrap();
        let end = chrono::DateTime::parse_from_rfc3339("2025-06-10T10:00:00+02:00").unwrap();
        let time_window = TimeWindowBuilder::default()
            .with_start(start.timestamp())
            .with_end(end.timestamp())
            .build();

        assert_eq!(time_window.start().unwrap(), start.timestamp());
        assert_eq!(time_window.end().unwrap(), end.timestamp());
    }

    #[test]
    fn test_iso_builder() {
        let time_window = TimeWindowBuilder::default()
            .with_iso_start("2025-06-10T08:00:00+02:00")
            .with_iso_end("2025-06-10T10:00:00+02:00")
            .build();

        let start = chrono::DateTime::parse_from_rfc3339("2025-06-10T08:00:00+02:00").unwrap();
        let end = chrono::DateTime::parse_from_rfc3339("2025-06-10T10:00:00+02:00").unwrap();

        assert_eq!(time_window.start().unwrap(), start.timestamp());
        assert_eq!(time_window.end().unwrap(), end.timestamp());
    }
}
