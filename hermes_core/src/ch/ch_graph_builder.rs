use std::{cmp, collections::BinaryHeap};

use crate::{
    base_graph::BaseGraph, edge_direction::EdgeDirection, graph::Graph, graph_edge::GraphEdge,
    types::NodeId, weighting::Weighting,
};

use super::{
    ch_graph::{CHBaseEdge, CHGraph},
    node_contractor,
    preparation_graph::{CHPreparationGraph, CHPreparationGraphEdge, PreparationGraphWeighting},
    witness_search::WitnessSearch,
};

// struct CHGraphBuilder {
//     ch_graph: CHGraph,
//     preparation_graph: CHGraph,
// }

#[derive(Eq, PartialEq)]
struct NodeWithPriority {
    node_id: NodeId,
    priority: i32,
}

impl PartialOrd for NodeWithPriority {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for NodeWithPriority {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.priority.cmp(&self.priority)
    }
}

pub fn build_ch_graph<W>(base_graph: &BaseGraph, weighting: &W) -> CHGraph
where
    W: Weighting<BaseGraph>,
{
    let mut ch_graph = CHGraph::new(base_graph);
    let mut preparation_graph = CHPreparationGraph::new(base_graph, weighting);
    let preparation_weighting = PreparationGraphWeighting::new(weighting);
    let mut witness_search = WitnessSearch::new();
    let mut priority_queue = BinaryHeap::with_capacity(base_graph.node_count());
    let mut ranks = vec![0; base_graph.node_count()];

    println!("Start CH contraction");

    for node_id in 0..base_graph.node_count() {
        let priority = node_contractor::calc_priority(
            &mut preparation_graph,
            &mut witness_search,
            &preparation_weighting,
            0,
            node_id,
        );

        if node_id % 100000 == 0 {
            println!("Priority for node {} is {}", node_id, priority);
        }

        priority_queue.push(NodeWithPriority { node_id, priority });
    }

    println!("Finish computing priority for every node");

    println!("{}", priority_queue.peek().unwrap().priority);

    let mut rank = 0;
    let mut added_shortcuts = 0;
    while let Some(NodeWithPriority { node_id, priority }) = priority_queue.pop() {
        // Lazy recomputation of the priority
        // If the recomputed priority is less than the next node to be contracted, we re-enqueue the node

        if priority != i32::MIN {
            if let Some(NodeWithPriority {
                priority: least_priority,
                ..
            }) = priority_queue.peek()
            {
                let recomputed_priority = node_contractor::calc_priority(
                    &mut preparation_graph,
                    &mut witness_search,
                    &preparation_weighting,
                    ranks[node_id],
                    node_id,
                );

                if recomputed_priority > *least_priority {
                    priority_queue.push(NodeWithPriority {
                        node_id,
                        priority: recomputed_priority,
                    });
                    continue;
                }
            }
        }

        for edge_id in preparation_graph.node_edges_iter(node_id) {
            let edge = preparation_graph.edge(edge_id);
            let adj_node = edge.adj_node(node_id);

            ranks[adj_node] = cmp::max(ranks[adj_node], ranks[node_id] + 1);

            let direction = preparation_graph.edge_direction(edge_id, node_id);

            // TODO: loop over incoming/outgoing edges
            match edge {
                CHPreparationGraphEdge::Edge(base_edge) => {
                    // TODO: make sure directions are correct
                    // From start to end
                    let forward_weight = weighting.calc_edge_weight(base_edge, direction);
                    let forward_time = weighting.calc_edge_ms(base_edge, direction);

                    // From end to start
                    let backward_weight =
                        weighting.calc_edge_weight(base_edge, direction.opposite());
                    let backward_time = weighting.calc_edge_ms(base_edge, direction.opposite());

                    ch_graph.add_edge(CHBaseEdge {
                        edge_id: base_edge.id(),
                        start: node_id,
                        end: adj_node,
                        distance: base_edge.distance(),
                        forward_time,
                        backward_time,
                        forward_weight,
                        backward_weight,
                    });
                }
                CHPreparationGraphEdge::Shortcut(shortcut) => {
                    ch_graph.add_shortcut(shortcut.clone());
                    added_shortcuts += 1;
                }
            }
        }

        rank += 1;
        if rank % 100000 == 0 {
            println!("Contracted nodes {}", rank);
            println!("added shortcuts {}", added_shortcuts)
        }

        node_contractor::contract_node(
            &mut preparation_graph,
            &mut witness_search,
            &preparation_weighting,
            node_id,
        );
    }

    println!("Finished contraction");
    println!(
        "Added {} shortcuts for {} base edges",
        added_shortcuts,
        base_graph.edge_count()
    );

    ch_graph
}

// impl CHGraphBuilder {
//     pub fn new(preparation_graph: CHGraph) -> Self {
//         CHGraphBuilder {
//             ch_graph: CHGraph::new(),
//             preparation_graph,
//         }
//     }
// }
