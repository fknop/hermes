use hermes_optimizer::{
    problem::{capacity::Capacity, job::JobIdx, meters::Meters, vehicle::VehicleIdx},
    solver::score::{Score, ScoreAnalysis},
};
use jiff::SignedDuration;
use serde::Serialize;

#[derive(Serialize)]
pub struct BenchmarkServiceActivity {
    pub service_id: JobIdx,
}

#[derive(Serialize)]
#[serde(tag = "type")]
pub enum BenchmarkSolutionActivity {
    Service(BenchmarkServiceActivity),
}

#[derive(Serialize)]
pub struct BenchmarkSolutionRoute {
    pub activities: Vec<BenchmarkSolutionActivity>,
    pub distance: Meters,
    pub total_demand: Capacity,
    pub vehicle_id: VehicleIdx,
    pub waiting_duration: SignedDuration,
    pub vehicle_max_load: f64,
}

#[derive(Serialize)]
pub struct BenchmarkSolution {
    pub routes: Vec<BenchmarkSolutionRoute>,
    pub distance: Meters,
    pub score: Score,
    pub score_analysis: ScoreAnalysis,
}
