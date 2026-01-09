use clap::{Parser, Subcommand};

#[cfg(not(feature = "dhat-heap"))]
use mimalloc::MiMalloc;

use crate::{
    generate::GenerateSubcommands, optimize::OptimizeArgs, optimize_dataset::OptimizeDatasetArgs,
};

mod file_utils;
mod generate;
mod optimize;
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

    #[arg(long, global = true)]
    env: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    Optimize {
        #[command(flatten)]
        args: OptimizeArgs,
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
    let cli = Cli::parse();

    if let Some(env) = cli.env {
        dotenvy::from_filename(env).ok();
    }

    #[cfg(feature = "dhat-heap")]
    let _profiler = dhat::Profiler::new_heap();

    tracing_subscriber::fmt()
        .with_max_level(if cli.debug {
            tracing::Level::DEBUG
        } else {
            tracing::Level::INFO
        })
        .init();

    match cli.command {
        Some(Commands::Optimize { args }) => optimize::run(args).await?,
        Some(Commands::OptimizeDataset { args }) => optimize_dataset::run(args)?,
        Some(Commands::Generate { commands }) => generate::run(commands)?,
        None => {
            // Handle no command provided
        }
    }

    Ok(())
}
