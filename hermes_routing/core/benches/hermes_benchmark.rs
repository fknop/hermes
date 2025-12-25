use std::collections::HashMap;

use criterion::{Criterion, black_box, criterion_group, criterion_main};

use fxhash::FxHashMap;

#[derive(Eq, Hash, PartialEq, Clone, Debug)]
enum Property {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    // P,
    // Q,
    // R,
    // S,
    // T,
    // U,
    // V,
    // W,
    // X,
    // Y,
    // Z,
}

#[derive(Default)]
struct SmallMap<T>(Vec<(Property, T)>);
impl<T> SmallMap<T> {
    fn get(&self, property: &Property) -> Option<&T> {
        self.0.iter().find(|(p, _)| p == property).map(|(_, v)| v)
    }

    fn insert(&mut self, property: Property, value: T) {
        self.0.push((property, value));
    }
}

fn small_map_benchmark(c: &mut Criterion) {
    let mut map: SmallMap<u32> = SmallMap::default();
    map.insert(Property::A, 1);
    map.insert(Property::B, 2);
    map.insert(Property::C, 3);
    map.insert(Property::D, 4);

    c.bench_function("SmallMap get", |b| {
        b.iter(|| black_box(map.get(&Property::C)))
    });

    map.insert(Property::E, 2);
    map.insert(Property::F, 2);
    map.insert(Property::G, 2);
    map.insert(Property::H, 2);
    map.insert(Property::I, 2);
    map.insert(Property::J, 2);
    map.insert(Property::K, 2);
    map.insert(Property::L, 2);
    map.insert(Property::M, 2);
    map.insert(Property::N, 2);
    map.insert(Property::O, 2);

    c.bench_function("SmallMap15Items get", |b| {
        b.iter(|| black_box(map.get(&Property::N)))
    });
}

fn hashmap_benchmark(c: &mut Criterion) {
    let mut map: HashMap<Property, u32> = HashMap::default();
    map.insert(Property::A, 1);
    map.insert(Property::B, 2);
    map.insert(Property::C, 3);
    map.insert(Property::D, 4);

    c.bench_function("HashMap get", |b| {
        b.iter(|| black_box(map.get(&Property::C)))
    });

    map.insert(Property::E, 2);
    map.insert(Property::F, 2);
    map.insert(Property::G, 2);
    map.insert(Property::H, 2);
    map.insert(Property::I, 2);
    map.insert(Property::J, 2);
    map.insert(Property::K, 2);
    map.insert(Property::L, 2);
    map.insert(Property::M, 2);
    map.insert(Property::N, 2);
    map.insert(Property::O, 2);

    c.bench_function("HashMap15Items get", |b| {
        b.iter(|| black_box(map.get(&Property::O)))
    });
}

fn fxhashmap_benchmark(c: &mut Criterion) {
    let mut map: FxHashMap<Property, u32> = FxHashMap::default();
    map.insert(Property::A, 1);
    map.insert(Property::B, 2);
    map.insert(Property::C, 3);
    map.insert(Property::D, 4);

    c.bench_function("FxHashMap get", |b| {
        b.iter(|| black_box(map.get(&Property::C)))
    });

    map.insert(Property::E, 5);
    map.insert(Property::F, 6);
    map.insert(Property::G, 7);
    map.insert(Property::H, 8);
    map.insert(Property::I, 9);
    map.insert(Property::J, 10);
    map.insert(Property::K, 11);
    map.insert(Property::L, 12);
    map.insert(Property::M, 13);
    map.insert(Property::N, 14);
    map.insert(Property::O, 15);

    c.bench_function("FxHashMap15Items get", |b| {
        b.iter(|| black_box(map.get(&Property::O)))
    });
}

criterion_group!(
    benches,
    small_map_benchmark,
    hashmap_benchmark,
    fxhashmap_benchmark
);
criterion_main!(benches);
