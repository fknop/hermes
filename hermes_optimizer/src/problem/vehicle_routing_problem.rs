use super::{
    location::Location,
    service::Service,
    travel_cost_matrix::{Cost, Distance, Time, TravelCostMatrix},
    vehicle::Vehicle,
};

pub struct VehicleRoutingProblem {
    locations: Vec<Location>,
    vehicles: Vec<Vehicle>,
    services: Vec<Service>,
    travel_costs: TravelCostMatrix,
}

impl VehicleRoutingProblem {
    fn get_distance(&self, from: &Location, to: &Location) -> Distance {
        self.travel_costs.get_distance(from.id(), to.id())
    }

    fn get_time(&self, from: &Location, to: &Location) -> Time {
        self.travel_costs.get_time(from.id(), to.id())
    }

    fn get_cost(&self, from: &Location, to: &Location) -> Cost {
        self.travel_costs.get_cost(from.id(), to.id())
    }
}

pub struct VehicleRoutingProblemBuilder {
    travel_costs: Option<TravelCostMatrix>,
}

impl VehicleRoutingProblemBuilder {
    pub fn new() -> Self {
        VehicleRoutingProblemBuilder { travel_costs: None }
    }

    pub fn with_travel_costs(&mut self, travel_costs: TravelCostMatrix) {
        self.travel_costs = Some(travel_costs)
    }

    pub fn build(self) -> VehicleRoutingProblem {
        VehicleRoutingProblem {
            locations: vec![],
            vehicles: vec![],
            services: vec![],
            travel_costs: self.travel_costs.expect("Missing travel_costs"),
        }
    }
}
