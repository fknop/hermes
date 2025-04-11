use std::collections::BinaryHeap;

use crate::{base_graph::BaseGraph, graph::Graph, types::NodeId, weighting::Weighting};

use super::{
    ch_graph::{self, CHGraph},
    node_contractor,
    preparation_graph::{CHPreparationGraph, PreparationGraphWeighting},
    witness_search::{self, WitnessSearch},
};

struct CHGraphBuilder {
    ch_graph: CHGraph,
    preparation_graph: CHGraph,
}

#[derive(Eq, PartialEq, PartialOrd)]
struct NodeWithPriority {
    node_id: NodeId,
    priority: i16,
}

impl Ord for NodeWithPriority {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.priority.cmp(&self.priority)
    }
}

fn build_ch_graph<W>(base_graph: &BaseGraph, weighting: &W) -> CHGraph
where
    W: Weighting<BaseGraph>,
{
    // let mut ch_graph = CHGraph::new();
    let mut preparation_graph = CHPreparationGraph::new(base_graph);
    let preparation_weighting = PreparationGraphWeighting::new(weighting);
    let mut witness_search = WitnessSearch::new(&preparation_graph);
    let mut priority_queue = BinaryHeap::with_capacity(base_graph.node_count());

    for node_id in 0..base_graph.node_count() {
        let priority = node_contractor::calc_priority(
            &mut preparation_graph,
            &mut witness_search,
            &preparation_weighting,
            node_id,
        );

        priority_queue.push(NodeWithPriority { node_id, priority });
    }

    let mut rank = 0;
    while let Some(NodeWithPriority { node_id, priority }) = priority_queue.pop() {
        // Lazy recomputation of the priority
        // If the recomputed priority is less than the next node to be contracted, we re-enqueue the node
        let recomputed_priority = node_contractor::calc_priority(
            &mut preparation_graph,
            &mut witness_search,
            &preparation_weighting,
            node_id,
        );

        if let Some(NodeWithPriority {
            priority: least_priority,
            ..
        }) = priority_queue.peek()
        {
            if recomputed_priority > *least_priority {
                priority_queue.push(NodeWithPriority {
                    node_id,
                    priority: recomputed_priority,
                });
                continue;
            }
        }

        node_contractor::contract_node(
            &mut preparation_graph,
            &mut witness_search,
            &preparation_weighting,
            node_id,
        );

        rank += 1;
    }

    ()
}

impl CHGraphBuilder {
    pub fn new(preparation_graph: CHGraph) -> Self {
        CHGraphBuilder {
            ch_graph: CHGraph::new(),
            preparation_graph,
        }
    }
}
