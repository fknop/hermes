use std::{cell::RefCell, hint::black_box};

use criterion::{Criterion, criterion_group, criterion_main};
use fxhash::FxHashSet;
use hermes_optimizer::problem::{
    amount::{Amount, AmountExpression, AmountSum},
    capacity::Capacity,
};
use rand::{Rng, SeedableRng, rng, rngs::SmallRng};
use thread_local::ThreadLocal;

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

fn sort_benchmark(c: &mut Criterion) -> () {
    let base_vec: Vec<usize> = (0..1000).map(|_| rng().random_range(0..5000)).collect();

    c.bench_function("sort unstable", |b| {
        let mut vec = base_vec.clone();
        b.iter(|| {
            vec.sort_unstable();
            black_box(&vec);
        })
    });

    c.bench_function("sort stable", |b| {
        let mut vec = base_vec.clone();
        b.iter(|| {
            vec.sort();
            black_box(&vec);
        })
    });
}

fn bench_direct_access(c: &mut Criterion) {
    const NUM_VECS: usize = 4;
    const VEC_LEN: usize = 10_000;
    const TOTAL_LEN: usize = NUM_VECS * VEC_LEN;

    // Nested: 4 vecs of 10,000 elements
    let nested: Vec<Vec<u64>> = (0..NUM_VECS)
        .map(|i| ((i * VEC_LEN) as u64..(i * VEC_LEN + VEC_LEN) as u64).collect())
        .collect();

    // Flat: 40,000 elements
    let flat: Vec<u64> = (0..TOTAL_LEN as u64).collect();

    // Pre-generate random indices
    let mut rng = SmallRng::seed_from_u64(42);
    let accesses: Vec<(usize, usize)> = (0..10_000)
        .map(|_| {
            let outer = rng.random_range(0..NUM_VECS);
            let inner = rng.random_range(0..VEC_LEN);
            (outer, inner)
        })
        .collect();

    let flat_indices: Vec<usize> = accesses
        .iter()
        .map(|(outer, inner)| outer * VEC_LEN + inner)
        .collect();

    let mut group = c.benchmark_group("direct_access");

    group.bench_function("nested_4x10000", |b| {
        b.iter(|| {
            let mut sum = 0u64;
            for &(outer, inner) in &accesses {
                sum += nested[outer][inner];
            }
            black_box(sum)
        })
    });

    group.bench_function("flat_40000", |b| {
        b.iter(|| {
            let mut sum = 0u64;
            for &idx in &flat_indices {
                sum += flat[idx];
            }
            black_box(sum)
        })
    });

    group.finish();
}

fn rng_bench(c: &mut Criterion) {
    let mut master_rng = SmallRng::seed_from_u64(42);

    let mut group = c.benchmark_group("rng_bench");

    group.bench_function("gen range", |b| {
        b.iter(|| {
            let range = master_rng.random_range(0..=1);
            black_box(range)
        })
    });

    group.bench_function("gen range from child", |b| {
        b.iter(|| {
            let mut child_rng = SmallRng::from_rng(&mut master_rng);
            let range = child_rng.random_range(0..=1);

            black_box(range)
        })
    });

    group.bench_function("gen range from clone", |b| {
        b.iter(|| {
            let mut child_rng = black_box(&master_rng).clone();
            let range = child_rng.random_range(0..=1);
            black_box(range)
        })
    });

    group.bench_function("gen range from mutex", |b| {
        let mutex = parking_lot::Mutex::new(master_rng.clone());
        b.iter(|| {
            let range = mutex.lock().random_range(0..=1);
            black_box(range)
        })
    });

    group.bench_function("gen range from thread_local", |b| {
        let tl: ThreadLocal<RefCell<SmallRng>> = ThreadLocal::new();
        b.iter(|| {
            let rng = tl.get_or(|| RefCell::new(SmallRng::from_rng(&mut master_rng)));
            let range = rng.borrow_mut().random_range(0..=1);
            black_box(range)
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    // bench_direct_access,
    // capacity_benchmark,
    // satisfies_demand_benchmark,
    // over_capacity_demand_benchmark,
    // find_in_set_benchmark,
    // sort_benchmark,
    rng_bench
);
criterion_main!(benches);
