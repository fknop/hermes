use std::path::Path;

use crate::problem::vehicle_routing_problem::VehicleRoutingProblem;

pub trait DatasetParser {
    fn parse<P: AsRef<Path>>(&self, file: P) -> Result<VehicleRoutingProblem, anyhow::Error>;
}
