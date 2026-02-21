
use crate::utils::bitset::BitSet;

#[derive(Clone)]
pub struct SparseTable {
    table: Vec<Vec<BitSet>>,
    len: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bitset(bits: &[usize], size: usize) -> BitSet {
        let mut bs = BitSet::with_capacity(size);
        for &b in bits {
            bs.insert(b);
        }
        bs
    }

    // --- build / empty ---

    #[test]
    fn empty_table_has_zero_len() {
        let table = SparseTable::empty();
        assert_eq!(table.len, 0);
        assert!(table.table.is_empty());
    }

    #[test]
    fn build_single_element() {
        let bs = bitset(&[0, 2], 4);
        let table = SparseTable::build(vec![bs.clone()]);
        assert_eq!(table.len, 1);
        // Only one level: the base row
        assert_eq!(table.table.len(), 1);
        assert_eq!(table.table[0][0], bs);
    }

    #[test]
    fn build_two_elements_unions_correctly() {
        // [0b0101, 0b1010] → level-1 entry should be 0b1111
        let a = bitset(&[0, 2], 4);
        let b = bitset(&[1, 3], 4);
        let table = SparseTable::build(vec![a.clone(), b.clone()]);
        assert_eq!(table.len, 2);

        let expected = bitset(&[0, 1, 2, 3], 4);
        assert_eq!(table.table[1][0], expected);
    }

    #[test]
    #[should_panic]
    fn build_panics_on_empty_input() {
        SparseTable::build(vec![]);
    }

    // range_covered_by(i, j, query) returns true iff for every element e in [i..=j]:
    //   e.skills ⊆ query  (i.e. the element only uses skills present in query)

    // --- range_covered_by (single element, i == j) ---

    #[test]
    fn range_covered_by_single_true_when_skills_within_query() {
        // element at index 1 has skills {1, 2}; query is {1, 2, 3} → {1,2} ⊆ {1,2,3} → true
        let bitsets = vec![bitset(&[0], 4), bitset(&[1, 2], 4), bitset(&[3], 4)];
        let table = SparseTable::build(bitsets);

        let query = bitset(&[1, 2, 3], 4);
        assert!(table.range_covered_by(1, 1, &query));
    }

    #[test]
    fn range_covered_by_single_false_when_element_has_extra_skill() {
        // element at index 1 has skills {1, 2}; query is {1} → {1,2} ⊄ {1} → false
        let bitsets = vec![bitset(&[0], 4), bitset(&[1, 2], 4), bitset(&[3], 4)];
        let table = SparseTable::build(bitsets);

        let query = bitset(&[1], 4);
        assert!(!table.range_covered_by(1, 1, &query));
    }

    #[test]
    fn range_covered_by_single_true_when_skills_match_exactly() {
        let bitsets = vec![bitset(&[1, 2], 4)];
        let table = SparseTable::build(bitsets);

        let query = bitset(&[1, 2], 4);
        assert!(table.range_covered_by(0, 0, &query));
    }

    #[test]
    fn range_covered_by_single_empty_element_always_true() {
        // An element with no skills is a subset of any query.
        let bitsets = vec![bitset(&[], 4)];
        let table = SparseTable::build(bitsets);

        let query = bitset(&[], 4);
        assert!(table.range_covered_by(0, 0, &query));

        let query2 = bitset(&[0, 1], 4);
        assert!(table.range_covered_by(0, 0, &query2));
    }

    // --- range_covered_by (multi-element range) ---

    #[test]
    fn range_covered_by_range_true_when_every_element_within_query() {
        // All elements use only bits 0 and 1; query covers {0,1,2,3}.
        let bitsets = vec![
            bitset(&[0, 1], 4),
            bitset(&[0], 4),
            bitset(&[1], 4),
            bitset(&[0, 1], 4),
        ];
        let table = SparseTable::build(bitsets);

        let query = bitset(&[0, 1, 2, 3], 4);
        assert!(table.range_covered_by(0, 3, &query));
    }

