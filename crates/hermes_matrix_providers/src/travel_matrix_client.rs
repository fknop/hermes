use crate::{
    as_the_crow_flies::as_the_crow_flies_matrices,
    cache::{FileCache, MatricesCache},
    graphhopper_api::{GraphHopperMatrixClient, GraphhopperMatrixClientParams},
    travel_matrices::TravelMatrices,
    travel_matrix_provider::TravelMatrixProvider,
};

pub struct TravelMatrixClient<C>
where
    C: MatricesCache,
{
    graphhopper_client: Option<GraphHopperMatrixClient>,
    cache: C,
}

impl<C> TravelMatrixClient<C>
where
    C: MatricesCache,
{
    pub fn new(cache: C) -> Self {
        Self {
            cache,
            graphhopper_client: Self::create_default_graphhopper_client(),
        }
    }

    fn create_default_graphhopper_client() -> Option<GraphHopperMatrixClient> {
        if let Ok(api_key) = std::env::var("GRAPHHOPPER_API_KEY") {
            Some(GraphHopperMatrixClient::new(
                GraphhopperMatrixClientParams {
                    api_key,
                    max_poll_attempts: 100, // max 20s, already really long time
                    poll_interval: std::time::Duration::from_millis(200),
                },
            ))
        } else {
            None
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
        let cached = self.cache.get_cached(&provider, points);

        if let Ok(Some(cached_matrices)) = cached {
            return Ok(cached_matrices);
        }

        let result = match &provider {
            TravelMatrixProvider::GraphHopperApi {
                gh_profile: profile,
            } => {
                let gh_client = self
                    .graphhopper_client
                    .as_ref()
                    .ok_or(anyhow::anyhow!("Missing GH api key"))?;

                gh_client.fetch_matrix(points, *profile).await
            }
            TravelMatrixProvider::AsTheCrowFlies { speed_kmh } => {
                Ok(as_the_crow_flies_matrices(points, *speed_kmh))
            }
            TravelMatrixProvider::Custom { matrices } => Ok(TravelMatrices {
                distances: matrices.distances.iter().flatten().copied().collect(),
                times: matrices.times.iter().flatten().copied().collect(),
                costs: Some(matrices.costs.iter().flatten().copied().collect()),
            }),
        };

        if let Ok(ref matrices) = result {
            self.cache.cache(&provider, points, matrices)?;
        }

        result
    }
}

impl Default for TravelMatrixClient<FileCache> {
    fn default() -> Self {
        Self {
            cache: FileCache::new(
                &std::env::var("HERMES_CACHE_FOLDER").expect("HERMES_CACHE_FOLDER must be set"),
            ),
            graphhopper_client: Self::create_default_graphhopper_client(),
        }
    }
}
