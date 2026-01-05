use clap::{Parser, Subcommand};

#[cfg(not(feature = "dhat-heap"))]
use mimalloc::MiMalloc;
use tracing::info;

use crate::{generate::GenerateSubcommands, optimize_dataset::OptimizeDatasetArgs};

mod file_utils;
mod generate;
mod optimize_dataset;
mod parsers;

#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

#[cfg(not(feature = "dhat-heap"))]
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

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
    OptimizeDataset {
        #[command(flatten)]
        args: OptimizeDatasetArgs,
    },
    #[command(visible_alias = "g")]
    Generate {
        #[command(subcommand)]
        commands: GenerateSubcommands,
    },
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    #[cfg(feature = "dhat-heap")]
    let _profiler = dhat::Profiler::new_heap();

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
        Some(Commands::OptimizeDataset { args }) => optimize_dataset::run(args)?,
        Some(Commands::Generate { commands }) => generate::run(commands)?,
        None => {
            // Handle no command provided
        }
    }

    Ok(())
}
