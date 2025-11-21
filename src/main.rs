mod config;
mod github;
mod prompt;

use anyhow::{Context, Result};
use std::env;
use colored::*;

#[derive(Debug, Clone)]
struct UpdateResult {
    secret_name: String,
    repository: String,
    success: bool,
    error: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    
    let token = env::var("GITHUB_TOKEN")
        .context("GITHUB_TOKEN not found in environment. Please set it in .env file")?;

    let config_path = env::var("CONFIG_PATH").unwrap_or_else(|_| "config.toml".to_string());
    let config = config::Config::from_file(&config_path)
        .with_context(|| format!("Failed to load config from {}", config_path))?;

    let repositories = config.get_repositories();
    let selected_indices = prompt::select_repositories(repositories)
        .context("Failed to select repositories")?;

    let secrets = prompt::prompt_secrets()
        .context("Failed to read secrets from user")?;

    if secrets.is_empty() {
        println!("{}", "No secrets to update.".yellow());
        return Ok(());
    }

    println!("\n{} {} {} {} {}...\n", 
        "Processing".cyan(),
        secrets.len().to_string().bright_cyan(),
        "secret(s) across".cyan(),
        selected_indices.len().to_string().bright_cyan(),
        "repository/repositories".cyan());

    let mut all_results = Vec::new();
    let mut all_failed_secrets: Vec<(usize, prompt::SecretPair)> = Vec::new();

    for &repo_index in &selected_indices {
        let selected_repo = &repositories[repo_index];
        let repo_display = selected_repo.display_name();
        
        println!("{}", "=".repeat(60).bright_black());
        println!("{} {}", "Repository:".bright_cyan(), repo_display.bright_cyan().bold());
        println!("{}", "=".repeat(60).bright_black());

        let github_client = github::GitHubClient::new(
            token.clone(),
            selected_repo.owner.clone(),
            selected_repo.name.clone(),
        )
        .context("Failed to initialize GitHub client")?;

        for secret in &secrets {
            let secret_info = github_client
                .get_secret_info(&secret.key)
                .await
                .context("Failed to check if secret exists")?;

            if let Some(info) = &secret_info {
                let last_updated = info.updated_at.as_deref();
                if !prompt::confirm_secret_update(&secret.key, last_updated)? {
                    println!("{} {} {} {}", 
                        "⊘".yellow(), 
                        "Skipping secret".yellow(),
                        format!("'{}'", secret.key).bright_yellow(),
                        format!("in {}", repo_display).yellow());
                    all_results.push(UpdateResult {
                        secret_name: secret.key.clone(),
                        repository: repo_display.clone(),
                        success: false,
                        error: Some("User declined to overwrite".to_string()),
                    });
                    continue;
                }
            }

            match github_client.update_secret(&secret.key, &secret.value).await {
                Ok(()) => {
                    println!("{} {} {} {} {}", 
                        "✓".green(),
                        "Successfully updated secret".green(),
                        format!("'{}'", secret.key).bright_green(),
                        "in".green(),
                        repo_display.bright_green());
                    all_results.push(UpdateResult {
                        secret_name: secret.key.clone(),
                        repository: repo_display.clone(),
                        success: true,
                        error: None,
                    });
                }
                Err(e) => {
                    // Extract detailed error message from error chain
                    let mut error_chain = vec![format!("{}", e)];
                    let mut current = e.source();
                    while let Some(err) = current {
                        error_chain.push(format!("{}", err));
                        current = err.source();
                    }
                    let detailed_error = error_chain.join(" → ");
                    
                    println!("{} {} {} {} {}",
                        "✗".red(),
                        "Failed to update secret".red(),
                        format!("'{}'", secret.key).bright_red(),
                        "in".red(),
                        repo_display.bright_red());
                    println!("{} {}", "  Reason:".bright_red(), detailed_error.bright_red());
                    
                    all_results.push(UpdateResult {
                        secret_name: secret.key.clone(),
                        repository: repo_display.clone(),
                        success: false,
                        error: Some(detailed_error.clone()),
                    });
                    all_failed_secrets.push((repo_index, secret.clone()));
                }
            }
        }
        println!();
    }

