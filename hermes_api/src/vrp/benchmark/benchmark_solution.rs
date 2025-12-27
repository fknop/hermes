use hermes_optimizer::{
    problem::{capacity::Capacity, service::ServiceId, vehicle::VehicleIdx},
    solver::score::{Score, ScoreAnalysis},
};
use jiff::SignedDuration;
use serde::Serialize;

#[derive(Serialize)]
pub struct BenchmarkServiceActivity {
    pub service_id: ServiceId,
}

#[derive(Serialize)]
#[serde(tag = "type")]
pub enum BenchmarkSolutionActivity {
    Service(BenchmarkServiceActivity),
}

#[derive(Serialize)]
pub struct BenchmarkSolutionRoute {
    pub activities: Vec<BenchmarkSolutionActivity>,
    pub distance: f64,
    pub total_demand: Capacity,
    pub vehicle_id: VehicleIdx,
    pub waiting_duration: SignedDuration,
    pub vehicle_max_load: f64,
}

#[derive(Serialize)]
pub struct BenchmarkSolution {
    pub routes: Vec<BenchmarkSolutionRoute>,
    pub distance: f64,
    pub score: Score,
    pub score_analysis: ScoreAnalysis,
}
