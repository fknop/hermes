use crate::graph::{Graph, GraphEdge};
use crate::properties::property::Property;
use crate::properties::property_map::EdgeDirection;

struct Weighting;

impl Weighting {
    fn new(graph: &Graph) -> Weighting {
        Weighting
    }

    fn calc_edge_weight(&self, edge: &GraphEdge, direction: EdgeDirection) -> f64 {
        let speed = Self::get_speed(edge, direction);
        if (speed == 0) {
            return f64::INFINITY;
        }

        let speed_meters_per_second = (speed as f64) * (1000.0 / 3600.0);
        return edge.get_distance() / speed_meters_per_second;
    }

    fn calc_edge_ms(&self, edge: &GraphEdge, direction: EdgeDirection) -> u64 {
        let time = self.calc_edge_weight(edge, direction) * 1000.0;
        time.round() as u64
    }

    fn get_speed(edge: &GraphEdge, direction: EdgeDirection) -> u8 {
        let access = edge
            .properties
            .get_bool(Property::VehicleAccess("car".to_string()), direction)
            .unwrap_or(false);

        if access == false {
            return 0;
        }

        let speed = edge.properties.get_u8(Property::MaxSpeed, direction);

        speed.unwrap_or(0)
    }
}

fn test(a: u8) {}

fn test2() {
    let a: u8 = 2;
    test(a);
    test(a);
}
