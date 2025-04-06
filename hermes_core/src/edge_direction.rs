#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum EdgeDirection {
    Forward,
    Backward,
}

impl EdgeDirection {
    pub fn opposite(&self) -> Self {
        match self {
            EdgeDirection::Forward => EdgeDirection::Backward,
            EdgeDirection::Backward => EdgeDirection::Forward,
        }
    }
}
