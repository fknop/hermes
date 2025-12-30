use crate::{
    as_the_crow_flies::as_the_crow_flies_matrices,
    cache::{cache_matrices, get_cached_matrices},
    graphhopper_api::{GraphHopperMatrixClient, GraphhopperMatrixClientParams},
    travel_matrices::TravelMatrices,
    travel_matrix_provider::TravelMatrixProvider,
};

pub struct TravelMatrixClient {
    graphhopper_client: GraphHopperMatrixClient,
}

impl TravelMatrixClient {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn fetch_matrix<P>(
        &self,
        points: &[P],
        provider: TravelMatrixProvider,
    ) -> anyhow::Result<TravelMatrices>
    where
        for<'a> &'a P: Into<geo_types::Point>,
    {
        let cached_results = get_cached_matrices(points, &provider);

        if let Ok(Some(results)) = cached_results {
            return Ok(results);
        }

        match provider {
            TravelMatrixProvider::GraphHopperApi {
                gh_profile: profile,
            } => self
                .graphhopper_client
                .fetch_matrix(points, profile)
                .await
                .inspect(|result| {
                    cache_matrices(points, &provider, &result);
                }),
            TravelMatrixProvider::AsTheCrowFlies { speed_kmh } => {
                Ok(as_the_crow_flies_matrices(points, speed_kmh))
            }
            TravelMatrixProvider::Custom { matrices } => Ok(matrices),
        }
    }
}

impl Default for TravelMatrixClient {
    fn default() -> Self {
        Self {
            graphhopper_client: GraphHopperMatrixClient::new(GraphhopperMatrixClientParams {
                api_key: std::env::var("GRAPHHOPPER_API_KEY")
                    .expect("GRAPHHOPPER_API_KEY must be set"),
                max_poll_attempts: 40, // max 20s, already really long time
                poll_interval: std::time::Duration::from_millis(500),
            }),
        }
    }
}
