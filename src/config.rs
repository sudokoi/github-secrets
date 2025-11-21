use anyhow::{Context, Result};
use serde::Deserialize;
use std::fs;

/// Configuration file structure containing repository definitions.
#[derive(Debug, Deserialize)]
pub struct Config {
    /// List of repositories to manage secrets for.
    #[serde(default)]
    pub repositories: Vec<Repository>,
    /// Single repository format (converted to repositories list during parsing).
    #[serde(default)]
    pub repository: Option<Repository>,
}

/// Repository configuration with owner, name, and optional display alias.
#[derive(Debug, Deserialize, Clone)]
pub struct Repository {
    /// GitHub username or organization name.
    pub owner: String,
    /// Repository name.
    pub name: String,
    /// Optional friendly name for display in selection menus.
    #[serde(default)]
    pub alias: Option<String>,
}

impl Repository {
    pub fn path(&self) -> String {
        format!("{}/{}", self.owner, self.name)
    }

    pub fn display_name(&self) -> String {
        if let Some(alias) = &self.alias {
            format!("{} ({})", alias, self.path())
        } else {
            self.path()
        }
    }
}

impl Config {
    /// Load configuration from a TOML file.
    /// Converts single repository format to repositories list if needed.
    pub fn from_file(path: &str) -> Result<Self> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path))?;
        let mut config: Config = toml::from_str(&content).context("Failed to parse config file")?;

        // Convert single repository to repositories list if present
        if let Some(repo) = config.repository.take()
            && config.repositories.is_empty()
        {
            config.repositories.push(repo);
        }

        if config.repositories.is_empty() {
            anyhow::bail!("No repositories found in config file");
        }

        Ok(config)
    }

    pub fn get_repositories(&self) -> &[Repository] {
        &self.repositories
    }
}
