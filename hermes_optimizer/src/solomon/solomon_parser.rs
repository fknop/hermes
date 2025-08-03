use std::error::Error;

use jiff::{SignedDuration, Timestamp};

use crate::problem::{
    capacity::Capacity,
    distance_method::DistanceMethod,
    location::Location,
    service::{Service, ServiceBuilder},
    time_window::TimeWindow,
    travel_cost_matrix::TravelCostMatrix,
    vehicle::{Vehicle, VehicleBuilder, VehicleShiftBuilder},
    vehicle_routing_problem::{VehicleRoutingProblem, VehicleRoutingProblemBuilder},
};

pub struct SolomonParser;

impl SolomonParser {
    pub fn from_file(file: &str) -> Result<VehicleRoutingProblem, Box<dyn Error>> {
        let file_content = std::fs::read_to_string(file)?;
        Self::from_solomon(&file_content)
    }

    fn from_solomon(file_content: &str) -> Result<VehicleRoutingProblem, Box<dyn Error>> {
        let mut builder = VehicleRoutingProblemBuilder::default();

        let mut lines = file_content.lines().peekable();

        // Skip initial descriptive lines until "VEHICLE" or "NUMBER" is found
        while let Some(line) = lines.next() {
            let trimmed_line = line.trim();
            if trimmed_line.starts_with("VEHICLE") || trimmed_line.starts_with("NUMBER") {
                // If it's "VEHICLE", consume the "NUMBER CAPACITY" line
                if trimmed_line.starts_with("VEHICLE") {
                    lines.next(); // Consume "NUMBER     CAPACITY" line
                }
                break;
            }
        }
        let mut vehicles: Vec<Vehicle> = vec![];

        // Parse Vehicle Capacity
        if let Some(line) = lines.next() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 2 {
                return Err("Invalid VEHICLE line format".into());
            }

            let num_vehicles = parts[0].parse::<usize>()?;
            let vehicle_capacity = parts[1].parse::<f64>()?;

            for index in 0..num_vehicles {
                let mut builder = VehicleBuilder::default();
                builder
                    .set_vehicle_id(index.to_string())
                    .set_capacity(Capacity::from_vec(vec![vehicle_capacity]))
                    .set_return(true);
                let vehicle = builder.build();
                vehicles.push(vehicle);
            }
        } else {
            return Err("Missing VEHICLE section or content".into());
        }

        // Skip lines until "CUSTOMER" is found
        while let Some(line) = lines.next() {
            let trimmed_line = line.trim();
            if trimmed_line.starts_with("CUSTOMER") {
                // Consume the header line "CUST NO. XCOORD. YCOORD. DEMAND READY TIME DUE TIME SERVICE TIME"
                lines.next();
                break;
            }
        }

        let mut locations: Vec<Location> = Vec::new();
        let mut services: Vec<Service> = Vec::new();

        // Parse Customers
        for (line_index, line) in lines.enumerate() {
            let trimmed_line = line.trim();
            if trimmed_line.is_empty() {
                continue; // Skip empty lines
            }

            let parts: Vec<&str> = trimmed_line.split_whitespace().collect();
            if parts.len() != 7 {
                eprintln!("Warning: Skipping malformed customer line: '{trimmed_line}'");
                continue;
            }

            let id = parts[0].to_string();
            let x = parts[1].parse::<f64>()?;
            let y = parts[2].parse::<f64>()?;
            let demand = parts[3].parse::<f64>()?;
            let ready_time = parts[4].parse::<i64>()?;
            let due_time = parts[5].parse::<i64>()?;
            let service_time = parts[6].parse::<i64>()?;

            let location_id = locations.len();
            let location = Location::from_cartesian(location_id, x, y);
            locations.push(location);

            let customer_index = line_index - 1;
            if customer_index == 0 {
                for vehicle in vehicles.iter_mut() {
                    let mut shift_builder = VehicleShiftBuilder::default();
                    shift_builder
                        .set_earliest_start(Timestamp::from_second(ready_time)?)
                        .set_latest_end(Timestamp::from_second(due_time)?);
                    vehicle.set_shift(shift_builder.build());
                    vehicle.set_depot_location(location_id);
                }
            } else {
                let mut service_builder = ServiceBuilder::default();

                let start = Timestamp::from_second(ready_time)?;
                let end = Timestamp::from_second(due_time)?;

                service_builder
                    .set_external_id(id)
                    .set_demand(Capacity::from_vec(vec![demand]))
                    .set_service_duration(SignedDuration::from_secs(service_time))
                    .set_location_id(location_id)
                    .set_time_window(TimeWindow::new(Some(start), Some(end)));

                services.push(service_builder.build());
            }
        }

        let travel_costs_matrix = TravelCostMatrix::from_euclidian(&locations);
        builder
            .set_vehicles(vehicles)
            .set_locations(locations)
            .set_services(services)
            .set_distance_method(DistanceMethod::Euclidean)
            .set_travel_costs(travel_costs_matrix);

        Ok(builder.build())
    }
}

#[cfg(test)]
mod tests {
    use std::env;

    use super::*;

    #[test]
    fn test_solomon_parser() {
        let current_dir = env::current_dir().unwrap();
        let root_directory = current_dir.parent().unwrap();

        let path = root_directory.join("./data/solomon/c1/c101.txt");

        let vrp = SolomonParser::from_file(path.to_str().unwrap()).unwrap();

        assert_eq!(vrp.vehicles().len(), 25);

        for vehicle in vrp.vehicles() {
            assert_eq!(vehicle.depot_location_id(), Some(0));
            assert_eq!(*vehicle.capacity(), Capacity::from_vec(vec![200.0]));
        }

        for (index, service) in vrp.services().iter().enumerate() {
            assert_eq!(service.external_id(), (index + 1).to_string().as_str());
            assert_eq!(service.service_duration(), SignedDuration::from_secs(90));
        }

        // Check one location
        let time_window = vrp.services()[9]
            .time_windows()
            .iter()
            .min_by_key(|tw| tw.start())
            .unwrap();
        let timestamp_zero = Timestamp::from_second(0).unwrap();
        assert_eq!(
            time_window.start().unwrap(),
            timestamp_zero + SignedDuration::from_secs(357)
        );
        assert_eq!(
            time_window.end().unwrap(),
            timestamp_zero + SignedDuration::from_secs(410)
        );
    }
}
