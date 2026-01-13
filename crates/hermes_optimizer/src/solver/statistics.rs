use std::sync::Arc;

use fxhash::FxHashMap;
use jiff::{SignedDuration, Timestamp};
use parking_lot::RwLock;
use serde::Serialize;
use serde_with::{DisplayFromStr, serde_as};

use super::{
    recreate::recreate_strategy::RecreateStrategy,
    ruin::ruin_strategy::RuinStrategy,
    score::{Score, ScoreAnalysis},
};

#[derive(Serialize)]
pub struct SearchStatistics {
    pub global_statistics: Arc<RwLock<GlobalStatistics>>,
    pub thread_statistics: Vec<Arc<RwLock<ThreadSearchStatistics>>>,
}

impl SearchStatistics {
    pub fn new(number_of_threads: usize) -> Self {
        Self {
            global_statistics: Arc::new(RwLock::new(GlobalStatistics::default())),
            thread_statistics: {
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
        &self.thread_statistics[thread]
    }

    pub fn aggregate(&self) -> AggregatedStatistics {
        AggregatedStatistics::from_statistics(self)
    }
}

#[derive(Serialize)]
pub struct ScoreEvolutionRow {
    pub timestamp: Timestamp,
    pub score: Score,
    pub score_analysis: ScoreAnalysis,
    pub thread: usize,
}

#[derive(Default, Serialize)]
pub struct GlobalStatistics {
    score_evolution: Vec<ScoreEvolutionRow>,
}

impl GlobalStatistics {
    pub fn add_best_score(&mut self, row: ScoreEvolutionRow) {
        self.score_evolution.push(row);
    }
}

#[derive(Serialize)]
pub enum SearchStatisticsIteration {
    RuinRecreate {
        timestamp: Timestamp,
        ruin_strategy: RuinStrategy,
        recreate_strategy: RecreateStrategy,
        improved: bool,
        is_best: bool,
        score_before: Score,
        score_after: Score,
        ruin_duration: SignedDuration,
        recreate_duration: SignedDuration,
    },
    Intensify {
        timestamp: Timestamp,
        improved: bool,
        is_best: bool,
    },
}

#[serde_as]
#[derive(Default, Serialize)]
pub struct ThreadSearchStatistics {
    #[serde(skip_serializing)]
    iterations: Vec<SearchStatisticsIteration>,

    #[serde_as(as = "FxHashMap<DisplayFromStr, _>")]
    ruin_strategies: FxHashMap<RuinStrategy, usize>,
    #[serde_as(as = "FxHashMap<DisplayFromStr, _>")]
    recreate_strategies: FxHashMap<RecreateStrategy, usize>,
}

impl ThreadSearchStatistics {
    pub fn add_iteration_info(&mut self, iteration: SearchStatisticsIteration) {
        if let SearchStatisticsIteration::RuinRecreate {
            ruin_strategy,
            recreate_strategy,
            ..
        } = iteration
        {
            self.ruin_strategies
                .entry(ruin_strategy)
                .and_modify(|entry| *entry += 1)
                .or_insert(1);
            self.recreate_strategies
                .entry(recreate_strategy)
                .and_modify(|entry| *entry += 1)
                .or_insert(1);
        }

        self.iterations.push(iteration);
    }
}

#[derive(Serialize)]
pub struct AggregatedStatistics {
    pub ruin_statistics: FxHashMap<RuinStrategy, AggregatedOperatorStatistics>,
    pub recreate_statistics: FxHashMap<RecreateStrategy, AggregatedOperatorStatistics>,
}

impl AggregatedStatistics {
    pub fn from_statistics(statistics: &SearchStatistics) -> Self {
        let mut ruin_statistics: FxHashMap<RuinStrategy, AggregatedOperatorStatistics> =
            FxHashMap::default();
        let mut recreate_statistics: FxHashMap<RecreateStrategy, AggregatedOperatorStatistics> =
            FxHashMap::default();

        for thread_statistics in statistics.thread_statistics.iter() {
            for iteration in &thread_statistics.read().iterations {
                if let SearchStatisticsIteration::RuinRecreate {
                    ruin_strategy,
                    recreate_strategy,
                    improved,
                    is_best,
                    ruin_duration,
                    recreate_duration,
                    score_before,
                    score_after,
                    ..
                } = iteration
                {
                    let ruin_stats = ruin_statistics.entry(*ruin_strategy).or_insert(
                        AggregatedOperatorStatistics {
                            total_invocations: 0,
                            total_improvements: 0,
                            total_best: 0,
                            avg_duration: SignedDuration::ZERO,
                            avg_score_improvement: 0.0,
                            avg_score_percentage_improvement: 0.0,
                        },
                    );

                    let recreate_stats = recreate_statistics.entry(*recreate_strategy).or_insert(
                        AggregatedOperatorStatistics {
                            total_invocations: 0,
                            total_improvements: 0,
                            total_best: 0,
                            avg_duration: SignedDuration::ZERO,
                            avg_score_improvement: 0.0,
                            avg_score_percentage_improvement: 0.0,
                        },
                    );

                    ruin_stats.total_invocations += 1;
                    recreate_stats.total_invocations += 1;

                    if *improved {
                        ruin_stats.total_improvements += 1;
                        recreate_stats.total_improvements += 1;
                    }
                    if *is_best {
                        ruin_stats.total_best += 1;
                        recreate_stats.total_best += 1;
                    }

                    ruin_stats.avg_duration = ruin_stats.avg_duration
                        + (*ruin_duration - ruin_stats.avg_duration)
                            / (ruin_stats.total_invocations as i32);

                    recreate_stats.avg_duration = recreate_stats.avg_duration
                        + (*recreate_duration - recreate_stats.avg_duration)
                            / (recreate_stats.total_invocations as i32);

                    let score_improvement = score_before.soft_score - score_after.soft_score;
                    let score_percentage_improvement =
                        if score_before.soft_score.abs() > f64::EPSILON {
                            score_improvement / score_before.soft_score.abs()
                        } else {
                            0.0
                        };

                    ruin_stats.avg_score_improvement = ruin_stats.avg_score_improvement
                        + (score_improvement - ruin_stats.avg_score_improvement)
                            / (ruin_stats.total_invocations as f64);
                    recreate_stats.avg_score_improvement = recreate_stats.avg_score_improvement
                        + (score_improvement - recreate_stats.avg_score_improvement)
                            / (recreate_stats.total_invocations as f64);

                    ruin_stats.avg_score_percentage_improvement = ruin_stats
                        .avg_score_percentage_improvement
                        + (score_percentage_improvement
                            - ruin_stats.avg_score_percentage_improvement)
                            / (ruin_stats.total_invocations as f64);

                    recreate_stats.avg_score_percentage_improvement = recreate_stats
                        .avg_score_percentage_improvement
                        + (score_percentage_improvement
                            - recreate_stats.avg_score_percentage_improvement)
                            / (recreate_stats.total_invocations as f64);
                }
            }
        }

        AggregatedStatistics {
            ruin_statistics,
            recreate_statistics,
        }
    }
}

#[derive(Serialize)]
pub struct AggregatedOperatorStatistics {
    pub total_invocations: usize,
    pub total_improvements: usize,
    pub total_best: usize,
    pub avg_duration: SignedDuration,
    pub avg_score_improvement: f64,
    pub avg_score_percentage_improvement: f64,
}