    #[test]
    fn range_covered_by_range_false_when_one_element_has_extra_bit() {
        // Element at index 1 has bit 2 which is not in the query.
        let bitsets = vec![
            bitset(&[0, 1], 4),
            bitset(&[0, 1, 2], 4), // has extra bit 2
            bitset(&[0, 1], 4),
        ];
        let table = SparseTable::build(bitsets);

        let query = bitset(&[0, 1], 4);
        assert!(!table.range_covered_by(0, 2, &query));
    }

    #[test]
    fn range_covered_by_subrange_excludes_bad_element() {
        // Only element at index 1 has the extra bit; subranges that skip it should pass.
        let bitsets = vec![
            bitset(&[0, 1], 4),
            bitset(&[0, 1, 2], 4), // extra bit 2
            bitset(&[0, 1], 4),
        ];
        let table = SparseTable::build(bitsets);

        let query = bitset(&[0, 1], 4);
        assert!(table.range_covered_by(0, 0, &query));
        assert!(table.range_covered_by(2, 2, &query));
        assert!(!table.range_covered_by(0, 2, &query));
    }

    #[test]
    fn range_covered_by_power_of_two_length() {
        // 4 elements – exercises the overlapping-block path inside range_covered_by
        let bitsets = vec![
            bitset(&[0, 1], 4),
            bitset(&[0, 1], 4),
            bitset(&[0, 1], 4),
            bitset(&[0, 1], 4),
        ];
        let table = SparseTable::build(bitsets);

        let query = bitset(&[0, 1, 2, 3], 4);
        assert!(table.range_covered_by(0, 3, &query));
        assert!(table.range_covered_by(1, 2, &query));

        let tight_query = bitset(&[0, 1], 4);
        assert!(table.range_covered_by(0, 3, &tight_query));
    }

    #[test]
    fn range_covered_by_large_table() {
        // 8 elements; element 5 has an extra bit (bit 3) not in the query.
        let mut bitsets: Vec<BitSet> = (0..8).map(|_| bitset(&[0, 1, 2], 4)).collect();
        bitsets[5] = bitset(&[0, 1, 2, 3], 4); // extra bit 3

        let table = SparseTable::build(bitsets);
        let query = bitset(&[0, 1, 2], 4);

        assert!(table.range_covered_by(0, 4, &query)); // does not include index 5
        assert!(!table.range_covered_by(0, 5, &query)); // includes index 5
        assert!(!table.range_covered_by(5, 7, &query)); // includes index 5
        assert!(table.range_covered_by(6, 7, &query)); // does not include index 5
    }
}

impl SparseTable {
    pub fn empty() -> Self {
        Self {
            len: 0,
            table: Vec::with_capacity(0),
        }
    }

    pub fn build(bitsets: Vec<BitSet>) -> Self {
        let n = bitsets.len();
        assert!(n > 0);

        let max_k = if n > 1 { n.ilog2() as usize + 1 } else { 1 };
        let mut table: Vec<Vec<BitSet>> = Vec::with_capacity(max_k);

        table.push(bitsets);

        for k in 1..max_k {
            let half = 1 << (k - 1);
            let prev = &table[k - 1];
            let row = (0..=n - (1 << k))
                .map(|i| {
                    let mut bits = prev[i].clone();
                    bits.union_with(&prev[i + half]);
                    bits
                })
                .collect();
            table.push(row);
        }

        Self { table, len: n }
    }

    pub fn with_len(len: usize) -> Self {
        Self {
            len,
            table: Vec::with_capacity(Self::max_k_from_len(len)),
        }
    }

    fn max_k_from_len(len: usize) -> usize {
        if len > 1 { len.ilog2() as usize + 1 } else { 1 }
    }

    pub fn range_covered_by(&self, i: usize, j: usize, bitset: &BitSet) -> bool {
        debug_assert!(i <= j && j < self.len);

        if i == j {
            self.table[0][i].is_subset(bitset.into())
        } else {
            let k = (j - i + 1).ilog2() as usize;
            let a = &self.table[k][i];
            let b = &self.table[k][j + 1 - (1 << k)];

            a.is_subset(bitset.into()) && b.is_subset(bitset.into())
        }
    }
}
