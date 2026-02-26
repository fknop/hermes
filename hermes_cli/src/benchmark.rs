use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, BufWriter},
    path::PathBuf,
};

use clap::{Args, Subcommand};
use comfy_table::{Cell, Color, ContentArrangement, Table};
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
    Compare {
        #[command(flatten)]
        args: CompareBenchmarkArgs,
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

#[derive(Args)]
pub struct CompareBenchmarkArgs {
    baseline: PathBuf,
    target: PathBuf,
}

pub fn run(subcommand: BenchmarkSubcommands) -> Result<(), anyhow::Error> {
    match subcommand {
        BenchmarkSubcommands::Run { args } => run_benchmark(args),
        BenchmarkSubcommands::Compare { args } => compare_benchmarks(args),
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
    pub instances: HashMap<String, InstanceResult>,
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
            instance: instance_name.clone(),
            iterations: result.iterations,
            vehicles: best_solution.solution.non_empty_routes_count(),
        };

        benchmark_run
            .instances
            .insert(instance_name, instance_result);
    }

    let mut out_path = args.out.clone();
    out_path.push(args.name);
    out_path.set_extension("json");

    let file = File::create(out_path)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, &benchmark_run)?;

    Ok(())
}

fn read_benchmark_run(path: PathBuf) -> anyhow::Result<BenchmarkRun> {
    let file = std::fs::File::open(path)?;
    let reader = BufReader::new(file);
    let run: BenchmarkRun = serde_json::from_reader(reader)?;
    Ok(run)
}

struct InstanceDiff {
    pub instance: String,
    pub baseline_cost: f64,
    pub target_cost: f64,
    pub delta_percent: f64, // (target - baseline) / baseline * 100
    pub is_regression: bool,
    pub is_improvement: bool,
    // Gap to BKS if available
    pub baseline_gap_percent: Option<f64>,
    pub target_gap_percent: Option<f64>,
}

fn compare_runs(
    baseline: &BenchmarkRun,
    target: &BenchmarkRun,
    threshold_pct: f64,
) -> Vec<InstanceDiff> {
    let mut diffs = vec![];

    for (name, target_result) in &target.instances {
        if let Some(baseline_result) = baseline.instances.get(name) {
            let delta_pct =
                (target_result.cost - baseline_result.cost) / baseline_result.cost * 100.0;

            diffs.push(InstanceDiff {
                instance: name.clone(),
                baseline_cost: baseline_result.cost,
                target_cost: target_result.cost,
                delta_percent: delta_pct,
                is_regression: delta_pct > threshold_pct,
                is_improvement: delta_pct < -threshold_pct,
                baseline_gap_percent: baseline_result
                    .bks
                    .map(|bks| (baseline_result.cost - bks.cost) / bks.cost * 100.0),
                target_gap_percent: target_result
                    .bks
                    .map(|bks| (target_result.cost - bks.cost) / bks.cost * 100.0),
            });
        }
    }

    // Sort: regressions first, then improvements, then neutral
    diffs.sort_by(|a, b| b.delta_percent.partial_cmp(&a.delta_percent).unwrap());
    diffs
}

fn print_comparison_table(diffs: &[InstanceDiff], threshold_pct: f64) {
    let mut table = Table::new();
    table.set_content_arrangement(ContentArrangement::Dynamic);
    table.set_header(vec![
        "Instance", "Baseline", "Target", "Delta%", "BKS Gap%", "Status",
    ]);

    for diff in diffs {
        let (status_str, row_color) = if diff.is_regression {
            ("⚠ REGRESSION", Some(Color::Red))
        } else if diff.is_improvement {
            ("✓ IMPROVEMENT", Some(Color::Green))
        } else {
            ("~ neutral", None)
        };

        let bks_gap = diff
            .target_gap_percent
            .map(|g| format!("{:+.1}%", g))
            .unwrap_or_else(|| "-".to_string());

        let mut row = vec![
            Cell::new(&diff.instance),
            Cell::new(format!("{:.1}", diff.baseline_cost)),
            Cell::new(format!("{:.1}", diff.target_cost)),
            Cell::new(format!("{:+.2}%", diff.delta_percent)),
            Cell::new(bks_gap),
            Cell::new(status_str),
        ];

        if let Some(color) = row_color {
            row = row.into_iter().map(|c| c.fg(color)).collect();
        }

        table.add_row(row);
    }

    // Summary line
    let regressions = diffs.iter().filter(|d| d.is_regression).count();
    let improvements = diffs.iter().filter(|d| d.is_improvement).count();

    println!("{table}");
    println!(
        "\n{} regression(s), {} improvement(s), {} neutral  [threshold: {:.1}%]",
        regressions,
        improvements,
        diffs.len() - regressions - improvements,
        threshold_pct
    );
}

// TODO: compare number of vehicles as well
fn compare_benchmarks(args: CompareBenchmarkArgs) -> Result<(), anyhow::Error> {
    let baseline = read_benchmark_run(args.baseline)?;
    let target = read_benchmark_run(args.target)?;

    let comparison = compare_runs(&baseline, &target, 0.1);
    print_comparison_table(&comparison, 0.1);

    Ok(())
}
