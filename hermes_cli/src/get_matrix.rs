use std::{fs::File, io::BufReader, path::PathBuf};

use clap::Args;
use hermes_matrix_providers::{cache::FileCache, travel_matrix_client::TravelMatrixClient};
use hermes_optimizer::json::types::JsonVehicleRoutingProblem;

use indicatif::ProgressBar;

use crate::file_utils::read_folder;

#[derive(Args)]
pub struct GetMatrixArgs {
    /// The file to optimize
    #[arg(short = 'i', long)]
    input: PathBuf,
}

async fn fetch_matrix(
    client: &TravelMatrixClient<FileCache>,
    file: &PathBuf,
) -> anyhow::Result<()> {
    let f = File::open(file)?;
    let reader = BufReader::new(f);
    let content: JsonVehicleRoutingProblem = serde_json::from_reader(reader)?;

    for profile in content.vehicle_profiles {
        let cost_provider = profile.cost_provider;

        client
            .fetch_matrix(&content.locations, cost_provider)
            .await?;
    }

    Ok(())
}

pub async fn run(args: GetMatrixArgs) -> anyhow::Result<()> {
    let paths = if args.input.is_file() {
        vec![args.input]
    } else {
        let mut files = read_folder(&args.input)?;
        files.retain(|path| path.extension().map(|ext| ext == "json").unwrap_or(false));
        files
    };

    let loading_bar = ProgressBar::new(0);
    loading_bar.set_message(format!("{}/{}", 0, paths.len()));

    let client = TravelMatrixClient::default();

    for (i, path) in paths.iter().enumerate() {
        let _ = fetch_matrix(&client, path).await;

        loading_bar.set_message(format!("{}/{}", i + 1, paths.len()));
    }

    Ok(())
}
