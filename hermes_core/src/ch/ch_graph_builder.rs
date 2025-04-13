use std::{cmp, collections::BinaryHeap};

use crate::{
    base_graph::BaseGraph, ch::priority_queue::PriorityQueue, edge_direction::EdgeDirection,
    graph::Graph, graph_edge::GraphEdge, types::NodeId, weighting::Weighting,
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

// #[derive(Eq, PartialEq)]
// struct NodeWithPriority {
//     node_id: NodeId,
//     priority: i32,
// }

// impl PartialOrd for NodeWithPriority {
//     fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
//         Some(self.cmp(other))
//     }
// }

// impl Ord for NodeWithPriority {
//     fn cmp(&self, other: &Self) -> std::cmp::Ordering {
//         other.priority.cmp(&self.priority)
//     }
// }

pub fn build_ch_graph<W>(base_graph: &BaseGraph, weighting: &W) -> CHGraph
where
    W: Weighting<BaseGraph>,
{
    let mut ch_graph = CHGraph::new(base_graph);
    let mut preparation_graph = CHPreparationGraph::new(base_graph, weighting);
    let preparation_weighting = PreparationGraphWeighting::new(weighting);
    let mut witness_search = WitnessSearch::new();
    let mut priority_queue = PriorityQueue::new(base_graph.node_count());
    let mut ranks = vec![0; base_graph.node_count()];

    println!("Start CH contraction");
    println!("Edge count {}", base_graph.edge_count());
    println!("Node count {}", base_graph.node_count());

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

        priority_queue
            .push(node_id, priority)
            .unwrap_or_else(|err| panic!("{}", err));
    }

    println!("Finish computing priority for every node");

    // let mut contracted_nodes = 0;
    let mut added_shortcuts = 0;
    let mut contracted_nodes = 0;

    while let Some((node_id, priority)) = priority_queue.pop() {
        // Lazy recomputation of the priority
        // If the recomputed priority is less than the next node to be contracted, we re-enqueue the node

        if contracted_nodes > 1800000 {
            println!("Remaining nodes {}", priority_queue.len());
        }

        let n = preparation_graph.outgoing_edges(node_id).len()
            * preparation_graph.incoming_edges(node_id).len();

        if priority_queue.len() < 100 {
            println!(
                "N {} - incoming {} - outgoing {}",
                n,
                preparation_graph.incoming_edges(node_id).len(),
                preparation_graph.outgoing_edges(node_id).len()
            );

            println!("added shortcuts {}", added_shortcuts);
            println!(
                "incoming shortcuts {}",
                preparation_graph
                    .incoming_edges(node_id)
                    .iter()
                    .filter(|edge_id| preparation_graph.is_shortcut(**edge_id))
                    .count()
            );
            println!(
                "outgoing shortcuts {}",
                preparation_graph
                    .outgoing_edges(node_id)
                    .iter()
                    .filter(|edge_id| preparation_graph.is_shortcut(**edge_id))
                    .count()
            );
        }

        if priority != i32::MIN {
            if let Some((_, least_priority)) = priority_queue.peek() {
                let recomputed_priority = node_contractor::calc_priority(
                    &mut preparation_graph,
                    &mut witness_search,
                    &preparation_weighting,
                    ranks[node_id],
                    node_id,
                );

                if recomputed_priority > *least_priority {
                    priority_queue
                        .push(node_id, recomputed_priority)
                        .unwrap_or_else(|err| panic!("{}", err));
                    continue;
                }
            }
        }

        let mut neighbors = Vec::new();

        for edge_id in preparation_graph.node_edges_iter(node_id) {
            let edge = preparation_graph.edge(edge_id);
            let adj_node = edge.adj_node(node_id);

            if node_id != adj_node {
                neighbors.push(adj_node);
            }

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

        // Only contract 95% of nodes
        //

        contracted_nodes += 1;
        if contracted_nodes > 0 && contracted_nodes % 10 == 0 {
            preparation_graph.disconnect_node(node_id);
            continue;
        }

        //

        node_contractor::contract_node(
            &mut preparation_graph,
            &mut witness_search,
            &preparation_weighting,
            node_id,
        );

        // TODO: better condition
        if contracted_nodes % 500000 == 0 && added_shortcuts > 0 {
            println!("Recompute all remaining priorities");
            let remaining_nodes: Vec<(NodeId, i32)> = priority_queue.to_vec();
            priority_queue.clear();
            for (node_id, _) in remaining_nodes {
                let priority = node_contractor::calc_priority(
                    &mut preparation_graph,
                    &mut witness_search,
                    &preparation_weighting,
                    ranks[node_id],
                    node_id,
                );

                priority_queue
                    .push(node_id, priority)
                    .unwrap_or_else(|err| panic!("{}", err));
            }
        } else {
            // let max_neighbor_update = 3;
            for neighbor in neighbors {
                let recomputed_neighbor_priority = node_contractor::calc_priority(
                    &mut preparation_graph,
                    &mut witness_search,
                    &preparation_weighting,
                    ranks[neighbor],
                    neighbor,
                );

                priority_queue.update_priority(neighbor, recomputed_neighbor_priority);
            }
        }

        if contracted_nodes % 100000 == 0 {
            println!("Contracted nodes {}", contracted_nodes);
            println!("added shortcuts {}", added_shortcuts)
        }
    }

    println!("Finished contraction");
    println!(
        "Added {} shortcuts for {} base edges",
        added_shortcuts,
        base_graph.edge_count()
    );

    ch_graph
}
