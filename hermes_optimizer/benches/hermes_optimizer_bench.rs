use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use hermes_optimizer::problem::capacity::Capacity;

#[inline]
fn capacity_add(a: &Capacity, b: &Capacity) -> Capacity {
    a + b
}

fn criterion_benchmark(c: &mut Criterion) -> () {
    let first = Capacity::from_vec(vec![10.0, 15.0, 25.0]);
    let second = Capacity::from_vec(vec![45.3, 15.2, 190.1]);
    c.bench_function("capacity add", |b| {
        b.iter(|| capacity_add(black_box(&first), black_box(&second)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
