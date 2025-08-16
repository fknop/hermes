use std::sync::Arc;

use fxhash::FxHashMap;
use jiff::Timestamp;
use parking_lot::RwLock;

use super::{
    recreate::recreate_strategy::RecreateStrategy,
    ruin::ruin_strategy::RuinStrategy,
    score::{Score, ScoreAnalysis},
};

pub struct SearchStatistics {
    global_statistics: Arc<RwLock<GlobalStatistics>>,
    thread_search_statistics: Vec<Arc<RwLock<ThreadSearchStatistics>>>,
}

impl SearchStatistics {
    pub fn new(number_of_threads: usize) -> Self {
        Self {
            global_statistics: Arc::new(RwLock::new(GlobalStatistics::default())),
            thread_search_statistics: {
                let mut v = Vec::with_capacity(number_of_threads);
                (0..number_of_threads)
                    .for_each(|_| v.push(Arc::new(RwLock::new(ThreadSearchStatistics::default()))));
                v
            },
        }
    }

    pub fn global_statistics(&self) -> &Arc<RwLock<GlobalStatistics>> {
        &self.global_statistics
    }

    pub fn thread_statistics(&self, thread: usize) -> &Arc<RwLock<ThreadSearchStatistics>> {
        &self.thread_search_statistics[thread]
    }
}

pub struct ScoreEvolutionRow {
    pub timestamp: Timestamp,
    pub score: Score,
    pub score_analysis: ScoreAnalysis,
    pub thread: usize,
}

#[derive(Default)]
pub struct GlobalStatistics {
    score_evolution: Vec<ScoreEvolutionRow>,
}

impl GlobalStatistics {
    pub fn add_best_score(&mut self, row: ScoreEvolutionRow) {
        self.score_evolution.push(row);
    }
}

pub struct SearchStatisticsIteration {
    pub timestamp: Timestamp,
    pub ruin_strategy: RuinStrategy,
    pub recreate_strategy: RecreateStrategy,
    pub improved: bool,
    pub is_best: bool,
}

#[derive(Default)]
pub struct ThreadSearchStatistics {
    iterations: Vec<SearchStatisticsIteration>,
    ruin_strategies: FxHashMap<RuinStrategy, usize>,
    recreate_strategies: FxHashMap<RecreateStrategy, usize>,
}

impl ThreadSearchStatistics {
    pub fn add_iteration_info(&mut self, iteration: SearchStatisticsIteration) {
        self.ruin_strategies
            .entry(iteration.ruin_strategy)
            .and_modify(|entry| *entry += 1)
            .or_insert(1);

        self.recreate_strategies
            .entry(iteration.recreate_strategy)
            .and_modify(|entry| *entry += 1)
            .or_insert(1);

        self.iterations.push(iteration);
    }
}
