use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use fxhash::FxHashSet;
use hermes_optimizer::problem::{
    amount::{Amount, AmountExpression, AmountSum},
    capacity::Capacity,
};
use rand::{Rng, random_range, rng};

#[inline]
fn capacity_add<'a>(a: &'a Capacity, b: &'a Capacity) -> AmountSum<&'a Amount, &'a Amount> {
    a + b
}

#[inline]
fn amount_sum_nested_no_dynamic<'a>(a: &'a Amount, b: &'a Amount, c: &Amount) -> bool {
    c >= &AmountSum {
        lhs: &AmountSum { lhs: a, rhs: b },
        rhs: &AmountSum { lhs: a, rhs: b },
    }
}

fn capacity_benchmark(c: &mut Criterion) -> () {
    let first = Capacity::from_vec(vec![10.0, 15.0, 25.0]);
    let second = Capacity::from_vec(vec![45.3, 15.2, 190.1]);
    let third: Amount = (&first + &second).into();
    c.bench_function("capacity add (smallvec)", |b| {
        b.iter(|| capacity_add(black_box(&first), black_box(&second)))
    });

    c.bench_function("amount_sum_nested_no_dynamic", |b| {
        b.iter(|| {
            amount_sum_nested_no_dynamic(black_box(&first), black_box(&second), black_box(&third))
        });
    });
}

fn satisfies_demand_zip(capacity: &Capacity, demand: &Capacity) -> bool {
    if capacity.len() < demand.len() {
        return false;
    }

    demand.iter().zip(capacity.iter()).all(|(d, c)| d <= c)
}

fn satisfies_demand_loop(capacity: &Capacity, demand: &Capacity) -> bool {
    if capacity.len() < demand.len() {
        return false;
    }

    for i in 0..demand.len() {
        if capacity[i] < demand[i] {
            return false;
        }
    }

    true
}

fn satisfies_demand_benchmark(c: &mut Criterion) -> () {
    let capacity = Capacity::from_vec(vec![10.0, 5.0, 8.0]);
    let demand = Capacity::from_vec(vec![5.0, 3.0, 2.0]);

    c.bench_function("satisfies demand zip", |b| {
        b.iter(|| satisfies_demand_zip(black_box(&capacity), black_box(&demand)))
    });

    c.bench_function("satisfies demand loop", |b| {
        b.iter(|| satisfies_demand_loop(black_box(&capacity), black_box(&demand)))
    });
}

pub fn over_capacity_demand_for_loop<C, D>(capacity: &C, demand: &D) -> f64
where
    C: AmountExpression,
    D: AmountExpression,
{
    let mut over_capacity = 0.0;

    for i in 0..demand.len() {
        if capacity.get(i) < demand.get(i) {
            over_capacity += demand.get(i) - capacity.get(i);
        }
    }

    over_capacity
}

pub fn over_capacity_demand_zip<C, D>(capacity: &C, demand: &D) -> f64
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

fn over_capacity_demand_benchmark(c: &mut Criterion) -> () {
    let capacity = Capacity::from_vec(vec![10.0, 5.0, 8.0]);
    let demand = Capacity::from_vec(vec![5.0, 3.0, 2.0]);

    c.bench_function("over_capacity_demand_for_loop", |b| {
        b.iter(|| over_capacity_demand_for_loop(black_box(&capacity), black_box(&demand)))
    });

    c.bench_function("over_capacity_demand_zip", |b| {
        b.iter(|| over_capacity_demand_zip(black_box(&capacity), black_box(&demand)))
    });
}

fn find_in_set_benchmark(c: &mut Criterion) -> () {
    let set: FxHashSet<usize> = (0..100).map(|_| rng().random_range(0..500)).collect();
    let vector: Vec<usize> = (0..100).map(|_| rng().random_range(0..500)).collect();

    c.bench_function("find in set", |b| {
        b.iter(|| set.contains(&rng().random_range(0..500)))
    });

    c.bench_function("find in vector", |b| {
        b.iter(|| vector.contains(&rng().random_range(0..500)))
    });
}

criterion_group!(
    benches,
    capacity_benchmark,
    satisfies_demand_benchmark,
    over_capacity_demand_benchmark,
    find_in_set_benchmark,
);
criterion_main!(benches);
