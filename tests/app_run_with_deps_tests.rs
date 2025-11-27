use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;

use github_secrets::app::App;
use github_secrets::app_deps::{
    GitHubApi, GitHubApiFactory, PromptInterface, RateLimiterInterface,
};
use github_secrets::config;
use github_secrets::prompt;

struct MockGitHubApi;

#[async_trait]
impl GitHubApi for MockGitHubApi {
    async fn get_secret_info(
        &self,
        _secret_name: &str,
    ) -> Result<Option<github_secrets::github::SecretInfo>> {
        Ok(None)
    }

    async fn update_secret(&self, _secret_name: &str, _secret_value: &str) -> Result<()> {
        Ok(())
    }
}

struct MockFactory;
impl GitHubApiFactory for MockFactory {
    fn create(&self, _token: String, _owner: String, _repo: String) -> Result<Box<dyn GitHubApi>> {
        Ok(Box::new(MockGitHubApi))
    }
}

struct MockPrompt;
impl PromptInterface for MockPrompt {
    fn select_repositories(&self, _repositories: &[config::Repository]) -> Result<Vec<usize>> {
        // select first repository
        Ok(vec![0])
    }

    fn prompt_secrets(&self) -> Result<Vec<prompt::SecretPair>> {
        Ok(vec![prompt::SecretPair {
            key: "TEST_KEY".to_string(),
            value: "secret".to_string(),
        }])
    }

    fn confirm_secret_update(&self, _key: &str, _last_updated: Option<&str>) -> Result<bool> {
        Ok(true)
    }

    fn confirm_retry(&self) -> Result<bool> {
        Ok(false)
    }
}

struct MockRateLimiter;

#[async_trait]
impl RateLimiterInterface for MockRateLimiter {
    async fn wait_if_needed(&mut self) {
        // no-op
    }

    fn release(&mut self) {}
}

#[tokio::test]
async fn test_run_with_deps_success_path() -> Result<()> {
    let factory = MockFactory;
    let prompt = MockPrompt;
    let mut rate_limiter = MockRateLimiter;

    let config = config::Config {
        repositories: vec![config::Repository {
            owner: "owner".to_string(),
            name: "repo".to_string(),
            alias: None,
        }],
        repository: None,
    };

    let token = Arc::new("token".to_string());

    let res = App::run_with_deps(&factory, &prompt, &mut rate_limiter, token, config).await;
    assert!(res.is_ok());
    Ok(())
}
