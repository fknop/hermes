use std::{
    cmp,
    time::{Duration, Instant},
};

use rand::distr::{Distribution, Uniform};
use tracing::{debug, info};

use crate::{
    base_graph::BaseGraph,
    ch::{ch_edge::CHBaseEdge, ch_storage::CHStorage, priority_queue::PriorityQueue},
    edge_direction::EdgeDirection,
    graph::{Graph, UndirectedEdgeAccess},
    graph_edge::GraphEdge,
    stopwatch::Stopwatch,
    types::NodeId,
    weighting::Weighting,
};

use super::{
    preparation_graph::{
        CHPreparationGraph, CHPreparationGraphEdge, PreparationGraphWeighting,
    },
    shortcut::PreparationShortcut,
    witness_search::WitnessSearch,
};

pub struct CHGraphBuilder<'a> {
    base_graph: &'a BaseGraph,
    build_stopwatch: Stopwatch,
    recompute_priority_stopwatch: Stopwatch,
    recompute_neighbors_priority_stopwatch: Stopwatch,
    contract_node_stopwatch: Stopwatch,
    contracted_nodes: usize,
    skipped_nodes: usize,
    added_shortcuts: usize,
}

impl<'a> CHGraphBuilder<'a> {
    pub fn from_base_graph(base_graph: &'a BaseGraph) -> Self {
        Self {
            base_graph,
            build_stopwatch: Stopwatch::new(String::from("build_ch_graph")),
            recompute_priority_stopwatch: Stopwatch::new(String::from("recompute_priority")),
            recompute_neighbors_priority_stopwatch: Stopwatch::new(String::from(
                "recompute_neighbors_priority",
            )),
            contract_node_stopwatch: Stopwatch::new(String::from("contract_node")),
            skipped_nodes: 0,
            contracted_nodes: 0,
            added_shortcuts: 0,
        }
    }

