use crate::{
    distance::{Distance, Meters},
    properties::property_map::EdgePropertyMap,
    types::NodeId,
};

pub trait GraphEdge {
    fn start_node(&self) -> NodeId;
    fn end_node(&self) -> NodeId;
    fn adj_node(&self, node: NodeId) -> NodeId {
        if self.start_node() == node {
            self.end_node()
        } else {
            self.start_node()
        }
    }

    fn distance(&self) -> Distance<Meters>;
    fn properties(&self) -> &EdgePropertyMap;
}
