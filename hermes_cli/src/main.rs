use std::path::PathBuf;

use clap::{Parser, Subcommand};
use tracing::info;

mod parsers;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    #[arg(short, long)]
    debug: bool,
}

#[derive(Subcommand)]
enum Commands {
    Optimize {
        // #[arg(short, long)]
        // input: PathBuf,
        /// The number of threads to use for optimization (default: 1)
        #[arg(short, long, default_value_t = 1)]
        threads: u8,

        /// Timeout for the solver (e.g., "30s", "5m", "PT1H30M")
        #[arg(short, long, value_parser = parsers::parse_duration)]
        duration: jiff::SignedDuration,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    tracing_subscriber::fmt()
        .with_max_level(if cli.debug {
            tracing::Level::DEBUG
        } else {
            tracing::Level::INFO
        })
        .init();

    match cli.command {
        Some(Commands::Optimize { duration, .. }) => {
            info!("{}", duration)
        }
        None => {
            // Handle no command provided
        }
    }
}
