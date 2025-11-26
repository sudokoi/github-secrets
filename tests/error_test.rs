use github_secrets::error::format_error_chain;
use github_secrets::errors::{ConfigError, GitHubError, ValidationError};

#[test]
fn test_format_error_chain_single_error() {
    let error = anyhow::anyhow!("Simple error");
    let formatted = format_error_chain(&error);
    assert_eq!(formatted, "Simple error");
}

#[test]
fn test_format_error_chain_nested_errors() {
    let base_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
    let error = anyhow::Error::new(base_error).context("Failed to read config");
    let formatted = format_error_chain(&error);

    // Should contain both error messages
    assert!(formatted.contains("Failed to read config"));
    assert!(formatted.contains("File not found"));
    assert!(formatted.contains("→"));
}

#[test]
fn test_format_error_chain_deeply_nested() {
    let base = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "Access denied");
    let error = anyhow::Error::new(base)
        .context("Failed to open file")
        .context("Failed to load configuration")
        .context("Application startup failed");

    let formatted = format_error_chain(&error);

    // Should contain all error messages in chain
    assert!(formatted.contains("Application startup failed"));
    assert!(formatted.contains("Failed to load configuration"));
    assert!(formatted.contains("Failed to open file"));
    assert!(formatted.contains("Access denied"));

    // Should have arrows connecting them
    let arrow_count = formatted.matches("→").count();
    assert!(
        arrow_count >= 3,
        "Should have at least 3 arrows for 4-level chain"
    );
}

#[test]
fn test_github_error_api_error_display() {
    let error = GitHubError::ApiError {
        status_code: 404,
        message: "Repository not found".to_string(),
        documentation_url: Some("https://docs.github.com".to_string()),
    };

    let display = format!("{}", error);
    assert!(display.contains("404"));
    assert!(display.contains("Repository not found"));
}

#[test]
fn test_github_error_http_error_display() {
    let error = GitHubError::HttpError("Connection timeout".to_string());

    let display = format!("{}", error);
    assert!(display.contains("HTTP error"));
    assert!(display.contains("Connection timeout"));
}

#[test]
fn test_github_error_encryption_display() {
    let error = GitHubError::EncryptionError("Invalid public key".to_string());

    let display = format!("{}", error);
    assert!(display.contains("encrypt secret"));
    assert!(display.contains("Invalid public key"));
}

#[test]
fn test_github_error_public_key_error_display() {
    let error = GitHubError::PublicKeyError("Failed to fetch key".to_string());

    let display = format!("{}", error);
    assert!(display.contains("public key"));
    assert!(display.contains("Failed to fetch key"));
}

#[test]
fn test_github_error_invalid_public_key_display() {
    let error = GitHubError::InvalidPublicKey("Malformed key data".to_string());

    let display = format!("{}", error);
    assert!(display.contains("Invalid public key"));
    assert!(display.contains("Malformed key data"));
}

#[test]
fn test_github_error_uri_error_display() {
    let error = GitHubError::UriError("Invalid URI format".to_string());

    let display = format!("{}", error);
    assert!(display.contains("URI error"));
    assert!(display.contains("Invalid URI format"));
}

#[test]
fn test_config_error_parse_display() {
    let error = ConfigError::ParseError("Invalid TOML syntax".to_string());

    let display = format!("{}", error);
    assert!(display.contains("parse config"));
    assert!(display.contains("Invalid TOML syntax"));
}

#[test]
fn test_config_error_read_display() {
    let error = ConfigError::ReadError("Permission denied".to_string());

    let display = format!("{}", error);
    assert!(display.contains("read config"));
    assert!(display.contains("Permission denied"));
}

#[test]
fn test_config_error_no_repositories_display() {
    let error = ConfigError::NoRepositories;

    let display = format!("{}", error);
    assert!(display.contains("No repositories"));
}

#[test]
fn test_config_error_invalid_repository_display() {
    let error = ConfigError::InvalidRepository("Missing owner field".to_string());

    let display = format!("{}", error);
    assert!(display.contains("Invalid repository"));
    assert!(display.contains("Missing owner field"));
}

#[test]
fn test_validation_error_secret_key_display() {
    let error = ValidationError::SecretKey("Key contains spaces".to_string());

    let display = format!("{}", error);
    assert!(display.contains("Secret key"));
    assert!(display.contains("Key contains spaces"));
}

#[test]
fn test_validation_error_repository_owner_display() {
    let error = ValidationError::RepositoryOwner("Owner name too long".to_string());

    let display = format!("{}", error);
    assert!(display.contains("Repository owner"));
    assert!(display.contains("Owner name too long"));
}

#[test]
fn test_validation_error_repository_name_display() {
    let error = ValidationError::RepositoryName("Name contains invalid characters".to_string());

    let display = format!("{}", error);
    assert!(display.contains("Repository name"));
    assert!(display.contains("Name contains invalid characters"));
}

#[test]
fn test_octocrab_error_conversion() {
    // Test that octocrab errors can be converted to GitHubError
    // This is a compile-time check more than runtime
    fn _accepts_github_error(_e: GitHubError) {}

    // If this compiles, the From trait is implemented correctly
    // We can't easily create an octocrab::Error without complex setup,
    // but we can verify the trait exists
    use std::convert::From;

    // This will fail to compile if From<octocrab::Error> is not implemented
    fn _test_conversion(e: octocrab::Error) -> GitHubError {
        GitHubError::from(e)
    }
}
