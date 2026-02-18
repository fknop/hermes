use fxhash::FxHashSet;

#[derive(Debug, Clone, PartialEq)]
pub struct BitSet {
    bitset: fixedbitset::FixedBitSet,
}

impl BitSet {
    pub fn with_capacity(capacity: usize) -> Self {
        BitSet {
            bitset: fixedbitset::FixedBitSet::with_capacity(capacity),
        }
    }

    pub fn empty() -> Self {
        BitSet {
            bitset: fixedbitset::FixedBitSet::with_capacity(0),
        }
    }

    pub fn from_registry<T: Eq + std::hash::Hash>(registry: &[T], skills: &FxHashSet<T>) -> Self {
        let mut bitset = BitSet::with_capacity(registry.len());

        for (i, skill) in registry.iter().enumerate() {
            bitset.set(i, skills.contains(skill));
        }

        bitset
    }

    pub fn intersects(&self, other: &BitSet) -> bool {
        self.bitset.intersection_count(&other.bitset) > 0
    }

    pub fn set(&mut self, index: usize, value: bool) {
        self.bitset.set(index, value);
    }

    pub fn union_with(&mut self, other: &BitSet) {
        self.bitset.union_with(&other.bitset)
    }

    pub fn insert(&mut self, index: usize) {
        self.bitset.insert(index);
    }

    pub fn is_subset(&self, other: &BitSet) -> bool {
        self.bitset.is_subset(&other.bitset)
    }

    pub(super) fn internal_bitset(&self) -> &fixedbitset::FixedBitSet {
        &self.bitset
    }
}

impl<'a> From<&'a BitSet> for &'a fixedbitset::FixedBitSet {
    fn from(bitset: &'a BitSet) -> Self {
        &bitset.bitset
    }
}
