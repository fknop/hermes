use std::{fmt::Display, hash::Hash};

use fxhash::FxHashMap;
use rand::seq::IndexedRandom;
use serde::Serialize;
use serde_with::serde_as;

use crate::solver::solver_params::SolverParams;

#[derive(Debug, Clone, Serialize)]
pub struct AlnsWeights<S>
where
    S: Copy + Eq + Hash,
{
    weights: Vec<Operator<S>>,
}

impl<S> std::fmt::Display for AlnsWeights<S>
where
    S: std::fmt::Debug,
    S: Copy + Eq + Hash,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f)?;
        writeln!(f, "{:<50} {:>8}", "Operator", "Weight")?;
        writeln!(f, "{:-<60}", "")?;
        for op in &self.weights {
            writeln!(f, "{:<50} {:>8.4}", format!("{:?}", op.strategy), op.weight)?;
        }
        Ok(())
    }
}

impl<S> AlnsWeights<S>
where
    S: Copy + Eq + Hash,
{
    pub fn new(strategies: Vec<S>) -> Self {
        let weights = strategies
            .into_iter()
            .map(|strategy| Operator {
                strategy,
                weight: 1.0,
            })
            .collect();

        AlnsWeights { weights }
    }

    pub fn update_weights(&mut self, scores: &mut AlnsScores<S>, alns_reaction_factor: f64) {
        for operator in self.weights.iter_mut() {
            if let Some(ruin_score) = scores.scores.get_mut(&operator.strategy) {
                operator.update_weight(ruin_score, alns_reaction_factor);
                ruin_score.reset();
            }
        }
    }

    pub fn select_strategy<R>(&self, rng: &mut R) -> S
    where
        R: rand::Rng,
    {
        self.weights
            .choose_weighted(rng, |operator| (operator.weight / 5.0).exp())
            .map(|operator| operator.strategy)
            .expect("No ruin strategy configured on solver")
    }

    pub fn reset(&mut self) {
        for operator in self.weights.iter_mut() {
            operator.reset();
        }
    }
}

#[derive(Debug)]
struct ScoreEntry {
    score: f64,
    iterations: usize,

    accumulated_score: f64,
    accumulated_iterations: usize,
}

impl ScoreEntry {
    pub fn from_score(score: f64) -> Self {
        ScoreEntry {
            score,
            iterations: 1,
            accumulated_score: score,
            accumulated_iterations: 1,
        }
    }

    pub fn reset(&mut self) {
        self.score = 0.0;
        self.iterations = 0;
    }

    pub fn increase_score(&mut self, score: f64) {
        self.score += score;
        self.accumulated_score += score;
        self.iterations += 1;
        self.accumulated_iterations += 1;
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Operator<S> {
    pub strategy: S,
    pub weight: f64,
}

const MIN_WEIGHT: f64 = 0.1;

impl<T> Operator<T> {
    fn update_weight(&mut self, entry: &ScoreEntry, reaction_factor: f64) {
        let new_weight = if entry.iterations == 0 {
            (1.0 - reaction_factor) * self.weight
        } else {
            (1.0 - reaction_factor) * self.weight
                + reaction_factor * (entry.score / entry.iterations as f64)
        };

        self.weight = new_weight.max(MIN_WEIGHT);
    }

    fn reset(&mut self) {
        self.weight = 1.0;
    }
}

#[derive(Debug)]
pub struct AlnsScores<S>
where
    S: Hash + Eq,
{
    scores: FxHashMap<S, ScoreEntry>,
}

pub struct UpdateScoreParams {
    pub is_best: bool,
    pub improved: bool,
    pub accepted: bool,
}

impl<S> AlnsScores<S>
where
    S: Hash + Eq + Copy,
{
    pub fn new(strategies: Vec<S>) -> Self {
        AlnsScores {
            scores: strategies
                .iter()
                .map(|strategy| {
                    (
                        *strategy,
                        ScoreEntry {
                            score: 0.0,
                            iterations: 0,
                            accumulated_score: 0.0,
                            accumulated_iterations: 0,
                        },
                    )
                })
                .collect(),
        }
    }

    pub fn reset(&mut self) {
        for score in self.scores.values_mut() {
            score.reset();
        }
    }

    pub fn reset_strategy(&mut self, strategy: S) {
        if let Some(score) = self.scores.get_mut(&strategy) {
            score.reset();
        }
    }

    pub fn update_score(&mut self, strategy: S, score: f64) {
        if let Some(entry) = self.scores.get_mut(&strategy) {
            entry.increase_score(score);
        } else {
            self.scores.insert(strategy, ScoreEntry::from_score(score));
        }
    }

    pub fn accumulate(&mut self, alns_scores: &mut AlnsScores<S>) {
        for (strategy, entry) in &mut alns_scores.scores {
            if let Some(self_entry) = self.scores.get_mut(strategy) {
                self_entry.score += entry.accumulated_score;
                self_entry.iterations += entry.accumulated_iterations;
                self_entry.accumulated_score += entry.accumulated_score;
                self_entry.accumulated_iterations += entry.accumulated_iterations;
            } else {
                self.scores.insert(
                    *strategy,
                    ScoreEntry {
                        score: entry.accumulated_score,
                        iterations: entry.accumulated_iterations,
                        accumulated_score: entry.score,
                        accumulated_iterations: entry.iterations,
                    },
                );
            }

            entry.accumulated_iterations = 0;
            entry.accumulated_score = 0.0;
        }
    }

    pub fn update_scores(
        &mut self,
        strategy: S,
        params: &SolverParams,
        UpdateScoreParams {
            accepted,
            is_best,
            improved,
        }: UpdateScoreParams,
    ) {
        if !accepted {
            self.update_score(strategy, 0.0);
        } else if is_best {
            self.update_score(strategy, params.alns_best_factor);
        } else if improved {
            self.update_score(strategy, params.alns_improvement_factor);
        } else {
            self.update_score(strategy, params.alns_accepted_worst_factor);
        }
    }
}
