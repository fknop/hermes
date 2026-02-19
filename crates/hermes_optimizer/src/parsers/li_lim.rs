use jiff::{SignedDuration, Timestamp};

use crate::{
    parsers::parser::DatasetParser,
    problem::{
        capacity::Capacity,
        distance_method::DistanceMethod,
        fleet::Fleet,
        location::Location,
        shipment::ShipmentBuilder,
        time_window::TimeWindow,
        travel_cost_matrix::TravelMatrices,
        vehicle::{Vehicle, VehicleBuilder, VehicleShiftBuilder},
        vehicle_profile::VehicleProfile,
        vehicle_routing_problem::{VehicleRoutingProblem, VehicleRoutingProblemBuilder},
    },
};

pub struct LiLimParser;

struct LiLimRow {
    id: usize,
    x: f64,
    y: f64,
    demand: f64,
    earliest: i64,
    latest: i64,
    service_time: i64,
    pickup_idx: usize,
    delivery_idx: usize,
}

impl DatasetParser for LiLimParser {
    fn parse(&self, content: &str) -> Result<VehicleRoutingProblem, anyhow::Error> {
        let mut lines = content.lines();

        // Parse header: <num_vehicles> <capacity> <speed>
        let header = lines
            .next()
            .ok_or_else(|| anyhow::anyhow!("Missing header line"))?;
        let header_parts: Vec<&str> = header.split_whitespace().collect();
        if header_parts.len() < 2 {
            return Err(anyhow::anyhow!("Invalid header format"));
        }
        let num_vehicles = header_parts[0].parse::<usize>()?;
        let vehicle_capacity = header_parts[1].parse::<f64>()?;

        // Parse all data rows
        let mut rows: Vec<LiLimRow> = Vec::new();
        for line in lines {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() < 9 {
                continue;
            }
            rows.push(LiLimRow {
                id: parts[0].parse()?,
                x: parts[1].parse()?,
                y: parts[2].parse()?,
                demand: parts[3].parse()?,
                earliest: parts[4].parse()?,
                latest: parts[5].parse()?,
                service_time: parts[6].parse()?,
                pickup_idx: parts[7].parse()?,
                delivery_idx: parts[8].parse()?,
            });
        }

        if rows.is_empty() {
            return Err(anyhow::anyhow!("No data rows found"));
        }

        // Build locations (one per row, in order)
        let locations: Vec<Location> = rows
            .iter()
            .map(|r| Location::from_cartesian(r.x, r.y))
            .collect();

        // Row 0 is the depot
        let depot = &rows[0];
        let mut vehicles: Vec<Vehicle> = Vec::with_capacity(num_vehicles);
        for index in 0..num_vehicles {
            let mut vb = VehicleBuilder::default();
            vb.set_vehicle_id(index.to_string())
                .set_capacity(Capacity::from_vec(vec![vehicle_capacity]))
                .set_profile_id(0)
                .set_return(true);

            let mut shift_builder = VehicleShiftBuilder::default();
            shift_builder
                .set_earliest_start(Timestamp::from_second(depot.earliest)?)
                .set_latest_end(Timestamp::from_second(depot.latest)?);
            let mut vehicle = vb.build();
            vehicle.set_shift(shift_builder.build());
            vehicle.set_depot_location(0.into());
            vehicles.push(vehicle);
        }

        // Build shipments from pickup-delivery pairs.
        // A pickup row has delivery_idx > 0 (pointing to its delivery).
        let mut shipments = Vec::new();
        for row in &rows[1..] {
            // Only process pickup rows (delivery_idx > 0 means this is a pickup)
            if row.delivery_idx == 0 {
                continue;
            }

            let delivery_row = &rows[row.delivery_idx];

            // Validate that the delivery row points back to this pickup
            if delivery_row.pickup_idx != row.id {
                return Err(anyhow::anyhow!(
                    "Inconsistent pickup-delivery pair: pickup {} points to delivery {}, but delivery points back to {}",
                    row.id,
                    row.delivery_idx,
                    delivery_row.pickup_idx
                ));
            }

            let mut sb = ShipmentBuilder::default();
            sb.set_external_id(format!("{}", row.id))
                .set_demand(Capacity::from_vec(vec![row.demand.abs()]))
                .set_pickup_location_id(row.id)
                .set_pickup_duration(SignedDuration::from_secs(row.service_time))
                .set_pickup_time_window(TimeWindow::new(
                    Some(Timestamp::from_second(row.earliest)?),
                    Some(Timestamp::from_second(row.latest)?),
                ))
                .set_delivery_location_id(delivery_row.id)
                .set_delivery_duration(SignedDuration::from_secs(delivery_row.service_time))
                .set_delivery_time_window(TimeWindow::new(
                    Some(Timestamp::from_second(delivery_row.earliest)?),
                    Some(Timestamp::from_second(delivery_row.latest)?),
                ));

            shipments.push(sb.build());
        }

        let travel_costs_matrix = TravelMatrices::from_euclidean(&locations, false);

        let mut builder = VehicleRoutingProblemBuilder::default();
        builder
            .set_vehicle_profiles(vec![VehicleProfile::new(
                "vehicle".to_owned(),
                travel_costs_matrix,
            )])
            .set_fleet(Fleet::Finite(vehicles))
            .set_locations(locations)
            .set_shipments(shipments)
            .set_distance_method(DistanceMethod::Euclidean)
            .set_penalize_waiting_duration(false);

        Ok(builder.build())
    }
}

