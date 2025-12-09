pub mod app;
pub mod app_deps;
pub mod cli;
pub mod config;
pub mod constants;
pub mod error;
pub mod errors;
pub mod github;
pub mod paths;
pub mod prompt;
pub mod rate_limit;
pub mod validation;

use anyhow::Result;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = cli::Cli::parse();

    match cli.command {
        Some(cli::Commands::Config) => app::App::config().await,
        None => app::App::run().await,
    }
}
