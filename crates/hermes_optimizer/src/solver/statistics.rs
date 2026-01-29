use std::sync::Arc;

use fxhash::FxHashMap;
use jiff::{SignedDuration, Timestamp};
use parking_lot::RwLock;
use schemars::JsonSchema;
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
        let mut aggregated_statistics = AggregatedStatistics::default();

        for thread_stats in &self.thread_statistics {
            let thread_stats = thread_stats.read();
            for (ruin_strategy, ruin_stats) in &thread_stats
                .aggregated_statistics
                .aggregated_ruin_statistics
            {
                let agg_stats = aggregated_statistics
                    .aggregated_ruin_statistics
                    .entry(*ruin_strategy)
                    .or_default();
                agg_stats.accumulate(ruin_stats);
            }

            for (recreate_strategy, recreate_stats) in &thread_stats
                .aggregated_statistics
                .aggregated_recreate_statistics
            {
                let agg_stats = aggregated_statistics
                    .aggregated_recreate_statistics
                    .entry(*recreate_strategy)
                    .or_default();
                agg_stats.accumulate(recreate_stats);
            }
        }

        aggregated_statistics
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
    aggregated_statistics: AggregatedStatistics,
}

impl ThreadSearchStatistics {
    pub fn add_iteration_info(&mut self, iteration: SearchStatisticsIteration) {
        if let SearchStatisticsIteration::RuinRecreate {
            ruin_strategy,
            recreate_strategy,
            ..
        } = iteration
        {
            let ruin_statistics = self
                .aggregated_statistics
                .aggregated_ruin_statistics
                .entry(ruin_strategy)
                .or_default();
            let recreate_statistics = self
                .aggregated_statistics
                .aggregated_recreate_statistics
                .entry(recreate_strategy)
                .or_default();

            Self::update_aggregated_statistics(ruin_statistics, recreate_statistics, &iteration);
        }

        self.iterations.push(iteration);
    }

    fn update_aggregated_statistics(
        ruin_stats: &mut AggregatedOperatorStatistics,
        recreate_stats: &mut AggregatedOperatorStatistics,
        iteration: &SearchStatisticsIteration,
    ) {
        if let &SearchStatisticsIteration::RuinRecreate {
            improved,
            is_best,
            ruin_duration,
            recreate_duration,
            score_before,
            score_after,
            ..
        } = iteration
        {
            ruin_stats.total_invocations += 1;
            recreate_stats.total_invocations += 1;

            if is_best {
                ruin_stats.total_best += 1;
                recreate_stats.total_best += 1;
            }

            if improved {
                ruin_stats.total_improvements += 1;
                recreate_stats.total_improvements += 1;
            }

            ruin_stats.total_duration += ruin_duration;
            recreate_stats.total_duration += recreate_duration;

            ruin_stats.avg_duration =
                ruin_stats.total_duration / ruin_stats.total_invocations as i32;
            recreate_stats.avg_duration =
                recreate_stats.total_duration / recreate_stats.total_invocations as i32;

            let score_improvement = score_before.soft_score - score_after.soft_score;
            ruin_stats.total_score_improvement += score_improvement;
            recreate_stats.total_score_improvement += score_improvement;

            ruin_stats.avg_score_improvement =
                ruin_stats.total_score_improvement / ruin_stats.total_invocations as f64;
            recreate_stats.avg_score_improvement =
                recreate_stats.total_score_improvement / recreate_stats.total_invocations as f64;

            let score_percentage_improvement = if score_before.soft_score.abs() > f64::EPSILON {
                score_improvement / score_before.soft_score.abs()
            } else {
                0.0
            };

            ruin_stats.total_score_percentage_improvement += score_percentage_improvement;
            recreate_stats.total_score_percentage_improvement += score_percentage_improvement;
            ruin_stats.avg_score_percentage_improvement =
                ruin_stats.total_score_percentage_improvement / ruin_stats.total_invocations as f64;
            recreate_stats.avg_score_percentage_improvement = recreate_stats
                .total_score_percentage_improvement
                / recreate_stats.total_invocations as f64;
        }
    }
}

#[serde_as]
#[derive(Serialize, Default)]
pub struct AggregatedStatistics {
    #[serde_as(as = "FxHashMap<DisplayFromStr, _>")]
    aggregated_ruin_statistics: FxHashMap<RuinStrategy, AggregatedOperatorStatistics>,
    #[serde_as(as = "FxHashMap<DisplayFromStr, _>")]
    aggregated_recreate_statistics: FxHashMap<RecreateStrategy, AggregatedOperatorStatistics>,
}

#[derive(Serialize, Default)]
pub struct AggregatedOperatorStatistics {
    pub total_invocations: usize,
    pub total_improvements: usize,
    pub total_best: usize,
    pub total_duration: SignedDuration,
    pub total_score_improvement: f64,
    pub total_score_percentage_improvement: f64,
    pub avg_duration: SignedDuration,
    pub avg_score_improvement: f64,
    pub avg_score_percentage_improvement: f64,
}

impl AggregatedOperatorStatistics {
    pub fn accumulate(&mut self, other: &AggregatedOperatorStatistics) {
        self.total_invocations += other.total_invocations;
        self.total_improvements += other.total_improvements;
        self.total_best += other.total_best;
        self.total_duration += other.total_duration;
        self.total_score_improvement += other.total_score_improvement;
        self.total_score_percentage_improvement += other.total_score_percentage_improvement;

        if self.total_invocations > 0 {
            self.avg_duration = self.total_duration / self.total_invocations as i32;
            self.avg_score_improvement =
                self.total_score_improvement / self.total_invocations as f64;
            self.avg_score_percentage_improvement =
                self.total_score_percentage_improvement / self.total_invocations as f64;
        }
    }
}