    println!("\n{}", "=".repeat(60).bright_black());
    println!("{}", "Overall Summary".bright_cyan().bold());
    println!("{}", "=".repeat(60).bright_black());
    
    let success_count = all_results.iter().filter(|r| r.success).count();
    let failure_count = all_results.len() - success_count;
    
    println!("{} {}", "Total operations:".cyan(), all_results.len().to_string().bright_cyan());
    println!("{} {}", "Successful:".green(), success_count.to_string().bright_green());
    println!("{} {}", "Failed:".red(), failure_count.to_string().bright_red());

    // Aggregate results by repository for breakdown
    use std::collections::HashMap;
    let mut repo_results: HashMap<String, Vec<&UpdateResult>> = HashMap::new();
    for result in &all_results {
        repo_results
            .entry(result.repository.clone())
            .or_insert_with(Vec::new)
            .push(result);
    }

    println!("\n{}", "Per-repository breakdown:".cyan());
    for (repo, results) in &repo_results {
        let repo_success = results.iter().filter(|r| r.success).count();
        let repo_failure = results.len() - repo_success;
        println!("  {}: {} {}, {} {}", 
            repo.bright_cyan(),
            repo_success.to_string().bright_green(),
            "successful".green(),
            repo_failure.to_string().bright_red(),
            "failed".red());
    }

    if failure_count > 0 {
        println!("\n{}", "Failed operations:".red().bold());
        for result in &all_results {
            if !result.success {
                println!("  {} {} {} {}",
                    "✗".red(),
                    format!("{}", result.secret_name).bright_red(),
                    format!("in {}", result.repository).red(),
                    format!("→ {}", result.error.as_ref().unwrap_or(&"Unknown error".to_string())).bright_red());
            }
        }

        if prompt::confirm_retry()? {
            println!("\n{}", "Retrying failed operations...\n".yellow());
            
            for (repo_index, secret) in &all_failed_secrets {
                let repo = &repositories[*repo_index];
                let repo_display = repo.display_name();
                
                let github_client = github::GitHubClient::new(
                    token.clone(),
                    repo.owner.clone(),
                    repo.name.clone(),
                )
                .context("Failed to initialize GitHub client")?;
                
                match github_client.update_secret(&secret.key, &secret.value).await {
                    Ok(()) => {
                        println!("{} {} {} {} {} {}",
                            "✓".green(),
                            "Successfully updated secret".green(),
                            format!("'{}'", secret.key).bright_green(),
                            "in".green(),
                            repo_display.bright_green(),
                            "(retry)".bright_black());
                        if let Some(result) = all_results.iter_mut()
                            .find(|r| r.secret_name == secret.key && r.repository == repo_display) {
                            result.success = true;
                            result.error = None;
                        }
                    }
                    Err(e) => {
                        let mut error_chain = vec![format!("{}", e)];
                        let mut current = e.source();
                        while let Some(err) = current {
                            error_chain.push(format!("{}", err));
                            current = err.source();
                        }
                        let detailed_error = error_chain.join(" → ");
                        
                        println!("{} {} {} {} {} {}",
                            "✗".red(),
                            "Failed to update secret".red(),
                            format!("'{}'", secret.key).bright_red(),
                            "in".red(),
                            repo_display.bright_red(),
                            "(retry)".bright_black());
                        println!("{} {}", "  Reason:".bright_red(), detailed_error.bright_red());
                    }
                }
            }

            let final_success = all_results.iter().filter(|r| r.success).count();
            let final_failure = all_results.len() - final_success;
            
            println!("\n{}", "=".repeat(60).bright_black());
            println!("{}", "Final Summary".bright_cyan().bold());
            println!("{}", "=".repeat(60).bright_black());
            println!("{} {}", "Total operations:".cyan(), all_results.len().to_string().bright_cyan());
            println!("{} {}", "Successful:".green(), final_success.to_string().bright_green());
            println!("{} {}", "Failed:".red(), final_failure.to_string().bright_red());
        }
    }

    Ok(())
}

