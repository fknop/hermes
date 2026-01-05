use std::path::PathBuf;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum GenerateSubcommands {
    JsonSchema {
        /// Output folder into .sol files
        #[arg(long, short = 'o')]
        out: PathBuf,
    },
}

pub fn run(subcommand: GenerateSubcommands) -> Result<(), anyhow::Error> {
    match subcommand {
        GenerateSubcommands::JsonSchema { out } => {
            let schema = hermes_optimizer::json::schema::generate_json_schema()?;

            if let Some(parent) = out.parent() {
                std::fs::create_dir_all(parent)?;
            }

            std::fs::write(out, schema)?;
        }
    }

    Ok(())
}
