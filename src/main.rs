pub mod app;
pub mod app_deps;
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

#[tokio::main]
async fn main() -> Result<()> {
    app::App::run().await
}
