use thiserror::Error;

/// Errors that can occur when working with GitHub API.
#[derive(Error, Debug)]
pub enum GitHubError {
    #[error("GitHub API error (status {status_code}): {message}")]
    ApiError {
        status_code: u16,
        message: String,
        documentation_url: Option<String>,
    },
    #[error("HTTP error: {0}")]
    HttpError(String),
    #[error("URI error: {0}")]
    UriError(String),
    #[error("Failed to encrypt secret: {0}")]
    EncryptionError(String),
    #[error("Failed to get public key: {0}")]
    PublicKeyError(String),
    #[error("Invalid public key format: {0}")]
    InvalidPublicKey(String),
}

/// Errors that can occur when working with configuration.
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to read config file: {0}")]
    ReadError(String),
    #[error("Failed to parse config file: {0}")]
    ParseError(String),
    #[error("No repositories found in config file")]
    NoRepositories,
    #[error("Invalid repository configuration: {0}")]
    InvalidRepository(String),
}

/// Errors that can occur during validation.
#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Secret key validation failed: {0}")]
    SecretKey(String),
    #[error("Repository owner validation failed: {0}")]
    RepositoryOwner(String),
    #[error("Repository name validation failed: {0}")]
    RepositoryName(String),
}

impl From<octocrab::Error> for GitHubError {
    fn from(err: octocrab::Error) -> Self {
        match err {
            octocrab::Error::GitHub { source, .. } => GitHubError::ApiError {
                status_code: source.status_code.as_u16(),
                message: source.message,
                documentation_url: source.documentation_url,
            },
            octocrab::Error::Http { source, .. } => {
                GitHubError::HttpError(source.to_string())
            }
            octocrab::Error::Uri { source, .. } => {
                GitHubError::UriError(source.to_string())
            }
            _ => GitHubError::HttpError(err.to_string()),
        }
    }
}

