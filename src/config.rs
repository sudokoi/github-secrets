//! Configuration file parsing and validation.
//!
//! This module handles loading and validating TOML configuration files
//! that define GitHub repositories for secret management.

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
    /// Get the repository path in the format "owner/repo".
    ///
    /// # Returns
    ///
    /// Returns a string in the format "{owner}/{name}".
    pub fn path(&self) -> String {
        format!("{}/{}", self.owner, self.name)
    }

    /// Get the display name for the repository.
    ///
    /// If an alias is set, returns "{alias} ({owner}/{name})", otherwise returns "{owner}/{name}".
    ///
    /// # Returns
    ///
    /// Returns a formatted string suitable for display in UI.
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

        // Validate all repositories
        for (idx, repo) in config.repositories.iter().enumerate() {
            crate::validation::validate_repo_owner(&repo.owner)
                .with_context(|| format!("Invalid owner in repository #{}", idx + 1))?;
            crate::validation::validate_repo_name(&repo.name)
                .with_context(|| format!("Invalid repository name in repository #{}", idx + 1))?;
        }

        Ok(config)
    }

    /// Get a reference to the list of repositories.
    ///
    /// # Returns
    ///
    /// Returns a slice of all configured repositories.
    pub fn get_repositories(&self) -> &[Repository] {
        &self.repositories
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_repository_path() {
        let repo = Repository {
            owner: "test-owner".to_string(),
            name: "test-repo".to_string(),
            alias: None,
        };
        assert_eq!(repo.path(), "test-owner/test-repo");
    }

    #[test]
    fn test_repository_display_name_without_alias() {
        let repo = Repository {
            owner: "test-owner".to_string(),
            name: "test-repo".to_string(),
            alias: None,
        };
        assert_eq!(repo.display_name(), "test-owner/test-repo");
    }

    #[test]
    fn test_repository_display_name_with_alias() {
        let repo = Repository {
            owner: "test-owner".to_string(),
            name: "test-repo".to_string(),
            alias: Some("My Repo".to_string()),
        };
        assert_eq!(repo.display_name(), "My Repo (test-owner/test-repo)");
    }

    #[test]
    fn test_config_from_file_with_repositories_list() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        let config_content = r#"
[[repositories]]
owner = "owner1"
name = "repo1"

[[repositories]]
owner = "owner2"
name = "repo2"
alias = "Repo 2"
"#;
        fs::write(&config_path, config_content).unwrap();

        let config = Config::from_file(config_path.to_str().unwrap()).unwrap();
        assert_eq!(config.repositories.len(), 2);
        assert_eq!(config.repositories[0].owner, "owner1");
        assert_eq!(config.repositories[0].name, "repo1");
        assert_eq!(config.repositories[1].owner, "owner2");
        assert_eq!(config.repositories[1].name, "repo2");
        assert_eq!(config.repositories[1].alias, Some("Repo 2".to_string()));
    }

    #[test]
    fn test_config_from_file_with_single_repository() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        let config_content = r#"
[repository]
owner = "owner1"
name = "repo1"
"#;
        fs::write(&config_path, config_content).unwrap();

        let config = Config::from_file(config_path.to_str().unwrap()).unwrap();
        assert_eq!(config.repositories.len(), 1);
        assert_eq!(config.repositories[0].owner, "owner1");
        assert_eq!(config.repositories[0].name, "repo1");
    }

    #[test]
    fn test_config_from_file_single_repository_ignored_when_repositories_exist() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        let config_content = r#"
[repository]
owner = "owner1"
name = "repo1"

[[repositories]]
owner = "owner2"
name = "repo2"
"#;
        fs::write(&config_path, config_content).unwrap();

        let config = Config::from_file(config_path.to_str().unwrap()).unwrap();
        // Single repository should be ignored when repositories list exists
        assert_eq!(config.repositories.len(), 1);
        assert_eq!(config.repositories[0].owner, "owner2");
        assert_eq!(config.repositories[0].name, "repo2");
    }

    #[test]
    fn test_config_from_file_empty_repositories() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        let config_content = r#"
"#;
        fs::write(&config_path, config_content).unwrap();

        let result = Config::from_file(config_path.to_str().unwrap());
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("No repositories found")
        );
    }

    #[test]
    fn test_config_get_repositories() {
        let config = Config {
            repositories: vec![
                Repository {
                    owner: "owner1".to_string(),
                    name: "repo1".to_string(),
                    alias: None,
                },
                Repository {
                    owner: "owner2".to_string(),
                    name: "repo2".to_string(),
                    alias: None,
                },
            ],
            repository: None,
        };

        let repos = config.get_repositories();
        assert_eq!(repos.len(), 2);
        assert_eq!(repos[0].owner, "owner1");
        assert_eq!(repos[1].owner, "owner2");
    }
}
