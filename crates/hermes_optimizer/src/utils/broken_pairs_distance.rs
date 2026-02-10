use fxhash::FxHashSet;

pub fn broken_pairs_distance<T: Eq + std::hash::Hash>(a: &FxHashSet<T>, b: &FxHashSet<T>) -> usize {
    let len = a.len().max(b.len());

    len - a.intersection(b).count()
}
