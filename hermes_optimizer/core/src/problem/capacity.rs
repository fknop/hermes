use crate::problem::amount::{Amount, AmountExpression};

pub type Capacity = Amount;

pub fn is_capacity_satisfied<C, D>(capacity: &C, demand: &D) -> bool
where
    C: AmountExpression,
    D: AmountExpression,
{
    if capacity.len() < demand.len() {
        return false;
    }

    demand.iter().zip(capacity.iter()).all(|(d, c)| d <= c)
}

pub fn over_capacity_demand<C, D>(capacity: &C, demand: &D) -> f64
where
    C: AmountExpression,
    D: AmountExpression,
{
    demand
        .iter()
        .zip(capacity.iter())
        .filter_map(|(d, c)| if d > c { Some(d - c) } else { None })
        .sum()
}
