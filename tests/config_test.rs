use github_secrets::config::{Config, Repository};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_config_parsing_valid() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    let content = r#"
    [[repositories]]
    owner = "test_owner"
    name = "test_repo"
    alias = "Test Repo"
    
    [[repositories]]
    owner = "another_owner"
    name = "another_repo"
    "#;

    fs::write(&config_path, content).unwrap();

    let config = Config::from_file(config_path.to_str().unwrap()).unwrap();
    assert_eq!(config.repositories.len(), 2);

    assert_eq!(config.repositories[0].owner, "test_owner");
    assert_eq!(config.repositories[0].name, "test_repo");
    assert_eq!(config.repositories[0].alias, Some("Test Repo".to_string()));

    assert_eq!(config.repositories[1].owner, "another_owner");
    assert_eq!(config.repositories[1].name, "another_repo");
    assert_eq!(config.repositories[1].alias, None);
}

#[test]
fn test_config_parsing_invalid_toml() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    fs::write(&config_path, "invalid toml content").unwrap();

    let result = Config::from_file(config_path.to_str().unwrap());
    assert!(result.is_err());
}

#[test]
fn test_config_parsing_missing_fields() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    let content = r#"
    [[repositories]]
    owner = "test_owner"
    # name is missing
    "#;

    fs::write(&config_path, content).unwrap();

    let result = Config::from_file(config_path.to_str().unwrap());
    assert!(result.is_err());
}

#[test]
fn test_config_validation_invalid_owner() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    let content = r#"
    [[repositories]]
    owner = ""  # Empty owner is invalid
    name = "test_repo"
    "#;

    fs::write(&config_path, content).unwrap();

    let result = Config::from_file(config_path.to_str().unwrap());
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Invalid owner"));
}

#[test]
fn test_repository_methods() {
    let repo = Repository {
        owner: "owner".to_string(),
        name: "repo".to_string(),
        alias: None,
    };

    assert_eq!(repo.path(), "owner/repo");
    assert_eq!(repo.display_name(), "owner/repo");

    let repo_with_alias = Repository {
        owner: "owner".to_string(),
        name: "repo".to_string(),
        alias: Some("Alias".to_string()),
    };

    assert_eq!(repo_with_alias.display_name(), "Alias (owner/repo)");
}
