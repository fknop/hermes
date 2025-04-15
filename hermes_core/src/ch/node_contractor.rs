use crate::{
    ch::{preparation_graph, shortcut::PreparationShortcut},
    graph::Graph,
    graph_edge::GraphEdge,
    types::NodeId,
    weighting::Weighting,
};

use super::{preparation_graph::CHPreparationGraph, witness_search::WitnessSearch};

pub fn contract_node<'a>(
    graph: &mut CHPreparationGraph<'a>,
    witness_search: &mut WitnessSearch,
    weighting: &impl Weighting<CHPreparationGraph<'a>>,
    node: NodeId,
) {
    let shortcuts = find_shortcuts(graph, witness_search, weighting, node, 250);

    for shortcut in shortcuts {
        graph.add_shortcut(shortcut);
    }

    graph.disconnect_node(node);
}

pub fn calc_priority<'a>(
    graph: &mut CHPreparationGraph<'a>,
    witness_search: &mut WitnessSearch,
    weighting: &impl Weighting<CHPreparationGraph<'a>>,
    rank: usize,
    node: NodeId,
) -> i32 {
    let shortcuts = find_shortcuts(graph, witness_search, weighting, node, 250);

    let incoming_edges = graph.incoming_edges(node).len();
    let outgoing_edges = graph.outgoing_edges(node).len();

    let edges = incoming_edges + outgoing_edges;

    // Isolated node
    if edges == 0 {
        return i32::MIN;
    }

    let edge_difference = (shortcuts.len() as f32) / (edges as f32);
    let priority = (rank as f32 * 10.0) + (edge_difference * 100.0);
    (priority * 1000.0).round() as i32
}

fn find_shortcuts<'a>(
    graph: &mut CHPreparationGraph<'a>,
    witness_search: &mut WitnessSearch,
    weighting: &impl Weighting<CHPreparationGraph<'a>>,
    node: NodeId,
    max_settled_nodes: usize,
) -> Vec<PreparationShortcut> {
    let mut shortcuts = Vec::new();
    for &incoming_edge_id in graph.incoming_edges(node) {
        let incoming_edge = graph.edge(incoming_edge_id);

        assert!(incoming_edge.start_node() == node || incoming_edge.end_node() == node);

        let incoming_edge_adj_node = incoming_edge.adj_node(node);
        let incoming_direction = graph.edge_direction(incoming_edge_id, incoming_edge_adj_node);

        if incoming_edge_adj_node == node {
            continue;
        }

        witness_search.init(incoming_edge_adj_node, node);

        for &outgoing_edge_id in graph.outgoing_edges(node) {
            // We ignore the same edge, no shortcut is needed
            if incoming_edge_id == outgoing_edge_id {
                continue;
            }

            let outgoing_edge = graph.edge(outgoing_edge_id);

            assert!(outgoing_edge.start_node() == node || outgoing_edge.end_node() == node);

            let outgoing_edge_adj_node = outgoing_edge.adj_node(node);

            if outgoing_edge_adj_node == node {
                continue;
            }

            let outgoing_direction = graph.edge_direction(outgoing_edge_id, node);

            let weight = weighting.calc_edge_weight(incoming_edge, incoming_direction)
                + weighting.calc_edge_weight(outgoing_edge, outgoing_direction);

            let witness_search_weight = witness_search.compute_weight_upperbound(
                graph,
                weighting,
                outgoing_edge_adj_node,
                weight,
                max_settled_nodes,
            );

            if witness_search_weight <= weight {
                continue;
            }

            shortcuts.push(PreparationShortcut {
                start: incoming_edge_adj_node,
                end: outgoing_edge_adj_node,
                incoming_edge: incoming_edge_id,
                outgoing_edge: outgoing_edge_id,
                distance: outgoing_edge.distance() + incoming_edge.distance(),
                time: weighting.calc_edge_ms(incoming_edge, incoming_direction)
                    + weighting.calc_edge_ms(outgoing_edge, outgoing_direction),
                weight,
            });
        }
    }

    shortcuts
}
