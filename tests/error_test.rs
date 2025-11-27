use github_secrets::errors::{ConfigError, GitHubError, ValidationError};

#[test]
fn test_github_error_display() {
    let api_error = GitHubError::ApiError {
        status_code: 404,
        message: "Not Found".to_string(),
        documentation_url: Some("http://doc.url".to_string()),
    };
    assert_eq!(
        api_error.to_string(),
        "GitHub API error (status 404): Not Found"
    );

    let http_error = GitHubError::HttpError("Connection failed".to_string());
    assert_eq!(http_error.to_string(), "HTTP error: Connection failed");

    let uri_error = GitHubError::UriError("Invalid URI".to_string());
    assert_eq!(uri_error.to_string(), "URI error: Invalid URI");

    let enc_error = GitHubError::EncryptionError("Key error".to_string());
    assert_eq!(enc_error.to_string(), "Failed to encrypt secret: Key error");

    let pk_error = GitHubError::PublicKeyError("PK error".to_string());
    assert_eq!(pk_error.to_string(), "Failed to get public key: PK error");

    let inv_pk = GitHubError::InvalidPublicKey("Bad format".to_string());
    assert_eq!(inv_pk.to_string(), "Invalid public key format: Bad format");
}

#[test]
fn test_config_error_display() {
    let read_err = ConfigError::ReadError("Permission denied".to_string());
    assert_eq!(
        read_err.to_string(),
        "Failed to read config file: Permission denied"
    );

    let parse_err = ConfigError::ParseError("Syntax error".to_string());
    assert_eq!(
        parse_err.to_string(),
        "Failed to parse config file: Syntax error"
    );

    let no_repos = ConfigError::NoRepositories;
    assert_eq!(no_repos.to_string(), "No repositories found in config file");

    let inv_repo = ConfigError::InvalidRepository("Missing owner".to_string());
    assert_eq!(
        inv_repo.to_string(),
        "Invalid repository configuration: Missing owner"
    );
}

#[test]
fn test_validation_error_display() {
    let key_err = ValidationError::SecretKey("Too short".to_string());
    assert_eq!(
        key_err.to_string(),
        "Secret key validation failed: Too short"
    );

    let owner_err = ValidationError::RepositoryOwner("Invalid chars".to_string());
    assert_eq!(
        owner_err.to_string(),
        "Repository owner validation failed: Invalid chars"
    );

    let name_err = ValidationError::RepositoryName("Empty".to_string());
    assert_eq!(
        name_err.to_string(),
        "Repository name validation failed: Empty"
    );
}
