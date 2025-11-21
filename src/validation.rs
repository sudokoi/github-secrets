//! Input validation utilities.
//!
//! This module provides validation functions for secret keys, repository names,
//! repository owners, and GitHub tokens to ensure they meet GitHub API requirements.

use anyhow::{Context, Result};
use regex::Regex;
use crate::constants;

/// Validate a secret key name according to GitHub API requirements.
///
/// # Arguments
///
/// * `key` - The secret key name to validate
///
/// # Returns
///
/// Returns `Ok(())` if the key is valid, or an error with a descriptive message.
///
/// # Errors
///
/// Returns an error if:
/// - The key is empty
/// - The key exceeds the maximum length
/// - The key contains invalid characters
pub fn validate_secret_key(key: &str) -> Result<()> {
    let trimmed = key.trim();
    
    if trimmed.is_empty() {
        anyhow::bail!("Secret key cannot be empty");
    }
    
    if trimmed.len() < constants::validation::MIN_SECRET_KEY_LENGTH {
        anyhow::bail!(
            "Secret key must be at least {} character(s) long",
            constants::validation::MIN_SECRET_KEY_LENGTH
        );
    }
    
    if trimmed.len() > constants::validation::MAX_SECRET_KEY_LENGTH {
        anyhow::bail!(
            "Secret key cannot exceed {} characters (got {})",
            constants::validation::MAX_SECRET_KEY_LENGTH,
            trimmed.len()
        );
    }
    
    // Validate character pattern
    let re = Regex::new(constants::validation::VALID_SECRET_KEY_PATTERN)
        .context("Failed to compile validation regex")?;
    
    if !re.is_match(trimmed) {
        anyhow::bail!(
            "Secret key can only contain letters, numbers, underscores, and hyphens. Got: '{}'",
            trimmed
        );
    }
    
    Ok(())
}

/// Validate a repository owner name.
///
/// # Arguments
///
/// * `owner` - The repository owner name to validate
///
/// # Returns
///
/// Returns `Ok(())` if the owner is valid, or an error with a descriptive message.
pub fn validate_repo_owner(owner: &str) -> Result<()> {
    let trimmed = owner.trim();
    
    if trimmed.is_empty() {
        anyhow::bail!("Repository owner cannot be empty");
    }
    
    if trimmed.len() < constants::repo::MIN_OWNER_LENGTH {
        anyhow::bail!(
            "Repository owner must be at least {} character(s) long",
            constants::repo::MIN_OWNER_LENGTH
        );
    }
    
    if trimmed.len() > constants::repo::MAX_OWNER_LENGTH {
        anyhow::bail!(
            "Repository owner cannot exceed {} characters (got {})",
            constants::repo::MAX_OWNER_LENGTH,
            trimmed.len()
        );
    }
    
    Ok(())
}

/// Validate a repository name.
///
/// # Arguments
///
/// * `name` - The repository name to validate
///
/// # Returns
///
/// Returns `Ok(())` if the name is valid, or an error with a descriptive message.
pub fn validate_repo_name(name: &str) -> Result<()> {
    let trimmed = name.trim();
    
    if trimmed.is_empty() {
        anyhow::bail!("Repository name cannot be empty");
    }
    
    if trimmed.len() < constants::repo::MIN_REPO_NAME_LENGTH {
        anyhow::bail!(
            "Repository name must be at least {} character(s) long",
            constants::repo::MIN_REPO_NAME_LENGTH
        );
    }
    
    if trimmed.len() > constants::repo::MAX_REPO_NAME_LENGTH {
        anyhow::bail!(
            "Repository name cannot exceed {} characters (got {})",
            constants::repo::MAX_REPO_NAME_LENGTH,
            trimmed.len()
        );
    }
    
    Ok(())
}

/// Validate a GitHub token format (basic checks).
///
/// GitHub tokens typically start with "ghp_" for personal access tokens
/// or "gho_" for OAuth tokens, but we'll do a more general validation.
///
/// # Arguments
///
/// * `token` - The token to validate
///
/// # Returns
///
/// Returns `Ok(())` if the token format appears valid, or an error.
pub fn validate_token(token: &str) -> Result<()> {
    let trimmed = token.trim();
    
    if trimmed.is_empty() {
        anyhow::bail!("GitHub token cannot be empty");
    }
    
    if trimmed.len() < 20 {
        anyhow::bail!("GitHub token appears too short (minimum 20 characters)");
    }
    
    if trimmed.len() > 200 {
        anyhow::bail!("GitHub token appears too long (maximum 200 characters)");
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_secret_key_valid() {
        assert!(validate_secret_key("MY_SECRET").is_ok());
        assert!(validate_secret_key("my-secret-123").is_ok());
        assert!(validate_secret_key("SECRET_KEY_123").is_ok());
    }

    #[test]
    fn test_validate_secret_key_invalid() {
        assert!(validate_secret_key("").is_err());
        assert!(validate_secret_key(" ").is_err());
        assert!(validate_secret_key("secret with spaces").is_err());
        assert!(validate_secret_key("secret@invalid").is_err());
        assert!(validate_secret_key(&"a".repeat(101)).is_err());
    }

    #[test]
    fn test_validate_repo_owner() {
        assert!(validate_repo_owner("owner").is_ok());
        assert!(validate_repo_owner("").is_err());
        assert!(validate_repo_owner(&"a".repeat(40)).is_err());
    }

    #[test]
    fn test_validate_repo_name() {
        assert!(validate_repo_name("repo").is_ok());
        assert!(validate_repo_name("").is_err());
        assert!(validate_repo_name(&"a".repeat(101)).is_err());
    }

    #[test]
    fn test_validate_token() {
        assert!(validate_token("ghp_1234567890123456789012345678901234567890").is_ok());
        assert!(validate_token("").is_err());
        assert!(validate_token("short").is_err());
        assert!(validate_token(&"a".repeat(201)).is_err());
    }
}
