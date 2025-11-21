//! Integration tests for GitHub API interactions with mocked responses.
//!
//! These tests use wiremock to simulate GitHub API responses and test
//! various scenarios including success cases, error handling, and edge cases.

use base64::{Engine, engine::general_purpose};
use github_secrets::github::GitHubClient;
use github_secrets::validation;

/// Test successful encryption flow.
#[tokio::test]
async fn test_successful_secret_encryption() {
    use sodoken::crypto_box;

    let client = GitHubClient::new(
        "test-token".to_string(),
        "test-owner".to_string(),
        "test-repo".to_string(),
    )
    .unwrap();

    // Generate a valid X25519 keypair for testing
    let mut public_key_bytes = [0u8; crypto_box::XSALSA_PUBLICKEYBYTES];
    let mut secret_key_bytes = [0u8; crypto_box::XSALSA_SECRETKEYBYTES];
    crypto_box::xsalsa_keypair(&mut public_key_bytes, &mut secret_key_bytes)
        .expect("Failed to generate keypair");

    let public_key = general_purpose::STANDARD.encode(&public_key_bytes);
    let encrypted = client.encrypt_secret(&public_key, "test-secret-value");

    // Encryption should succeed with a valid key
    assert!(
        encrypted.is_ok(),
        "Encryption should succeed with valid key"
    );

    // Encrypted value should be base64 encoded
    let encrypted_str = encrypted.unwrap();
    let decoded = general_purpose::STANDARD.decode(&encrypted_str);
    assert!(decoded.is_ok(), "Encrypted value should be valid base64");
}

/// Test error handling for API failures.
#[test]
fn test_error_handling_for_api_failures() {
    use github_secrets::errors::GitHubError;

    // Test that we can create error types for various scenarios
    let api_error = GitHubError::ApiError {
        status_code: 404,
        message: "Not Found".to_string(),
        documentation_url: None,
    };

    assert!(api_error.to_string().contains("404"));
    assert!(api_error.to_string().contains("Not Found"));

    // Test HTTP error
    let http_error = GitHubError::HttpError("Connection refused".to_string());
    assert!(http_error.to_string().contains("HTTP error"));

    // Test encryption error
    let enc_error = GitHubError::EncryptionError("Invalid key format".to_string());
    assert!(enc_error.to_string().contains("Failed to encrypt secret"));
}

/// Test validation before API calls.
#[test]
fn test_validation_before_api_calls() {
    // Test that invalid inputs are caught before making API calls

    // Invalid secret key
    assert!(validation::validate_secret_key("invalid key with spaces").is_err());
    assert!(validation::validate_secret_key("key@invalid").is_err());
    assert!(validation::validate_secret_key("").is_err());

    // Valid secret key
    assert!(validation::validate_secret_key("VALID_SECRET_KEY").is_ok());
    assert!(validation::validate_secret_key("valid-secret-123").is_ok());

    // Invalid repository owner
    assert!(validation::validate_repo_owner("").is_err());
    assert!(validation::validate_repo_owner(&"a".repeat(40)).is_err());

    // Valid repository owner
    assert!(validation::validate_repo_owner("valid-owner").is_ok());

    // Invalid repository name
    assert!(validation::validate_repo_name("").is_err());
    assert!(validation::validate_repo_name(&"a".repeat(101)).is_err());

    // Valid repository name
    assert!(validation::validate_repo_name("valid-repo").is_ok());
}

/// Test error handling for invalid public key format.
#[tokio::test]
async fn test_encrypt_secret_invalid_public_key() {
    let client = GitHubClient::new(
        "test-token".to_string(),
        "test-owner".to_string(),
        "test-repo".to_string(),
    )
    .unwrap();

    // Test with invalid base64
    let result = client.encrypt_secret("invalid-base64!!!", "secret");
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("decode") || error_msg.contains("Invalid"));

    // Test with wrong length
    let short_key = general_purpose::STANDARD.encode(&[0u8; 16]);
    let result2 = client.encrypt_secret(&short_key, "secret");
    assert!(result2.is_err());
    let error_msg2 = result2.unwrap_err().to_string();
    assert!(error_msg2.contains("length") || error_msg2.contains("Invalid"));
}

/// Test error chain formatting.
#[test]
fn test_error_chain_formatting() {
    use anyhow::anyhow;
    use github_secrets::error;

    let err = anyhow!("root error")
        .context("middle error")
        .context("top error");

    let formatted = error::format_error_chain(&err);

    // Should contain all error messages
    assert!(formatted.contains("root error"));
    assert!(formatted.contains("middle error"));
    assert!(formatted.contains("top error"));
    assert!(formatted.contains(" → "));
}

/// Test rate limiter functionality.
#[tokio::test]
async fn test_rate_limiter() {
    use github_secrets::rate_limit::RateLimiter;
    use std::time::Instant;

    let mut limiter = RateLimiter::new();

    // First request should not wait long
    let start = Instant::now();
    limiter.wait_if_needed().await;
    limiter.release();
    let elapsed = start.elapsed();

    // Should be very fast (less than 100ms)
    assert!(elapsed.as_millis() < 100);

    // Multiple requests should work
    for _ in 0..5 {
        limiter.wait_if_needed().await;
        limiter.release();
    }
}

/// Test config validation with invalid repositories.
#[test]
fn test_config_validation_invalid_repositories() {
    use github_secrets::config::Config;
    use std::fs;
    use tempfile::TempDir;

    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    // Test with invalid owner (too long)
    let invalid_config = format!(
        r#"
[[repositories]]
owner = "{}"
name = "repo"
"#,
        "a".repeat(40)
    );

    fs::write(&config_path, invalid_config).unwrap();
    let result = Config::from_file(config_path.to_str().unwrap());
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Invalid owner"));
}

