use std::{fs::File, io::BufWriter, path::PathBuf};

use clap::{Args, Subcommand};
use hermes_optimizer::{
    parsers::{
        cvrplib::{Bks, parse_bks_for_file, parse_solution_file},
        parser::parse_dataset,
    },
    solver::{
        solver::Solver,
        solver_params::{SolverParams, SolverParamsDebugOptions, Termination, Threads},
    },
};
use indicatif::{ProgressBar, ProgressStyle};
use jiff::SignedDuration;
use serde::{Deserialize, Serialize};

use crate::{file_utils::read_folder, parsers};

#[derive(Subcommand)]
pub enum BenchmarkSubcommands {
    Run {
        #[command(flatten)]
        args: RunBenchmarkArgs,
    },
}

#[derive(Args)]
pub struct RunBenchmarkArgs {
    /// The file to optimize
    #[arg(short, long)]
    dataset: PathBuf,

    #[arg(short, long)]
    name: String,

    #[arg(short, long, value_parser=parsers::parse_duration)]
    timeout: Option<jiff::SignedDuration>,

    #[arg(long, default_value_t = 4)]
    ithreads: u8,

    #[arg(long, default_value_t = 1)]
    sthreads: u8,

    #[arg(long, short = 'n')]
    iterations: Option<usize>,

    /// Output folder into .sol files
    #[arg(long, short = 'o')]
    out: PathBuf,
}

pub fn run(subcommand: BenchmarkSubcommands) -> Result<(), anyhow::Error> {
    match subcommand {
        BenchmarkSubcommands::Run { args } => run_benchmark(args),
    }
}

#[derive(Serialize, Deserialize)]
struct InstanceResult {
    pub instance: String,
    pub cost: f64,
    pub vehicles: usize,
    pub duration: SignedDuration,
    pub feasible: bool,
    pub iterations: usize,
    pub bks: Option<Bks>,
}

#[derive(Serialize, Deserialize, Default)]
struct BenchmarkRun {
    pub instances: Vec<InstanceResult>,
}

impl InstanceResult {
    pub fn gap_percent(&self) -> Option<f64> {
        self.bks
            .map(|bks| (self.cost - bks.cost) / bks.cost * 100.0)
    }

    pub fn is_bks(&self) -> bool {
        if !self.feasible {
            return false;
        }

        if let Some(bks) = &self.bks {
            bks.cost == self.cost && bks.vehicles == self.vehicles
        } else {
            false
        }
    }
}

fn run_benchmark(args: RunBenchmarkArgs) -> Result<(), anyhow::Error> {
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

    let mut benchmark_run = BenchmarkRun::default();

    let style = ProgressStyle::with_template("{msg}").unwrap();

    let progress_bar = ProgressBar::new(0);
    progress_bar.set_style(style);

    for (i, path) in paths.iter().enumerate().filter(|(_, p)| {
        p.extension()
            .map(|ext| ext == "txt" || ext == "vrp")
            .unwrap_or(false)
    }) {
        // Try to load an accompanying .sol file for optimal solution reference
        let mut solution_path = path.clone();
        solution_path.set_extension("sol");
        let bks = if let Some(bks) = parse_solution_file(solution_path) {
            Some(bks)
        } else {
            // If no sol file, try to find bks.json file
            parse_bks_for_file(path).ok()
        };

        let instance_name = path
            .strip_prefix("./data")
            .or_else(|_| path.strip_prefix("data"))
            .unwrap_or(path)
            .with_extension("")
            .to_string_lossy()
            .replace('\\', "/")
            .to_string();

        progress_bar.set_message(format!(
            "Running benchmark instance {} ({}/{})",
            instance_name,
            i + 1,
            paths.len()
        ));

        let vrp = parse_dataset(path)?;

        let mut terminations: Vec<Termination> = vec![];

        if let Some(timeout) = args.timeout {
            terminations.push(Termination::Duration(timeout));
        }

        if let Some(iterations) = args.iterations {
            terminations.push(Termination::Iterations(iterations));
        }

        if let Some(optimal_sol) = bks {
            terminations.push(Termination::VehiclesAndCosts {
                vehicles: optimal_sol.vehicles,
                costs: optimal_sol.cost,
            });
        }

        let solver_params = SolverParams {
            terminations,
            search_threads: Threads::Multi(args.sthreads as usize),
            insertion_threads: Threads::Multi(args.ithreads as usize),
            debug_options: SolverParamsDebugOptions {
                enable_local_search: true,
            },
            ..SolverParams::default_from_problem(&vrp)
        };

        let solver = Solver::new(vrp, solver_params);

        let result = solver.solve()?;
        let best_solution = result
            .best_solution
            .ok_or(anyhow::anyhow!("No solution found"))?;

        let instance_result = InstanceResult {
            bks,
            cost: best_solution.solution.total_transport_costs(),
            duration: result.duration,
            feasible: best_solution.solution.unassigned_jobs().is_empty()
                && best_solution.score.is_feasible(),
            instance: instance_name,
            iterations: result.iterations,
            vehicles: best_solution.solution.non_empty_routes_count(),
        };

        benchmark_run.instances.push(instance_result);
    }

    let mut out_path = args.out.clone();
    out_path.push(args.name);
    out_path.set_extension("json");

    let file = File::create(out_path)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, &benchmark_run)?;

    Ok(())
}
