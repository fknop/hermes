pub struct IndexedIter<I, Idx> {
    inner: std::iter::Enumerate<I>,
    _marker: std::marker::PhantomData<Idx>,
}

pub trait EnumerateIdx<Idx>: Iterator + Sized {
    fn enumerate_idx(self) -> IndexedIter<Self, Idx> {
        IndexedIter {
            inner: self.enumerate(),
            _marker: std::marker::PhantomData,
        }
    }
}

impl<I: Iterator, Idx> EnumerateIdx<Idx> for I {}

impl<I: Iterator, Idx: From<usize>> Iterator for IndexedIter<I, Idx> {
    type Item = (Idx, I::Item);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(i, item)| (Idx::from(i), item))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}
