use std::{fs::File, io::BufReader, path::PathBuf};

use clap::{Args, arg};
use hermes_matrix_providers::travel_matrix_client::TravelMatrixClient;
use hermes_optimizer::{
    json::types::JsonVehicleRoutingProblem,
    solver::{
        solver::Solver,
        solver_params::{SolverParams, Termination, Threads},
    },
};
use indicatif::ProgressBar;

use crate::parsers;

#[derive(Args)]
pub struct OptimizeArgs {
    /// The file to optimize
    #[arg(short = 'i', long)]
    input: PathBuf,

    #[arg(short, long, value_parser=parsers::parse_duration, default_value = "5s")]
    timeout: jiff::SignedDuration,

    #[arg(long, default_value_t = 1)]
    threads: u8,

    /// Output folder into .sol files
    #[arg(long, short = 'o')]
    out: Option<PathBuf>,
}

pub async fn run(args: OptimizeArgs) -> anyhow::Result<()> {
    let file_name = args
        .input
        .file_name()
        .unwrap()
        .to_string_lossy()
        .into_owned();

    let mut loading_bar = ProgressBar::new(args.timeout.as_secs() as u64);
    loading_bar.set_prefix(file_name);
    loading_bar.set_message("pending...");

    let f = File::open(args.input)?;
    let content: JsonVehicleRoutingProblem = serde_json::from_reader(f)?;
    let client = TravelMatrixClient::default();
    let problem = content.build_problem(&client).await?;

    let mut solver = Solver::new(
        problem,
        SolverParams {
            terminations: vec![Termination::Duration(args.timeout)],
            insertion_threads: Threads::Multi(args.threads as usize),
            run_intensify_search: true,
            ..SolverParams::default()
        },
    );

    // solver.on_best_solution(|best_solution| {
    //     loading_bar.set_message("running...");
    // });

    loading_bar.set_message("running...");

    solver.solve();
    let best_solution = solver.current_best_solution();
    if let Some(best_solution) = best_solution {
        let n_routes = best_solution.solution.non_empty_routes_count();
        let total_transport_cost = best_solution.solution.total_transport_costs();
        loading_bar.finish_with_message(format!(
            "Finished: routes = {}, costs = {}, unassigned = {}",
            n_routes,
            total_transport_cost,
            best_solution.solution.unassigned_jobs().len(),
        ));
    } else {
        loading_bar.finish_with_message("No solution".to_string());
    }

    Ok(())
}
