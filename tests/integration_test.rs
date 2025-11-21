use github_secrets::config::{Config, Repository};
use std::fs;
use tempfile::TempDir;

/// Integration test for config parsing with various scenarios
#[test]
fn test_config_parsing_integration() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    // Test complex config with multiple repositories
    let config_content = r#"
[[repositories]]
owner = "org1"
name = "repo1"
alias = "Production Repo"

[[repositories]]
owner = "org2"
name = "repo2"

[[repositories]]
owner = "org3"
name = "repo3"
alias = "Staging"
"#;
    fs::write(&config_path, config_content).unwrap();

    let config = Config::from_file(config_path.to_str().unwrap()).unwrap();
    assert_eq!(config.repositories.len(), 3);

    // Verify first repository
    assert_eq!(config.repositories[0].owner, "org1");
    assert_eq!(config.repositories[0].name, "repo1");
    assert_eq!(
        config.repositories[0].alias,
        Some("Production Repo".to_string())
    );
    assert_eq!(
        config.repositories[0].display_name(),
        "Production Repo (org1/repo1)"
    );

    // Verify second repository (no alias)
    assert_eq!(config.repositories[1].owner, "org2");
    assert_eq!(config.repositories[1].name, "repo2");
    assert_eq!(config.repositories[1].alias, None);
    assert_eq!(config.repositories[1].display_name(), "org2/repo2");

    // Verify third repository
    assert_eq!(config.repositories[2].owner, "org3");
    assert_eq!(config.repositories[2].name, "repo3");
    assert_eq!(config.repositories[2].alias, Some("Staging".to_string()));
}

/// Test repository path formatting
#[test]
fn test_repository_path_formatting() {
    let repo = Repository {
        owner: "my-org".to_string(),
        name: "my-repo".to_string(),
        alias: None,
    };

    assert_eq!(repo.path(), "my-org/my-repo");
}

/// Test repository display name variations
#[test]
fn test_repository_display_name_variations() {
    let repo_with_alias = Repository {
        owner: "org".to_string(),
        name: "repo".to_string(),
        alias: Some("My Awesome Repo".to_string()),
    };

    let repo_without_alias = Repository {
        owner: "org".to_string(),
        name: "repo".to_string(),
        alias: None,
    };

    assert_eq!(repo_with_alias.display_name(), "My Awesome Repo (org/repo)");
    assert_eq!(repo_without_alias.display_name(), "org/repo");
}
