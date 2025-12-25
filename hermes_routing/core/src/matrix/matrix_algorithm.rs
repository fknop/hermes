use std::time::Duration;

use crate::types::NodeId;

use super::matrix::Matrix;

pub struct MatrixAlgorithmResult {
    pub matrix: Matrix,
    pub visited_nodes: usize,
    pub duration: Duration,
}

pub trait MatrixAlgorithm {
    fn calc_matrix(&mut self, sources: &[NodeId], targets: &[NodeId]) -> MatrixAlgorithmResult;
}
