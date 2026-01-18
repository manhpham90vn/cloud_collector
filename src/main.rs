mod aws;
mod cli;
mod commands;
mod models;
mod output;
mod parallel;
mod ui;
mod utils;
// Future cloud providers (not yet implemented)
// mod gcp;
// mod azure;

use anyhow::Result;
use clap::Parser;

use crate::cli::{AwsCommands, Cli, Provider};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.provider {
        Provider::Aws { command } => {
            match command {
                AwsCommands::Collect {
                    profile,
                    regions,
                    region_services,
                    create_new_file,
                    concurrency,
                } => {
                    // Validate concurrency
                    let concurrency = concurrency.clamp(1, 10);
                    commands::aws::collect_resources(
                        &profile,
                        regions.as_deref(),
                        region_services.as_deref(),
                        create_new_file,
                        concurrency,
                    )
                    .await?;
                }
                AwsCommands::ListServices => {
                    commands::aws::list_services();
                }
            }
        }
        Provider::Gcp => {
            eprintln!("❌ Google Cloud Platform support is not yet implemented");
            eprintln!("   Coming soon!");
            std::process::exit(1);
        }
        Provider::Azure => {
            eprintln!("❌ Microsoft Azure support is not yet implemented");
            eprintln!("   Coming soon!");
            std::process::exit(1);
        }
    }

    Ok(())
}
