use std::cmp;

use jiff::{SignedDuration, Timestamp};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

#[derive(Deserialize, JsonSchema, Debug, Serialize, Clone)]
pub struct TimeWindow {
    start: Option<Timestamp>,
    end: Option<Timestamp>,
}

impl TimeWindow {
    pub fn new(start: Option<Timestamp>, end: Option<Timestamp>) -> Self {
        TimeWindow { start, end }
    }

    pub fn from_iso(start: Option<&str>, end: Option<&str>) -> Self {
        let start_ts = start.map(|s| s.parse().expect("Error parsing ISO"));
        let end_ts = end.map(|e| e.parse().expect("Error parsing ISO"));
        TimeWindow {
            start: start_ts,
            end: end_ts,
        }
    }

    pub fn start(&self) -> Option<Timestamp> {
        self.start
    }

    pub fn end(&self) -> Option<Timestamp> {
        self.end
    }

    pub fn is_empty(&self) -> bool {
        self.start.is_none() && self.end.is_none()
    }
}

impl TimeWindow {
    pub fn is_satisfied(&self, arrival: Timestamp) -> bool {
        match self.end {
            Some(end) => arrival <= end,
            None => true,
        }
    }

    pub fn overtime(&self, arrival: Timestamp) -> SignedDuration {
        match self.end {
            Some(end) => {
                let secs = arrival.duration_since(end);

                if secs > SignedDuration::ZERO {
                    secs
                } else {
                    SignedDuration::ZERO
                }
            }
            None => SignedDuration::ZERO,
        }
    }

