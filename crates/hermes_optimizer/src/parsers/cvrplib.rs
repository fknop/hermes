use std::path::Path;

use geo::Coord;

use crate::{
    parsers::parser::DatasetParser,
    problem::{
        capacity::Capacity,
        distance_method::DistanceMethod,
        fleet::Fleet,
        location::Location,
        service::ServiceBuilder,
        travel_cost_matrix::TravelMatrices,
        vehicle::VehicleBuilder,
        vehicle_profile::VehicleProfile,
        vehicle_routing_problem::{VehicleRoutingProblem, VehicleRoutingProblemBuilder},
    },
};

pub struct CVRPLibParser;

impl DatasetParser for CVRPLibParser {
    fn parse<P: AsRef<Path>>(&self, file: P) -> Result<VehicleRoutingProblem, anyhow::Error> {
        let file_content = std::fs::read_to_string(file)?;
        let instance = parse(&file_content)?;

        let mut builder = VehicleRoutingProblemBuilder::default();

        let locations = instance
            .coords
            .iter()
            .map(|coord| Location::from_cartesian(coord.x, coord.y))
            .collect::<Vec<_>>();

        let services = instance
            .coords
            .iter()
            .enumerate()
            .filter(|(id, _)| !instance.depots.contains(id))
            .map(|(id, _)| {
                let mut service_builder = ServiceBuilder::default();

                service_builder.set_demand(Capacity::from_vec(vec![instance.demands[id]]));
                service_builder.set_location_id(id);
                service_builder.set_external_id(format!("{id}"));

                service_builder.build()
            })
            .collect::<Vec<_>>();

        let mut vb = VehicleBuilder::default();
        vb.set_capacity(Capacity::from_vec(vec![instance.capacity]));
        vb.set_profile_id(0);
        vb.set_vehicle_id(String::from("vehicle"));
        vb.set_depot_location_id(instance.depots[0]);
        vb.set_return(true);

        let vehicle = vb.build();

        builder.set_vehicle_profiles(vec![VehicleProfile::new(
            String::from("profile"),
            TravelMatrices::from_euclidean(&locations, true),
        )]);
        builder.set_fleet(Fleet::Infinite(vec![vehicle]));
        builder.set_locations(locations);
        builder.set_services(services);
        builder.set_distance_method(DistanceMethod::Euclidean);
        builder.set_penalize_waiting_duration(false);

        Ok(builder.build())
    }
}

#[derive(Debug, Clone)]
pub struct CvrpInstance {
    pub dimension: usize,
    pub capacity: f64,
    pub coords: Vec<geo::Coord<f64>>,
    pub demands: Vec<f64>,
    pub depots: Vec<usize>,
}

