use fxhash::FxHashSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BitSet {
    repr: fixedbitset::FixedBitSet,
}

impl BitSet {
    pub fn with_capacity(capacity: usize) -> Self {
        BitSet {
            repr: fixedbitset::FixedBitSet::with_capacity(capacity),
        }
    }

    pub fn empty() -> Self {
        BitSet {
            repr: fixedbitset::FixedBitSet::with_capacity(0),
        }
    }

    pub fn from_registry<T: Eq + std::hash::Hash>(registry: &[T], skills: &FxHashSet<T>) -> Self {
        let mut bitset = BitSet::with_capacity(registry.len());

        for (i, skill) in registry.iter().enumerate() {
            bitset.set(i, skills.contains(skill));
        }

        bitset
    }

    pub fn is_all_zeroes(&self) -> bool {
        self.repr.is_clear()
    }

    pub fn clear(&mut self) {
        self.repr.clear();
    }

    pub fn ones(&self) -> Vec<usize> {
        self.repr.ones().collect()
    }

    pub fn intersects(&self, other: &BitSet) -> bool {
        self.repr.intersection_count(&other.repr) > 0
    }

    pub fn set(&mut self, index: usize, value: bool) {
        self.repr.set(index, value);
    }

    pub fn union_with(&mut self, other: &BitSet) {
        self.repr.union_with(&other.repr)
    }

    pub fn insert(&mut self, index: usize) {
        self.repr.insert(index);
    }

    pub fn is_subset(&self, other: &BitSet) -> bool {
        self.repr.is_subset(&other.repr)
    }

    pub fn difference_count(&self, other: &BitSet) -> usize {
        self.repr.difference_count(&other.repr)
    }
}
