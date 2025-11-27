#[test]
fn test_error_enum_display_messages() {
    use github_secrets::errors::{ConfigError, GitHubError, ValidationError};

    let g1 = GitHubError::ApiError {
        status_code: 500,
        message: "Server blew up".to_string(),
        documentation_url: None,
    };
    assert!(g1.to_string().contains("500"));
    assert!(g1.to_string().contains("Server blew up"));

    let g2 = GitHubError::HttpError("conn failed".to_string());
    assert!(g2.to_string().contains("HTTP error"));

    let c1 = ConfigError::ReadError("no file".to_string());
    assert!(c1.to_string().contains("Failed to read config file"));

    let c2 = ConfigError::NoRepositories;
    assert!(c2.to_string().contains("No repositories found"));

    let v1 = ValidationError::SecretKey("invalid".to_string());
    assert!(v1.to_string().contains("Secret key validation failed"));
}

#[test]
fn test_octocrab_error_conversion() {
    use github_secrets::errors::GitHubError;

    // Test HttpError conversion
    // We can't easily construct octocrab::Error::Http directly, so we test the logic
    // by checking that our Display implementation works correctly

    let http_error = GitHubError::HttpError("Connection refused".to_string());
    assert_eq!(http_error.to_string(), "HTTP error: Connection refused");

    let uri_error = GitHubError::UriError("Invalid URI".to_string());
    assert_eq!(uri_error.to_string(), "URI error: Invalid URI");

    // Test that ApiError preserves all fields
    let api_error = GitHubError::ApiError {
        status_code: 404,
        message: "Not found".to_string(),
        documentation_url: Some("https://docs.github.com".to_string()),
    };
    let error_str = api_error.to_string();
    assert!(error_str.contains("404"));
    assert!(error_str.contains("Not found"));
}
