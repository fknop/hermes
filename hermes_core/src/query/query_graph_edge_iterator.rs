use crate::types::EdgeId;

/// An iterator that combines base edges and virtual edges from a QueryGraph
///
/// This iterator will first yield all base edges, followed by all virtual edges.
/// It is used internally by the QueryGraph to provide a unified view of both
/// the original graph edges and dynamically added virtual edges.
pub struct QueryGraphEdgeIterator<'a> {
    base_edges: &'a [EdgeId],
    virtual_edges: &'a [EdgeId],
    index: usize,
}

impl<'a> QueryGraphEdgeIterator<'a> {
    pub fn new(base_edges: &'a [EdgeId], virtual_edges: &'a [EdgeId]) -> Self {
        QueryGraphEdgeIterator {
            base_edges,
            virtual_edges,
            index: 0,
        }
    }
}

impl Iterator for QueryGraphEdgeIterator<'_> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.base_edges.len() {
            let edge = self.base_edges[self.index];
            self.index += 1;
            return Some(edge);
        }

        let virtual_index = self.index - self.base_edges.len();

        if virtual_index < self.virtual_edges.len() {
            let edge = self.virtual_edges[virtual_index];
            self.index += 1;
            return Some(edge);
        }

        None
    }
}
