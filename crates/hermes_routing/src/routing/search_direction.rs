#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(crate) enum SearchDirection {
    Forward,
    Backward,
}

impl SearchDirection {
    pub fn opposite(&self) -> Self {
        match self {
            SearchDirection::Forward => SearchDirection::Backward,
            SearchDirection::Backward => SearchDirection::Forward,
        }
    }
}
