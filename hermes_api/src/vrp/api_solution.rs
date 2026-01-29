use geojson::Feature;
use hermes_optimizer::{
    problem::{capacity::Capacity, job::JobIdx, meters::Meters, vehicle::VehicleIdx},
    solver::score::{Score, ScoreAnalysis},
};
use jiff::{SignedDuration, Timestamp};
use schemars::{JsonSchema, Schema, SchemaGenerator};
use serde::Serialize;

#[derive(Serialize, JsonSchema)]
pub struct ApiServiceActivity {
    pub id: String,
    pub arrival_time: Timestamp,
    pub departure_time: Timestamp,
    pub waiting_duration: SignedDuration,
}

#[derive(Serialize, JsonSchema)]
pub struct ApiStartActivity {
    pub arrival_time: Timestamp,
    pub departure_time: Timestamp,
}

#[derive(Serialize, JsonSchema)]
pub struct ApiEndActivity {
    pub arrival_time: Timestamp,
    pub departure_time: Timestamp,
}

#[derive(Serialize, JsonSchema)]
#[serde(tag = "type")]
pub enum ApiSolutionActivity {
    Start(ApiStartActivity),
    Service(ApiServiceActivity),
    End(ApiEndActivity),
}

fn feature_schema(_gen: &mut SchemaGenerator) -> Schema {
    schemars::schema_for_value!(Feature::default())
}

#[derive(Serialize, JsonSchema)]
pub struct ApiSolutionRoute {
    pub duration: SignedDuration,
    pub transport_duration: SignedDuration,
    pub activities: Vec<ApiSolutionActivity>,
    pub distance: Meters,
    pub total_demand: Capacity,
    pub vehicle_id: String,
    pub waiting_duration: SignedDuration,
    #[schemars(schema_with = "feature_schema")]
    pub polyline: Feature,
    pub vehicle_max_load: f64,
}

#[derive(Serialize, JsonSchema)]
pub struct ApiSolution {
    pub routes: Vec<ApiSolutionRoute>,
    pub duration: SignedDuration,
    pub score: Score,
    pub score_analysis: ScoreAnalysis,
    pub unassigned_jobs: Vec<String>,
}
