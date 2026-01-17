use geojson::Feature;
use hermes_optimizer::{
    problem::{capacity::Capacity, job::JobIdx, vehicle::VehicleIdx},
    solver::score::{Score, ScoreAnalysis},
};
use jiff::{SignedDuration, Timestamp};
use serde::Serialize;

#[derive(Serialize)]
pub struct ApiServiceActivity {
    pub service_id: JobIdx,
    pub arrival_time: Timestamp,
    pub departure_time: Timestamp,
    pub waiting_duration: SignedDuration,
}

#[derive(Serialize)]
pub struct ApiStartActivity {
    pub arrival_time: Timestamp,
    pub departure_time: Timestamp,
}

#[derive(Serialize)]
pub struct ApiEndActivity {
    pub arrival_time: Timestamp,
    pub departure_time: Timestamp,
}

#[derive(Serialize)]
#[serde(tag = "type")]
pub enum ApiSolutionActivity {
    Start(ApiStartActivity),
    Service(ApiServiceActivity),
    End(ApiEndActivity),
}

#[derive(Serialize)]
pub struct ApiSolutionRoute {
    pub duration: SignedDuration,
    pub transport_duration: SignedDuration,
    pub activities: Vec<ApiSolutionActivity>,
    pub distance: f64,
    pub total_demand: Capacity,
    pub vehicle_id: VehicleIdx,
    pub waiting_duration: SignedDuration,
    pub polyline: Feature,
    pub vehicle_max_load: f64,
}

#[derive(Serialize)]
pub struct ApiSolution {
    pub routes: Vec<ApiSolutionRoute>,
    pub duration: SignedDuration,
    pub score: Score,
    pub score_analysis: ScoreAnalysis,
    pub unassigned_jobs: Vec<JobIdx>,
}
