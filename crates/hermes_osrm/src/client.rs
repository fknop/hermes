use flatbuffers::InvalidFlatbuffer;
use serde::Deserialize;
use thiserror::Error;

use crate::fbresult_generated;

#[derive(Debug, Error)]
pub enum OsrmError {
    #[error("HTTP request failed: {0}")]
    Request(#[from] reqwest::Error),

    #[error("Deserialization error: {0}")]
    Deserialize(#[from] InvalidFlatbuffer),

    #[error("Incomplete response")]
    IncompleteResponse,
}

#[derive(Deserialize)]
pub struct OsrmMatrices {
    /// Travel times in seconds
    pub times: Vec<f64>,

    /// Distances in meters
    pub distances: Vec<f64>,
}

pub struct OsrmMatrixClientParams {
    pub osrm_url: String,
}

pub const OSRM_TABLE_API_PATH: &str = "/table/v1/driving/";

pub struct OsrmMatrixClient {
    params: OsrmMatrixClientParams,
    client: reqwest::Client,
}

impl OsrmMatrixClient {
    pub fn new(params: OsrmMatrixClientParams) -> Self {
        Self {
            params,
            client: reqwest::Client::new(),
        }
    }

    pub async fn fetch_matrix<P>(&self, points: &[P]) -> Result<OsrmMatrices, OsrmError>
    where
        for<'a> &'a P: Into<geo_types::Point>,
    {
        let mut url = self.params.osrm_url.clone();
        url.push_str(OSRM_TABLE_API_PATH);

        for (i, point) in points.iter().enumerate() {
            let point: geo_types::Point = point.into();
            url.push_str(&format!("{},{}", point.x(), point.y()));

            if i < points.len() - 1 {
                url.push(';');
            }
        }

        url.push_str(".flatbuffers");

        let response = self
            .client
            .post(url)
            .query(&[
                ("annotations", "duration,distance"),
                ("skip_waypoints", "true"),
            ])
            .send()
            .await?;

        let bytes = response.bytes().await?;
        let result =
            fbresult_generated::osrm::engine::api::fbresult::root_as_fbresult(bytes.as_ref());

        match result {
            Ok(result) => {
                let table = result.table().ok_or(OsrmError::IncompleteResponse)?;

                let durations = table.durations().ok_or(OsrmError::IncompleteResponse)?;
                let distances = table.distances().ok_or(OsrmError::IncompleteResponse)?;

                let times = durations
                    .into_iter()
                    .map(|duration| duration as f64)
                    .collect::<Vec<f64>>();
                let distances = distances
                    .into_iter()
                    .map(|distance| distance as f64)
                    .collect::<Vec<f64>>();

                Ok(OsrmMatrices { times, distances })
            }
            Err(err) => Err(OsrmError::Deserialize(err)),
        }
    }
}