    pub fn waiting_duration(&self, arrival: Timestamp) -> SignedDuration {
        if let Some(start) = self.start
            && start > arrival
        {
            start.duration_since(arrival)
        } else {
            SignedDuration::ZERO
        }
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct TimeWindows(SmallVec<[TimeWindow; 1]>);

impl TimeWindows {
    pub fn new(time_windows: SmallVec<[TimeWindow; 1]>) -> Self {
        TimeWindows(time_windows)
    }

    pub fn from_vec(time_windows: Vec<TimeWindow>) -> Self {
        TimeWindows::new(SmallVec::from_vec(time_windows))
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty() || self.0.iter().all(|tw| tw.is_empty())
    }

    pub fn is_satisfied(&self, arrival: Timestamp) -> bool {
        self.0.iter().any(|tw| tw.is_satisfied(arrival))
    }

    pub fn overtime(&self, arrival: Timestamp) -> SignedDuration {
        if self.is_satisfied(arrival) {
            return SignedDuration::ZERO;
        }

        self.0
            .iter()
            .filter(|tw| !tw.is_satisfied(arrival))
            .map(|tw| tw.overtime(arrival))
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(SignedDuration::ZERO)
    }

    pub fn end(&self) -> Option<Timestamp> {
        self.0.iter().filter_map(|tw| tw.end()).max()
    }

    pub fn waiting_duration(&self, arrival: Timestamp) -> SignedDuration {
        self.0
            .iter()
            .filter(|tw| tw.is_satisfied(arrival))
            .map(|tw| tw.waiting_duration(arrival))
            .min()
            .unwrap_or(SignedDuration::ZERO)
    }

    pub fn iter(&self) -> std::slice::Iter<'_, TimeWindow> {
        self.0.iter()
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

    #[test]
    fn test_is_satisfied() {
        let time_window = TimeWindowBuilder::default()
            .with_iso_start("2025-06-10T08:00:00+02:00")
            .with_iso_end("2025-06-10T10:00:00+02:00")
            .build();

        assert!(time_window.is_satisfied("2025-06-10T07:00:00+02:00".parse().unwrap()));
        assert!(time_window.is_satisfied("2025-06-10T08:00:00+02:00".parse().unwrap()));
        assert!(time_window.is_satisfied("2025-06-10T09:00:00+02:00".parse().unwrap()));
        assert!(time_window.is_satisfied("2025-06-10T10:00:00+02:00".parse().unwrap()));
        assert!(!time_window.is_satisfied("2025-06-10T10:00:01+02:00".parse().unwrap()));

        let time_window = TimeWindowBuilder::default()
            .with_iso_end("2025-06-10T10:00:00+02:00")
            .build();

        assert!(time_window.is_satisfied("2025-06-10T07:00:00+02:00".parse().unwrap()));
        assert!(time_window.is_satisfied("2025-06-10T08:00:00+02:00".parse().unwrap()));
        assert!(time_window.is_satisfied("2025-06-10T09:00:00+02:00".parse().unwrap()));
        assert!(time_window.is_satisfied("2025-06-10T10:00:00+02:00".parse().unwrap()));
        assert!(!time_window.is_satisfied("2025-06-10T10:00:01+02:00".parse().unwrap()));

        let time_window = TimeWindowBuilder::default()
            .with_iso_start("2025-06-10T08:00:00+02:00")
            .build();

        assert!(time_window.is_satisfied("2025-06-10T07:00:00+02:00".parse().unwrap()));
        assert!(time_window.is_satisfied("2025-06-10T08:00:00+02:00".parse().unwrap()));
        assert!(time_window.is_satisfied("2025-06-10T09:00:00+02:00".parse().unwrap()));
        assert!(time_window.is_satisfied("2025-06-10T10:00:00+02:00".parse().unwrap()));
        assert!(time_window.is_satisfied("2025-06-10T10:00:01+02:00".parse().unwrap()));
    }

    #[test]
    fn test_overtime() {
        let time_window = TimeWindowBuilder::default()
            .with_iso_start("2025-06-10T08:00:00+02:00")
            .with_iso_end("2025-06-10T10:00:00+02:00")
            .build();

        assert_eq!(
            time_window.overtime("2025-06-10T07:00:00+02:00".parse().unwrap()),
            SignedDuration::ZERO
        );
        assert_eq!(
            time_window.overtime("2025-06-10T10:00:01+02:00".parse().unwrap()),
            SignedDuration::from_secs(1)
        );

        let time_window = TimeWindowBuilder::default()
            .with_iso_end("2025-06-10T10:00:00+02:00")
            .build();

        assert_eq!(
            time_window.overtime("2025-06-10T07:00:00+02:00".parse().unwrap()),
            SignedDuration::ZERO
        );
        assert_eq!(
            time_window.overtime("2025-06-10T10:00:01+02:00".parse().unwrap()),
            SignedDuration::from_secs(1)
        );

        let time_window = TimeWindowBuilder::default()
            .with_iso_start("2025-06-10T08:00:00+02:00")
            .build();

        assert_eq!(
            time_window.overtime("2025-06-10T07:00:00+02:00".parse().unwrap()),
            SignedDuration::ZERO
        );
        assert_eq!(
            time_window.overtime("2025-06-10T10:00:01+02:00".parse().unwrap()),
            SignedDuration::ZERO
        );
    }

    #[test]
    fn test_waiting_duration() {
        let time_window = TimeWindowBuilder::default()
            .with_iso_start("2025-06-10T08:00:00+02:00")
            .with_iso_end("2025-06-10T10:00:00+02:00")
            .build();

        assert_eq!(
            time_window.waiting_duration("2025-06-10T07:00:00+02:00".parse().unwrap()),
            SignedDuration::from_hours(1)
        );
        assert_eq!(
            time_window.waiting_duration("2025-06-10T08:00:00+02:00".parse().unwrap()),
            SignedDuration::ZERO
        );
        assert_eq!(
            time_window.waiting_duration("2025-06-10T09:00:00+02:00".parse().unwrap()),
            SignedDuration::ZERO
        );
        assert_eq!(
            time_window.waiting_duration("2025-06-10T10:00:00+02:00".parse().unwrap()),
            SignedDuration::ZERO
        );
        assert_eq!(
            time_window.waiting_duration("2025-06-10T10:00:01+02:00".parse().unwrap()),
            SignedDuration::ZERO
        );
    }

    #[test]
    fn test_is_empty() {
        assert!(TimeWindow::new(None, None).is_empty());
        assert!(
            !TimeWindow::new(None, Some("2025-06-10T10:00:01+02:00".parse().unwrap())).is_empty()
        );
    }

    #[test]
    fn test_time_windows_is_satisfied() {
        let tw1 = TimeWindowBuilder::default()
            .with_iso_start("2025-06-10T08:00:00+02:00")
            .with_iso_end("2025-06-10T10:00:00+02:00")
            .build();
        let tw2 = TimeWindowBuilder::default()
            .with_iso_start("2025-06-10T14:00:00+02:00")
            .with_iso_end("2025-06-10T16:00:00+02:00")
            .build();

        let tws = TimeWindows::from_vec(vec![tw1, tw2]);

        assert!(tws.is_satisfied("2025-06-10T07:00:00+02:00".parse().unwrap()));
        assert!(tws.is_satisfied("2025-06-10T08:00:00+02:00".parse().unwrap()));
        assert!(tws.is_satisfied("2025-06-10T09:00:00+02:00".parse().unwrap()));
        assert!(tws.is_satisfied("2025-06-10T10:00:00+02:00".parse().unwrap()));
        assert!(tws.is_satisfied("2025-06-10T10:00:01+02:00".parse().unwrap()));
        assert!(tws.is_satisfied("2025-06-10T15:00:00+02:00".parse().unwrap()));
        assert!(!tws.is_satisfied("2025-06-10T16:30:00+02:00".parse().unwrap()));
    }

    #[test]
    fn test_time_windows_overtime() {
        let tw1 = TimeWindowBuilder::default()
            .with_iso_start("2025-06-10T08:00:00+02:00")
            .with_iso_end("2025-06-10T10:00:00+02:00")
            .build();
        let tw2 = TimeWindowBuilder::default()
            .with_iso_start("2025-06-10T14:00:00+02:00")
            .with_iso_end("2025-06-10T16:00:00+02:00")
            .build();

        let tws = TimeWindows::from_vec(vec![tw1, tw2]);

        assert_eq!(
            tws.overtime("2025-06-10T07:00:00+02:00".parse().unwrap()),
            SignedDuration::ZERO
        );
        assert_eq!(
            tws.overtime("2025-06-10T08:00:00+02:00".parse().unwrap()),
            SignedDuration::ZERO
        );
        assert_eq!(
            tws.overtime("2025-06-10T09:00:00+02:00".parse().unwrap()),
            SignedDuration::ZERO
        );
        assert_eq!(
            tws.overtime("2025-06-10T10:00:00+02:00".parse().unwrap()),
            SignedDuration::ZERO
        );
        assert_eq!(
            tws.overtime("2025-06-10T10:00:01+02:00".parse().unwrap()),
            SignedDuration::ZERO
        );
        assert_eq!(
            tws.overtime("2025-06-10T15:00:00+02:00".parse().unwrap()),
            SignedDuration::ZERO
        );
        assert_eq!(
            tws.overtime("2025-06-10T16:30:00+02:00".parse().unwrap()),
            SignedDuration::from_mins(30)
        );
    }

    #[test]
    fn test_time_windows_waiting_duration() {
        let tw1 = TimeWindowBuilder::default()
            .with_iso_start("2025-06-10T08:00:00+02:00")
            .with_iso_end("2025-06-10T10:00:00+02:00")
            .build();
        let tw2 = TimeWindowBuilder::default()
            .with_iso_start("2025-06-10T14:00:00+02:00")
            .with_iso_end("2025-06-10T16:00:00+02:00")
            .build();

        let tws = TimeWindows::from_vec(vec![tw1, tw2]);

        assert_eq!(
            tws.waiting_duration("2025-06-10T07:00:00+02:00".parse().unwrap()),
            SignedDuration::from_hours(1)
        );
        assert_eq!(
            tws.waiting_duration("2025-06-10T08:00:00+02:00".parse().unwrap()),
            SignedDuration::ZERO
        );
        assert_eq!(
            tws.waiting_duration("2025-06-10T09:00:00+02:00".parse().unwrap()),
            SignedDuration::ZERO
        );
        assert_eq!(
            tws.waiting_duration("2025-06-10T10:00:00+02:00".parse().unwrap()),
            SignedDuration::ZERO
        );
        assert_eq!(
            tws.waiting_duration("2025-06-10T11:00:00+02:00".parse().unwrap()),
            SignedDuration::from_hours(3)
        );
        assert_eq!(
            tws.waiting_duration("2025-06-10T15:00:00+02:00".parse().unwrap()),
            SignedDuration::ZERO
        );
        assert_eq!(
            tws.waiting_duration("2025-06-10T16:30:00+02:00".parse().unwrap()),
            SignedDuration::ZERO
        );
    }
}
