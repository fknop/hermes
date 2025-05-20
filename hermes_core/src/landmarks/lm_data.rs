use tracing::{debug, info};

use crate::{
    storage::{read_bytes, write_bytes},
    weighting::Weight,
};

#[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
pub(crate) struct Landmark {
    node_id: usize,
    weight_from_landmark: Vec<Weight>,
    weight_to_landmark: Vec<Weight>,
}

impl Landmark {
    pub fn new(
        node_id: usize,
        weight_from_landmark: Vec<Weight>,
        weight_to_landmark: Vec<Weight>,
    ) -> Self {
        Landmark {
            node_id,
            weight_from_landmark,
            weight_to_landmark,
        }
    }
}

#[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
pub(crate) struct LMData {
    landmarks: Vec<Landmark>,
}

impl LMData {
    pub fn from_file(path: &str) -> Self {
        debug!("Reading from path {}", path);
        let bytes = read_bytes(path);
        debug!("Read from path {}, size {}", path, bytes.len());
        let data = rkyv::from_bytes::<Self, rkyv::rancor::Error>(&bytes[..]).unwrap();
        info!("Deserialized landmarks from buffer");
        data
    }

    pub fn save_to_file(&self, path: &str) -> Result<(), std::io::Error> {
        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(self).expect("to_bytes failed");
        write_bytes(&bytes[..], path)
    }

    pub fn get_node_ids(&self) -> Vec<usize> {
        self.landmarks.iter().map(|lm| lm.node_id).collect()
    }

    pub fn new(landmarks: Vec<Landmark>) -> Self {
        LMData { landmarks }
    }

    pub fn num_landmarks(&self) -> usize {
        self.landmarks.len()
    }

    pub fn weight_from_landmark(&self, landmark_index: usize, node_id: usize) -> Weight {
        self.landmarks[landmark_index].weight_from_landmark[node_id]
    }

    pub fn weight_to_landmark(&self, landmark_index: usize, node_id: usize) -> Weight {
        self.landmarks[landmark_index].weight_to_landmark[node_id]
    }
}
