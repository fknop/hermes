use fxhash::FxHashSet;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Skill(String);

impl Skill {
    pub fn new(skill: String) -> Self {
        Skill(skill)
    }
}

impl std::fmt::Display for Skill {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct SkillBitset {
    bitset: fixedbitset::FixedBitSet,
}

impl SkillBitset {
    pub fn new(capacity: usize) -> Self {
        SkillBitset {
            bitset: fixedbitset::FixedBitSet::with_capacity(capacity),
        }
    }

    pub fn empty() -> Self {
        SkillBitset {
            bitset: fixedbitset::FixedBitSet::with_capacity(0),
        }
    }

    pub fn from_registry(registry: &[Skill], skills: &FxHashSet<Skill>) -> Self {
        let mut bitset = SkillBitset::new(registry.len());

        for (i, skill) in registry.iter().enumerate() {
            bitset.set(i, skills.contains(skill));
        }

        bitset
    }

    pub fn intersects(&self, other: &SkillBitset) -> bool {
        self.bitset.intersection_count(&other.bitset) > 0
    }

    pub fn set(&mut self, index: usize, value: bool) {
        self.bitset.set(index, value);
    }

    pub fn bitset(&self) -> &fixedbitset::FixedBitSet {
        &self.bitset
    }
}
