use crate::{
    constants::MAX_WEIGHT,
    edge_direction::EdgeDirection,
    graph::Graph,
    graph_edge::GraphEdge,
    types::{EdgeId, NodeId},
    weighting::Weighting,
};

use super::{
    preparation_graph::CHPreparationGraph, shortcut::Shortcut, witness_search::WitnessSearch,
};

pub fn contract_node<'a>(
    graph: &mut CHPreparationGraph<'a>,
    witness_search: &mut WitnessSearch<'a>,
    weighting: &impl Weighting<CHPreparationGraph<'a>>,
    node: NodeId,
) {
    let shortcuts = find_shortcuts(graph, witness_search, weighting, node);

    for shortcut in shortcuts {
        graph.add_shortcut(shortcut);
    }

    graph.disconnect_node(node);
}

pub fn calc_priority<'a>(
    graph: &mut CHPreparationGraph<'a>,
    witness_search: &mut WitnessSearch<'a>,
    weighting: &impl Weighting<CHPreparationGraph<'a>>,
    node: NodeId,
) -> i16 {
    let shortcuts = find_shortcuts(graph, witness_search, weighting, node);

    let incoming_edges = graph
        .node_edges_iter(node)
        .filter(|edge_id| filter_incoming_edge(graph, weighting, edge_id, node))
        .count();

    let outgoing_edges = graph
        .node_edges_iter(node)
        .filter(|edge_id| filter_outgoing_edge(graph, weighting, edge_id, node))
        .count();

    let edges = incoming_edges + outgoing_edges;
    shortcuts.len() as i16 - edges as i16
}

pub fn find_shortcuts<'a>(
    graph: &mut CHPreparationGraph<'a>,
    witness_search: &mut WitnessSearch<'a>,
    weighting: &impl Weighting<CHPreparationGraph<'a>>,
    node: NodeId,
) -> Vec<Shortcut> {
    let mut shortcuts = Vec::new();
    for incoming_edge_id in graph
        .node_edges_iter(node)
        .filter(|edge_id| filter_incoming_edge(graph, weighting, edge_id, node))
    {
        let incoming_edge = graph.edge(incoming_edge_id);
        let incoming_edge_adj_node = incoming_edge.adj_node(node);

        witness_search.init(incoming_edge_adj_node, node);

        for outgoing_edge_id in graph
            .node_edges_iter(node)
            .filter(|edge_id| filter_outgoing_edge(graph, weighting, edge_id, node))
        {
            // We ignore the same edge, no shortcut is needed
            if incoming_edge_id == outgoing_edge_id {
                continue;
            }

            let outgoing_edge = graph.edge(outgoing_edge_id);
            let outgoing_edge_adj_node = outgoing_edge.adj_node(node);

            let weight = weighting.calc_edge_weight(incoming_edge, EdgeDirection::Backward)
                + weighting.calc_edge_weight(outgoing_edge, EdgeDirection::Forward);

            // TODO: max weight
            // TODO: max settled nodes
            let witness_search_weight =
                witness_search.find_max_weight(weighting, outgoing_edge_adj_node, MAX_WEIGHT, 200);

            if witness_search_weight <= weight {
                continue;
            }

            shortcuts.push(Shortcut {
                from: incoming_edge_adj_node,
                to: outgoing_edge_adj_node,
                incoming_edge: incoming_edge_id,
                outgoing_edge: outgoing_edge_id,
                distance: outgoing_edge.distance() + incoming_edge.distance(),
                time: weighting.calc_edge_ms(incoming_edge, EdgeDirection::Backward)
                    + weighting.calc_edge_ms(outgoing_edge, EdgeDirection::Forward),
                weight,
            });
        }
    }

    shortcuts
}

/// Filter incoming edge edge_id to the node
fn filter_incoming_edge<'a>(
    graph: &CHPreparationGraph<'a>,
    weighting: &impl Weighting<CHPreparationGraph<'a>>,
    edge_id: &EdgeId,
    node: NodeId,
) -> bool {
    let edge_direction = graph.edge_direction(*edge_id, node);
    let edge = graph.edge(*edge_id);
    let weight = weighting.calc_edge_weight(edge, edge_direction.opposite());
    weight != MAX_WEIGHT
}

/// Filter outgoing edge edge_id to the node
fn filter_outgoing_edge<'a>(
    graph: &CHPreparationGraph<'a>,
    weighting: &impl Weighting<CHPreparationGraph<'a>>,
    edge_id: &EdgeId,
    node: NodeId,
) -> bool {
    let edge_direction = graph.edge_direction(*edge_id, node);
    let edge = graph.edge(*edge_id);
    let weight = weighting.calc_edge_weight(edge, edge_direction);
    weight != MAX_WEIGHT
}
