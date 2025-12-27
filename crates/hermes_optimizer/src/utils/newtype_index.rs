#[macro_export]
macro_rules! define_index_newtype {
    ($name:ident, $t:ident) => {
        #[derive(
            serde::Serialize, Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default,
        )]
        pub struct $name(usize);

        impl $name {
            pub const fn new(index: usize) -> Self {
                Self(index)
            }

            pub const fn get(&self) -> usize {
                self.0
            }
        }

        impl From<usize> for $name {
            fn from(index: usize) -> Self {
                Self(index)
            }
        }

        impl std::ops::Index<$name> for Vec<$t> {
            type Output = $t;
            fn index(&self, index: $name) -> &Self::Output {
                &self[index.0]
            }
        }

        impl std::ops::IndexMut<$name> for Vec<$t> {
            fn index_mut(&mut self, index: $name) -> &mut Self::Output {
                &mut self[index.0]
            }
        }

        impl std::ops::Index<$name> for [$t] {
            type Output = $t;
            fn index(&self, index: $name) -> &Self::Output {
                &self[index.0]
            }
        }

        pub struct IndexedIter<I> {
            inner: std::iter::Enumerate<I>,
            _marker: std::marker::PhantomData<$name>,
        }

        pub trait EnumerateIdx: Iterator + Sized {
            fn enumerate_idx(self) -> IndexedIter<Self> {
                IndexedIter {
                    inner: self.enumerate(),
                    _marker: std::marker::PhantomData,
                }
            }
        }

        impl<I: Iterator> EnumerateIdx for I {}

        impl<I: Iterator> Iterator for IndexedIter<I> {
            type Item = ($name, I::Item);

            fn next(&mut self) -> Option<Self::Item> {
                self.inner.next().map(|(i, item)| ($name::from(i), item))
            }

            fn size_hint(&self) -> (usize, Option<usize>) {
                self.inner.size_hint()
            }
        }
    };
}
