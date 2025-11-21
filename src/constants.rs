//! Application constants for API endpoints, validation rules, and rate limiting.
//!
//! This module contains all constant values used throughout the application,
//! including GitHub API endpoints, validation constraints, and rate limiting parameters.

/// GitHub API endpoint constants.
pub mod api {
    /// Base path for GitHub Actions secrets API.
    pub const SECRETS_BASE_PATH: &str = "/repos/{owner}/{repo}/actions/secrets";

    /// Path to get the public key for encrypting secrets.
    pub const PUBLIC_KEY_PATH: &str = "/repos/{owner}/{repo}/actions/secrets/public-key";

    /// Path template for updating a specific secret.
    pub const SECRET_PATH_TEMPLATE: &str = "/repos/{owner}/{repo}/actions/secrets/{secret_name}";
}

/// Secret validation constants.
pub mod validation {
    /// Maximum length for a secret key name (GitHub API limit).
    pub const MAX_SECRET_KEY_LENGTH: usize = 100;

    /// Minimum length for a secret key name.
    pub const MIN_SECRET_KEY_LENGTH: usize = 1;

    /// Valid characters for secret key names (alphanumeric, underscore, hyphen).
    /// GitHub allows: letters, numbers, underscores, and hyphens.
    pub const VALID_SECRET_KEY_PATTERN: &str = r"^[a-zA-Z0-9_-]+$";
}

/// Rate limiting constants for GitHub API.
pub mod rate_limit {
    /// Maximum number of requests per hour for authenticated requests (5000 for GitHub).
    pub const REQUESTS_PER_HOUR: u32 = 5000;

    /// Maximum number of concurrent requests to avoid hitting rate limits.
    pub const MAX_CONCURRENT_REQUESTS: usize = 5;

    /// Delay between batches of requests (in milliseconds).
    pub const BATCH_DELAY_MS: u64 = 100;
}

/// Repository validation constants.
pub mod repo {
    /// Maximum length for repository owner name.
    pub const MAX_OWNER_LENGTH: usize = 39; // GitHub username limit

    /// Maximum length for repository name.
    pub const MAX_REPO_NAME_LENGTH: usize = 100;

    /// Minimum length for repository owner name.
    pub const MIN_OWNER_LENGTH: usize = 1;

    /// Minimum length for repository name.
    pub const MIN_REPO_NAME_LENGTH: usize = 1;
}
