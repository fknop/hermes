use super::{constraint::GlobalConstraint, score::Score, working_solution::WorkingSolution};

pub struct MinimizeCompletionTime;

impl GlobalConstraint for MinimizeCompletionTime {
    fn compute_score(&self, solution: &WorkingSolution) -> Score {
        let mut seconds = 0;
        for route in solution.routes() {
            let start = route.start();
            let end = route.end();
            seconds += end.get_departure_time().as_second() - start.arrival_time().as_second();
        }

        Score::soft(seconds)
    }
}
