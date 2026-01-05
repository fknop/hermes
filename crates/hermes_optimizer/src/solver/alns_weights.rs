use std::hash::Hash;

use fxhash::FxHashMap;
use rand::seq::IndexedRandom;

use crate::solver::solver_params::SolverParams;

#[derive(Debug)]
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
            .choose_weighted(rng, |operator| operator.weight)
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
    pub score: f64,
    pub iterations: usize,
}

impl ScoreEntry {
    pub fn reset(&mut self) {
        self.score = 0.0;
        self.iterations = 0;
    }
}

#[derive(Debug)]
pub struct Operator<S> {
    pub strategy: S,
    pub weight: f64,
}

impl<T> Operator<T> {
    fn update_weight(&mut self, entry: &ScoreEntry, reaction_factor: f64) {
        let new_weight = if entry.iterations == 0 {
            (1.0 - reaction_factor) * self.weight
        } else {
            (1.0 - reaction_factor) * self.weight
                + reaction_factor * (entry.score / entry.iterations as f64)
        };

        self.weight = new_weight.max(0.01);
    }

    fn reset(&mut self) {
        self.weight = 1.0;
    }
}

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
            entry.score += score;
            entry.iterations += 1;
        } else {
            self.scores.insert(
                strategy,
                ScoreEntry {
                    score,
                    iterations: 1,
                },
            );
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
