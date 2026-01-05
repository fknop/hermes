use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

use clap::Args;
use hermes_optimizer::{
    parsers::{
        cvrplib::{CVRPLibParser, parse_solution_file},
        parser::DatasetParser,
        solomon::SolomonParser,
    },
    solver::{
        solution::working_solution::WorkingSolution,
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

    #[arg(long, default_value_t = 1)]
    threads: u8,

    /// Output folder into .sol files
    #[arg(long, short = 'o')]
    out: Option<PathBuf>,
}

pub fn run(args: OptimizeDatasetArgs) -> Result<(), anyhow::Error> {
    let paths = if args.dataset.is_file() {
        vec![args.dataset]
    } else {
        let mut files = read_folder(&args.dataset)?;
        files.retain(|path| {
            path.extension()
                .map(|ext| ext == "txt" || ext == "vrp")
                .unwrap_or(false)
        });
        files
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

    for (i, path) in paths.iter().enumerate().filter(|(_, p)| {
        p.extension()
            .map(|ext| ext == "txt" || ext == "vrp")
            .unwrap_or(false)
    }) {
        let mut optimal_cost: Option<f64> = None;
        let extension = path.extension().unwrap();
        let now = Instant::now();

        let vrp = if extension == "txt" {
            let parser = SolomonParser;
            parser.parse(path)?
        } else if extension == "vrp" {
            let parser = CVRPLibParser;

            let mut solution_path = path.clone();
            solution_path.set_extension("sol");
            optimal_cost = parse_solution_file(solution_path);

            parser.parse(path)?
        } else {
            continue;
        };

        let solver = Solver::new(
            vrp,
            SolverParams {
                terminations: vec![Termination::Duration(args.timeout)],
                insertion_threads: Threads::Multi(args.threads as usize),
                run_intensify_search: true,
                ..SolverParams::default()
            },
        );

        let bar = &mut bars[i]; //Arc::new(ProgressBar::new(seconds as u64));
        bar.set_message("running...");
        bar.reset_elapsed();
        bar.enable_steady_tick(Duration::from_millis(100));

        bar.set_style(style.clone());

        solver.solve();
        let best_solution = solver.current_best_solution();

        if let Some(best_solution) = best_solution {
            let n_routes = best_solution.solution.non_empty_routes_count();
            let total_transport_cost = best_solution.solution.total_transport_costs();
            bar.finish_with_message(format!(
                "Finished - routes = {}, costs = {}, unassigned = {}, gap = {}",
                n_routes,
                total_transport_cost,
                best_solution.solution.unassigned_jobs().len(),
                optimal_cost
                    .map(|oc| format!("{:+.2}%", gap_percent(total_transport_cost, oc)))
                    .unwrap_or_else(|| "n/a".to_string())
            ));

            if let Some(out) = &args.out {
                let mut out_path = out.clone();
                if out_path.is_dir() {
                    let file_stem = path.file_stem().unwrap();
                    out_path.push(file_stem);
                    out_path.set_extension("sol");
                }
                std::fs::write(out_path, create_sol_file_contents(&best_solution.solution))?;
            }

            // println!("{}", create_sol_file_contents(&best_solution.solution));
        } else {
            bar.finish_with_message(format!("No solution"));
        }
    }

    Ok(())
}

fn create_sol_file_contents(solution: &WorkingSolution) -> String {
    // Should create content like this:
    /*
    Route #1: 21 31 19 17 13 7 26
    Route #2: 12 1 16 30
    Route #3: 27 24
    Route #4: 29 18 8 9 22 15 10 25 5 20
    Route #5: 14 28 11 4 23 3 2 6
    Cost 784
    */

    let mut contents = String::new();
    let problem = solution.problem();

    for (idx, route) in solution.non_empty_routes_iter().enumerate() {
        let route_number = idx + 1;
        contents.push_str(&format!("Route #{}:", route_number));

        for activity_id in route.activity_ids() {
            let job = problem.job(activity_id.job_id());
            let external_id = job.external_id();
            contents.push_str(&format!(" {}", external_id));
        }

        contents.push('\n');
    }

    let total_cost = solution.total_transport_costs();
    contents.push_str(&format!("Cost {}", total_cost as i64));

    contents
}

fn gap_percent(cost: f64, optimal_cost: f64) -> f64 {
    (cost - optimal_cost) / optimal_cost * 100.0
}
