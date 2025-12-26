use std::{path::PathBuf, sync::Arc, thread, time::Duration};

use clap::Args;
use hermes_optimizer::{
    solomon::solomon_parser::SolomonParser,
    solver::{
        solver::Solver,
        solver_params::{SolverParams, Termination, Threads},
    },
};
use indicatif::{ProgressBar, ProgressStyle};
use tracing::info;

use crate::{file_utils::read_folder, parsers};

#[derive(Args)]
pub struct OptimizeDatasetArgs {
    /// The file to optimize
    #[arg(short, long)]
    dataset: PathBuf,

    #[arg(short, long, value_parser=parsers::parse_duration, default_value = "5s")]
    timeout: jiff::SignedDuration,

    #[arg(short, long, default_value_t = 1)]
    threads: u8,

    /// Output folder into .solution files
    #[arg(short, long)]
    output: Option<PathBuf>,
}

pub fn run(args: OptimizeDatasetArgs) -> Result<(), anyhow::Error> {
    info!("Optimizing dataset {:?}", args.dataset);
    let paths = if args.dataset.is_file() {
        vec![args.dataset]
    } else {
        read_folder(&args.dataset)?
    };

    for path in paths {
        let vrp = SolomonParser::from_file(&path).unwrap();

        let solver = Solver::new(
            vrp,
            SolverParams {
                terminations: vec![Termination::Duration(args.timeout)],
                insertion_threads: Threads::Multi(args.threads as usize),
                ..SolverParams::default()
            },
        );

        let seconds = args.timeout.as_secs();
        let bar = Arc::new(ProgressBar::new(seconds as u64));

        bar.enable_steady_tick(Duration::from_secs(1));

        bar.set_style(
            ProgressStyle::default_bar()
                .template("[{bar:40}] ({elapsed}/{len}s)")
                .unwrap(),
        );
        let t_bar = Arc::clone(&bar);
        let ticker = thread::spawn(move || {
            for i in 0..seconds {
                t_bar.set_position(i as u64);
                thread::sleep(Duration::from_secs(1));
            }
        });

        solver.solve();
        let best_solution = solver.current_best_solution();

        bar.finish_and_clear();
        ticker.join().unwrap();

        if let Some(best_solution) = best_solution {
            info!("Best solution found: {:?}", best_solution.score);
        } else {
            info!("No solution found");
        }
    }

    Ok(())
}
