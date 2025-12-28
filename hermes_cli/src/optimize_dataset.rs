use std::{path::PathBuf, time::Duration};

use clap::Args;
use hermes_optimizer::{
    parsers::{cvrplib::CVRPLibParser, parser::DatasetParser, solomon::SolomonParser},
    solver::{
        solver::Solver,
        solver_params::{SolverParams, Termination, Threads},
    },
};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
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

    let multi_bar = MultiProgress::new();
    let style = ProgressStyle::with_template("{prefix:.bold} [{elapsed_precise}] {msg}").unwrap();
    let seconds = args.timeout.as_secs();

    let mut bars: Vec<_> = paths
        .iter()
        .map(|path| {
            let pb = multi_bar.add(ProgressBar::new(seconds as u64));
            pb.set_style(style.clone());
            pb.set_prefix(path.file_name().unwrap().to_string_lossy().into_owned());
            pb.set_message("pending...");
            pb
        })
        .collect();

    for (i, path) in paths.iter().enumerate() {
        let vrp = if path.ends_with(".txt") {
            let parser = SolomonParser;
            parser.parse(path)?
        } else {
            let parser = CVRPLibParser;
            parser.parse(path)?
        };

        let solver = Solver::new(
            vrp,
            SolverParams {
                terminations: vec![Termination::Duration(args.timeout)],
                insertion_threads: Threads::Multi(args.threads as usize),
                ..SolverParams::default()
            },
        );

        let bar = &mut bars[i]; //Arc::new(ProgressBar::new(seconds as u64));
        bar.set_message("running...");
        bar.reset_elapsed();
        bar.enable_steady_tick(Duration::from_millis(100));

        bar.set_style(
            style.clone(), // ProgressStyle::default_bar()
                           //     .template("({elapsed}/{len}s)")
                           //     .unwrap(),
        );
        // let t_bar = Arc::clone(&bar);
        // let ticker = thread::spawn(move || {
        //     for i in 0..seconds {
        //         t_bar.set_position(i as u64);
        //         thread::sleep(Duration::from_secs(1));
        //     }
        // });

        solver.solve();
        let best_solution = solver.current_best_solution();

        if let Some(best_solution) = best_solution {
            let n_routes = best_solution.solution.non_empty_routes_count();
            let total_transport_cost = best_solution.solution.total_transport_costs();
            bar.finish_with_message(format!(
                "Finished - routes = {}, costs = {}",
                n_routes, total_transport_cost
            ));
        } else {
            bar.finish_with_message(format!("No solution"));
        }
    }

    Ok(())
}
