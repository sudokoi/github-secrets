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
