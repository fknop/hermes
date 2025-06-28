use std::error::Error;

use jiff::{SignedDuration, Timestamp};

use crate::problem::{
    capacity::Capacity,
    location::Location,
    service::{Service, ServiceBuilder},
    time_window::TimeWindow,
    travel_cost_matrix::TravelCostMatrix,
    vehicle::{Vehicle, VehicleBuilder},
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

        // Parse Vehicle Capacity
        if let Some(line) = lines.next() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 2 {
                return Err("Invalid VEHICLE line format".into());
            }
            let mut vehicles: Vec<Vehicle> = vec![];

            let num_vehicles = parts[0].parse::<usize>()?;
            let vehicle_capacity = parts[1].parse::<f64>()?;

            for _ in 0..num_vehicles {
                let vehicle = VehicleBuilder::default()
                    .with_capacity(Capacity::new(vec![vehicle_capacity]))
                    .build();
                vehicles.push(vehicle);
            }

            builder = builder.with_vehicles(vehicles)
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
        for line in lines {
            let trimmed_line = line.trim();
            if trimmed_line.is_empty() {
                continue; // Skip empty lines
            }

            let parts: Vec<&str> = trimmed_line.split_whitespace().collect();
            if parts.len() != 7 {
                eprintln!(
                    "Warning: Skipping malformed customer line: '{}'",
                    trimmed_line
                );
                continue;
            }

            let id = parts[0].to_string();
            let x = parts[1].parse::<f64>()?;
            let y = parts[2].parse::<f64>()?;
            let demand = parts[3].parse::<f64>()?;
            let ready_time = parts[4].parse::<i64>()?;
            let due_time = parts[5].parse::<i64>()?;
            let service_time = parts[6].parse::<i64>()?;

            let location = Location::from_cartesian(locations.len(), x, y);
            locations.push(location);

            let mut service_builder = ServiceBuilder::default();

            let start = Timestamp::from_second(ready_time)?;
            let end = Timestamp::from_second(due_time)?;

            service_builder
                .set_external_id(parts[0].to_string())
                .set_demand(Capacity::new(vec![demand]))
                .set_service_duration(SignedDuration::from_secs(service_time))
                .set_location_id(locations.len())
                .set_time_window(TimeWindow::new(Some(start), Some(end)));

            services.push(service_builder.build());
        }

        let travel_costs_matrix = TravelCostMatrix::from_euclidian(&locations);
        builder = builder
            .with_services(services)
            .with_locations(locations)
            .with_travel_costs(travel_costs_matrix);

        Ok(builder.build())
    }
}