    pub fn build<W>(&mut self, weighting: &W) -> CHStorage
    where
        W: Weighting<BaseGraph> + Send + Sync,
    {
        self.build_stopwatch.start();
        let mut last_reported_time = Instant::now();

        let mut rng = rand::rng();
        let dist = Uniform::new_inclusive(0, 100).unwrap();

        let mut ch_storage = CHStorage::new(self.base_graph);
        let mut preparation_graph = CHPreparationGraph::new(self.base_graph, weighting);
        let preparation_weighting = PreparationGraphWeighting::new(weighting);
        let mut witness_search = WitnessSearch::new();
        let mut priority_queue = PriorityQueue::new(self.base_graph.node_count());
        let mut hierarchies = vec![0; self.base_graph.node_count()];

        info!("Start CH contraction");
        info!(
            node_count = self.base_graph.node_count(),
            edge_count = self.base_graph.edge_count(),
        );

        for node_id in 0..self.base_graph.node_count() {
            let priority = self.calc_priority(
                &mut preparation_graph,
                &mut witness_search,
                &preparation_weighting,
                0,
                node_id,
            );

            priority_queue
                .push(node_id, priority)
                .unwrap_or_else(|err| panic!("{}", err));
        }

        info!("Finish computing priority for every node");

        let mut rank = 0;

        while let Some((node_id, priority)) = priority_queue.pop() {
            // Lazy recomputation of the priority
            // If the recomputed priority is less than the next node to be contracted, we re-enqueue the node

            if priority != i32::MIN
                && let Some((_, least_priority)) = priority_queue.peek() {
                    self.recompute_priority_stopwatch.start();
                    let recomputed_priority = self.calc_priority(
                        &mut preparation_graph,
                        &mut witness_search,
                        &preparation_weighting,
                        hierarchies[node_id],
                        node_id,
                    );
                    self.recompute_priority_stopwatch.stop();

                    if recomputed_priority > *least_priority {
                        priority_queue
                            .push(node_id, recomputed_priority)
                            .unwrap_or_else(|err| panic!("{}", err));
                        continue;
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
                        self.added_shortcuts += 1;
                    }
                }
            }

            // Update hierarchy of neighbors
            for &neighbor in neighbors.iter() {
                hierarchies[neighbor] = cmp::max(hierarchies[neighbor], hierarchies[node_id] + 1);
            }

            ch_storage.set_node_rank(node_id, rank);
            rank += 1;

            // Only contract 95% of nodes
            let percentage = 100;

            if preparation_graph.node_degree(node_id) == 0 || dist.sample(&mut rng) > percentage {
                self.skipped_nodes += 1;
                preparation_graph.disconnect_node(node_id);
                continue;
            }

            self.contract_node(
                &mut preparation_graph,
                &mut witness_search,
                &preparation_weighting,
                node_id,
            );

            self.contracted_nodes += 1;

            // TODO: better condition
            if self.contracted_nodes.is_multiple_of(500000) && self.added_shortcuts > 0 {
                debug!("Recompute all remaining priorities");
                let remaining_nodes: Vec<(NodeId, i32)> = priority_queue.to_vec();
                priority_queue.clear();
                for (node_id, _) in remaining_nodes {
                    let priority = self.calc_priority(
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
                self.recompute_neighbors_priority_stopwatch.start();
                for neighbor in neighbors {
                    let recomputed_neighbor_priority = self.calc_priority(
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
                self.recompute_neighbors_priority_stopwatch.stop();
            }

            if last_reported_time.elapsed().as_millis() > 3000 {
                self.report_timings(&preparation_graph);
                last_reported_time = Instant::now();
            }
        }

        self.report_timings(&preparation_graph);
        info!(
            "Added {} shortcuts for {} base edges",
            self.added_shortcuts,
            self.base_graph.edge_count()
        );
        info!("Finished contraction");

        ch_storage.check();

        ch_storage
    }

    fn report_timings(&self, preparation_graph: &CHPreparationGraph) {
        let current_duration = self.build_stopwatch.elapsed();
        let contract_node_duration = self.contract_node_stopwatch.total_duration();
        let recompute_priority_duration = self.recompute_priority_stopwatch.total_duration();
        let recompute_neighbors_duration =
            self.recompute_neighbors_priority_stopwatch.total_duration();

        println!(
            "{:20} {:20} {:20} {:20} {:20} {:20} {:20} {:20} {:20}",
            "Total",
            "Contract",
            "Recompute Prio.",
            "Recompute N. Prio.",
            "Contracted nodes",
            "Skipped nodes",
            "Remaining nodes",
            "Shortcuts",
            "Mean degree"
        );

        println!(
            "{:20} {:20} {:20} {:20} {:20} {:20} {:20} {:20} {:20}",
            format!("{}ms", current_duration.as_millis()),
            CHGraphBuilder::format_percentage(&current_duration, &contract_node_duration),
            CHGraphBuilder::format_percentage(&current_duration, &recompute_priority_duration),
            CHGraphBuilder::format_percentage(&current_duration, &recompute_neighbors_duration),
            format!("{}", self.contracted_nodes),
            format!("{}", self.skipped_nodes),
            format!(
                "{}",
                self.base_graph.node_count() - self.contracted_nodes - self.skipped_nodes
            ),
            format!("{}", self.added_shortcuts),
            format!("{:.2}", preparation_graph.mean_degree())
        );
    }

    fn format_percentage(total: &Duration, duration: &Duration) -> String {
        let percentage = duration.as_millis() as f64 / total.as_millis() as f64 * 100.0;
        format!("{percentage:.2}%")
    }

    fn contract_node(
        &mut self,
        graph: &mut CHPreparationGraph<'a>,
        witness_search: &mut WitnessSearch,
        weighting: &impl Weighting<CHPreparationGraph<'a>>,
        node: NodeId,
    ) {
        self.contract_node_stopwatch.start();

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

        self.contract_node_stopwatch.stop();
    }

    fn calc_priority(
        &mut self,
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

        let edge_quotient = (shortcuts.len() as f32) / (degree as f32);
        let priority = (hierarchy as f32 * 20.0) + (edge_quotient * 100.0);
        (priority * 1000.0).round() as i32
    }

    fn find_shortcuts(
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
