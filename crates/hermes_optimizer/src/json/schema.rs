use schemars::schema_for;

use crate::json::types;

pub fn generate_json_schema() -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(&schema_for!(types::JsonVehicleRoutingProblem))
}