pub fn parse(text: &str) -> Result<CvrpInstance, anyhow::Error> {
    let mut dimension: Option<usize> = None;
    let mut capacity: Option<f64> = None;
    let mut coords: Option<Vec<geo::Coord<f64>>> = None;
    let mut demands: Option<Vec<f64>> = None;
    let mut depots: Option<Vec<usize>> = None;

    let lines: Vec<&str> = text.lines().map(|l| l.trim()).collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];

        if line.is_empty() || line == "EOF" {
            i += 1;
            continue;
        }

        // Parse specifications (KEY : VALUE)
        if let Some((key, value)) = line.split_once(':') {
            let key = key.trim().to_uppercase();
            let value = value.trim();

            match key.as_str() {
                "DIMENSION" => {
                    dimension =
                        Some(value.parse().map_err(|_| {
                            anyhow::anyhow!(format!("Invalid dimension: {}", value))
                        })?);
                }
                "CAPACITY" => {
                    capacity =
                        Some(value.parse().map_err(|_| {
                            anyhow::anyhow!(format!("Invalid capacity: {}", value))
                        })?);
                }
                _ => {} // Ignore other specifications
            }
            i += 1;
            continue;
        }

        // Parse sections
        if line.contains("NODE_COORD_SECTION") {
            i += 1;
            let mut parsed_coords = Vec::new();
            while i < lines.len() && !lines[i].contains("SECTION") && lines[i] != "EOF" {
                let parts: Vec<&str> = lines[i].split_whitespace().collect();
                if parts.len() >= 3 {
                    let x: f64 = parts[1].parse().map_err(|_| {
                        anyhow::anyhow!(format!("Invalid x coordinate: {}", parts[1]))
                    })?;
                    let y: f64 = parts[2].parse().map_err(|_| {
                        anyhow::anyhow!(format!("Invalid y coordinate: {}", parts[2]))
                    })?;
                    parsed_coords.push(Coord { x, y });
                }
                i += 1;
            }
            coords = Some(parsed_coords);
            continue;
        }

        if line.contains("DEMAND_SECTION") {
            i += 1;
            let mut parsed_demands: Vec<f64> = Vec::new();
            while i < lines.len() && !lines[i].contains("SECTION") && lines[i] != "EOF" {
                let parts: Vec<&str> = lines[i].split_whitespace().collect();
                if parts.len() >= 2 {
                    let demand: f64 = parts[1]
                        .parse()
                        .map_err(|_| anyhow::anyhow!(format!("Invalid demand: {}", parts[1])))?;
                    parsed_demands.push(demand);
                }
                i += 1;
            }
            demands = Some(parsed_demands);
            continue;
        }

        if line.contains("DEPOT_SECTION") {
            i += 1;
            let mut parsed_depots = Vec::new();
            while i < lines.len() && !lines[i].contains("SECTION") && lines[i] != "EOF" {
                for part in lines[i].split_whitespace() {
                    let idx: i32 = part
                        .parse()
                        .map_err(|_| anyhow::anyhow!(format!("Invalid depot index: {}", part)))?;
                    if idx == -1 {
                        break;
                    }
                    // Convert to 0-indexed
                    parsed_depots.push((idx - 1) as usize);
                }
                i += 1;
            }
            depots = Some(parsed_depots);
            continue;
        }

        i += 1;
    }

    Ok(CvrpInstance {
        dimension: dimension.ok_or_else(|| anyhow::anyhow!("Missing DIMENSION"))?,
        capacity: capacity.ok_or_else(|| anyhow::anyhow!("Missing CAPACITY"))?,
        coords: coords.ok_or_else(|| anyhow::anyhow!("Missing NODE_COORD_SECTION"))?,
        demands: demands.ok_or_else(|| anyhow::anyhow!("Missing DEMAND_SECTION"))?,
        depots: depots.unwrap_or_else(|| vec![0]),
    })
}

pub fn parse_solution_file<P: AsRef<Path>>(path: P) -> Option<f64> {
    if !path.as_ref().exists() {
        return None;
    }

    let content = std::fs::read_to_string(path).ok()?;

    content
        .lines()
        .rev()
        .find_map(|line| line.strip_prefix("Cost "))
        .and_then(|cost| cost.trim().parse().ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"
NAME : A-n32-k5
COMMENT : (Augerat et al, No of trucks: 5, Optimal value: 784)
TYPE : CVRP
DIMENSION : 32
EDGE_WEIGHT_TYPE : EUC_2D
CAPACITY : 100
NODE_COORD_SECTION
 1 82 76
 2 96 44
 3 50 5
 4 49 8
 5 13 7
DEMAND_SECTION
1 0
2 19
3 21
4 6
5 19
DEPOT_SECTION
 1
 -1
EOF
"#;

    #[test]
    fn test_parse() {
        let instance = parse(SAMPLE).unwrap();

        assert_eq!(instance.dimension, 32);
        assert_eq!(instance.capacity, 100.0);
        assert_eq!(instance.coords.len(), 5);
        assert_eq!(instance.demands.len(), 5);
        assert_eq!(instance.depots, vec![0]);

        assert_eq!(instance.coords[0].x, 82.0);
        assert_eq!(instance.coords[0].y, 76.0);
        assert_eq!(instance.demands[0], 0.0);
        assert_eq!(instance.demands[1], 19.0);
    }
}
