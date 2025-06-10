use crate::problem::service::ServiceId;

pub type Cost = f64;

pub struct SolutionActivity {
    service_id: ServiceId,
}

pub struct SolutionRoute {
    activities: Vec<SolutionActivity>,
}

pub struct Solution {
    cost: Cost,
}

impl Solution {
    pub fn get_cost(&self) -> Cost {
        self.cost
    }
}