#[cfg(test)]
mod tests {
    use std::env;

    use super::*;

    #[test]
    fn test_li_lim_parser() {
        let current_dir = env::current_dir().unwrap();
        let root_directory = current_dir.parent().unwrap();

        let path = root_directory.join("../data/pdptw/li-lim/100/lc101.txt");
        let content = std::fs::read_to_string(&path).unwrap();
        let parser = LiLimParser;
        let vrp = parser.parse(&content).unwrap();

        assert_eq!(vrp.vehicles().len(), 25);

        for vehicle in vrp.vehicles() {
            assert_eq!(vehicle.depot_location_id(), Some(0.into()));
            assert_eq!(*vehicle.capacity(), Capacity::from_vec(vec![200.0]));
        }

        // 107 rows total (0=depot + 106 tasks), 53 pickup-delivery pairs
        assert_eq!(vrp.jobs().len(), 53);

        for job in vrp.jobs() {
            assert!(job.is_shipment(), "Expected all jobs to be shipments");
        }

        // Check first shipment (pickup row id=3, delivery row id=75)
        // Row 3: x=42 y=66 demand=10 earliest=65 latest=146 service=90 pickup_idx=0 delivery_idx=75
        // Row 75: x=45 y=65 demand=-10 earliest=997 latest=1068 service=90 pickup_idx=3 delivery_idx=0
        let first_shipment = vrp.shipment(0);
        assert_eq!(first_shipment.external_id(), "3");
        assert_eq!(*first_shipment.demand(), Capacity::from_vec(vec![10.0]));

        let pickup_tw = first_shipment
            .pickup()
            .time_windows()
            .iter()
            .next()
            .unwrap();
        let timestamp_zero = Timestamp::from_second(0).unwrap();
        assert_eq!(
            pickup_tw.start().unwrap(),
            timestamp_zero + SignedDuration::from_secs(65)
        );
        assert_eq!(
            pickup_tw.end().unwrap(),
            timestamp_zero + SignedDuration::from_secs(146)
        );

        let delivery_tw = first_shipment
            .delivery()
            .time_windows()
            .iter()
            .next()
            .unwrap();
        assert_eq!(
            delivery_tw.start().unwrap(),
            timestamp_zero + SignedDuration::from_secs(997)
        );
        assert_eq!(
            delivery_tw.end().unwrap(),
            timestamp_zero + SignedDuration::from_secs(1068)
        );
    }
}
