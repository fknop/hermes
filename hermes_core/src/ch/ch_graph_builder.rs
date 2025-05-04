use std::cmp;

use rand::distr::{Distribution, Uniform};

use crate::{
    base_graph::BaseGraph,
    ch::{ch_edge::CHBaseEdge, ch_storage::CHStorage, priority_queue::PriorityQueue},
    edge_direction::EdgeDirection,
    graph::{Graph, UndirectedEdgeAccess},
    graph_edge::GraphEdge,
    types::NodeId,
    weighting::Weighting,
};

use super::{
    preparation_graph::{CHPreparationGraph, CHPreparationGraphEdge, PreparationGraphWeighting},
    shortcut::PreparationShortcut,
    witness_search::WitnessSearch,
};

pub struct CHGraphBuilder<'a> {
    base_graph: &'a BaseGraph,
}

impl<'a> CHGraphBuilder<'a> {
    pub fn from_base_graph(base_graph: &'a BaseGraph) -> Self {
        Self { base_graph }
    }

    pub fn build<W>(&self, weighting: &W) -> CHStorage
    where
        W: Weighting<BaseGraph> + Send + Sync,
    {
        let mut rng = rand::rng();
        let dist = Uniform::new_inclusive(0, 100).unwrap();

        let mut ch_storage = CHStorage::new(self.base_graph);
        let mut preparation_graph = CHPreparationGraph::new(self.base_graph, weighting);
        let preparation_weighting = PreparationGraphWeighting::new(weighting);
        let mut witness_search = WitnessSearch::new();
        let mut priority_queue = PriorityQueue::new(self.base_graph.node_count());
        let mut hierarchies = vec![0; self.base_graph.node_count()];

        println!("Start CH contraction");
        println!("Edge count {}", self.base_graph.edge_count());
        println!("Node count {}", self.base_graph.node_count());

        for node_id in 0..self.base_graph.node_count() {
            let priority = CHGraphBuilder::calc_priority(
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

        let mut added_shortcuts = 0;
        let mut contracted_nodes = 0;
        let mut skipped_nodes = 0;
        let mut rank = 0;

        while let Some((node_id, priority)) = priority_queue.pop() {
            // Lazy recomputation of the priority
            // If the recomputed priority is less than the next node to be contracted, we re-enqueue the node

            if contracted_nodes > 1800000 {
                println!("Remaining nodes {}", priority_queue.len());
            }

            if priority != i32::MIN {
                if let Some((_, least_priority)) = priority_queue.peek() {
                    let recomputed_priority = CHGraphBuilder::calc_priority(
                        &mut preparation_graph,
                        &mut witness_search,
                        &preparation_weighting,
                        hierarchies[node_id],
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

                match edge {
                    CHPreparationGraphEdge::Edge(base_edge) => {
                        // TODO: make sure directions are correct
                        // From start to end
                        let forward_weight =
                            weighting.calc_edge_weight(base_edge, EdgeDirection::Forward);
                        let forward_time =
                            weighting.calc_edge_ms(base_edge, EdgeDirection::Forward);

                        // From end to start
                        let backward_weight =
                            weighting.calc_edge_weight(base_edge, EdgeDirection::Backward);
                        let backward_time =
                            weighting.calc_edge_ms(base_edge, EdgeDirection::Backward);

                        ch_storage.add_edge(CHBaseEdge {
                            id: base_edge.id(),
                            start: edge.start_node(),
                            end: edge.end_node(),
                            distance: base_edge.distance(),
                            forward_time,
                            backward_time,
                            forward_weight,
                            backward_weight,
                        });
                    }
                    CHPreparationGraphEdge::Shortcut(shortcut) => {
                        ch_storage.add_shortcut(shortcut.clone());
                        added_shortcuts += 1;
                    }
                }
            }

            // Update hierarchy of neighbors
            for &neighbor in neighbors.iter() {
                if neighbor != node_id {
                    hierarchies[neighbor] =
                        cmp::max(hierarchies[neighbor], hierarchies[node_id] + 1);
                }
            }

            ch_storage.set_node_rank(node_id, rank);
            rank += 1;

            // Only contract 95% of nodes
            let percentage = 95;

            if preparation_graph.node_degree(node_id) == 0 || dist.sample(&mut rng) > percentage {
                skipped_nodes += 1;
                preparation_graph.disconnect_node(node_id);
                continue;
            }

            CHGraphBuilder::contract_node(
                &mut preparation_graph,
                &mut witness_search,
                &preparation_weighting,
                node_id,
            );

            contracted_nodes += 1;

            // TODO: better condition
            if contracted_nodes % 500000 == 0 && added_shortcuts > 0 {
                println!("Recompute all remaining priorities");
                let remaining_nodes: Vec<(NodeId, i32)> = priority_queue.to_vec();
                priority_queue.clear();
                for (node_id, _) in remaining_nodes {
                    let priority = CHGraphBuilder::calc_priority(
                        &mut preparation_graph,
                        &mut witness_search,
                        &preparation_weighting,
                        hierarchies[node_id],
                        node_id,
                    );

                    priority_queue
                        .push(node_id, priority)
                        .unwrap_or_else(|err| panic!("{}", err));
                }
            } else {
                let max_neighbor_update = 3;
                let mut neighbor_count = 0;
                for neighbor in neighbors {
                    let recomputed_neighbor_priority = CHGraphBuilder::calc_priority(
                        &mut preparation_graph,
                        &mut witness_search,
                        &preparation_weighting,
                        hierarchies[neighbor],
                        neighbor,
                    );

                    priority_queue.update_priority(neighbor, recomputed_neighbor_priority);
                    neighbor_count += 1;
                    if neighbor_count >= max_neighbor_update {
                        break;
                    }
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
            self.base_graph.edge_count()
        );
        println!("Contracted nodes {}", contracted_nodes);
        println!("Skipped nodes {}", skipped_nodes);

        ch_storage.check();

        ch_storage
    }

    fn contract_node(
        graph: &mut CHPreparationGraph<'a>,
        witness_search: &mut WitnessSearch,
        weighting: &impl Weighting<CHPreparationGraph<'a>>,
        node: NodeId,
    ) {
        let shortcuts = CHGraphBuilder::find_shortcuts(
            graph,
            witness_search,
            weighting,
            node,
            (graph.mean_degree() * 200.0).round() as usize,
        );

        for shortcut in shortcuts {
            graph.add_shortcut(shortcut);
        }

        graph.disconnect_node(node);
    }

    fn calc_priority(
        graph: &mut CHPreparationGraph<'a>,
        witness_search: &mut WitnessSearch,
        weighting: &impl Weighting<CHPreparationGraph<'a>>,
        hierarchy: usize,
        node: NodeId,
    ) -> i32 {
        let shortcuts = CHGraphBuilder::find_shortcuts(
            graph,
            witness_search,
            weighting,
            node,
            (graph.mean_degree() * 5.0).round() as usize,
        );

        let degree = graph.node_degree(node);

        // Isolated node
        if degree == 0 {
            return i32::MIN;
        }

        let edge_difference = (shortcuts.len() as f32) / (degree as f32);
        let priority = (hierarchy as f32 * 20.0) + (edge_difference * 100.0);
        (priority * 1000.0).round() as i32
    }

    fn find_shortcuts(
        graph: &mut CHPreparationGraph<'a>,
        witness_search: &mut WitnessSearch,
        weighting: &impl Weighting<CHPreparationGraph<'a>>,
        node: NodeId,
        max_settled_nodes: usize,
    ) -> Vec<PreparationShortcut> {
        let product = graph.incoming_edges(node).len() * graph.outgoing_edges(node).len();

        let mut shortcuts = Vec::new();

        if product > 1_000_000 {
            return shortcuts;
        }

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

                if incoming_edge_adj_node == outgoing_edge_adj_node {
                    continue;
                }

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
                    10,
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
}
