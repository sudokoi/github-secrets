use anyhow::Result;
use async_trait::async_trait;

use crate::{config, github, prompt, rate_limit};

#[async_trait]
pub trait GitHubApi: Send + Sync {
    async fn get_secret_info(&self, secret_name: &str) -> Result<Option<github::SecretInfo>>;
    async fn update_secret(&self, secret_name: &str, secret_value: &str) -> Result<()>;
}

pub trait GitHubApiFactory: Send + Sync {
    fn create(&self, token: String, owner: String, repo: String) -> Result<Box<dyn GitHubApi>>;
}

pub struct RealGitHubApi {
    inner: github::GitHubClient,
}

#[async_trait]
impl GitHubApi for RealGitHubApi {
    async fn get_secret_info(&self, secret_name: &str) -> Result<Option<github::SecretInfo>> {
        self.inner.get_secret_info(secret_name).await
    }

    async fn update_secret(&self, secret_name: &str, secret_value: &str) -> Result<()> {
        self.inner.update_secret(secret_name, secret_value).await
    }
}

pub struct RealGitHubApiFactory;

impl GitHubApiFactory for RealGitHubApiFactory {
    fn create(&self, token: String, owner: String, repo: String) -> Result<Box<dyn GitHubApi>> {
        let client = github::GitHubClient::new(token, owner, repo)?;
        Ok(Box::new(RealGitHubApi { inner: client }))
    }
}

pub trait PromptInterface: Send + Sync {
    fn select_repositories(&self, repositories: &[config::Repository]) -> Result<Vec<usize>>;
    fn prompt_secrets(&self) -> Result<Vec<prompt::SecretPair>>;
    fn confirm_secret_update(&self, key: &str, last_updated: Option<&str>) -> Result<bool>;
    fn confirm_retry(&self) -> Result<bool>;

    fn manage_config(&self, initial: config::Config) -> Result<Option<config::Config>>;
}

pub struct RealPrompt;

impl PromptInterface for RealPrompt {
    fn select_repositories(&self, repositories: &[config::Repository]) -> Result<Vec<usize>> {
        crate::prompt::select_repositories(repositories)
    }

    fn prompt_secrets(&self) -> Result<Vec<prompt::SecretPair>> {
        crate::prompt::prompt_secrets()
    }

    fn confirm_secret_update(&self, name: &str, last_updated: Option<&str>) -> Result<bool> {
        crate::prompt::confirm_secret_update(name, last_updated)
    }

    fn confirm_retry(&self) -> Result<bool> {
        crate::prompt::confirm_retry()
    }

    fn manage_config(&self, initial: config::Config) -> Result<Option<config::Config>> {
        crate::prompt::manage_config(initial)
    }
}

#[async_trait]
pub trait RateLimiterInterface: Send + Sync {
    async fn wait_if_needed(&mut self);
    fn release(&mut self);
}

#[derive(Default)]
pub struct RealRateLimiter {
    inner: rate_limit::RateLimiter,
}

impl RealRateLimiter {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl RateLimiterInterface for RealRateLimiter {
    async fn wait_if_needed(&mut self) {
        self.inner.wait_if_needed().await;
    }

    fn release(&mut self) {
        self.inner.release();
    }
}
