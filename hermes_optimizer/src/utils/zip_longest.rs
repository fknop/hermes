use std::{cmp, iter::Fuse};

/// Inspired from https://github.com/rust-itertools/itertools/blob/master/src/zip_longest.rs
pub struct ZipLongest<T, U> {
    lhs: Fuse<T>,
    rhs: Fuse<U>,
}

pub fn zip_longest<T, U>(lhs: T, rhs: U) -> ZipLongest<T, U>
where
    T: Iterator,
    U: Iterator,
{
    ZipLongest {
        lhs: lhs.fuse(),
        rhs: rhs.fuse(),
    }
}

impl<T, U> Iterator for ZipLongest<T, U>
where
    T: Iterator,
    U: Iterator,
{
    type Item = (Option<T::Item>, Option<U::Item>);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match (self.lhs.next(), self.rhs.next()) {
            (None, None) => None,
            (Some(a), None) => Some((Some(a), None)),
            (None, Some(b)) => Some((None, Some(b))),
            (Some(a), Some(b)) => Some((Some(a), Some(b))),
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (a_lower, a_upper) = self.lhs.size_hint();
        let (b_lower, b_upper) = self.rhs.size_hint();

        let lower = cmp::max(a_lower, b_lower);

        let upper = match (a_upper, b_upper) {
            (Some(x), Some(y)) => Some(cmp::max(x, y)),
            _ => None,
        };

        (lower, upper)
    }
}
