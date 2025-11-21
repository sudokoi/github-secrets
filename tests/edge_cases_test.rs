use github_secrets::config::{Config, Repository};
use github_secrets::validation;

#[test]
fn test_config_empty_repositories() {
    let config_str = r#"
        repositories = []
    "#;

    let result: Result<Config, _> = toml::from_str(config_str);
    assert!(result.is_ok());

    let config = result.unwrap();
    assert!(config.repositories.is_empty());
}

#[test]
fn test_config_single_repository_format() {
    let config_str = r#"
        [repository]
        owner = "test-owner"
        name = "test-repo"
    "#;

    let result: Result<Config, _> = toml::from_str(config_str);
    assert!(result.is_ok());
}

#[test]
fn test_config_multiple_repositories() {
    let config_str = r#"
        [[repositories]]
        owner = "owner1"
        name = "repo1"
        
        [[repositories]]
        owner = "owner2"
        name = "repo2"
        alias = "My Repo"
    "#;

    let result: Result<Config, _> = toml::from_str(config_str);
    assert!(result.is_ok());

    let config = result.unwrap();
    assert_eq!(config.repositories.len(), 2);
}

#[test]
fn test_repository_path_formatting() {
    let repo = Repository {
        owner: "test-owner".to_string(),
        name: "test-repo".to_string(),
        alias: None,
    };

    assert_eq!(repo.path(), "test-owner/test-repo");
}

#[test]
fn test_repository_display_name_with_alias() {
    let repo = Repository {
        owner: "test-owner".to_string(),
        name: "test-repo".to_string(),
        alias: Some("My Test Repo".to_string()),
    };

    let display = repo.display_name();
    assert!(display.contains("My Test Repo"));
    assert!(display.contains("test-owner/test-repo"));
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
fn test_validate_secret_key_edge_cases() {
    // Test minimum length
    assert!(validation::validate_secret_key("a").is_ok());

    // Test maximum length
    let max_key = "a".repeat(100);
    assert!(validation::validate_secret_key(&max_key).is_ok());

    // Test over maximum length
    let too_long = "a".repeat(101);
    assert!(validation::validate_secret_key(&too_long).is_err());

    // Test with leading/trailing spaces (should be trimmed)
    assert!(validation::validate_secret_key("  valid_key  ").is_ok());

    // Test special characters
    assert!(validation::validate_secret_key("key_with_underscore").is_ok());
    assert!(validation::validate_secret_key("key-with-hyphen").is_ok());
    assert!(validation::validate_secret_key("key123").is_ok());
    assert!(validation::validate_secret_key("KEY_UPPER").is_ok());
    assert!(validation::validate_secret_key("key_lower").is_ok());
    assert!(validation::validate_secret_key("MixedCase123").is_ok());
}

#[test]
fn test_validate_secret_key_invalid_characters() {
    assert!(validation::validate_secret_key("key with spaces").is_err());
    assert!(validation::validate_secret_key("key@invalid").is_err());
    assert!(validation::validate_secret_key("key#invalid").is_err());
    assert!(validation::validate_secret_key("key$invalid").is_err());
    assert!(validation::validate_secret_key("key%invalid").is_err());
    assert!(validation::validate_secret_key("key&invalid").is_err());
    assert!(validation::validate_secret_key("key*invalid").is_err());
    assert!(validation::validate_secret_key("key(invalid").is_err());
    assert!(validation::validate_secret_key("key)invalid").is_err());
    assert!(validation::validate_secret_key("key=invalid").is_err());
    assert!(validation::validate_secret_key("key+invalid").is_err());
    assert!(validation::validate_secret_key("key[invalid").is_err());
    assert!(validation::validate_secret_key("key]invalid").is_err());
    assert!(validation::validate_secret_key("key{invalid").is_err());
    assert!(validation::validate_secret_key("key}invalid").is_err());
    assert!(validation::validate_secret_key("key|invalid").is_err());
    assert!(validation::validate_secret_key("key\\invalid").is_err());
    assert!(validation::validate_secret_key("key:invalid").is_err());
    assert!(validation::validate_secret_key("key;invalid").is_err());
    assert!(validation::validate_secret_key("key\"invalid").is_err());
    assert!(validation::validate_secret_key("key'invalid").is_err());
    assert!(validation::validate_secret_key("key<invalid").is_err());
    assert!(validation::validate_secret_key("key>invalid").is_err());
    assert!(validation::validate_secret_key("key,invalid").is_err());
    assert!(validation::validate_secret_key("key.invalid").is_err());
    assert!(validation::validate_secret_key("key?invalid").is_err());
    assert!(validation::validate_secret_key("key/invalid").is_err());
    assert!(validation::validate_secret_key("key!invalid").is_err());
}

#[test]
fn test_validate_repo_owner_edge_cases() {
    // Test minimum length
    assert!(validation::validate_repo_owner("a").is_ok());

    // Test maximum length
    let max_owner = "a".repeat(39);
    assert!(validation::validate_repo_owner(&max_owner).is_ok());

    // Test over maximum length
    let too_long = "a".repeat(40);
    assert!(validation::validate_repo_owner(&too_long).is_err());

    // Test with spaces (should be trimmed)
    assert!(validation::validate_repo_owner("  valid_owner  ").is_ok());
}

#[test]
fn test_validate_repo_name_edge_cases() {
    // Test minimum length
    assert!(validation::validate_repo_name("a").is_ok());

    // Test maximum length
    let max_name = "a".repeat(100);
    assert!(validation::validate_repo_name(&max_name).is_ok());

    // Test over maximum length
    let too_long = "a".repeat(101);
    assert!(validation::validate_repo_name(&too_long).is_err());

    // Test with spaces (should be trimmed)
    assert!(validation::validate_repo_name("  valid_repo  ").is_ok());
}

#[test]
fn test_validate_token_edge_cases() {
    // Test minimum length
    let min_token = "a".repeat(20);
    assert!(validation::validate_token(&min_token).is_ok());

    // Test maximum length
    let max_token = "a".repeat(200);
    assert!(validation::validate_token(&max_token).is_ok());

    // Test over maximum length
    let too_long = "a".repeat(201);
    assert!(validation::validate_token(&too_long).is_err());

    // Test too short
    let too_short = "a".repeat(19);
    assert!(validation::validate_token(&too_short).is_err());

    // Test empty
    assert!(validation::validate_token("").is_err());

    // Test with spaces (should be trimmed)
    let token_with_spaces = format!("  {}  ", "a".repeat(20));
    assert!(validation::validate_token(&token_with_spaces).is_ok());
}
