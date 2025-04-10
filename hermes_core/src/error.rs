use thiserror::Error;

#[derive(Error, Debug)]
pub enum ImportError {
    #[error("Failed to save graph file")]
    SaveGraph(std::io::Error),
    #[error("Failed to save LM file")]
    SaveLandmarks(std::io::Error),
    #[error("Failed to save location index file")]
    SaveLocationIndex(bincode::error::EncodeError),
}
