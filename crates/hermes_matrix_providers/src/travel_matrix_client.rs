use crate::{
    as_the_crow_flies::as_the_crow_flies_matrices,
    graphhopper::{GraphHopperMatrixClient, GraphhopperMatrixClientParams},
    travel_matrices::TravelMatrices,
    travel_matrix_provider::TravelMatrixProvider,
};

pub struct TravelMatrixClient {
    graphhopper_client: GraphHopperMatrixClient,
}

impl TravelMatrixClient {
    pub fn new() -> Self {
        Self {
            graphhopper_client: GraphHopperMatrixClient::new(GraphhopperMatrixClientParams {
                api_key: std::env::var("GRAPHHOPPER_API_KEY").unwrap(),
                max_poll_attempts: 40, // max 20s, already really long time
                poll_interval: std::time::Duration::from_millis(500),
            }),
        }
    }

    pub async fn fetch_matrix<P>(
        &self,
        points: &[P],
        provider: TravelMatrixProvider,
    ) -> anyhow::Result<TravelMatrices>
    where
        for<'a> &'a P: Into<geo_types::Point>,
    {
        match provider {
            TravelMatrixProvider::GraphHopperApi {
                gh_profile: profile,
            } => self.graphhopper_client.fetch_matrix(points, profile).await,
            TravelMatrixProvider::AsTheCrowFlies { speed_kmh } => {
                Ok(as_the_crow_flies_matrices(points, speed_kmh))
            }
            TravelMatrixProvider::Custom { matrices } => Ok(matrices),
        }
    }
}
