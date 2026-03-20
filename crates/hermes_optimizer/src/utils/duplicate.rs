pub fn find_duplicate<'a, T: PartialEq + Eq + std::hash::Hash + ?Sized>(
    mut iter: impl Iterator<Item = &'a T>,
) -> Option<&'a T> {
    let mut seen = std::collections::HashSet::new();
    iter.find(|&item| !seen.insert(item))
}
