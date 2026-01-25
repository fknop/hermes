#[macro_export]
macro_rules! define_index_newtype {
    ($name:ident, $t:ident) => {
        #[derive(
            serde::Serialize, Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default,
        )]
        pub struct $name(usize);

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

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

        impl std::ops::Index<$name> for [&$t] {
            type Output = $t;
            fn index(&self, index: $name) -> &Self::Output {
                &self[index.0]
            }
        }
    };
}