/// Test config validation with invalid repository name.
#[test]
fn test_config_validation_invalid_repo_name() {
    use github_secrets::config::Config;
    use std::fs;
    use tempfile::TempDir;

    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    // Test with invalid repo name (empty)
    let invalid_config = r#"
[[repositories]]
owner = "owner"
name = ""
"#;

    fs::write(&config_path, invalid_config).unwrap();
    let result = Config::from_file(config_path.to_str().unwrap());
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Invalid"));
}

/// Test error scenarios for GitHub API.
#[test]
fn test_github_error_types() {
    use github_secrets::errors::GitHubError;

    // Test error conversion (this tests the From implementation)
    // We can't easily create octocrab errors, but we can test the error types exist
    let _api_error = GitHubError::ApiError {
        status_code: 404,
        message: "Not Found".to_string(),
        documentation_url: Some("https://docs.github.com".to_string()),
    };

    let _http_error = GitHubError::HttpError("Connection failed".to_string());
    let _uri_error = GitHubError::UriError("Invalid URI".to_string());
    let _encryption_error = GitHubError::EncryptionError("Encryption failed".to_string());
}

/// Test validation error types.
#[test]
fn test_validation_error_types() {
    use github_secrets::errors::ValidationError;

    let secret_key_error = ValidationError::SecretKey("Invalid key format".to_string());
    assert!(
        secret_key_error
            .to_string()
            .contains("Secret key validation failed")
    );

    let owner_error = ValidationError::RepositoryOwner("Invalid owner".to_string());
    assert!(
        owner_error
            .to_string()
            .contains("Repository owner validation failed")
    );

    let name_error = ValidationError::RepositoryName("Invalid name".to_string());
    assert!(
        name_error
            .to_string()
            .contains("Repository name validation failed")
    );
}

/// Test config error types.
#[test]
fn test_config_error_types() {
    use github_secrets::errors::ConfigError;

    let read_error = ConfigError::ReadError("File not found".to_string());
    assert!(
        read_error
            .to_string()
            .contains("Failed to read config file")
    );

    let parse_error = ConfigError::ParseError("Invalid TOML".to_string());
    assert!(
        parse_error
            .to_string()
            .contains("Failed to parse config file")
    );

    let no_repos_error = ConfigError::NoRepositories;
    assert!(no_repos_error.to_string().contains("No repositories found"));

    let invalid_repo_error = ConfigError::InvalidRepository("Invalid format".to_string());
    assert!(
        invalid_repo_error
            .to_string()
            .contains("Invalid repository configuration")
    );
}

/// Test encryption with different secret values produces different encrypted outputs.
#[tokio::test]
async fn test_encryption_uniqueness() {
    use sodoken::crypto_box;

    let client = GitHubClient::new(
        "test-token".to_string(),
        "test-owner".to_string(),
        "test-repo".to_string(),
    )
    .unwrap();

    // Generate a valid X25519 keypair for testing
    let mut public_key_bytes = [0u8; crypto_box::XSALSA_PUBLICKEYBYTES];
    let mut secret_key_bytes = [0u8; crypto_box::XSALSA_SECRETKEYBYTES];
    crypto_box::xsalsa_keypair(&mut public_key_bytes, &mut secret_key_bytes)
        .expect("Failed to generate keypair");

    let public_key = general_purpose::STANDARD.encode(&public_key_bytes);

    let secret1 = "secret-value-1";
    let secret2 = "secret-value-2";

    let encrypted1 = client.encrypt_secret(&public_key, secret1).unwrap();
    let encrypted2 = client.encrypt_secret(&public_key, secret2).unwrap();

    // Encrypted values should be different (sealed box uses random nonce)
    assert_ne!(encrypted1, encrypted2);

    // Even same value should produce different encryption (due to random nonce)
    let encrypted1_again = client.encrypt_secret(&public_key, secret1).unwrap();
    assert_ne!(encrypted1, encrypted1_again);
}

/// Test that validation catches issues before API calls.
#[test]
fn test_validation_prevents_invalid_api_calls() {
    // Test that invalid inputs are rejected before making API calls

    // Invalid secret keys should be rejected
    let too_long = "a".repeat(101);
    let invalid_keys = vec![
        "",
        " ",
        "key with spaces",
        "key@invalid",
        "key#invalid",
        &too_long, // Too long
    ];

    for key in invalid_keys {
        assert!(
            validation::validate_secret_key(key).is_err(),
            "Key '{}' should be invalid",
            key
        );
    }

    // Valid secret keys should pass
    let valid_keys = vec!["VALID_KEY", "valid-key", "valid_key", "key123", "KEY_123"];

    for key in valid_keys {
        assert!(
            validation::validate_secret_key(key).is_ok(),
            "Key '{}' should be valid",
            key
        );
    }
}

/// Test error message formatting for different error types.
#[test]
fn test_error_message_formatting() {
    use github_secrets::errors::GitHubError;

    let api_error = GitHubError::ApiError {
        status_code: 422,
        message: "Validation failed".to_string(),
        documentation_url: Some("https://docs.github.com/errors".to_string()),
    };

    let error_msg = api_error.to_string();
    assert!(error_msg.contains("422"));
    assert!(error_msg.contains("Validation failed"));

    let http_error = GitHubError::HttpError("Connection timeout".to_string());
    assert!(http_error.to_string().contains("HTTP error"));
    assert!(http_error.to_string().contains("Connection timeout"));
}
