mod config;
mod github;
mod prompt;

use anyhow::{Context, Result};
use std::env;

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
        println!("No secrets to update.");
        return Ok(());
    }

    println!("\nProcessing {} secret(s) across {} repository/repositories...\n", 
        secrets.len(), selected_indices.len());

    let mut all_results = Vec::new();
    let mut all_failed_secrets: Vec<(usize, prompt::SecretPair)> = Vec::new();

    for &repo_index in &selected_indices {
        let selected_repo = &repositories[repo_index];
        let repo_display = selected_repo.display_name();
        
        println!("{}", "=".repeat(60));
        println!("Repository: {}", repo_display);
        println!("{}", "=".repeat(60));

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

            // Prompt for confirmation if secret already exists
            if let Some(info) = &secret_info {
                let last_updated = info.updated_at.as_deref();
                if !prompt::confirm_secret_update(&secret.key, last_updated)? {
                    println!("Skipping secret '{}' in {}", secret.key, repo_display);
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
                    println!("✓ Successfully updated secret '{}' in {}", secret.key, repo_display);
                    all_results.push(UpdateResult {
                        secret_name: secret.key.clone(),
                        repository: repo_display.clone(),
                        success: true,
                        error: None,
                    });
                }
                Err(e) => {
                    let error_msg = format!("{}", e);
                    println!("✗ Failed to update secret '{}' in {}: {}", secret.key, repo_display, error_msg);
                    all_results.push(UpdateResult {
                        secret_name: secret.key.clone(),
                        repository: repo_display.clone(),
                        success: false,
                        error: Some(error_msg.clone()),
                    });
                    all_failed_secrets.push((repo_index, secret.clone()));
                }
            }
        }
        println!();
    }

    println!("\n{}", "=".repeat(60));
    println!("Overall Summary");
    println!("{}", "=".repeat(60));
    
    let success_count = all_results.iter().filter(|r| r.success).count();
    let failure_count = all_results.len() - success_count;
    
    println!("Total operations: {}", all_results.len());
    println!("Successful: {}", success_count);
    println!("Failed: {}", failure_count);

    // Aggregate results by repository for breakdown
    use std::collections::HashMap;
    let mut repo_results: HashMap<String, Vec<&UpdateResult>> = HashMap::new();
    for result in &all_results {
        repo_results
            .entry(result.repository.clone())
            .or_insert_with(Vec::new)
            .push(result);
    }

    println!("\nPer-repository breakdown:");
    for (repo, results) in &repo_results {
        let repo_success = results.iter().filter(|r| r.success).count();
        let repo_failure = results.len() - repo_success;
        println!("  {}: {} successful, {} failed", repo, repo_success, repo_failure);
    }

    if failure_count > 0 {
        println!("\nFailed operations:");
        for result in &all_results {
            if !result.success {
                println!("  - {} in {}: {}", 
                    result.secret_name, 
                    result.repository,
                    result.error.as_ref().unwrap_or(&"Unknown error".to_string()));
            }
        }

        if prompt::confirm_retry()? {
            println!("\nRetrying failed operations...\n");
            
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
                        println!("✓ Successfully updated secret '{}' in {} (retry)", secret.key, repo_display);
                        if let Some(result) = all_results.iter_mut()
                            .find(|r| r.secret_name == secret.key && r.repository == repo_display) {
                            result.success = true;
                            result.error = None;
                        }
                    }
                    Err(e) => {
                        println!("✗ Failed to update secret '{}' in {} (retry): {}", secret.key, repo_display, e);
                    }
                }
            }

            let final_success = all_results.iter().filter(|r| r.success).count();
            let final_failure = all_results.len() - final_success;
            
            println!("\n{}", "=".repeat(60));
            println!("Final Summary");
            println!("{}", "=".repeat(60));
            println!("Total operations: {}", all_results.len());
            println!("Successful: {}", final_success);
            println!("Failed: {}", final_failure);
        }
    }

    Ok(())
}

