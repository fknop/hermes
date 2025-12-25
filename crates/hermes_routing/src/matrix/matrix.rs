use std::{fmt, time::Duration};

use crate::{
    constants::MAX_WEIGHT,
    distance::{Distance, Meters},
    weighting::{Milliseconds, Weight},
};

#[derive(Clone)]
pub struct MatrixEntry {
    weight: Weight,
    distance: Distance<Meters>,
    time: Milliseconds,
}

impl MatrixEntry {
    pub fn weight(&self) -> Weight {
        self.weight
    }

    pub fn distance(&self) -> Distance<Meters> {
        self.distance
    }

    pub fn time(&self) -> Milliseconds {
        self.time
    }
}

impl fmt::Debug for MatrixEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[weight={}, time={:?}, distance={}km]",
            self.weight,
            Duration::from_millis(self.time as u64),
            self.distance.value() / 1000.0
        )
    }
}

#[derive(Debug)]
pub struct Matrix {
    entries: Vec<Vec<Option<MatrixEntry>>>,
}

impl Matrix {
    pub fn new(sources: usize, targets: usize) -> Self {
        Matrix {
            entries: vec![vec![None; targets]; sources],
        }
    }

    pub fn update_entry(
        &mut self,
        source: usize,
        target: usize,
        weight: Weight,
        distance: Distance<Meters>,
        time: Milliseconds,
    ) {
        self.entries[source][target] = Some(MatrixEntry {
            weight,
            distance,
            time,
        });
    }

    pub fn weight(&self, source_index: usize, target_index: usize) -> Weight {
        match &self.entries[source_index][target_index] {
            Some(entry) => entry.weight,
            None => MAX_WEIGHT,
        }
    }

    pub fn entry(&self, source_index: usize, target_index: usize) -> Option<&MatrixEntry> {
        self.entries[source_index][target_index].as_ref()
    }
}
