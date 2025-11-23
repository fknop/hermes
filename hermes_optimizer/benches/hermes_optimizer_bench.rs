use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use hermes_optimizer::problem::{
    amount::{AmountExpression, AmountSub, AmountSum},
    capacity::Capacity,
};
use std::ops::AddAssign;

#[inline]
fn capacity_add<'a>(a: &'a Capacity, b: &'a Capacity) -> AmountSum<'a, Capacity, Capacity> {
    a + b
}

fn capacity_benchmark(c: &mut Criterion) -> () {
    let first = Capacity::from_vec(vec![10.0, 15.0, 25.0]);
    let second = Capacity::from_vec(vec![45.3, 15.2, 190.1]);
    c.bench_function("capacity add (smallvec)", |b| {
        b.iter(|| capacity_add(black_box(&first), black_box(&second)))
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

criterion_group!(benches, capacity_benchmark, satisfies_demand_benchmark);
criterion_main!(benches);
