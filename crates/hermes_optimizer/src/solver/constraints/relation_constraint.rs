use crate::{
    problem::task_dependencies::TaskDependencyType,
    solver::{
        insertion::Insertion, insertion_context::InsertionContext, score::Score,
        score_level::ScoreLevel, solution::working_solution::WorkingSolution,
    },
};

use super::global_constraint::GlobalConstraint;

#[derive(Clone)]
pub struct RelationConstraint;

const SCORE_LEVEL: ScoreLevel = ScoreLevel::Hard;

pub const RELATION_VIOLATION_WEIGHT: f64 = 10000.0;

impl GlobalConstraint for RelationConstraint {
    fn score_level(&self) -> ScoreLevel {
        SCORE_LEVEL
    }

    fn compute_score(&self, solution: &WorkingSolution) -> Score {
        let problem = solution.problem();

        if !problem.has_task_dependencies() {
            return Score::ZERO;
        }

        let task_dependencies = problem.task_dependencies();
        let mut total_violations = 0.0;

        for route in solution.non_empty_routes_iter() {
            // 1: check not in same route constraints
            if route.contains_not_in_same_route_violations(problem) {
                total_violations += RELATION_VIOLATION_WEIGHT;
            }

            // 2: check in same route constraints
            for other_route in solution.non_empty_routes_iter() {
                if route.version() == other_route.version() {
                    continue;
                }

                if route.contains_in_same_route_violations(problem, other_route) {
                    total_violations += RELATION_VIOLATION_WEIGHT;
                }
            }

            for (position, &activity_id) in route.activity_ids().iter().enumerate() {
                // 3: Check in sequence constraints
                for dependency in task_dependencies.traverse(activity_id, TaskDependencyType::After)
                {
                    if let Some(dep_position) = route.job_position(dependency)
                        && dep_position < position
                    {
                        total_violations += RELATION_VIOLATION_WEIGHT;
                    }
                }

                // 4: Check in direct sequence constraints
                for dependency in
                    task_dependencies.traverse(activity_id, TaskDependencyType::DirectlyAfter)
                {
                    if let Some(dep_position) = route.job_position(dependency)
                        && dep_position != position + 1
                    {
                        total_violations += RELATION_VIOLATION_WEIGHT;
                    }
                }
            }
        }

        Score::of(self.score_level(), total_violations)
    }

    fn compute_insertion_score(&self, context: &InsertionContext) -> Score {
        let problem = context.problem();
        if !problem.has_task_dependencies() {
            return Score::ZERO;
        }

        let route = context.route();

        let is_valid = match context.insertion {
            Insertion::Service(insertion) => route.is_valid_dependency_change(
                problem,
                insertion.inserted_activity_ids(),
                insertion.position,
                insertion.position,
            ),
            Insertion::Shipment(insertion) => route.is_valid_dependency_change(
                problem,
                insertion.inserted_activity_ids(route),
                insertion.pickup_position,
                insertion.delivery_position,
            ),
        };

        if is_valid {
            Score::zero()
        } else {
            Score::hard(RELATION_VIOLATION_WEIGHT)
        }
    }
}
