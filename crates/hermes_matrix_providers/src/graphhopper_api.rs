use std::{fmt::Display, time::Duration};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::debug;

use crate::travel_matrices::TravelMatrices;

pub type GHPoint = [f64; 2];

#[derive(Deserialize, Serialize, JsonSchema, Copy, Clone, Hash)]
#[serde(rename_all = "snake_case")]
pub enum GraphHopperProfile {
    Car,
    Bike,
    Foot,
    SmallTruck,
    Truck,
}

impl Display for GraphHopperProfile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                GraphHopperProfile::Car => "car",
                GraphHopperProfile::Bike => "bike",
                GraphHopperProfile::Foot => "foot",
                GraphHopperProfile::SmallTruck => "small_truck",
                GraphHopperProfile::Truck => "truck",
            }
        )
    }
}

#[derive(Debug, Error)]
pub enum GraphHopperError {
    #[error("HTTP request failed: {0}")]
    Request(#[from] reqwest::Error),

    #[error("API error: {status} - {message}")]
    Api { status: u16, message: String },

    #[error("Job failed with status: {0}")]
    JobFailed(String),

    #[error("Polling timeout after {0} attempts")]
    Timeout(u32),

    #[error("Deserialization error: {0}")]
    Deserialize(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Serialize)]
pub struct MatrixRequestBody {
    /// Points for symmetric matrix (all-to-all)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub points: Option<Vec<GHPoint>>,

    /// Street hints for source points (helps snapping)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_point_hints: Option<Vec<String>>,

    /// Street hints for destination points
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to_point_hints: Option<Vec<String>>,

    /// Which arrays to return: "weights", "times", "distances"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub out_arrays: Option<Vec<String>>,

    /// Routing profile (e.g., "car", "bike", "foot")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<String>,

    /// Fail fast on unreachable points
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fail_fast: Option<bool>,
}

#[derive(Deserialize)]
struct MatrixSolution {
    /// Travel times in seconds
    pub times: Vec<Vec<f64>>,

    /// Distances in meters
    pub distances: Vec<Vec<f64>>,

    /// Weights
    pub weights: Vec<Vec<f64>>,
}

#[derive(Deserialize)]
struct AsyncMatrixJobResponse {
    job_id: String,
}

#[derive(Deserialize)]
struct AsyncMatrixResponse {
    status: String,
    solution: Option<MatrixSolution>,
}

pub struct GraphhopperMatrixClientParams {
    pub api_key: String,
    pub poll_interval: Duration,
    pub max_poll_attempts: u32,
}

pub const GRAPHOPPER_MATRIX_SYNC_API_URL: &str = "https://graphhopper.com/api/1/matrix";
pub const GRAPHOPPER_MATRIX_ASYNC_POST_API_URL: &str =
    "https://graphhopper.com/api/1/matrix/calculate";
pub const GRAPHOPPER_MATRIX_ASYNC_POLL_API_URL: &str =
    "https://graphhopper.com/api/1/matrix/solution";

pub struct GraphHopperMatrixClient {
    params: GraphhopperMatrixClientParams,
    client: reqwest::Client,
}

impl GraphHopperMatrixClient {
    pub fn new(params: GraphhopperMatrixClientParams) -> Self {
        Self {
            params,
            client: reqwest::Client::new(),
        }
    }

    pub async fn fetch_matrix<P>(
        &self,
        points: &[P],
        profile: GraphHopperProfile,
    ) -> anyhow::Result<TravelMatrices>
    where
        for<'a> &'a P: Into<geo_types::Point>,
    {
        let gh_points: Vec<GHPoint> = points
            .iter()
            .map(|p| {
                let point: geo_types::Point = p.into();
                [point.x(), point.y()]
            })
            .collect();

        // TODO: validate profile

        let body = MatrixRequestBody {
            points: Some(gh_points),
            from_point_hints: None,
            to_point_hints: None,
            out_arrays: Some(vec![
                "times".to_string(),
                "distances".to_string(),
                "weights".to_string(),
            ]),
            profile: Some(profile.to_string()),
            fail_fast: Some(true),
        };

        let result = if points.len() < 25 {
            self.sync_matrix_request(&body).await
        } else {
            self.async_matrix_requset(&body).await
        };

        match result {
            Ok(solution) => {
                let times = solution.times.into_iter().flatten().collect();
                let distances = solution.distances.into_iter().flatten().collect();
                let costs = solution.weights.into_iter().flatten().collect();

                Ok(TravelMatrices {
                    times,
                    distances,
                    costs: Some(costs),
                })
            }
            Err(e) => Err(anyhow::anyhow!(e)),
        }
    }

    async fn sync_matrix_request(
        &self,
        body: &MatrixRequestBody,
    ) -> Result<MatrixSolution, GraphHopperError> {
        let response = self
            .client
            .post(GRAPHOPPER_MATRIX_SYNC_API_URL)
            .query(&[("key", &self.params.api_key)])
            .json(body)
            .send()
            .await?;

        self.handle_response(response).await
    }

    async fn async_matrix_requset(
        &self,
        body: &MatrixRequestBody,
    ) -> Result<MatrixSolution, GraphHopperError> {
        let post_response = self
            .client
            .post(GRAPHOPPER_MATRIX_ASYNC_POST_API_URL)
            .query(&[("key", &self.params.api_key)])
            .json(body)
            .send()
            .await?;

        if !post_response.status().is_success() {
            let status = post_response.status().as_u16();
            let message = post_response.text().await.unwrap_or_default();
            return Err(GraphHopperError::Api { status, message });
        }

        debug!("GraphHopperApi: Posted matrix request",);

        let job_response: AsyncMatrixJobResponse = post_response.json().await?;

        self.poll_until_completed(&job_response.job_id).await
    }

    async fn get_solution(&self, job_id: &str) -> Result<Option<MatrixSolution>, GraphHopperError> {
        let url = format!("{}/{}", GRAPHOPPER_MATRIX_ASYNC_POLL_API_URL, job_id);
        let poll_response = self
            .client
            .get(url)
            .query(&[("key", &self.params.api_key)])
            .send()
            .await?;

        if !poll_response.status().is_success() {
            let status = poll_response.status().as_u16();
            let message = poll_response.text().await.unwrap_or_default();
            return Err(GraphHopperError::Api { status, message });
        }

        let async_response: AsyncMatrixResponse = poll_response.json().await?;

        match async_response.status.as_str() {
            "finished" => Ok(async_response.solution),
            "waiting" | "processing" => Ok(None),
            other => Err(GraphHopperError::JobFailed(other.to_string())),
        }
    }

    async fn poll_until_completed(&self, job_id: &str) -> Result<MatrixSolution, GraphHopperError> {
        for attempt in 1..=self.params.max_poll_attempts {
            debug!(
                "GraphHopperApi: Polling for job completion {}/{}",
                attempt, self.params.max_poll_attempts
            );
            if let Some(solution) = self.get_solution(job_id).await? {
                return Ok(solution);
            }

            tokio::time::sleep(self.params.poll_interval).await;
        }

        Err(GraphHopperError::Timeout(self.params.max_poll_attempts))
    }

    async fn handle_response(
        &self,
        response: reqwest::Response,
    ) -> Result<MatrixSolution, GraphHopperError> {
        if response.status().is_success() {
            let matrix_solution: MatrixSolution = response.json().await?;
            Ok(matrix_solution)
        } else {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();
            Err(GraphHopperError::Api { status, message })
        }
    }
}
