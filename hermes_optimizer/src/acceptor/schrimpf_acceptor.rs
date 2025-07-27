use crate::solver::{
    accepted_solution::AcceptedSolution, score::Score, working_solution::WorkingSolution,
};

use super::accept_solution::{AcceptSolution, AcceptSolutionContext};

pub struct SchrimpfAcceptor {
    initial_threshold: f64,
    alpha: f64,
}

impl SchrimpfAcceptor {
    pub fn new() -> Self {
        SchrimpfAcceptor {
            // TODO: compute this initial value based on the problem size
            initial_threshold: 10000.0,
            alpha: 0.15,
        }
    }

    // * threshold(i) = initialThreshold * Math.exp(-Math.log(2) * (i / nuOfTotalIterations) / alpha)
    fn compute_threshold(&self, context: &AcceptSolutionContext) -> f64 {
        self.initial_threshold
            * (-(2.0_f64).ln()
                * (context.iteration as f64 / context.max_iterations.unwrap_or(15000) as f64)
                / self.alpha)
                .exp()
    }
}

impl AcceptSolution for SchrimpfAcceptor {
    fn accept(
        &self,
        current_solutions: &[AcceptedSolution],
        _: &WorkingSolution,
        score: &Score,
        context: AcceptSolutionContext,
    ) -> bool {
        if current_solutions.len() < context.max_solutions {
            return true; // Accept the first solution
        }

        let worst_current_solution = current_solutions.iter().max_by_key(|s| s.score);

        if let Some(worst_solution) = worst_current_solution {
            let threshold = self.compute_threshold(&context);

            let new_score = worst_solution.score + Score::soft(threshold);
            if score < &new_score {
                return true;
            }

            false
        } else {
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_threshold() {
        let acceptor = SchrimpfAcceptor::new();

        let mut threshold = acceptor.compute_threshold(&AcceptSolutionContext {
            iteration: 0,
            max_solutions: 100,
            max_iterations: Some(1000),
        });
        println!("{threshold:?}");
        threshold = acceptor.compute_threshold(&AcceptSolutionContext {
            iteration: 1,
            max_solutions: 100,
            max_iterations: Some(1000),
        });

        println!("{threshold:?}");

        threshold = acceptor.compute_threshold(&AcceptSolutionContext {
            iteration: 999,
            max_solutions: 100,
            max_iterations: Some(1000),
        });

        println!("{threshold:?}");

        threshold = acceptor.compute_threshold(&AcceptSolutionContext {
            iteration: 1000,
            max_solutions: 100,
            max_iterations: Some(1000),
        });

        println!("{threshold:?}");

        threshold = acceptor.compute_threshold(&AcceptSolutionContext {
            iteration: 2000,
            max_solutions: 100,
            max_iterations: Some(1000),
        });

        println!("{threshold:?}");

        assert!(threshold > 0.0);
    }
}
